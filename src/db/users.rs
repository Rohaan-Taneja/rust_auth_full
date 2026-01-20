// we will add queries/struct/function to inteact with the user tables
// which can be called via handlers , so it is same as .servide file

use axum::http::StatusCode;
use chrono::*;
use diesel::prelude::*;

use crate::{
    errors::HttpError,
    models::{NewUser, Users},
    schema::users::{self},
};

// a manager which holds db connection
pub struct UserRepository<'a> {
    pub db_con: &'a mut PgConnection,
}


// implementing user related functions
impl<'a> UserRepository<'a> {
    // function to give db connection access to this repository/manager
    pub fn new(db_connection: &'a mut PgConnection) -> Self {
        UserRepository {
            db_con: db_connection,
        }
    }

    /**
     * we wil take all the user details as input and db conn also and call db to store it
     * it is like a service function
     */
    pub async fn save_user(
        &mut self,
        name: impl Into<String>,
        email: impl Into<String>,
        pass: impl Into<String>,
        verification_token: impl Into<String>,
        token_expires_at: DateTime<Utc>,
    ) -> Result<Users, HttpError> {

        let new_user = NewUser {
            name: name.into(),
            email: email.into(),
            verified : false,
            password: pass.into(),
            verification_token: Some(verification_token.into()),
            token_expires_at: Some(token_expires_at.into()),
        };

        let saved_user = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(self.db_con)
            .map_err(|e| HttpError {
                message: e.to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            })?;

            Ok(saved_user)

       
    }
}
