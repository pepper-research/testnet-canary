#[cfg(feature = "offchain")]
use {
    super::models::{Log, LogLevel, LogScope, NewLog},
    crate::{db::utils::establish_connection, schema::log},
    diesel::{RunQueryDsl, SelectableHelper},
};

#[cfg(feature = "offchain")]
pub fn insert_log(level: LogLevel, scope: LogScope, message: &str) -> Log {
    let mut connection = establish_connection();

    let new_log: NewLog = NewLog {
        timestamp: chrono::Utc::now().naive_utc(),
        level,
        scope,
        message: message.to_string(),
    };

    diesel::insert_into(log::table)
        .values(&new_log)
        .returning(Log::as_returning())
        .get_result(&mut connection)
        .expect("Error inserting log")
}

#[cfg(feature = "offchain")]
#[cfg(test)]
mod tests {
    use crate::db::{
        logs::insert_log,
        models::{LogLevel, LogScope},
        utils::establish_connection,
    };

    #[test]
    fn test_insert_log() {
        let log = insert_log(LogLevel::Info, LogScope::None, "Test log");
        assert_eq!(log.level, LogLevel::Info);
        assert_eq!(log.scope, LogScope::None);
        assert_eq!(log.message, "Test log");
    }
}
