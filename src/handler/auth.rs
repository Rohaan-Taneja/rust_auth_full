// we will write auth handler/controller code here

use std::sync::Arc;

use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse, routing::post};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;
use validator::Validate;

use crate::{
    AppState,
    db::{UserRepository, auth::AuthRepository},
    dtos::{
        login_dto::loggedInUser,
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
        .route("/login", post(login_user))
}

/**
 * register user handler and send otp to user email
 * inputs
 * user details in register user dto format details
 * shared app state (containg db connection) ,
 * data will be coming in json format , we need data in struct format ,
 * we will save new user in users table and otp details in user_email_verification table , to verify user email
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
    let otp = generate_otp(); // otp/verification token , will send on email
    let exp_duration = Utc::now() + Duration::minutes(5); // 5 minutes time to validate the otp 

    let hashed_pass =
        password::hash_pass(&body.password).map_err(|e| HttpError::bad_request(e.to_string()))?;

    // we will create a clone of arc referece of the
    let mut db_conn = app_state.db.clone();

    // initilizing user repo and giving db con so as to use its functions
    let mut user_repo = AuthRepository::new(db_conn);

    let saved_user = user_repo
        .save_new_user(&body.name, &body.email, hashed_pass)
        .await;

    match saved_user {
        Ok(user) => {
            // save user otp details in user_emai_verification table
            let res = user_repo
                .add_new_user_to_user_verification_table(
                    otp.to_string(),
                    user.email.to_string(),
                    exp_duration,
                )
                .await
                .map_err(|e| e);

            // sending otp verification rew to the user
            let ans = construct_mail(
                user.email.clone(),
                &[otp.to_string(), user.name.to_string()], //[otp , name_of_the_user]
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

    let mut db_con = app_state.db.clone();
    let mut auth_repo = AuthRepository::new(db_con);

    // string uuidd to uuid conversion
    let mut userId = Uuid::parse_str(&body.user_id)
        .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // finding user from the db for rhe user id came in with req body
    let user = auth_repo.get_user(userId).await.map_err(|e| e)?;

    // getting new_users user_email_verification status
    let user_verification_data = auth_repo
        .get_user_verification_status(user.email.clone())
        .await
        .map_err(|e| e)?;

    // if otp already used , then user is already verified , take to login state
    if user_verification_data.used {
        HttpError::bad_request("user already verified".to_string());
    }

    // extracting option<data time > to datatime
    let exp_time = &user_verification_data.expires_at.ok_or_else(|| {
        HttpError::new(
            "error while extracting expiration time",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // if exp< current ( time khtmm ) = > then show error expire
    //  check for if otp not equal , show error
    // if exp > current (if time is there) => check for otp equal or not
    // first we will return error , then do for corret

    // if otp has expired
    if exp_time < &Utc::now() {
        HttpError::new("otp has expred".to_string(), StatusCode::BAD_REQUEST);
    }

    // getting token from the user object as string
    let saved_otp = &user_verification_data.otp;

    // if otp not euqal , show error
    if saved_otp != &body.otp {
        HttpError::new("otp not equal".to_string(), StatusCode::BAD_REQUEST);
    }

    // till here , exp is okay and otp are equal
    // we will create auth_token/jwt token for the user verification
    let auth_tokens = create_token(&body.user_id).map_err(|e| {
        HttpError::new(
            "error while generating auth tokens",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // token exp time will be 24 hours
    let exp_time = Utc::now() + Duration::hours(24);

    // updating otp used status to true
    auth_repo
        .update_new_user_otp_status(&user.email)
        .await
        .map_err(|e| e)?;

    auth_repo
        .update_jwt_token_to_user(&user.email, &auth_tokens, exp_time)
        .await
        .map_err(|e| e)?;

    // // we need to update the user verification status to used and save auth jwt token in db also

    Ok((StatusCode::CREATED, Json(auth_tokens)))
}


/**
 * @inputs => we will get app state and login data as input
 * 
 * @result => we will login user and return auth token and basic  user details to the user
 */
pub async fn login_user(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(login_info): Json<loggedInUser>,
) -> Result< impl IntoResponse, HttpError> {
    // we will get user name and password
    // we will send to the db to check if it is okay or not ?
    // if no , we will send unauthorized request
    // if yes , we will call utils/token function with user id to create token 
    // we will then cal update token db function to save tokens and exp
    // and then response okay and send tokens
    
    Ok("hello".to_string())
}
