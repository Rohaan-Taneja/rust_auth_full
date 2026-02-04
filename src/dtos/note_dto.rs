use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Validate, Serialize, Deserialize, Clone)]
pub struct NoteDTO {
    #[validate(length(min = 1, message = "title of a note cannot be empty"))]
    pub title: String,

    #[validate(length(min = 1, message = "content of a note cannot be empty"))]
    pub content: String,
}
