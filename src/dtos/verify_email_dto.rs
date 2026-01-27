use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize , Deserialize , Validate)]
pub struct VerifyEmailDTO{

    // it may be of 6 char , we have to see and change it whle tetsing
    #[validate(length(min=1 , message = "otp length should be greater that 1"))]
    pub otp : String ,
    
    #[validate(length(min= 1 , message = "user id length  should be greater than 1"))]
    pub user_id : Option<String>,

    #[validate(email)]
    pub user_email : Option<String>

}