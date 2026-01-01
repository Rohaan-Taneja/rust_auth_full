// we will add queries/struct/function to inteact with the user tables
// which can be called via handlers , so it is same as .servide file

use diesel::prelude::*;

// a manager which holds db connection
pub struct UserRepository<'a> {
    pub db_con: &'a mut PgConnection,
}

// implementing user related functions
// impl UserRepository<'a> {

//     // function to give db connection access to this repository/manager 
//     pub fn new(& mut db_connection: PgConnection) -> UserRepository {
        
//         UserRepository {
//             db_con: db_connection,
//         }
//     }
// }
