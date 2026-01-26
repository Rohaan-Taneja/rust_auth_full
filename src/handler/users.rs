use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::get};

use crate::{errors::HttpError, middleware::JwtAuthMiddleware};





pub fn users_handler()->Router{
    Router::new()
        .route("/me", get(get_user_data))
        


}




/**
 * input , wew ill get auth token from the frontend 
 * if tokens are corrects , we will extract user details from it
 * return => we are returning user name from this api 
 */
pub async fn get_user_data(Extension(user) : Extension<JwtAuthMiddleware>)-> Result<impl IntoResponse , HttpError>{

    println!("middleware worked {:?}" , user.user);

    Ok((StatusCode::ACCEPTED , Json(user.user.name)))
}


