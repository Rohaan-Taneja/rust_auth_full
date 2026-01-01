// hashing and veryfying of passwords

use core::error;

use argon2::{
    Argon2,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::{Error, OsRng},
    },
};

use crate::errors::ErrorMessage;

// maxxing the size of the password
const MAX_PASSWORD_LENGTH: usize = 64;

// hashing the function

pub fn hash_pass(pass: impl Into<String>) -> Result<String, ErrorMessage> {
    let pass = pass.into();
    let salt = SaltString::generate(&mut OsRng);

    if pass.is_empty() {
        return Err(ErrorMessage::EmptyPassword);
    }
    if pass.len() > MAX_PASSWORD_LENGTH {
        return Err(ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH));
    }

    let hashed_password = Argon2::default()
        .hash_password(pass.as_bytes(), &salt)
        .map_err(|_| ErrorMessage::HashingError)?
        .to_string();

    return Ok(hashed_password);
}

// validate password
/**
 * user will give up password that he entered
 * and the hashed pass from the db
 * and hence we will verify it with our db stored hashed pass
 */
pub fn validate_pas(
    pass: impl Into<String>,
    hashed_user_pass: impl Into<String>,
) -> Result<bool, ErrorMessage> {
    // converting into actual Strings
    let pass = pass.into();
    let hashed_user_pass = hashed_user_pass.into();

    if pass.is_empty() {
        return Err(ErrorMessage::EmptyPassword);
    }

    if pass.len() > MAX_PASSWORD_LENGTH {
        return Err(ErrorMessage::ExceededMaxPasswordLength(MAX_PASSWORD_LENGTH));
    }

    let parsed_hash =
        PasswordHash::new(&hashed_user_pass).map_err(|_| ErrorMessage::InvalidHashFormat)?;

    let res = Argon2::default()
        .verify_password(pass.as_bytes(), &parsed_hash)
        .map_or( false, |_|true); //when we want to do something on error(false) and on correct value (true value) 

    return Ok(res);
    
}
