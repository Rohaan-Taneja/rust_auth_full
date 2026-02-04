use axum::{Json, http::StatusCode, response::IntoResponse};
use diesel::serialize;
use serde::Serialize;



#[derive(Serialize , Debug , Clone)]
pub struct UserOkResponsesDTO{
    #[serde(with = "http_serde::status_code")]
    pub status : StatusCode ,
    pub message : String,
    pub data : Option<Vec<String>>
    

}

impl IntoResponse for UserOkResponsesDTO{
    fn into_response(self) -> axum::response::Response {
        return Json(self).into_response();
    }
}