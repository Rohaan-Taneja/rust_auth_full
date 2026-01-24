// we will add queries/struct/function to inteact with the user tables
// which can be called via handlers , so it is same as .servide file

use axum::http::StatusCode;
use chrono::*;
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

use crate::{
    DbPool,
    errors::HttpError,
    models::{NewUser, Users},
    schema::users::{self},
};

pub struct UserRepository {
    pub db_con: DbPool,
}

// implementing user related functions
impl UserRepository {
    // function to give db pool access to this repository/manager
    pub fn new(db_connection: DbPool) -> Self {
        UserRepository {
            db_con: db_connection,
        }
    }
}
