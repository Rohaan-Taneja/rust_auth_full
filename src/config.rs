// here we are writig our confiration things

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_maxage: i64,
    pub port: i16,
}

impl Config {
    pub fn init() -> Config {
        // reading and storing values from env
        // constructing cnfig struct and returning

        let database_url = env::var("DATABASE_URL").expect("database url must be set");
        let jwt_secret = env::var("JWT_SECRET").expect("jwt secret must be set");
        let jwt_maxage = env::var("JWT_MAXAGE").expect("max age must be set");

        return Config {
            database_url: database_url,
            jwt_secret: jwt_secret,
            jwt_maxage: jwt_maxage.parse::<i64>().unwrap(),
            port: 8080,
        };
    }
}
