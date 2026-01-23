use axum::{Extension, Router, response::IntoResponse, routing::get};

use crate::{errors::HttpError, middleware::JwtAuthMiddleware};





pub fn users_handler()->Router{
    Router::new()
        .route("/user_details", get(get_user_data))


}




pub async fn get_user_data(Extension(user) : Extension<JwtAuthMiddleware>)-> Result<impl IntoResponse , HttpError>{

    println!("middleware worked {:?}" , user.user);

    Ok("hello")
}