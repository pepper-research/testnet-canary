use super::models::{NewSysSampleObject, SysSampleObject};
use crate::schema::sys_samples;
use crate::PerfSamples;
use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::{Connection, PgConnection, Queryable, RunQueryDsl, Selectable, SelectableHelper};
use std::env;

pub fn insert_sys_sample(
    connection: &mut PgConnection,
    perf_samples_obj: PerfSamples,
) -> SysSampleObject {
    let new_sys_sample_object = NewSysSampleObject {
        timestamp: chrono::Utc::now().naive_utc(),
        total_memory: BigDecimal::from(perf_samples_obj.total_memory.unwrap()),
        total_cpu: BigDecimal::from(perf_samples_obj.total_cpu.unwrap()),
        memory_usage: BigDecimal::from(perf_samples_obj.memory_usage.unwrap()),
        swap_usage: BigDecimal::from(perf_samples_obj.swap_usage.unwrap()),
        cpu_usage: BigDecimal::from((perf_samples_obj.cpu_usage.unwrap() * 100.0).round() as i32)
            / 100,
        process_cpu_usage: BigDecimal::from(
            (perf_samples_obj.process_cpu_usage.unwrap() * 100.0).round() as i32,
        ) / 100,
        process_memory_usage: BigDecimal::from(perf_samples_obj.process_memory_usage.unwrap()),
        network_down: BigDecimal::from(perf_samples_obj.network_down.unwrap()),
        network_up: BigDecimal::from(perf_samples_obj.network_up.unwrap()),
    };

    diesel::insert_into(sys_samples::table)
        .values(&new_sys_sample_object)
        .returning(SysSampleObject::as_returning())
        .get_result(connection)
        .expect("Error inserting sample")
}

#[cfg(test)]
mod tests {
    use crate::db::{
        models::{NewSysSampleObject, SysSampleObject},
        samples::insert_sys_sample,
        utils::establish_connection,
    };
    use crate::PerfSamples;

    #[test]
    fn test_insert_log() {
        let mut connection = establish_connection();
        let mut test_perf_samples_obj = PerfSamples::new();
        test_perf_samples_obj.total_memory = Some(100);
        test_perf_samples_obj.total_cpu = Some(200);
        test_perf_samples_obj.memory_usage = Some(300);
        test_perf_samples_obj.swap_usage = Some(400);
        test_perf_samples_obj.cpu_usage = Some(500 as f32);
        test_perf_samples_obj.process_cpu_usage = Some(600 as f32);
        test_perf_samples_obj.process_memory_usage = Some(700);
        test_perf_samples_obj.network_down = Some(800);
        test_perf_samples_obj.network_up = Some(900);

        let log = insert_sys_sample(&mut connection, test_perf_samples_obj);
        assert_eq!(log.total_memory.to_string(), "100");
        assert_eq!(log.total_cpu.to_string(), "200");
        assert_eq!(log.memory_usage.to_string(), "300");
        assert_eq!(log.swap_usage.to_string(), "400");
        assert_eq!(log.cpu_usage.to_string(), "500");
        assert_eq!(log.process_cpu_usage.to_string(), "600");
        assert_eq!(log.process_memory_usage.to_string(), "700");
        assert_eq!(log.network_down.to_string(), "800");
        assert_eq!(log.network_up.to_string(), "900");
    }
}
