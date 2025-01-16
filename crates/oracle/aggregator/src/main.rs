use prost::Message;
use spicenet_shared::oracle::NUMBER_OF_MARKETS;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::interval;
use tonic::{transport::Server, Code, Request, Response, Status};

pub mod oracle {
    tonic::include_proto!("oracle");
}

use oracle::oracle_aggregator_server::{OracleAggregator, OracleAggregatorServer};
use oracle::{DataRequest, DataResponse};

#[derive(Debug, Default)]
struct OracleAggregatorService {
    data: Arc<Mutex<HashMap<SocketAddr, Vec<u64>>>>,
}

#[tonic::async_trait]
impl OracleAggregator for OracleAggregatorService {
    async fn send_data(
        &self,
        request: Request<DataRequest>,
    ) -> Result<Response<DataResponse>, Status> {
        let node_address = request
            .remote_addr()
            .ok_or_else(|| Status::new(Code::Internal, "Remote address not available"))?;

        let data = request.into_inner().data;

        if data.len() == NUMBER_OF_MARKETS {
            let mut storage = self.data.lock().await;
            storage.insert(node_address, data);
            return Ok(Response::new(DataResponse { success: true }));
        }
        return Err(Status::new(
            Code::InvalidArgument,
            "Invalid number of markets. Update your node runner to the latest",
        ));
    }
}

async fn aggregate_data(data: Arc<Mutex<HashMap<SocketAddr, Vec<u64>>>>) {
    let mut interval = interval(Duration::from_millis(5));

    loop {
        interval.tick().await;

        let mut storage = data.lock().await;

        if storage.is_empty() {
            continue;
        }

        let market_data = build_market_data(&storage, NUMBER_OF_MARKETS);
        let (median_data, aggregate_conf_intervals) =
            compute_median_and_conf_intervals(&market_data);

        storage.clear();
    }
}

fn build_market_data(
    storage: &HashMap<SocketAddr, Vec<u64>>,
    number_of_markets: usize,
) -> Vec<Vec<u64>> {
    let values = storage.values().collect::<Vec<_>>();
    let mut iters: Vec<_> = values.into_iter().map(|n| n.into_iter()).collect();
    (0..number_of_markets)
        .map(|_| {
            iters
                .iter_mut()
                .map(|n| n.next().unwrap().to_owned())
                .collect::<Vec<u64>>()
        })
        .collect()
}

fn compute_median_and_conf_intervals(market_data: &[Vec<u64>]) -> (Vec<u64>, Vec<u64>) {
    let Some(no_of_nodes) = market_data.first().map(|market| market.len()) else {
        return (Vec::new(), Vec::new());
    };

    // Using inclusive quartile function (N - 1)
    let q1_index = (no_of_nodes + 1) as f64 * 0.25 - 1.0;
    let q3_index = (no_of_nodes + 1) as f64 * 0.75 - 1.0;

    let mut median_data = Vec::with_capacity(market_data.len());
    let mut aggregate_conf_intervals = Vec::with_capacity(market_data.len());

    for mut market_values in market_data.to_owned() {
        market_values.sort_unstable();

        let mid = no_of_nodes / 2;
        let median = if no_of_nodes % 2 == 0 {
            (market_values[mid - 1] + market_values[mid]) / 2
        } else {
            market_values[mid]
        };
        median_data.push(median);

        // Calculating Aggregate Confidence Interval
        let q1: u64 = interpolate(&market_values, q1_index);
        let q3: u64 = interpolate(&market_values, q3_index);

        // println!("q1_index: {} q1: {}, q3: {}", q1_index, q1, q3);
        // println!("median: {}", median);

        if median - q1 > q3 - median {
            aggregate_conf_intervals.push(median - q1);
        } else {
            aggregate_conf_intervals.push(q3 - median)
        }
    }

    (median_data, aggregate_conf_intervals)
}

fn interpolate(values: &[u64], index: f64) -> u64 {
    let floor = index.floor() as usize;
    let ceil = index.ceil() as usize;
    let fraction = index.fract();

    // Return the last value if the index is out of bounds (happens if
    if floor >= values.len() || ceil >= values.len() {
        return *values.last().unwrap();
    }

    if fraction == 0.0 {
        values[index as usize]
    } else {
        let lower = values[floor] as f64;
        let upper = values[ceil] as f64;
        (lower + (upper - lower) * fraction) as u64
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:9090".parse()?;
    let aggregator = OracleAggregatorService {
        data: Arc::new(Mutex::new(HashMap::new())),
    };

    let data_clone = aggregator.data.clone();
    tokio::spawn(async move {
        aggregate_data(data_clone).await;
    });

    Server::builder()
        .add_service(OracleAggregatorServer::new(aggregator))
        .serve(addr)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_median_and_conf_intervals_empty() {
        let market_data: Vec<Vec<u64>> = vec![];
        let (medians, conf_intervals) = compute_median_and_conf_intervals(&market_data);
        assert!(medians.is_empty());
        assert!(conf_intervals.is_empty());
    }

    #[test]
    fn test_compute_median_and_conf_intervals_single_value() {
        let market_data = vec![vec![100]];
        let (medians, conf_intervals) = compute_median_and_conf_intervals(&market_data);
        assert_eq!(medians, vec![100]);
        assert_eq!(conf_intervals, vec![0]);
    }

    #[test]
    fn test_compute_median_and_conf_intervals_multiple_values() {
        let market_data = vec![vec![100, 200, 300, 400, 500]];
        let (medians, conf_intervals) = compute_median_and_conf_intervals(&market_data);
        assert_eq!(medians, vec![300]);
        assert_eq!(conf_intervals, vec![150]);
    }

    #[test]
    fn test_compute_median_and_conf_intervals_empty_market() {
        let market_data = vec![];
        let (medians, conf_intervals) = compute_median_and_conf_intervals(&market_data);
        assert!(medians.is_empty());
        assert!(conf_intervals.is_empty());
    }

    #[test]
    fn test_compute_median_and_conf_intervals_multiple_markets() {
        let market_data = vec![vec![100, 200, 300], vec![50, 150, 250]];
        let (medians, conf_intervals) = compute_median_and_conf_intervals(&market_data);
        assert_eq!(medians, vec![200, 150]);
        assert_eq!(conf_intervals, vec![100, 100]);
    }

    #[test]
    fn test_compute_median_and_conf_intervals_even_nodes() {
        let market_data = vec![vec![100, 200, 300, 400]];
        let (medians, conf_intervals) = compute_median_and_conf_intervals(&market_data);
        assert_eq!(medians, vec![250]);
        assert_eq!(conf_intervals, vec![125]);
    }

    #[test]
    fn test_build_market_data() {
        let mut storage: HashMap<SocketAddr, Vec<u64>> = HashMap::new();
        storage.insert("127.0.0.1:8000".parse().unwrap(), vec![100, 150]);
        storage.insert("127.0.0.1:8080".parse().unwrap(), vec![200, 250]);
        let market_data = build_market_data(&storage, 2);
        assert_eq!(market_data, vec![vec![200, 100], vec![250, 150]]);
    }
}
