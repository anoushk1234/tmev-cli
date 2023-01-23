use reqwest;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;
const TIP_ACCOUNT: &'static str = "Gnt27xtC473ZT2Mw5u8wZ68Z3gULkSTb5DuxJy7eJotD";
pub async fn get_bundle_feed() -> Result<Vec<BlockBundles>, Box<dyn Error>> {
    let url = "http://0.0.0.0:8000/bundles";
    let client = reqwest::Client::new();
    let resp = client.get(url).send().await?;
    let parsed: Vec<BlockBundles> =
        serde_json::from_str(resp.text().await.unwrap().as_str()).unwrap();
    let helius_rpc =
        "https://api.helius.xyz/v0/transactions/?api-key=f7e6a7ce-99ec-4bfc-b487-4fbda0defd20";
    // let helius_client = reqwest::Client::new();

    for mut b in parsed.clone() {
        let body = serde_json::json!({
            "transactions":b.bundles.iter().map(|f| f.transaction_hash.clone()).collect::<Vec<String>>().as_slice().to_owned()
        })
        .to_string();
        let helius_data = client
            .post(helius_rpc)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?
            .text()
            .await?;
        // println!("hd : {:?}", helius_data);
        let txns: serde_json::Result<Vec<HeliusTransactionResponse>> =
            serde_json::from_str(&helius_data.as_str());
        // println!("txns : {:?}", txns);
        if let Ok(txns) = txns {
            for txn in txns {
                let is_tip = serde_json::to_string(&txn).unwrap().contains(TIP_ACCOUNT);
                // println!("is tip: {:?}", is_tip);
                if is_tip {
                    let tip_amt = txn
                        .token_transfers // since we are simulating bundles using pyth's acc we are using token atransfer but u shud use native transfer for tip accs
                        .get(0)
                        .unwrap_or(&TokenTransfer::default())
                        .token_amount;
                    // println!("ta: {:?}", tip_amt);
                    b.tip_amt = Some(tip_amt);
                }
            }
        } else {
            b.tip_amt = Some(9.0)
        }
    }

    Ok(parsed)
}
// use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockBundles {
    pub id: Option<String>,
    pub bundles: Vec<SingleBundle>,
    pub tip_amt: Option<f64>,
}
// pardon the naming scheme too many "Bundles"
#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct SingleBundle {
    pub searcher_key: String,
    pub uuid: String,
    pub transaction_hash: String,
    pub slot: String,
}

use futures::StreamExt;
// use reqwest::Response;
// use serde_derive::Deserialize;
// use serde_derive::Serialize;
// use serde_json::Value;
// use std::error::Error;
use std::time::Duration;
use tmev_protos::tmev_proto::bundle_service_client::BundleServiceClient;
use tmev_protos::tmev_proto::Bundle;
use tmev_protos::tmev_proto::SubscribeBundlesRequest;
// use tokio::sync::mpsc::Receiver;
// use tokio::sync::mpsc::Sender;
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

use serde_derive::Deserialize;
use serde_derive::Serialize;
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeliusTransactionResponse {
    pub description: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub source: String,
    pub fee: i64,
    pub fee_payer: String,
    pub signature: String,
    pub slot: i64,
    pub timestamp: i64,
    pub native_transfers: Vec<NativeTransfer>,
    pub token_transfers: Vec<TokenTransfer>,
    pub account_data: Vec<AccountDaum>,
    pub transaction_error: Option<TransactionError>,
    pub instructions: Vec<Instruction>,
    pub events: Events,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeTransfer {
    pub from_user_account: String,
    pub to_user_account: String,
    pub amount: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTransfer {
    pub from_user_account: String,
    pub to_user_account: String,
    pub from_token_account: String,
    pub to_token_account: String,
    pub token_amount: f64,
    pub mint: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountDaum {
    pub account: String,
    pub native_balance_change: i64,
    pub token_balance_changes: Vec<TokenBalanceChange>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceChange {
    pub user_account: String,
    pub token_account: String,
    pub mint: String,
    pub raw_token_amount: RawTokenAmount,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawTokenAmount {
    pub token_amount: String,
    pub decimals: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionError {
    pub error: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instruction {
    pub accounts: Vec<String>,
    pub data: String,
    pub program_id: String,
    pub inner_instructions: Vec<InnerInstruction>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerInstruction {
    pub accounts: Vec<String>,
    pub data: String,
    pub program_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Events {
    pub nft: Option<Nft>,
    pub swap: Option<Swap>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nft {
    pub description: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub amount: i64,
    pub fee: i64,
    pub fee_payer: String,
    pub signature: String,
    pub slot: i64,
    pub timestamp: i64,
    pub sale_type: String,
    pub buyer: String,
    pub seller: String,
    pub staker: String,
    pub nfts: Vec<Nft2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Nft2 {
    pub mint: String,
    pub token_standard: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Swap {
    pub native_input: NativeInput,
    pub token_inputs: Vec<Value>,
    pub token_outputs: Vec<Value>,
    pub token_fees: Vec<Value>,
    pub native_fees: Vec<Value>,
    pub inner_swaps: Vec<InnerSwap>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeInput {
    pub account: String,
    pub amount: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InnerSwap {
    pub token_inputs: Vec<Value>,
    pub token_outputs: Vec<Value>,
    pub token_fees: Vec<Value>,
    pub native_fees: Vec<Value>,
    pub program_info: ProgramInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgramInfo {
    pub source: String,
    pub account: String,
    pub program_name: String,
    pub instruction_name: String,
}
