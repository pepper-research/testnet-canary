use bigdecimal::BigDecimal;
use chrono::NaiveDateTime;
use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::sys_samples)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct SysSampleObject {
    pub id: i32,
    pub timestamp: NaiveDateTime,
    pub total_memory: BigDecimal,
    pub total_cpu: BigDecimal,
    pub memory_usage: BigDecimal,
    pub swap_usage: BigDecimal,
    pub cpu_usage: BigDecimal,
    pub process_cpu_usage: BigDecimal,
    pub process_memory_usage: BigDecimal,
    pub network_down: BigDecimal,
    pub network_up: BigDecimal,
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::sys_samples)]
pub struct NewSysSampleObject {
    pub timestamp: NaiveDateTime,
    pub total_memory: BigDecimal,
    pub total_cpu: BigDecimal,
    pub memory_usage: BigDecimal,
    pub swap_usage: BigDecimal,
    pub cpu_usage: BigDecimal,
    pub process_cpu_usage: BigDecimal,
    pub process_memory_usage: BigDecimal,
    pub network_down: BigDecimal,
    pub network_up: BigDecimal,
}
