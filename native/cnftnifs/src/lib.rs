use rustler::{Atom, Error, NifResult};
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
mod constants;
mod utils;
use mpl_bubblegum::{
    accounts::TreeConfig,
    instructions::{CreateTreeConfigBuilder, MintV1Builder, TransferBuilder},
    types::{Creator, MetadataArgs, TokenProgramVersion, TokenStandard},
    utils::get_asset_id,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};

pub use constants::*;
pub use utils::*;
// Merkle Tree & Compression
use spl_account_compression::{state::CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1, ConcurrentMerkleTree};

pub async fn create_tree_config(tree_string: String) -> Result<String, Box<dyn stdError>> {
    let payer = read_keypair_file(KEYPAIR_PATH).expect("Failed to read keypair file");
    let rpc_client =
        RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    let tree = get_keypair(tree_string).unwrap();
    let (tree_config, _) = TreeConfig::find_pda(&tree.pubkey());

    //   Merkle Tree accounts with associated Tree Config accounts will be reffrered as  Bubblegum Trees.
    let size = CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1
        + std::mem::size_of::<ConcurrentMerkleTree<MAX_DEPTH, MAX_BUFFER_SIZE>>();

    let rent = rpc_client.get_minimum_balance_for_rent_exemption(size)?;

    let create_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &tree.pubkey(),
        rent,
        size as u64,
        &spl_account_compression::ID,
    );
    let create_config_ix = CreateTreeConfigBuilder::new()
        .tree_config(tree_config)
        .merkle_tree(tree.pubkey())
        .payer(payer.pubkey())
        .tree_creator(payer.pubkey())
        .max_depth(MAX_DEPTH as u32)
        .max_buffer_size(MAX_BUFFER_SIZE as u32)
        .instruction();
    let signers = vec![&payer, &tree]; // Use references (&)
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, create_config_ix],
        Some(&payer.pubkey()),
        &signers, // Signers as references
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&tx)?;
    Ok(signature.to_string())
}
pub async fn mint(
    tree_string: String,
    meta_data: MetadataArgs,
) -> Result<String, Box<dyn stdError>> {
    let payer = read_keypair_file(KEYPAIR_PATH).expect("Failed to read keypair file");

    let rpc_client =
        RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    let tree = get_keypair(tree_string).unwrap();
    let (tree_config, _) = TreeConfig::find_pda(&tree.pubkey());
    let account_data = rpc_client.get_account_data(&tree_config).unwrap();
    let tree_config_account = TreeConfig::from_bytes(&account_data).unwrap();

    let mint_count = tree_config_account.num_minted;
    let asset_id = get_asset_id(&tree.pubkey(), mint_count);
    let mint_ix = MintV1Builder::new()
        .leaf_delegate(payer.pubkey())
        .leaf_owner(payer.pubkey())
        .merkle_tree(tree.pubkey())
        .payer(payer.pubkey())
        .tree_config(tree_config)
        .tree_creator_or_delegate(payer.pubkey())
        .metadata(meta_data.clone())
        .instruction();

    let signers: Vec<&Keypair> = vec![&payer];
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &signers, // Signers
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&tx).map_err(|e| {
        println!("Transaction failed: {:?}", e);
        e
    })?;
    println!("cNFTs minted on Tx: {}", signature);
    Ok(asset_id.to_string())
}

