use serde::{Deserialize, Serialize};
use validator::Validate;
use crate::models::{UserRole, Users};


#[derive(Validate, Serialize, Clone , Deserialize)]
pub struct RegisterUser {
    #[validate(length(min = 1, message = "name is required"))]
    pub name: String,

    #[validate(email)] // it will use regex and check for email patter
    pub email: String,

    #[validate(length(min = 6, message = " passwrod length should be min 6 characters"))]
    pub password: String,

    #[validate(length(
        min = 6,
        message = "confirm passwrod min length should be 6 characters"
    ))]
    #[validate(must_match(
        other = "password",
        message = "confirm password should be equal to password"
    ))]
    #[serde(rename = "confirmPassword")]
    pub confirm_password: String,
}

