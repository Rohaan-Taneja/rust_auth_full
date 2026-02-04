// we will add queries/struct/function to inteact with the user tables
// which can be called via handlers , so it is same as .servide file

use axum::{http::StatusCode, response::IntoResponse};
use chrono::*;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use uuid::Uuid;

use crate::{
    DbPool,
    dtos::note_dto::NoteDTO,
    errors::HttpError,
    models::{NewUser, NewUserNote, UserNotes, Users},
    schema::user_notes,
};

use crate::schema::{user_email_verifications, users};

pub struct UserRepository {
    pub db_con: DbPool,
}

// implementing user related functions
impl UserRepository {
    // function to give db pool access to this repository/manager
    pub fn new(db_connection: DbPool) -> Self {
        UserRepository {
            db_con: db_connection,
        }
    }

    /**
     * In this service we will update the password of the loggedn in user
     *  @inputs => we will get id of the user and new password
     * @we will update the password and return tru
     */
    pub async fn update_loggedIn_user_pass(
        &mut self,
        user_id: Uuid,
        new_pass: impl Into<String>,
    ) -> Result<bool, HttpError> {
        let mut conn = self.db_con.get().map_err(|e| {
            HttpError::new(
                "unable to to db connection".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let user_pass = new_pass.into();

        let result = tokio::task::spawn_blocking(move || {
            diesel::update(users::table)
                .filter(users::id.eq(user_id.clone()))
                .set(users::password.eq(user_pass))
                .execute(&mut conn)
        })
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .map_err(|e| {
            HttpError::new(
                "error while updating the user password".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        Ok(true)
    }

    /**
     * saving user notes to table
     */
    pub async fn create_user_note(
        &mut self,
        user_id: Uuid,
        note_data: NoteDTO,
    ) -> Result<String, HttpError> {
        let title = note_data.title;
        let content = note_data.content;

        // get connection
        let mut con = self
            .db_con
            .get()
            .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

        let new_note = NewUserNote {
            user_id: user_id.clone(),
            title: title,
            content: content,
        };

        // saving user notes
        let result = tokio::task::spawn_blocking(move || {
            diesel::insert_into(user_notes::table)
                .values(&new_note)
                .returning(UserNotes::as_returning())
                .get_result(&mut con)
        })
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .map_err(|e| {
            HttpError::new(
                "getting error in saving user notes".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        // returning note id
        Ok(result.id.to_string())
    }

    /**
     * we will get note id and we will update it title/cotent
     * @input => we will get note id , updated note data
     * @result => we will update update and returd bool
     */
    pub async fn update_user_note(
        &mut self,
        note_id: Uuid,
        note_data: NoteDTO,
    ) -> Result<bool, HttpError> {
        let title = note_data.title;
        let content = note_data.content;

        // get connection
        let mut con = self
            .db_con
            .get()
            .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

        // updating user note user notes
        let result = tokio::task::spawn_blocking(move || {
            diesel::update(user_notes::table)
                .filter(user_notes::id.eq(note_id))
                .set((user_notes::title.eq(title), user_notes::content.eq(content)))
                .execute(&mut con)
        })
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .map_err(|e| {
            HttpError::new(
                "getting error in updating user notes".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        Ok(true)
    }

    /**
     * we will delete this specific note of the user
     * @input => we will get note id
     * @result => we will delete this note
     */
    pub async fn delete_user_note(&mut self, note_id: Uuid) -> Result<bool, HttpError> {
        // get connection
        let mut con = self
            .db_con
            .get()
            .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

        // deleting user note
        let result = tokio::task::spawn_blocking(move || {
            diesel::delete(user_notes::table.find(note_id)).execute(&mut con)
        })
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .map_err(|e| {
            HttpError::new(
                "getting error in deleting the user note".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        Ok(true)
    }

    /**
     * we will give user notes in 5-5 sets of pages
     * @input => we will get page number , user id . we will have offset(which 5 set , 1st 5 or 2nd five or 3rd five , etc)
     * @return => we will return nuser notes in 5 sets of that specific offset or page
     */
    pub async fn get_user_notes_in_pages(
        &mut self,
        user_id: Uuid,
        page: i64,
    ) -> Result<Vec<UserNotes>, HttpError> {
        // how many to leave , before starting returning the result
        let offset = (&page - 1) * 5;

        let mut con = self
            .db_con
            .get()
            .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

        let user_notes = tokio::task::spawn_blocking(move || {
            user_notes::table
                .filter(user_notes::user_id.eq(&user_id))
                .order_by(user_notes::created_at.desc())
                .limit(5)
                .offset(offset)
                .load::<UserNotes>(&mut con)
        })
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?
        .map_err(|e| HttpError::server_error(e.to_string()))?;

        Ok(user_notes)
    }
}
