use futures::StreamExt;
use reqwest::Response;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;
use std::error::Error;
use std::time::Duration;
use tmev_protos::tmev_proto::bundle_service_client::BundleServiceClient;
use tmev_protos::tmev_proto::Bundle;
use tmev_protos::tmev_proto::SubscribeBundlesRequest;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::sleep;

pub async fn run_bundle_request_loop(mut tx: UnboundedSender<Vec<Bundle>>) {
    loop {
        let mut client = BundleServiceClient::connect("http://0.0.0.0:5005")
            .await
            .unwrap();
        let mut stream = client
            .subscribe_bundles(SubscribeBundlesRequest {
                searcher_key: "test".to_string(),
            })
            .await
            .unwrap()
            .into_inner();

        // stream is infinite - take just 5 elements and then disconnect
        // let mut stream = stream.t(num);
        while let Some(item) = stream.next().await {
            match item {
                Ok(bundles_response) => {
                    println!("Received bundle: {:?}", bundles_response.bundles);
                    let sent = tx.send(bundles_response.bundles);
                    if let Ok(sent) = sent {
                        println!("sent bundle tx to thread");
                    }
                }
                Err(status) => {
                    println!("err {:?}", status);
                }
            }
        }
        sleep(Duration::from_millis(400)).await;
    }
}
