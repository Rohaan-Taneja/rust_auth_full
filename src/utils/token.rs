// creation and verification of user specific token

use std::env;

use axum::http::header;
use chrono::{Days, Duration, prelude::*};
use diesel::{deserialize, serialize};
use jsonwebtoken::{
    Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation, decode, encode,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}


// uuid of the user will be given and we crete a token out of it 
// xxx.xxx.xxx (header.payload.signature) signature containing hash of header , payload and secret
pub fn create_token(user_id: impl Into<String>) -> Result<String, jsonwebtoken::errors::Error> {
    // create dates
    // create claim struct
    // encode the details
    // return the claim result

    // converted to string
    let user = user_id.into();

    if user.is_empty() {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidSubject.into());
    };

    let now = Utc::now();
    let issue_date = now.timestamp() as usize;
    let exp_date = (now + Duration::days(90)).timestamp() as usize;

    let claim = Claims {
        sub: user,
        exp: exp_date,
        iat: issue_date,
    };

    // storing the string first;
    let secret = env::var("JWT_SECRET").unwrap();

    let token = encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(secret.as_bytes()),
    );

    return token;
}

pub fn decode_token(token: impl Into<String>) -> Result<String, jsonwebtoken::errors::Error> {
    // convert token to actual string
    // check if it is not empty

    let token = token.into();

    if token.is_empty() {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidToken.into());
    }

    let secret = env::var("JWT_SECRET").unwrap();

    // getting token result
    let decode = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    );

    // matching result and returning user_id or error
    match decode {
        Ok(token_data) => Ok(token_data.claims.sub),
        Err(e) => Err(e),
    }
}
