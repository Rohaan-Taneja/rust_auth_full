use axum::{Json, http::StatusCode, response::IntoResponse};
use lettre::message;
use serde::{Deserialize, Serialize};

use crate::models::UserNotes;

#[derive( Debug , Serialize )]
pub struct UserNotesVecResponseDTO{

    #[serde(with = "http_serde::status_code")] // conming from http_serde , helps in ser/deser http types , nomrla serde crate do not know
    pub status : StatusCode,
    pub message : String ,
    pub notesVec : Vec<UserNotes>
}


impl IntoResponse for UserNotesVecResponseDTO{

    fn into_response(self) -> axum::response::Response {

        // converting to json then calling intoresponse 
        return Json(self).into_response()
        
    }
}