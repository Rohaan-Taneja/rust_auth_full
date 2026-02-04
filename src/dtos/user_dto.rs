use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct UserDTO {
    pub name: String,
    pub email: String,
}

impl IntoResponse for UserDTO {
    fn into_response(self) -> axum::response::Response {
        return Json(self).into_response();
    }
}
