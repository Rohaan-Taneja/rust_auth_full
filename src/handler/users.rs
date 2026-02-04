use std::sync::Arc;

use axum::{
    Extension, Json, Router, extract::Path, http::StatusCode, response::IntoResponse, routing::get,
};
use diesel::result;
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::users::UserRepository,
    dtos::{
        loggedIn_user_reset_password_dto::LoggedInUserResetPasswordDTO, note_dto::NoteDTO, user_dto::UserDTO, user_notes_vec_response_dto::UserNotesVecResponseDTO, user_ok_response_dto::UserOkResponsesDTO
    },
    errors::HttpError,
    middleware::JwtAuthMiddleware,
    utils::password::validate_pas,
};

pub fn users_handler() -> Router {
    Router::new()
    .route("/user_details", get(get_user_data))
    // .route("/user_details", get(get_user_data))
}

/**
 * input , wew ill get auth token from the frontend
 * if tokens are corrects , we will extract user details from it
 * return => we are returning user name from this api
 */
pub async fn get_user_data(
    Extension(user): Extension<JwtAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    println!("middleware worked {:?}", user.user);

    let user_data = UserDTO{
        name : user.user.name,
        email : user.user.email,
    };

    Ok((StatusCode::ACCEPTED, user_data))
}

/**
 * in this service we will update the the logged in user password
 * @input => we will get user from auth midleware(from jwt tokens sent by the frontend) , old pass , new pass
 * @result => we will verify user and its old password , if they are eqaul , we will update the new password
 */
pub async fn update_loggedIn_user_password(
    Extension(app_state): Extension<Arc<AppState>>,

    Extension(user_data): Extension<JwtAuthMiddleware>,
    Json(passwords_data): Json<LoggedInUserResetPasswordDTO>,
) -> Result<impl IntoResponse, HttpError> {
    // we will validate the inputs dto conditions
    passwords_data
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let old_incoming_pass = passwords_data.old_password;
    let new_incoming_pass = passwords_data.new_password;

    let user = user_data.user;

    let pass_compared_result = validate_pas(&old_incoming_pass, &user.password)
        .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // if old passowrd is not equal , then we will return error
    if !pass_compared_result {
        return Err(HttpError::bad_request(
            "the old password is not equal".to_string(),
        ));
    }

    let db_con = app_state.db.clone();
    let mut user_repo = UserRepository::new(db_con);

    let res = user_repo
        .update_loggedIn_user_pass(user.id.clone(), new_incoming_pass.to_string())
        .await
        .map_err(|e| e)?;

    Ok(UserOkResponsesDTO {
        status: StatusCode::OK,
        message: "password updated".to_string(),
        data: None,
    })
}

/**
 * we will create/add note to user name
 * @inputs => app state , user(from auth middleware) , title , content
 * @result => we will save post in user name and return true or error
 */
pub async fn create_user_note(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_data): Extension<JwtAuthMiddleware>,
    Json(note_data): Json<NoteDTO>,
) -> Result<impl IntoResponse, HttpError> {
    // validating the oncoming body content
    note_data
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let userId = user_data.user.id;
    let db_con_pool = app_state.db.clone();

    let mut user_repo = UserRepository::new(db_con_pool);

    // saving the usernote to db
    let noteId = user_repo
        .create_user_note(userId.clone(), note_data)
        .await
        .map_err(|e| e)?;

        // let mut data = Vec::new();
        // data.
    Ok(UserOkResponsesDTO {
        status: StatusCode::CREATED,
        message: "new user not created".to_string(),
        data: Some(vec![noteId]),
    })
}

/**
 * if we want to update title or content we will call this function
 * @input => it will get app_state , user from auth middleware , notedto format body
 * @result => we will update and return true
 */
pub async fn update_user_note(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_data): Extension<JwtAuthMiddleware>,
    Path(noteId): Path<String>,
    Json(updated_user_note): Json<NoteDTO>,
) -> Result<impl IntoResponse, HttpError> {
    updated_user_note
        .validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let userId = user_data.user.id;
    let db_con_pool = app_state.db.clone();

    let noteId_uuid =
        Uuid::parse_str(&noteId).map_err(|e| HttpError::bad_request("noteId is not a valid Id"))?;

    let mut user_repo = UserRepository::new(db_con_pool);

    // updating the usernote to db
    let noteId = user_repo
        .update_user_note(noteId_uuid, updated_user_note)
        .await
        .map_err(|e| e)?;

    Ok(UserOkResponsesDTO {
        status: StatusCode::OK,
        message: "updated user note".to_string(),
        data: None,
    })
}

/**
 * when user wants to delete his note
 * @input => app state , user from auth middleware , noteId
 * @result => we will delete user note
 */
pub async fn delete_user_note(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_data): Extension<JwtAuthMiddleware>,
    Path(noteId): Path<String>,
) -> Result<impl IntoResponse, HttpError> {
    let userId = user_data.user.id;
    let db_con_pool = app_state.db.clone();

    let noteId_uuid =
        Uuid::parse_str(&noteId).map_err(|e| HttpError::bad_request("noteId is not a valid Id"))?;

    let mut user_repo = UserRepository::new(db_con_pool);

    // deleting the usernote to db
    let noteId = user_repo
        .delete_user_note(noteId_uuid)
        .await
        .map_err(|e| e)?;

    Ok(UserOkResponsesDTO {
        status: StatusCode::OK,
        message: "updated user note".to_string(),
        data: None,
    })
}

/**
 * we will get user notes in order by created_at
 * recently created to oldest , based on created_at
 * we will get page , offset would be 5-5
 * we will return 5 notes at a time
 * we will give result in pages
*/
pub async fn get_users_notes(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user_obj): Extension<JwtAuthMiddleware>,
    Path(page): Path<i64>,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = user_obj.user.id;

    let db_con = app_state.db.clone();

    let mut user_repo = UserRepository::new(db_con);

    let vec_of_users_notes = user_repo
        .get_user_notes_in_pages(user_id, page)
        .await
        .map_err(|e| e)?;

    Ok(UserNotesVecResponseDTO {
        status: StatusCode::OK,
        message: "success".to_string(),
        notesVec: vec_of_users_notes,
    })
}
