use serde::{Deserialize, Serialize};
use validator::Validate;


#[derive(Validate , Serialize , Deserialize , Clone)]
pub struct SendOtpDTO{

    #[validate(email)]
    pub user_email : String
}