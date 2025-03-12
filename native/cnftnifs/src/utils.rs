use solana_sdk::signature::Keypair;
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
