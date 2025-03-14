use solana_sdk::signature::Keypair;
// Standard Library
use std::error::Error as stdError;
mod atoms {
    rustler::atoms! {
        ok,
        error,
        eof,
        unknown // Other error
    }
}
use bs58;
use reqwest::Client;
use serde_json::Value;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};
pub fn vec_to_array(vec: Vec<u8>) -> Result<[u8; 32], &'static str> {
    vec.try_into().map_err(|_| "Vector length is not 32 bytes")
}

pub fn get_keypair_bs58(keypair: Keypair) -> String {
    bs58::encode(keypair.to_bytes()).into_string()
}
pub fn get_keypair(keypair: String) -> Result<Keypair, Box<dyn std::error::Error>> {
    let decoded = bs58::decode(&keypair).into_vec()?;
    Ok(Keypair::from_bytes(&decoded)?)
}
pub async fn get_proof_from_api(
    asset_id: &str,
) -> Result<(Vec<AccountMeta>, [u8; 32]), Box<dyn stdError>> {
    let client = Client::new();

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAssetProof",
        "params": {
            "id": asset_id
        }
    });

    let response: Value = client
        .post("https://aura-devnet.metaplex.com")
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    // Extract and decode root as [u8; 32]
    let root_str = response["result"]["root"]
        .as_str()
        .ok_or("Missing root field")?;
    let root_vec = bs58::decode(root_str)
        .into_vec()
        .map_err(|_| "Failed to decode root from base58")?;
    let root: [u8; 32] = root_vec
        .try_into()
        .map_err(|_| "Root key has invalid length")?;

    // Extract proof list
    let proof_list = response["result"]["proof"]
        .as_array()
        .ok_or("Missing proof array")?
        .iter()
        .filter_map(|entry| entry.as_str())
        .filter_map(|proof_str| Pubkey::try_from(proof_str).ok()) // Skip invalid keys
        .map(|pubkey| AccountMeta {
            pubkey,
            is_signer: false,
            is_writable: false,
        })
        .collect();

    Ok((proof_list, root))
}
pub async fn get_asset_data(
    asset_id: &str,
) -> Result<([u8; 32], [u8; 32], u64), Box<dyn stdError>> {
    let client = Client::new();

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAsset",
        "params": { "id": asset_id }
    });

    let response: Value = client
        .post("https://aura-devnet.metaplex.com")
        .json(&payload)
        .send()
        .await?
        .json()
        .await?;

    // Navigate the JSON to get compression fields
    let compression = response["result"]["compression"]
        .as_object()
        .ok_or("Missing compression field")?;

    // Extract and decode `creator_hash`
    let creator_hash_str = compression
        .get("creator_hash")
        .and_then(Value::as_str)
        .ok_or("Missing creator_hash")?;
    let creator_hash: [u8; 32] = bs58::decode(creator_hash_str)
        .into_vec()?
        .try_into()
        .map_err(|_| "Invalid creator_hash length")?;

    // Extract and decode `data_hash`
    let data_hash_str = compression
        .get("data_hash")
        .and_then(Value::as_str)
        .ok_or("Missing data_hash")?;
    let data_hash: [u8; 32] = bs58::decode(data_hash_str)
        .into_vec()?
        .try_into()
        .map_err(|_| "Invalid data_hash length")?;

    let nonce = compression
        .get("leaf_id")
        .and_then(Value::as_u64)
        .ok_or("Missing data_hash")?;
    Ok((creator_hash, data_hash, nonce))
}
