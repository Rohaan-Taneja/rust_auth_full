// we will add all the db related function related to user auth here

use std::string;

use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;

use crate::dtos::login_dto::loggedInUser;
use crate::models::{NewUser, NewUserEmailVerifications, UserEmailVerifications};
use crate::schema::{user_email_verifications, users};
use crate::utils::password::{hash_pass, validate_pas};
use crate::{errors::HttpError, models::Users};
use diesel::result::Error;

// parkinglol/pool of db connection
pub type PgPool = Pool<ConnectionManager<PgConnection>>;
// manager which have db connection and have all the function impl for auth related things ,
//  sign up , signin etc'
pub struct AuthRepository {
    pub db_con: PgPool,
}

// implementing all the auth functions
// this struct will get onwership/clone of arc referece pointer of the db_connection
impl<'a> AuthRepository {
    pub fn new(con: PgPool) -> Self {
        AuthRepository { db_con: con }
    }

    // getting/funding user on the basis of user_id(primary key)
    pub async fn get_user(&mut self, user_id: Uuid) -> Result<Users, HttpError> {
        // we are asking for 1 db connection from pool
        // the con variable will get 1 pooled db connection temporarily
        // after the function ends the connection will automatically goes back to the pool
        // connection is just temporarily give and will automatically taked back after the variable scope ends
        let mut con = self.db_con.get().map_err(|e| {
            HttpError::new(
                "error is connection pool".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        // putting diesel db call in async environment , so that it does not bock thread
        let res = tokio::task::spawn_blocking(move || {
            users::table
                .find(user_id)
                .select(Users::as_select())
                .get_result::<Users>(&mut con)
            // either a diesel error or not found , so matching that
        })
        .await
        .map_err(|e| HttpError::server_error("internal server in resolving asyn task"))? //spwn closure error
        .map_err(|e| match e {
            // db call error
            Error::NotFound => HttpError::new("user not found", StatusCode::NOT_FOUND),
            _ => HttpError::server_error("internal server error in diesel"),
        })?;

        Ok(res)
    }

    /**
     * we will take user id as input
     * @result => we will fetch the user_email_verification data/status
     */
    pub async fn get_user_verification_status(
        &mut self,
        userr_email: impl Into<String>,
    ) -> Result<UserEmailVerifications, HttpError> {
        let email = userr_email.into();

        let mut con = self.db_con.get().map_err(|e| {
            HttpError::new(
                "error is connection pool".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let result = tokio::task::spawn_blocking(move || {
            user_email_verifications::table
                .filter(user_email_verifications::user_email.eq(&email))
                .first::<UserEmailVerifications>(&mut con)
        })
        .await
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .map_err(|e| {
            HttpError::new(
                "error while getting user email verification status".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        Ok(result)
    }

    /**
     * we wil take all the user details as input and db conn also and call db to store it
     * it is like a service function
     */
    pub async fn save_new_user(
        &mut self,
        name: impl Into<String>,
        email: impl Into<String>,
        pass: impl Into<String>,
    ) -> Result<Users, HttpError> {
        let new_user = NewUser {
            name: name.into(),
            email: email.into(),
            verified: false,
            password: pass.into(),
        };

        let mut con = self.db_con.get().map_err(|e| {
            HttpError::new(
                "error is connection pool".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        let saved_user = tokio::task::spawn_blocking(move || {
            diesel::insert_into(users::table)
                .values(&new_user)
                .get_result(&mut con)
        })
        .await
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(saved_user)
    }

    // in this we will save the newly regiters(not verifed ) user to the user_verification table (otp , email, exprity etc)
    pub async fn add_new_user_to_user_verification_table(
        &mut self,
        otp: impl Into<String>,
        email: impl Into<String>,
        expiry: DateTime<Utc>,
    ) -> Result<bool, HttpError> {
        let ver_otp = otp.into();
        let ver_email = email.into();

        // we will pool of connection mamnagers and we will get connection manager
        let mut con = self.db_con.get().map_err(|e| {
            HttpError::new(
                "error is connection pool".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        // created given value struct , now we will give it to the save function
        // it will add other values(automated genrated value) and save it in the db
        let new_user_verification_data = NewUserEmailVerifications {
            user_email: ver_email,
            otp: ver_otp,
            expires_at: Some(expiry),
        };

        // added db call to the tokio async enviromenet , now the db call wont block main thread
        let saved_user_verification_data = tokio::task::spawn_blocking(move || {
            diesel::insert_into(user_email_verifications::table)
                .values(&new_user_verification_data)
                .returning(UserEmailVerifications::as_returning())
                .get_result(&mut con)
        })
        .await
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(true)
    }

    /**
     * updating new user otp verification stats(used) to true
     */
    pub async fn update_new_user_otp_status(
        &mut self,
        user_email: impl Into<String>,
    ) -> Result<bool, HttpError> {
        let email = user_email.into();

        let mut con = self.db_con.get().map_err(|e| {
            HttpError::new(
                "error is connection pool".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        tokio::task::spawn_blocking(move || {
            diesel::update(user_email_verifications::table)
                .filter(user_email_verifications::user_email.eq(email))
                .set(user_email_verifications::used.eq(true))
                .execute(&mut con)
        })
        .await
        .map_err(|e| {
            HttpError::new(
                "inetrnal error is thread",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?
        .map_err(|e| {
            HttpError::new(
                "inetrnal error is thread",
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

        Ok(true)
    }

    /**
     * in this function we will update/add users auth/jwt token + exp to the user table
     */
    pub async fn update_jwt_token_to_user(
        &mut self,
        user_email: impl Into<String>,
        jwt_token: impl Into<String>,
        token_exp: DateTime<Utc>,
    ) -> Result<bool, HttpError> {
        let email = user_email.into();
        let token = jwt_token.into();

        let mut con = self
            .db_con
            .get()
            .map_err(|e| HttpError::server_error("error in getting db ppol"))?;

        // hadled db req in asyn environment , as else it will block the main thread
        tokio::task::spawn_blocking(move || {
            diesel::update(users::table)
                .filter(users::email.eq(email))
                .set((
                    users::verification_token.eq(token),
                    users::verified.eq(true),
                    users::token_expires_at.eq(Some(token_exp)),
                ))
                .execute(&mut con)
        })
        .await
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?;

        Ok(true)
    }

    /**
     * @inputs => we will get email and password
     * @response => either user is verified user , we will return user id else unauthorized user , we will send unauthoried error
     */
    pub async fn verify_login_user(
        &mut self,
        email: impl Into<String>,
        password: impl Into<String>,
    ) -> Result<String, HttpError> {
        // we will get user from email
        // if does not exist , show unauthoried error

        //  we will havw incoming pass and compare it with the saved hashed password
        // if not equal , we will show wrong password error , or unauthorized
        // now we will check for verified ,
        //  if not verified, show error , user not verified , go to signup screen , will see this flow

        // if equal , verified , we will return okay and user id

        let mut conn = self
            .db_con
            .get()
            .map_err(|e| HttpError::server_error("error in getting db ppol"))?;

        let user_email = email.into();
        let pass = password.into();

        // finding user from from the users table
        let res = tokio::task::spawn_blocking(move || {
            users::table
                .filter(users::email.eq(user_email.clone()))
                .first::<Users>(&mut conn)
        })
        .await
        .map_err(|e| HttpError {
            message: e.to_string(),
            status: StatusCode::INTERNAL_SERVER_ERROR,
        })?
        .map_err(|e| match e {
            Error::NotFound => HttpError {
                message: "user not found".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR,
            },
            _ => HttpError::server_error("error while fetching users table data"),
        })?;

        // check password equal
        let validate_pass = validate_pas(&pass, &res.password).map_err(|e|HttpError::new(e.to_string() , StatusCode::INTERNAL_SERVER_ERROR))?;

        // password validation is false , wrong password
        if !validate_pass {
             return Err(HttpError::unauthorized(
                "wrong password",
            ));
            
        }
        // now pass and email is correct but
        // user not verified
        if !res.verified {
            return Err(HttpError::unauthorized(
                "users email is not verfied , please sign and verify email",
            ));
        }

        // now user pass ,email and verifed everything is okayy
        // we will return user id in response

        Ok(res.id.to_string())
    }
}
