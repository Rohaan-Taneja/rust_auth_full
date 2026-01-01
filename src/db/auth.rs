// we will add all the db related function related to user auth here

use axum::http::StatusCode;
use diesel::prelude::*;
use uuid::Uuid;

use crate::schema::users;
use crate::{errors::HttpError, models::Users};
use diesel::result::Error;

// manager which have db connection and have all the function impl for auth related things ,
//  sign up , signin etc'
pub struct AuthRepository<'a> {
    pub db_con: &'a mut PgConnection,
}

// implementing all the auth functions
impl<'a> AuthRepository<'a> {
    pub fn new(con: &'a mut PgConnection) -> Self {
        AuthRepository { db_con: con }
    }

    // getting/funding user on the basis of user_id(primary key)
    pub fn get_user(&mut self, user_id: Uuid) -> Result<Users, HttpError> {
        users::table
            .find(user_id)
            .select(Users::as_select())
            .get_result::<Users>(self.db_con)
            // either a diesel error or not found , so matching that
            .map_err(|e| match e {
                Error::NotFound => HttpError::new("user not found", StatusCode::NOT_FOUND),
                _ => HttpError::server_error("internal server error in diesel"),
            })
    }
}
