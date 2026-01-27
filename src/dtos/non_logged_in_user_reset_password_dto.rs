use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate , Serialize  , Deserialize , Clone)]
pub struct NonLoggedInUserResetPasswordDTO{

    #[validate(email)]
    pub user_email : String,


    pub reset_token : String,

    #[validate(length(min = 6 , message = "password length should be greater than 6"))]
    pub new_password : String

}