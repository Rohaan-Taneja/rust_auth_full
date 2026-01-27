// we will write general function for mails

use serde::{Deserialize, Serialize};

use crate::{errors::HttpError, mail::sendMail::send_mail};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailData {
    content: String,
    subject: String,
}

#[derive(Clone)]
pub enum EmailType {
    NewUserEmailVerification,
    ResetPasswordEmailVerification,
}

/**
 * we will get to , variables : [ "" , ""] and email type as input
 * we will get the content_template and subject from the render_email function and and it will construct the email content and subject and
 * then call send_email with all the details
 */
pub async fn construct_mail(
    to: impl Into<String>,
    variables: &[String],
    email_type: EmailType,
) -> Result<bool, HttpError> {
    let reciever = to.into();

    // getting the email data(subject and content)
    let email_data = render_email(email_type.clone(), variables).map_err(|e| e)?;

    // this will send email to the user(reciever) and return true or errror
    let result = send_mail(
        &reciever,
        email_data.content.to_string(),
        email_data.subject.to_string(),
    )
    .await
    .map_err(|e| e)?;

    Ok(result)
}

/**
 * we will render the content , on the basis of the email type
 */
fn render_email(email_type: EmailType, vars: &[String]) -> Result<EmailData, HttpError> {
    match email_type {
        EmailType::NewUserEmailVerification => {
            let data = EmailData {
                content: format! {"Hi {},\n\nWelcome to RustAuth! We are glad to have you. \n\nYour OTP for email verification is {}" , vars.get(1).ok_or_else(|| HttpError::bad_request("user name missing".to_string()))? , vars.get(0).ok_or_else(|| HttpError::bad_request("otp is missing"))?},
                subject: "Welcome to RustAuth 🎉".to_string(),
            };

            Ok(data)
        }
        EmailType::ResetPasswordEmailVerification => {
            let data = EmailData {
                content: format!(
                    "Your password reset token is {}",
                    vars.get(0)
                        .ok_or_else(|| HttpError::bad_request("otp missing"))?
                ),
                subject: "Reset your password".to_string(),
            };

            Ok(data)
        }
    }
}