use std::str::FromStr;
pub async fn transfer(
    receiver: Pubkey,
    tree_string: String,
    asset_id: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    let owner = read_keypair_file(KEYPAIR_PATH).expect("Failed to read keypair file");
    let payer = read_keypair_file(KEYPAIR_PATH).expect("Failed to read keypair file");
    let rpc_client =
        RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    let tree = get_keypair(tree_string).unwrap();
    let (tree_config, _) = TreeConfig::find_pda(&tree.pubkey());

    let (creator_hash, data_hash, nonce) = get_asset_data(&asset_id).await?;
    let (proof, root) = get_asset_proof(&asset_id).await?;
    let transfer_ix = TransferBuilder::new()
        .leaf_delegate(owner.pubkey(), false)
        .leaf_owner(owner.pubkey(), true)
        .merkle_tree(tree.pubkey())
        .tree_config(tree_config)
        .new_leaf_owner(receiver)
        .root(root)
        .nonce(nonce)
        .creator_hash(creator_hash)
        .data_hash(data_hash)
        .index(nonce as u32)
        .add_remaining_accounts(&proof)
        .instruction();

    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let signers: Vec<&Keypair> = vec![&owner, &payer];
    let tx = Transaction::new_signed_with_payer(
        &[transfer_ix],
        Some(&payer.pubkey()),
        &signers,
        recent_blockhash,
    );
    let signature = rpc_client.send_and_confirm_transaction(&tx).map_err(|e| {
        println!("Transaction failed: {:?}", e);
        e
    })?;
    println!("Transfer Tx: {}", signature);

    Ok(signature.to_string())
}
#[rustler::nif(schedule = "DirtyIo")]
fn create_tree_nif() -> NifResult<(Atom, String)> {
    // the payer will be creator for tree as well
    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.block_on(async {
        let tree_bs58 = get_keypair_bs58(Keypair::new());
        let _signature = create_tree_config(tree_bs58.clone())
            .await
            .map_err(|_| Error::Atom("failed_to_create_tree_config"))?; //  Handle errors properly

        println!("Your tree keypair is: {}", tree_bs58);

        Ok((atoms::ok(), tree_bs58)) //  Return `{:ok, keypair}`
    })
}
#[rustler::nif(schedule = "DirtyIo")]
fn mint_nft_nif(
    tree: String,
    nft_name: String,
    nft_url: String,
    nft_symbol: String,
    creator_share: u8,
    creator_verification_status: bool,
    seller_fee_basis_points: u16,
    primary_sale_happened: bool,
    is_mutable: bool,
    token_program_version: u8, // 1 for original and 2 for Token22
) -> NifResult<(Atom, String)> {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    let creator_address =
        read_keypair_file(CREATOR_KEYPAIR_PATH).expect("Failed to read keypair file");

    let meta_data = MetadataArgs {
        name: nft_name,
        uri: nft_url,
        symbol: nft_symbol,
        creators: vec![Creator {
            address: creator_address.pubkey(), //  Creator is payer
            share: creator_share,
            verified: creator_verification_status,
        }],
        edition_nonce: None,
        is_mutable,
        primary_sale_happened,
        seller_fee_basis_points,
        token_program_version: if token_program_version == 1 {
            TokenProgramVersion::Original
        } else {
            TokenProgramVersion::Token2022
        },
        token_standard: Some(TokenStandard::NonFungible),
        collection: None,
        uses: None,
    };

    runtime.block_on(async {
        match mint(tree.clone(), meta_data).await {
            Ok(asset_id) => Ok((atoms::ok(), asset_id)),
            Err(err) => {
                eprintln!("Minting failed: {:?}", err);
                Err(Error::Atom("failed_to_mint_nft"))
            }
        }
    })
}

#[rustler::nif(schedule = "DirtyIo")]
fn transfer_nft_nif(
    reciever: String, // receiver pubkey
    tree: String,
    asset_id: String,
) -> NifResult<(Atom, String)> {
    let runtime = tokio::runtime::Runtime::new().unwrap();

    runtime.block_on(async {
        let reciever_pubkey =
            Pubkey::from_str(&reciever).map_err(|_| Error::Atom("invalid_pubkey"))?; // ✅ Proper error handling

        let signature = transfer(reciever_pubkey, tree, &asset_id)
            .await
            .map_err(|e| Error::Term(Box::new(format!("failed_to_transfer: {}", e))))?;

        Ok((atoms::ok(), signature))
    })
}
rustler::init!("Elixir.CnftNifs");
/*
usage
"""
{:ok, tree_bs58} = CnftNifs.create_tree()
{:ok, asset_id} = CnftNifs.mint_nft(tree_bs58, "CoolNFT", "https://example.com/nft.png", "CNFT", 100, true, 500, false, true, 1)
{:ok, signature} = CnftNifs.transfer_nft("ap5oPFPVSnxtc8bbvcCeKwy9Xnu5NePhMGzX2hexDVh", tree_bs58,asset_id)
76r2MK11WgnaW42cEHE8eYQc4nPJgbtVoqvm1zkgMo4w
 """
*/
