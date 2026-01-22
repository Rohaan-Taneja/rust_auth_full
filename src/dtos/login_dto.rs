use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, Clone)]
pub struct loggedInUser {
    #[validate(email)]
    pub email: String,

    #[validate(length(min = 6, message = "passowrd length should be min 6 chars"))]
    pub password: String,
}
