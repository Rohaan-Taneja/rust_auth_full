use std::sync::Arc;

use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::get};
use validator::Validate;

use crate::{
    AppState, dtos::loggedIn_user_reset_password_dto::LoggedInUserResetPasswordDTO,
    errors::HttpError, middleware::JwtAuthMiddleware, utils::password::validate_pas,
};

pub fn users_handler() -> Router {
    Router::new().route("/me", get(get_user_data))
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

    Ok((StatusCode::ACCEPTED, Json(user.user.name)))
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
    passwords_data.validate().map_err(|e| HttpError::bad_request(e.to_string()))?;

    let old_incoming_pass = passwords_data.old_password;
    let new_incoming_pass = passwords_data.new_password;

    let user = user_data.user;

    let pass_compared_result = validate_pas(&old_incoming_pass, &user.password).map_err(|e| HttpError::new(e.to_string() , StatusCode::INTERNAL_SERVER_ERROR))?;

    // if old passowrd is not equal , then we will return error 
    if !pass_compared_result {
        return Err(HttpError::bad_request("the old password is not equal".to_string()))
    }

    
    Ok("hello".to_string())
}
