// we will write auth handler/controller code here

use std::sync::Arc;

use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use chrono::{Duration, Utc};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::{UserRepository, auth::AuthRepository},
    dtos::{
        register_dto::{self, RegisterUser},
        verify_email_dto::{self, VerifyEmailDTO},
    },
    errors::HttpError,
    mail::{
        mail::construct_mail,
        sendMail::{self, send_mail},
    },
    utils::{
        self,
        password::{self, generate_otp},
        token::create_token,
    },
};

pub fn auth_handler() -> Router {
    Router::new()
        .route("/register", post(register_user))
        .route("/verify-email", post(verify_user))
}

/**
 * register user handler and send otp to user email
 * inputs
 * user details in register user dto format details
 * shared app state (containg db connection) ,
 * data will be coming in json format , we need data in struct format ,
 */
pub async fn register_user(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<RegisterUser>,
) -> Result<impl IntoResponse, HttpError> {
    //return type is result<T , E> , both T and E has IntoResponse trait implemented
    // calling validate function given by validator trait applied on register_dto
    // it will check if all the validation written for fields is valid or not
    body.validate()
        .map_err(|e| HttpError::bad_request(e.to_string()))?;

    // otp verification
    let verification_token = generate_otp(); // otp/verification token , will send on email
    let exp_duration = Utc::now() + Duration::minutes(5); // 5 minutes time to validate the otp 

    let hashed_pass =
        password::hash_pass(&body.password).map_err(|e| HttpError::bad_request(e.to_string()))?;

    let mut db_conn = app_state.db.get().map_err(|_| {
        HttpError::new(
            "error is getting db connection",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // initilizing user repo and giving db con so as to use its functions
    let mut user_repo = UserRepository::new(&mut db_conn);

    let saved_user = user_repo
        .save_user(
            &body.name,
            &body.email,
            hashed_pass,
            &verification_token,
            exp_duration,
        )
        .await;

    match saved_user {
        Ok(user) => {
            // sending otp verification rew to the user
            let ans = construct_mail(
                user.email.clone(),
                &[verification_token.to_string(), user.name.to_string()], //[otp , name_of_the_user]
                "otp_verification",
            )
            .await
            .map_err(|e| HttpError::new(e.message.to_string(), e.status))?;

            if !ans {
                HttpError::new(
                    "could not send the emial".to_string(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            }
            // and return response , we may return user id in the frontend , so that when req comes back , we have user_id to find user and verify the verification token
            // we are returning a tuple as return type
            // for tuple intoresponse is already implemeted
            // so basically a response struct is created and sent it to the frontent
            // response (status_code , content_type , body : userId(converted into json))
            // why we wrote json wrapper , to tell intoresponse that we need to convet this uuid into json format
            Ok((StatusCode::CREATED, Json(user.id)))
        }
        Err(e) => {
            return Err(HttpError::new(e.message.to_string(), e.status));
            // if error comes in saving the user , then it might be a db , user_already exist types of error , we will return that error only
        }
    }
}

/**
 * input => we will take userid and otp entered and app state(we will get db pool from this)
 * return type => intoresponse => (status , ( tokens))
 */
pub async fn verify_user(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(body): Json<VerifyEmailDTO>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
        .map_err(|e| HttpError::new(e.to_string(), StatusCode::BAD_REQUEST))?;

    let mut db_con = app_state
        .db
        .get()
        .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    let mut auth_repo = AuthRepository::new(&mut db_con);

    // string uuidd to uuid conversion
    let mut userId = Uuid::parse_str(&body.user_id)
        .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // finding user from the db
    let user = auth_repo.get_user(userId).await.map_err(|e| e)?;

    // extracting option<data time > to datatime
    let exp_time = &user.token_expires_at.ok_or_else(|| {
        HttpError::new(
            "error while extracting expiration time",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // if exp< current ( time khtmm ) = > then show error expire
    //  check for if otp not equal , show error
    // if exp > current (if time is there) => check for otp equal or not
    // first we will return error , then do for corret

    // if verification token has expired
    if exp_time < &Utc::now() {
        HttpError::new(
            "verification token has expred".to_string(),
            StatusCode::BAD_REQUEST,
        );
    }

    // getting token from the user object as string
    let saved_verification_token = &user.verification_token.ok_or_else(|| {
        HttpError::new(
            "error in user saved verification token",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // if token not euqal
    if saved_verification_token != &body.otp {
        HttpError::new(
            "verification token not equal".to_string(),
            StatusCode::BAD_REQUEST,
        );
    }

    // till here , exp is okay and otp are equal
    let auth_tokens = create_token(&body.user_id).map_err(|e| {
        HttpError::new(
            "error while gerating auth tokens",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // // we need to update the user verification status and save auth jwt token in db also

    Ok((StatusCode::CREATED, Json(auth_tokens)))
}

// now create route for this and then check the basic flow , if it is working or not , then make other controller and function
