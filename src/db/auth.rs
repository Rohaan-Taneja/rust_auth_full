// we will add all the db related function related to user auth here


use diesel::prelude::*;


// manager which have db connection and have all the function impl for auth related things ,
//  sign up , signin etc
pub struct AuthRepository {
    db_con : PgConnection
}


// implementing all the auth functions
impl AuthRepository {

    pub fn save_user(){

    }
    
    pub fn get_user(){}
}