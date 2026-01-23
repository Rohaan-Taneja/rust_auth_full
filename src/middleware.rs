use std::{f32::consts::E, sync::Arc};

use axum::{
    Extension,
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::extract::CookieJar;
use lettre::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    AppState, db::auth::AuthRepository, errors::HttpError, models::Users,
    utils::token::decode_token,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
// this is the resp struct that we will return/attach to req header
pub struct JwtAuthMiddleware {
    pub user: Users,
}

/**
 * inputs
 * app state
 * this function is extracting tokens from cookie or auth header
 * then getting user from the token
 * then finding the user struct from the db using userid
 * the creting a jwtAuthMiddleware struct , adding user to it and returning it
 */
pub async fn auth(
    cookie_jar: CookieJar,
    Extension(app_state): Extension<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, HttpError> {
    // step-1
    // extracting token either from cookies or authorization header
    let token_data = req
        .headers()
        .get("AUTHORIZATION")
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        });

    // if token id then ok else throw error token not found
    let token = token_data.ok_or_else(|| HttpError::unauthorized("token not found"))?;

    // calling decode function to decode token and get user id from it
    let token_data =
        decode_token(token.as_str()).map_err(|_| HttpError::unauthorized("invalid token"))?;

    // coverting string uuid(user id ) to uuid data type
    let user_id =
        Uuid::parse_str(&token_data).map_err(|_| HttpError::unauthorized("Invalid Token"))?;

    // calling db user function to get the user struct from user id(uuid)
    let mut db_pool = app_state.db.clone();
    let mut auth_repo = AuthRepository::new(db_pool);

    let user_data = auth_repo.get_user(user_id).await.map_err(|e| e)?;

    // adding data to the req haspmap
    req.extensions_mut().insert(JwtAuthMiddleware {
        user: user_data.clone(),
    });
    // moving to the next request , (await because rew will give future , so we have to await)
    Ok(next.run(req).await)
}
