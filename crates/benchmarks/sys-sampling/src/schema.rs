// @generated automatically by Diesel CLI.

diesel::table! {
    sys_samples (id) {
        id -> Int4,
        timestamp -> Timestamp,
        total_memory -> Numeric,
        total_cpu -> Numeric,
        memory_usage -> Numeric,
        swap_usage -> Numeric,
        cpu_usage -> Numeric,
        process_cpu_usage -> Numeric,
        process_memory_usage -> Numeric,
        network_down -> Numeric,
        network_up -> Numeric,
    }
}
