use std::env;

use axum::http::StatusCode;
use chrono::Utc;
use http_serde::option::status_code;
use lettre::{
    Message, SmtpTransport, Transport,
    message::{Mailbox, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use tokio::time::error;

use crate::errors::HttpError;

/**
 * it will take to and content in the input ,
 * it will be a universal email sending function
 */
pub async fn send_mail(
    to: impl Into<String>,
    content: impl Into<String>,
    subject: impl Into<String>,
) -> Result<bool, HttpError> {
    // converted to string/owned string
    let reciever = to.into();
    let email_content = content.into();
    let email_subject = subject.into();

    let from = env::var("SMTP_USERNAME").unwrap();

    let smtp_server = env::var("SMTP_SERVER")
        .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;

    let pass = env::var("SMTP_PASS").unwrap();

    // constructing email to send
    let email = Message::builder()
        .from(Mailbox::new(
            Some("rust_auth_system".to_owned()),
            "tanejarohan60@gmail.com".clone().parse().unwrap(),
        ))
        .to(Mailbox::new(
            Some("user".to_owned()),
            "tanejarohan6@gmail.com".to_string().parse().unwrap(),
        ))
        .subject(email_subject.clone())
        .header(ContentType::TEXT_PLAIN)
        .body(email_content.clone())
        .unwrap();

    let creds = Credentials::new(from.clone(), pass.clone());

    // opening a remote server to send email
    let mailer = SmtpTransport::relay(&smtp_server)
        .unwrap()
        .credentials(creds)
        .build();

    // this sending functionality will block main thread , so we will put it in a waiting tread , when it will be completed then we will put it back
    // letter crate is not by default asyn await
    // so we will put this blocking req in a async /await enviroment
    // and when it will done , we will be notified and we will trasnfer
    println!("time now before sending email in function only {:?}" , Utc::now());
    let result = tokio::task::spawn_blocking(move || mailer.send(&email))
        .await
        .map_err(|e| HttpError::new(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?; // sending the email to the user
    println!("time now after sending the mail in function only {:?}" , Utc::now());
    match result {
        Ok(_) => {
            println!("we have sent the mail ");
            Ok(true)
        }
        Err(e) => {
            return Err(HttpError::new(
                e.to_string(),
                StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    }
}
