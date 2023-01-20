#![feature(async_closure)]
use std::error::Error;
// use std::sync::Arc;
// use std::thread;
// use std::time::Duration;

use crate::arb_feed::ArbFeedResponse;
use arb_feed::QueryData;
use clap::arg;
// use clap::command;
// use clap::ArgMatches;
use clap::Command;
// use clap::Parser;
use spinners::{Spinner, Spinners};
mod arb_feed;
mod arb_table;
mod bundle_sub;
// mod searcher_grpc;

use arb_table::*;
use futures::stream::iter;
use tmev_protos::SubscribeBundlesRequest;
// use tmev_protos::bundle_service_client;
// use tmev_protos::SubscribeBundlesRequest;
use tmev_protos::bundle_service_client::BundleServiceClient;
use tokio::sync::Mutex;
use tokio::time::delay_for;
#[tokio::main]
async fn main() {
    let cmd = Command::new("tmev").args(&[
        arg!(--address <ADDRESS> "An address to filter transactions by"),
        arg!(--arbs "View a table of the recent arbs in order of most profitable"),
        arg!(--bundles "View a table of the recent bundles"),
    ]);

    let matches = cmd.get_matches();
    println!("matches: {:?}", matches);
    for arg in matches.ids().into_iter() {
        println!("matches: {:?}", arg);
        let a = arg.as_str();
        match a {
            "address" => {
                println!("{}", a);
                break;
            }
            "arbs" => {
                // println!("{}", a);

                let mut sp = Spinner::new(Spinners::Dots8Bit, " loading arbs 🥩".into());
                let feed = arb_feed::get_arb_feed().await;
                let parsed = serde_json::from_str::<ArbFeedResponse>(
                    feed.unwrap().text().await.unwrap().as_str(),
                )
                .unwrap()
                .query_data;

                let profit_amts = parsed
                    .profit_amount
                    .iter()
                    .map(|a| a.to_string())
                    .collect::<Vec<String>>();
                let prices_usd = parsed
                    .price_usd
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>();
                let profits_usd = parsed
                    .price_usd
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<String>>();
                let QueryData {
                    block_time,
                    slot_id,
                    transaction_hash,
                    profit_amount,
                    currency,
                    signers,
                    price_usd,
                    profit_usd,
                } = parsed;
                let mut row_vec = Vec::new();

                for index in 0..transaction_hash.len() {
                    let row: Vec<String> = vec![
                        block_time[index].clone(),
                        slot_id[index].clone(),
                        transaction_hash[index].clone(),
                        profit_amts[index].clone(),
                        currency[index].clone(),
                        signers[index].clone(),
                        prices_usd[index].clone(),
                        profits_usd[index].clone(),
                    ];
                    row_vec.push(row);
                }

                sp.stop();

                display_table(row_vec).await.unwrap();
                break;
            }
            "bundles" => {
                loop {
                    let channel = tonic::transport::Channel::from_static("http://0.0.0.0:5005")
                        .connect()
                        .await
                        .unwrap();

                    let mut client: BundleServiceClient<tonic::transport::Channel> =
                        BundleServiceClient::new(channel);
                    let request = tonic::Request::new(SubscribeBundlesRequest {
                        searcher_key: String::from(""),
                    });
                    // now the response is stream
                    let mut response = client
                        .subscribe_bundles(request)
                        .await
                        .ok()
                        .expect("failed to subscribe to bundles")
                        .into_inner();
                    // listening to stream
                    while let res = response.message().await {
                        match res {
                            Ok(r) => println!("RESPONSE={:?}", r),
                            Err(e) => println!("ERR={:?}", e),
                        }
                    }
                }

                // break;
            }

            _ => {
                // overview ui
                break;
            }
        }
    }
}

// type Something = Arc<tokio::sync::Mutex<Vec<Vec<String>>>>;
// impl Something for SomethingStruct {}
// type Res<T> = Result<T, dyn Error>;
pub async fn get_and_parse_arb_feed() -> Result<Vec<Vec<String>>, Box<dyn Error + std::marker::Send>>
{
    let feed = arb_feed::get_arb_feed().await;
    let parsed =
        serde_json::from_str::<ArbFeedResponse>(feed.unwrap().text().await.unwrap().as_str())
            .unwrap()
            .query_data;

    let profit_amts = parsed
        .profit_amount
        .iter()
        .map(|a| a.to_string())
        .collect::<Vec<String>>();
    let prices_usd = parsed
        .price_usd
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>();
    let profits_usd = parsed
        .price_usd
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>();
    let QueryData {
        block_time,
        slot_id,
        transaction_hash,
        profit_amount,
        currency,
        signers,
        price_usd,
        profit_usd,
    } = parsed;
    let mut row_vec = Vec::new();

    for index in 0..transaction_hash.len() {
        let row: Vec<String> = vec![
            block_time[index].clone(),
            slot_id[index].clone(),
            transaction_hash[index].clone(),
            profit_amts[index].clone(),
            currency[index].clone(),
            signers[index].clone(),
            prices_usd[index].clone(),
            profits_usd[index].clone(),
        ];
        row_vec.push(row);
    }
    Ok(row_vec)
}
