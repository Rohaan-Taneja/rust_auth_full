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
        non_logged_in_user_reset_password_dto::NonLoggedInUserResetPasswordDTO,
        register_dto::{self, RegisterUser},
        send_otp::SendOtpDTO,
        verify_email_dto::{self, VerifyEmailDTO},
    },
    errors::HttpError,
    mail::{
        mail::{
            EmailType::{NewUserEmailVerification, ResetPasswordEmailVerification},
            construct_mail,
        },
        sendMail::{self, send_mail},
    },
    utils::{
        self,
        password::{self, generate_otp, hash_pass, validate_pas},
        token::create_token,
    },
};

pub fn auth_handler() -> Router {
    Router::new()
        .route("/register", post(register_user))
        .route("/verify-email", post(verify_user))
        .route("/login", post(login_user))
        .nest("/reset-password", reset_pass_handler())
}

// api routes for reset-pass for non logged-in user
pub fn reset_pass_handler() -> Router {
    Router::new()
        .route("/send-otp", post(send_otp))//to send otp and register user_reset_password_email_verifications
        .route("/verify-otp", post(verify_forget_pass_emails_otp)) //to verify otp and save reset_token in user_reset_pass_validations
        .route("/save-new-password", post(save_new_pass)) // to verify reset-token and save new pass and send new jwt tokens to user
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
    let exp_duration = Utc::now() + Duration::minutes(5); // 5 minutes exp time to validate the otp 

    let hashed_pass =
        password::hash_pass(&body.password).map_err(|e| HttpError::bad_request(e.to_string()))?;

    println!("time now beofre db pool clone {:?}", Utc::now());
    // we will create a clone of referec
    let mut db_conn = app_state.db.clone();

    println!("time now after db pool clone {:?}", Utc::now());

    // initilizing user repo and giving db con so as to use its functions
    let mut user_repo = AuthRepository::new(db_conn);

    println!("time now after creating repo {:?}", Utc::now());

    let saved_user = user_repo
        .save_new_user(&body.name, &body.email, hashed_pass)
        .await;

    println!("time now after saving user {:?}", Utc::now());

    match saved_user {
        Ok(user) => {
            // save user otp details in user_emai_verification table
            user_repo
                .add_new_user_to_user_verification_table(
                    otp.to_string(),
                    user.email.to_string(),
                    exp_duration,
                )
                .await
                .map_err(|e| e)?;

            println!("time now after saving otp details {:?}", Utc::now());

            // sending otp verification req to the user
            // if we could not able to send email , error will show up
            construct_mail(
                user.email.clone(),
                &[otp.to_string(), user.name.to_string()], //[otp , name_of_the_user]
                NewUserEmailVerification.clone(),
            )
            .await
            .map_err(|e| HttpError::new(e.message.to_string(), e.status))?;

            println!("time now after sending email req {:?}", Utc::now());

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
    let user_id = body.user_id.ok_or_else(|| {
        HttpError::new(
            "cannot extract user id ".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // string uuidd to uuid conversion
    let mut userId = Uuid::parse_str(&user_id)
        .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    // finding user from the db for rhe user id came in with req body
    let user = auth_repo.get_user(userId).await.map_err(|e| e)?;

    // if verified user tris to verify again , we will return this okay status
    if user.verified {
        return Ok((
            StatusCode::ACCEPTED,
            Json("user already verified".to_string()),
        ));
    }

    // getting new_users user_email_verification status
    let user_verification_data = auth_repo
        .get_user_verification_status(user.email.clone())
        .await
        .map_err(|e| e)?;

    // if otp already used , then user is already verified , take to login state
    if user_verification_data.used {
        return Err(HttpError::bad_request("user already verified".to_string()));
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
        return Err(HttpError::new(
            "otp has expred".to_string(),
            StatusCode::BAD_REQUEST,
        ));
    }

    // getting token from the user object as string
    let saved_otp = &user_verification_data.otp;

    // if otp not euqal , show error
    if saved_otp != &body.otp {
        return Err(HttpError::new(
            "otp not equal".to_string(),
            StatusCode::BAD_REQUEST,
        ));
    }

    // till here , exp is okay and otp are equal
    // we will create auth_token/jwt token for the user verification
    let auth_tokens = create_token(&user_id).map_err(|e| {
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
 * @result => we will login user and return auth token and basic  user details to the user
 */
pub async fn login_user(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(login_info): Json<loggedInUser>,
) -> Result<impl IntoResponse, HttpError> {
    // we will get user name and password
    // we will send to the db to check if it is okay or not ?
    // if no , we will send unauthorized request
    // if yes , we will call utils/token function with user id to create token
    // we will then cal update token db function to save tokens and exp
    // and then response okay and send tokens

    // validate incoming data
    login_info.validate().map_err(|e| {
        HttpError::bad_request("data not according to the login dto format".to_string())
    })?;

    // trasnferring ownershit to our vars
    let user_email = login_info.email.to_string();
    let user_pass = login_info.password.to_string();

    // cloned the app state arc pointer
    let db_pool = app_state.db.clone();

    // created auth repo instance , so that we call its db function
    let mut auth_repo = AuthRepository::new(db_pool);

    // check for user authenticity and verify else will show unauthorized error
    let logged_in_user = auth_repo
        .verify_login_user(&user_email, &user_pass)
        .await
        .map_err(|e| e)?;

    // creating auth tokens for the user
    let auth_token = create_token(&logged_in_user).map_err(|e| {
        HttpError::new(
            "error while creating used auth tokens",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let token_exp = Utc::now() + Duration::hours(24);

    // storing the auth tokens in the users table
    auth_repo
        .update_jwt_token_to_user(&user_email, &auth_token, token_exp)
        .await
        .map_err(|e| e)?;

    Ok((StatusCode::CREATED, auth_token))
}

/**
 * In this function we will send otp to the incoming user email
 * @input => we will get user email as input
 * @ result => we will create , save otp in the user_verification table and sent it to the user email and return bool after sending
 */
pub async fn send_otp(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(user_email): Json<SendOtpDTO>,
) -> Result<impl IntoResponse, HttpError> {
    // we will get user email

    let email = user_email.user_email;

    // 6 digit random otp
    let otp = generate_otp();
    let otp_exp = Utc::now() + Duration::minutes(5); //5 min of exp time for otp

    // clone of db connection pointer
    let db_con = app_state.db.clone();

    let mut auth_repo = AuthRepository::new(db_con);

    // add user email and otp to table to verify on the next step
    auth_repo
        .add_otp_details_to_user_reset_password_verification_table(
            otp.to_string(),
            email.to_string(),
            otp_exp,
        )
        .await
        .map_err(|e| e)?;

    // sending otp email to the user
    construct_mail(
        email.to_string(),
        &[otp.to_string()],
        ResetPasswordEmailVerification.clone(),
    )
    .await
    .map_err(|e| HttpError::new(e.message.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok("otp sent to the email".to_string())
}

/**
 * this function is to verify the otp of the user/email(not loggedIn user/no_auth_token user) ,(stading on the login page) who has forget his pass and now wants to reset his password
 * we will verify his email via sent otp , here
 * @inputs => we will get otp and email
 * @result => we will verify his otp and return true or false
 */
pub async fn verify_forget_pass_emails_otp(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(verify_data): Json<VerifyEmailDTO>,
) -> Result<impl IntoResponse, HttpError> {
    let email = verify_data.user_email.ok_or_else(|| {
        HttpError::new(
            "cannot extract email from the inputs".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    let to_be_verified_otp = verify_data.otp;

    let db_con = app_state.db.clone();

    let mut auth_repo = AuthRepository::new(db_con);

    let user_otp_details = auth_repo
        .get_user_reset_password_email_verification_status(&email)
        .await
        .map_err(|e| e)?;

    if user_otp_details.used {
        return Err(HttpError::bad_request("otp already used".to_string()));
    }

    let otp_exp = user_otp_details.expires_at.ok_or_else(|| {
        HttpError::new(
            "cannot extract expiry details from the details ",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    if &otp_exp < &Utc::now() {
        return Err(HttpError::bad_request("otp expired".to_string()));
    }

    let saved_otp = user_otp_details.otp;

    if to_be_verified_otp != saved_otp {
        return Err(HttpError::bad_request("otp not eqaul".to_string()));
    }

    // if we reached here , otp is eqaul , now we will create reset token and save it and return it to the user
    let reset_token = uuid::Uuid::new_v4();
    let exp_at = Utc::now() + Duration::minutes(5);

    let hashed_reset_token = hash_pass(reset_token.clone()).map_err(|e| {
        HttpError::new(
            "getting error in hshing pass",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    // saving reset token details in the user_reset_pass_validations table
    auth_repo
        .save_reset_token_details(email.clone(), hashed_reset_token, exp_at)
        .await
        .map_err(|e| e)?;

    // returning reset tokens to the user for the next step validation
    Ok((StatusCode::CREATED, Json(reset_token)))
}

/**
 * this function is to update the pass of the user/email(non loggedIn user/no_auth_token user) , who has verified his email via otp
 * we will update the pass of the email
 * @input => email  , reset_pass_token and new_pass
 * @result => we will return new auth tokens to the user
 */
pub async fn save_new_pass(
    Extension(app_state): Extension<Arc<AppState>>,
    Json(new_pass_data): Json<NonLoggedInUserResetPasswordDTO>,
) -> Result<impl IntoResponse, HttpError> {
    let email = new_pass_data.user_email;

    let unhashed_reset_token = new_pass_data.reset_token;
    let new_password = new_pass_data.new_password;

    let db_con = app_state.db.clone();

    let mut auth_repo = AuthRepository::new(db_con);

    let reset_pass_data = auth_repo
        .get_user_reset_password_validations_details(&email)
        .await
        .map_err(|e| e)?;

    if reset_pass_data.used {
        return Err(HttpError::bad_request(
            "password already updated".to_string(),
        ));
    }

    let otp_exp = reset_pass_data.expires_at.ok_or_else(|| {
        HttpError::new(
            "cannot extract expiry details from the details ",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;

    if &otp_exp < &Utc::now() {
        return Err(HttpError::bad_request("otp expired".to_string()));
    }

    let verify_reset_token =
        validate_pas(&unhashed_reset_token, &reset_pass_data.hashed_reset_token).map_err(|e| {
            HttpError::new(
                "not able to validate reset token".to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        })?;

    let user_details = auth_repo
        .get_user_from_email(&reset_pass_data.user_email)
        .await
        .map_err(|e| e)?;

    let jwt_token = create_token(user_details.id.to_string()).map_err(|e| {
        HttpError::new(
            "error while generating auth tokens",
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    })?;
    let expiry = Utc::now() + Duration::hours(24);

    let ans = auth_repo
        .update_user_password_and_jwt_token_and_exp(&email, new_password, jwt_token.clone(), expiry)
        .await
        .map_err(|e| e)?;

    Ok((StatusCode::ACCEPTED, Json(jwt_token)))
}
