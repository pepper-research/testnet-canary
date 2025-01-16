mod db;
mod schema;

use crate::db::samples::insert_sys_sample;
use crate::db::utils::establish_connection;
use bigdecimal::ToPrimitive;
use chrono::Utc;
use clap::Parser;
use cron::Schedule;
use diesel::serialize::IsNull::No;
use diesel::PgConnection;
use lazy_static::lazy_static;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use sysinfo::{
    get_current_pid, CpuRefreshKind, Networks, Pid, ProcessRefreshKind, ProcessesToUpdate,
    RefreshKind, System,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to the rollup config.
    #[arg(long)]
    node_pid: String,
}

#[derive(Debug)]
pub struct PerfSamples {
    pub total_memory: Option<u64>,
    pub total_cpu: Option<u64>,
    pub memory_usage: Option<u64>,
    pub swap_usage: Option<u64>,
    pub cpu_usage: Option<f32>,
    pub process_cpu_usage: Option<f32>,
    pub process_memory_usage: Option<u64>,
    pub network_down: Option<u64>,
    pub network_up: Option<u64>,
}

impl PerfSamples {
    pub fn new() -> Self {
        PerfSamples {
            total_memory: None,
            total_cpu: None,
            memory_usage: None,
            swap_usage: None,
            cpu_usage: None,
            process_cpu_usage: None,
            process_memory_usage: None,
            network_down: None,
            network_up: None,
        }
    }

    pub fn verify(&self) -> bool {
        self.total_memory.is_some()
            && self.total_cpu.is_some()
            && self.memory_usage.is_some()
            && self.swap_usage.is_some()
            && self.cpu_usage.is_some()
            && self.process_cpu_usage.is_some()
            && self.process_memory_usage.is_some()
            && self.network_down.is_some()
            && self.network_up.is_some()
    }
}

#[tokio::main]
async fn main() {
    let expression = "0 */1 * * * * *";
    let schedule = Schedule::from_str(expression).unwrap();

    let args = Args::parse();

    let NODE_PID = Pid::from_str(args.node_pid.as_str()).unwrap();

    let mut connection = establish_connection();

    while let Some(time) = schedule.upcoming(Utc).next() {
        let mut system =
            System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::everything()));
        let mut networks = Networks::new_with_refreshed_list();

        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);

        networks.refresh();

        let now = chrono::Utc::now();
        if time > now {
            let duration = time - now;
            tokio::time::sleep(tokio::time::Duration::from_secs(
                duration.num_seconds() as u64
            ))
            .await;
        };

        println!("Running Sampler");
        let perf_samples_obj = measure_perf(NODE_PID, system, networks);

        if perf_samples_obj.verify() {
            println!("Object verified");
            insert_sys_sample(&mut connection, perf_samples_obj);
        } else {
            println!("failed");
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

pub fn measure_perf(NODE_PID: Pid, mut system: System, mut networks: Networks) -> PerfSamples {
    let mut perf_samples_obj = PerfSamples::new();

    system.refresh_all();
    perf_samples_obj.total_memory = Some(system.total_memory());
    perf_samples_obj.total_cpu = Some(system.cpus().iter().len().to_u64().unwrap());
    perf_samples_obj.memory_usage = Some(system.used_memory());
    let mut total_cpu_usage = 0.0;
    for cpu in system.cpus() {
        total_cpu_usage += cpu.cpu_usage();
    }
    perf_samples_obj.cpu_usage =
        Some(total_cpu_usage / system.cpus().iter().len().to_f32().unwrap());
    perf_samples_obj.swap_usage = Some(system.used_swap());

    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        sysinfo::ProcessRefreshKind::new().with_cpu(),
    );
    let process = system.process(NODE_PID).unwrap();
    perf_samples_obj.process_cpu_usage = Some(process.cpu_usage() / 8.0);
    perf_samples_obj.process_memory_usage = Some(process.memory());

    let mut total_network_up = 0;
    let mut total_network_down = 0;
    for network in networks.iter() {
        total_network_up += network.1.received();
        total_network_down += network.1.transmitted();
    }

    perf_samples_obj.network_up = Some(total_network_up);
    perf_samples_obj.network_down = Some(total_network_down);

    perf_samples_obj
}
