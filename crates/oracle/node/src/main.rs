use base64::prelude::*;
use clap::Arg;
use futures_util::{SinkExt, StreamExt};
use http::Request;
use oracle::oracle_aggregator_client::OracleAggregatorClient;
use oracle::DataRequest;
use rand::Rng;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub mod oracle {
    tonic::include_proto!("oracle");
}

const MARKETS: [&str; 1] = ["BTC/USD"];

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let source = "sidecar";

    let mut aggregator = {
        let mut res = OracleAggregatorClient::connect("http://[::1]:9090").await;
        let mut interval = interval(Duration::from_secs(1));
        while !res.is_ok() {
            interval.tick().await;
            println!("Connecting to aggregator failed! Retrying in 1s...");
            res = OracleAggregatorClient::connect("http://[::1]:9090").await;
        }
        res.unwrap()
    };

    match source {
        "sidecar" => run_sidecar(aggregator).await,
        "stork" => run_stork(aggregator).await,
        _ => Err("Invalid source. Use 'sidecar' or 'stork'.".into()),
    }
}

async fn run_sidecar(
    mut aggregator: OracleAggregatorClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn Error>> {
    let sidecar = Client::new();
    let sidecar_url = "http://localhost:8080/slinky/oracle/v1/prices";
    let mut interval = interval(Duration::from_millis(5));

    loop {
        interval.tick().await;

        let response = sidecar.get(sidecar_url).send().await?;
        let body = response.text().await?;
        let json: Value = serde_json::from_str(&body)?;

        let prices: Vec<u64> = MARKETS
            .iter()
            .map(|&asset| {
                json["prices"]
                    .get(asset)
                    .and_then(Value::as_str)
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(0)
            })
            .collect();

        let request = tonic::Request::new(DataRequest {
            data: prices.clone(),
        });

        match aggregator.send_data(request).await {
            Ok(_) => println!("Sent data: {:?}", prices),
            Err(e) => eprintln!("Error sending data: {}", e),
        }
    }
}

#[derive(Deserialize, Debug)]
struct StorkProxyPrices {
    prices: Vec<u64>,
}

async fn run_stork(
    mut aggregator: OracleAggregatorClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn Error>> {
    let url = "ws://localhost:8081"; // Connect to the local proxy
    let (ws_stream, response) = connect_async(url).await?;

    println!("WebSocket handshake response: {:?}", response);

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to BTC/USD
    // let subscribe_msg = serde_json::json!({
    //     "type": "subscribe",
    //     "data": ["BTCUSD"]
    // });
    // write.send(Message::Text(subscribe_msg.to_string())).await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        // println!("Received message: {:?}", msg);
        // match msg {
        //
        // }
        if let Message::Text(text) = msg {
            println!("Received message serialized: {}", text);
            match serde_json::from_str::<StorkProxyPrices>(&text) {
                Ok(prices) => {
                    let request = tonic::Request::new(DataRequest {
                        data: prices.prices.clone(),
                    });

                    match aggregator.send_data(request).await {
                        Ok(_) => println!("Sent data: {:?}", prices.prices),
                        Err(e) => eprintln!("Error sending data: {}", e),
                    }
                }
                Err(e) => eprintln!("Error deserializing message: {}", e),
            }
        }
    }

    Ok(())
}
