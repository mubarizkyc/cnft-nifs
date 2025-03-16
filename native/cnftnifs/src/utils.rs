use bs58;
use reqwest::Client;
use serde_json::Value;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, signature::Keypair};
use std::error::Error as stdError;
pub const AURA_URL: &str = "https://devnet-aura.metaplex.com/df9a341a-4158-439c-bbde-28635dfd1cad";
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
pub async fn get_asset_proof(
    asset_id: &str,
) -> Result<(Vec<AccountMeta>, [u8; 32]), Box<dyn stdError>> {
    let client = Client::new();

    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getAssetProof",
        "params": { "id": asset_id }
    });

    let response: Value = client
        .post(AURA_URL)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    let root_str = response["result"]["root"]
        .as_str()
        .ok_or_else(|| "API response missing 'root' field".to_string())?;

    let root_vec = bs58::decode(root_str)
        .into_vec()
        .map_err(|e| format!("Failed to decode 'root' from base58: {}", e))?;

    let root: [u8; 32] = root_vec
        .try_into()
        .map_err(|_| "Decoded 'root' has invalid length (expected 32 bytes)".to_string())?;

    let proof_array = response["result"]["proof"]
        .as_array()
        .ok_or_else(|| "API response missing 'proof' array".to_string())?;

    let proof_list = proof_array
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .ok_or_else(|| format!("Invalid proof entry: {:?}", entry))
                .and_then(|proof_str| {
                    Pubkey::try_from(proof_str)
                        .map_err(|_| format!("Invalid Pubkey format in proof: {}", proof_str))
                })
                .map(|pubkey| AccountMeta {
                    pubkey,
                    is_signer: false,
                    is_writable: false,
                })
        })
        .collect::<Result<Vec<AccountMeta>, String>>()?;

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
        .post(AURA_URL)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    let compression = response["result"]["compression"]
        .as_object()
        .ok_or_else(|| "API response missing 'compression' field".to_string())?;

    let creator_hash_str = compression
        .get("creator_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "API response missing 'creator_hash' field".to_string())?;

    let creator_hash: [u8; 32] = bs58::decode(creator_hash_str)
        .into_vec()
        .map_err(|e| format!("Failed to decode 'creator_hash' from base58: {}", e))?
        .try_into()
        .map_err(|_| "Decoded 'creator_hash' has invalid length (expected 32 bytes)".to_string())?;

    let data_hash_str = compression
        .get("data_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "API response missing 'data_hash' field".to_string())?;

    let data_hash: [u8; 32] = bs58::decode(data_hash_str)
        .into_vec()
        .map_err(|e| format!("Failed to decode 'data_hash' from base58: {}", e))?
        .try_into()
        .map_err(|_| "Decoded 'data_hash' has invalid length (expected 32 bytes)".to_string())?;

    let nonce = compression
        .get("leaf_id")
        .and_then(Value::as_u64)
        .ok_or_else(|| "API response missing 'leaf_id' field or invalid type".to_string())?;

    Ok((creator_hash, data_hash, nonce))
}
