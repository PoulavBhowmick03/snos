use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::json;
use starknet_types_core::felt::Felt;

use crate::pathfinder::proofs::{PathfinderClassProof, PathfinderProof};

fn jsonrpc_request(method: &str, params: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": method,
        "params": params,
    })
}

async fn post_jsonrpc_request<T: DeserializeOwned>(
    client: &reqwest::Client,
    rpc_provider: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<T, reqwest::Error> {
    let request = jsonrpc_request(method, params);
    let response = client.post(format!("{}/rpc/pathfinder/v0.1", rpc_provider)).json(&request).send().await?;

    #[derive(Deserialize)]
    struct TransactionReceiptResponse<T> {
        result: T,
    }

    let response_text = response.text().await?;
    let response: TransactionReceiptResponse<T> =
        serde_json::from_str(&response_text).unwrap_or_else(|_| panic!("Error: {}", response_text));
    Ok(response.result)
}

pub struct PathfinderRpcClient {
    /// A raw client to access endpoints not covered by starknet-rs.
    http_client: reqwest::Client,
    /// The base URL of the RPC client
    rpc_base_url: String,
}

impl PathfinderRpcClient {
    pub fn new(base_url: &str) -> Self {
        let starknet_rpc_url = format!("{}/rpc/v0_7", base_url);
        log::info!("Starknet RPC URL: {}", starknet_rpc_url);
        let http_client =
            reqwest::ClientBuilder::new().build().unwrap_or_else(|e| panic!("Could not build reqwest client: {e}"));

        Self { http_client, rpc_base_url: base_url.to_string() }
    }

    pub async fn get_proof(
        &self,
        block_number: u64,
        contract_address: Felt,
        keys: &[Felt],
    ) -> Result<PathfinderProof, reqwest::Error> {
        post_jsonrpc_request(
            &self.http_client,
            &self.rpc_base_url,
            "pathfinder_getProof",
            json!({ "block_id": { "block_number": block_number }, "contract_address": contract_address, "keys": keys }),
        )
        .await
    }

    pub async fn get_class_proof(
        &self,
        block_number: u64,
        class_hash: &Felt,
    ) -> Result<PathfinderClassProof, reqwest::Error> {
        log::debug!("querying pathfinder_getClassProof for {:x}", class_hash);
        post_jsonrpc_request(
            &self.http_client,
            &self.rpc_base_url,
            "pathfinder_getClassProof",
            json!({ "block_id": { "block_number": block_number }, "class_hash": class_hash }),
        )
        .await
    }
}