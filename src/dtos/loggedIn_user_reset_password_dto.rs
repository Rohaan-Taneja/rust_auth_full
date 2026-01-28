use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, Clone)]
pub struct LoggedInUserResetPasswordDTO {
    #[validate(length(min = 6, message = "old password min length should be greater that 6"))]
    pub old_password: String,

    #[validate(length(min = 6, message = "new password min length should be greater that 6"))]
    pub new_password: String,
}
