#[cfg(feature = "offchain")]
use {
    diesel::{pg::PgConnection, prelude::*},
    dotenvy::dotenv,
    std::env,
};

#[cfg(feature = "offchain")]
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect("Error connecting to database")
}
