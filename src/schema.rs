// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "user_type"))]
    pub struct UserType;
}

diesel::table! {
    user_email_verifications (id) {
        id -> Uuid,
        #[max_length = 255]
        user_email -> Varchar,
        #[max_length = 6]
        otp -> Varchar,
        expires_at -> Nullable<Timestamptz>,
        used -> Bool,
    }
}

diesel::table! {
    user_reset_pass_validations (id) {
        id -> Uuid,
        #[max_length = 255]
        user_email -> Varchar,
        #[max_length = 100]
        hashed_reset_token -> Varchar,
        expires_at -> Nullable<Timestamptz>,
        used -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::UserType;

    users (id) {
        id -> Uuid,
        #[max_length = 100]
        name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        verified -> Bool,
        #[max_length = 100]
        password -> Varchar,
        #[max_length = 255]
        verification_token -> Nullable<Varchar>,
        token_expires_at -> Nullable<Timestamptz>,
        role -> UserType,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    user_email_verifications,
    user_reset_pass_validations,
    users,
);
