// we will add queries/struct/function to inteact with the user tables
// which can be called via handlers , so it is same as .servide file

use axum::http::StatusCode;
use chrono::*;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use uuid::Uuid;

use crate::{
    DbPool,
    errors::HttpError,
    models::{NewUser, Users},
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
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .map_err(|e| {
            HttpError::new(
                "error while updating the user password".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        Ok(true)
    }
}
