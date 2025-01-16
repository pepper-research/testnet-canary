use super::models::{NewSampleObject, SampleObject};
use crate::schema::samples;
use crate::Latency;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::{Connection, PgConnection, Queryable, RunQueryDsl, Selectable, SelectableHelper};
use std::env;

pub fn insert_sample(connection: &mut PgConnection, latency_object: Latency) -> SampleObject {
    let new_sample_object = NewSampleObject {
        timestamp: chrono::Utc::now().naive_utc(),
        nonce_latency: BigDecimal::from(latency_object.nonce_latency.unwrap()),
        publish_batch_latency: BigDecimal::from(latency_object.publish_batch_latency.unwrap()),
        ping_latency: BigDecimal::from(latency_object.ping_latency.unwrap()),
        confirmation_latency: BigDecimal::from(latency_object.confirmation_latency.unwrap()),
        e2e_latency: BigDecimal::from(latency_object.e2e_latency.unwrap()),
    };

    diesel::insert_into(samples::table)
        .values(&new_sample_object)
        .returning(SampleObject::as_returning())
        .get_result(connection)
        .expect("Error inserting sample")
}

#[cfg(test)]
mod tests {
    use crate::db::{
        models::{NewSampleObject, SampleObject},
        samples::insert_sample,
        utils::establish_connection,
    };
    use crate::Latency;

    #[test]
    fn test_insert_log() {
        let mut connection = establish_connection();
        let mut test_latency_obj = Latency::new();
        test_latency_obj.set_confirmation_latency(30000);
        test_latency_obj.set_nonce_latency(20000);
        test_latency_obj.set_ping_latency(10000);
        test_latency_obj.set_publish_batch_latency(40000);
        test_latency_obj.set_e2e_latency(50000);

        let log = insert_sample(&mut connection, test_latency_obj);
        assert_eq!(log.nonce_latency.to_string(), "20000");
        assert_eq!(log.publish_batch_latency.to_string(), "40000");
        assert_eq!(log.ping_latency.to_string(), "10000");
        assert_eq!(log.confirmation_latency.to_string(), "30000");
        assert_eq!(log.e2e_latency.to_string(), "50000");
    }
}
