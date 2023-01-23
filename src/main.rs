#![feature(async_closure)]
use std::error::Error;
use std::time::Duration;
// use std::sync::Arc;
// use std::thread;
// use std::time::Duration;

use crate::arb_feed::ArbFeedResponse;
use crate::bundle_feed::get_bundle_feed;
use arb_feed::QueryData;
use clap::arg;
// use clap::command;
// use clap::ArgMatches;
use clap::Command;
use futures::StreamExt;
// use clap::Parser;
use spinners::{Spinner, Spinners};
mod arb_feed;
mod arb_table;
mod bundle_feed;
mod events;
mod key;
use key::Key;
use reqwest::Response;
// mod bundle_table;
// mod searcher_grpc;

use arb_table::*;
use futures::stream::iter;
use tmev_protos::tmev_proto::SubscribeBundlesRequest;
// use tmev_protos::bundle_service_client;
// use tmev_protos::SubscribeBundlesRequest;
use tmev_protos::tmev_proto::bundle_service_client::BundleServiceClient;
use tmev_protos::tmev_proto::Bundle;
use tokio::sync::Mutex;
use tokio::time::sleep;
#[tokio::main]
async fn main() {
    let cmd = Command::new("tmev").args(&[
        arg!(--address <ADDRESS> "Deprecated."),
        arg!(--arbs <ADDRESS>"View a table of the recent arbs in order of most profitable"),
        arg!(--bundles "View a table of the recent bundles"),
    ]);

    let matches = cmd.get_matches();
    // println!("matches: {:?}", matches);
    for arg in matches.ids().into_iter() {
        // println!("matches: {:?}", arg);
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
                let raw_bundles = get_bundle_feed().await.unwrap();
                let mut bundle_vec = Vec::new();
                let mut searcher_bundle_vec = Vec::new();
                let sk = "CQzPyC5xVhkuBfWFJiPCvPEnBshmRium4xxUxnX1ober"; // adding this to filter searcher only for demo purposes
                for raw in raw_bundles.iter() {
                    let searcher = raw.bundles.get(0).unwrap().searcher_key.clone();
                    let uuid = raw.bundles.get(0).unwrap().uuid.clone();
                    let slot = raw.bundles.get(0).unwrap().slot.clone();
                    let mut tip = raw.tip_amt.clone().unwrap_or(0.0).to_string();
                    // println!("out: {:?}", tip);
                    tip.push_str(" 💰");
                    bundle_vec.push(vec![
                        slot.clone(),
                        searcher.clone(),
                        uuid.clone(),
                        tip.clone(),
                    ]);
                    if searcher == sk.to_string() {
                        searcher_bundle_vec.push(vec![
                            slot.clone(),
                            searcher.clone(),
                            uuid.clone(),
                            tip.clone(),
                        ]);
                    }
                }

                sp.stop();

                display_table(row_vec, bundle_vec, searcher_bundle_vec)
                    .await
                    .unwrap();
            }
            "bundles" => {
                // loop {
                //     // let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
                //     // tokio::spawn(run_bundle_request_loop(tx));

                //     //display_table(rows)
                // }
                // break;
                // let feed = get_bundle_feed().await;

                // display_table();
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
