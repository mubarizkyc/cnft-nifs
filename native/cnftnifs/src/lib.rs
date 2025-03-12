use rustler::{Atom, Error, NifResult, NifStruct};
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
mod serializable_merkle_tree;

pub use serializable_merkle_tree::SerializableMerkleTree;
mod utils;
pub use utils::*;
//use bincode;
// External Crates
use bs58;
use mpl_bubblegum::{
    accounts::TreeConfig,
    hash::{hash_creators, hash_metadata},
    instructions::{CreateTreeConfigBuilder, MintV1Builder, TransferBuilder},
    types::{Creator, LeafSchema, MetadataArgs, TokenProgramVersion, TokenStandard},
    utils::get_asset_id,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::AccountMeta,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
    system_instruction,
    transaction::Transaction,
};
use std::convert::TryInto;

// Merkle Tree & Compression
use spl_account_compression::{state::CONCURRENT_MERKLE_TREE_HEADER_SIZE_V1, ConcurrentMerkleTree};
use spl_merkle_tree_reference::{MerkleTree, Node};
const RPC_URL: &str = "https://api.devnet.solana.com";
const KEYPAIR_PATH: &str = "/home/mubariz/.config/solana/id.json"; // Change to your actual path
const MAX_DEPTH: usize = 6;
const MAX_BUFFER_SIZE: usize = 16;
use bincode;
/// Rustler-compatible `LeafSchema`
#[derive(NifStruct)]
#[module = "MyModule.LeafSchema"]
pub struct LeafSchemaReplica {
    id: String,       // bs58 encoded PubKey
    owner: String,    // bs58 encoded PubKey
    delegate: String, // bs58 encoded PubKey
    nonce: u64,
    data_hash: Vec<u8>,
    creator_hash: Vec<u8>,
}

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
    merkle_tree: String,
) -> Result<(LeafSchemaReplica, String), Box<dyn stdError>> {
    let payer = read_keypair_file(KEYPAIR_PATH).expect("Failed to read keypair file");

    let (_, mut proof_tree) = decode_merkle_tree(merkle_tree).map_err(|_| "Decoding failed")?;

    let rpc_client =
        RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    let tree = get_keypair(tree_string).unwrap();
    let (tree_config, _) = TreeConfig::find_pda(&tree.pubkey());
    let account_data = rpc_client.get_account_data(&tree_config).unwrap();
    let tree_config_account = TreeConfig::from_bytes(&account_data).unwrap();

    let mint_ix = MintV1Builder::new()
        .leaf_delegate(payer.pubkey())
        .leaf_owner(payer.pubkey())
        .merkle_tree(tree.pubkey())
        .payer(payer.pubkey())
        .tree_config(tree_config)
        .tree_creator_or_delegate(payer.pubkey())
        .metadata(meta_data.clone())
        .instruction();
    let data_hash = hash_metadata(&meta_data).unwrap();
    let creator_hash = hash_creators(&meta_data.creators);
    let mint_count = tree_config_account.num_minted;
    let asset_id = get_asset_id(&tree.pubkey(), mint_count);

    let leaf_replica = LeafSchemaReplica {
        id: asset_id.to_string(),
        owner: payer.pubkey().to_string(),
        delegate: payer.pubkey().to_string(),
        nonce: mint_count,
        data_hash: data_hash.to_vec(),
        creator_hash: creator_hash.to_vec(),
    };
    let leaf = LeafSchema::V1 {
        id: asset_id,
        owner: payer.pubkey(),
        delegate: payer.pubkey(),
        nonce: mint_count,
        data_hash,
        creator_hash,
    };
    proof_tree.add_leaf(leaf.hash(), mint_count as usize);

    let signers: Vec<&Keypair> = vec![&payer];
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[mint_ix],
        Some(&payer.pubkey()),
        &signers, // Signers
        recent_blockhash,
    );

    let signature = rpc_client.send_and_confirm_transaction(&tx)?;
    println!("cNFTs minted on Tx: {}", signature);
    Ok((leaf_replica, encode_merkle_tree(&proof_tree)))
}
use std::str::FromStr;
pub async fn transfer(
    receiver: Pubkey,
    tree_string: String,
    merkle_tree: String,
    asset: &LeafSchemaReplica,
) -> Result<(String, String), Box<dyn std::error::Error>> {
    let (_, mut proof_tree) = decode_merkle_tree(merkle_tree).map_err(|_| "Decoding failed")?;

    let owner = read_keypair_file(KEYPAIR_PATH).expect("Failed to read keypair file");
    let payer = read_keypair_file(KEYPAIR_PATH).expect("Failed to read keypair file");
    let rpc_client =
        RpcClient::new_with_commitment(RPC_URL.to_string(), CommitmentConfig::confirmed());
    let tree = get_keypair(tree_string).unwrap();
    let (tree_config, _) = TreeConfig::find_pda(&tree.pubkey());

    let creator_hash = vec_to_array(asset.creator_hash.clone())?;
    let data_hash = vec_to_array(asset.data_hash.clone())?;
    let nonce = asset.nonce;

    let proof: Vec<AccountMeta> = proof_tree
        .get_proof_of_leaf(nonce as usize)
        .iter()
        .map(|node| AccountMeta {
            pubkey: Pubkey::new_from_array(*node),
            is_signer: false,
            is_writable: false,
        })
        .collect();

    let transfer_ix = TransferBuilder::new()
        .leaf_delegate(owner.pubkey(), false)
        .leaf_owner(owner.pubkey(), true)
        .merkle_tree(tree.pubkey())
        .tree_config(tree_config)
        .new_leaf_owner(receiver)
        .root(proof_tree.root)
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
    let signature = rpc_client.send_and_confirm_transaction(&tx)?;
    println!("Transfer Tx: {}", signature);

    let leaf = LeafSchema::V1 {
        id: Pubkey::from_str(&asset.id).expect("Invalid public key"),
        owner: receiver,
        delegate: receiver,
        nonce,
        data_hash,
        creator_hash,
    };

    proof_tree.add_leaf(leaf.hash(), nonce as usize);

    Ok((signature.to_string(), encode_merkle_tree(&proof_tree)))
}
fn encode_merkle_tree(merkle_tree: &MerkleTree) -> String {
    let serializable_tree = SerializableMerkleTree {
        leaf_nodes: merkle_tree
            .leaf_nodes
            .iter()
            .map(|node| node.borrow().node)
            .collect(),
        root: merkle_tree.get_root(), // Include the root node
    };

    let encoded_bytes = bincode::serialize(&serializable_tree).expect("Serialization failed");
    bs58::encode(encoded_bytes).into_string()
}
fn decode_merkle_tree(encoded_string: String) -> NifResult<(Atom, MerkleTree)> {
    let decoded_bytes = bs58::decode(&encoded_string)
        .into_vec()
        .map_err(|_| rustler::Error::Atom("base58_decode_failed"))?;

    let serializable_tree: SerializableMerkleTree = bincode::deserialize(&decoded_bytes)
        .map_err(|_| rustler::Error::Atom("deserialization_failed"))?;

    //  Use `TreeNode::new_empty` and manually assign node hashes
    let leaves: Vec<Node> = serializable_tree.leaf_nodes.clone();
    let merkle_tree = MerkleTree::new(&leaves);
    //  Ensure root consistency
    if merkle_tree.get_root() != serializable_tree.root {
        return Err(rustler::Error::Atom("root_mismatch"));
    }

    println!("Successfully decoded Merkle Tree!");

    Ok((atoms::ok(), merkle_tree))
}

// **Expose create_tree_config as an NIF**
#[rustler::nif]
fn getbs58_payer_nif() -> NifResult<(Atom, String)> {
    // will by defult use the keypair in your keypair path
    let payer = read_keypair_file(KEYPAIR_PATH).expect("Failed to read keypair file");
    let payer_bs58 = get_keypair_bs58(payer);

    Ok((atoms::ok(), payer_bs58)) //  Return `{:ok, keypair}`
}
#[rustler::nif(schedule = "DirtyIo")]
fn create_tree_nif() -> NifResult<(Atom, String)> {
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
#[rustler::nif]

fn create_merkle_tree_nif() -> NifResult<(Atom, String)> {
    let tree = spl_merkle_tree_reference::MerkleTree::new(
        vec![Node::default(); 1 << MAX_DEPTH].as_slice(),
    );

    Ok((atoms::ok(), encode_merkle_tree(&tree))) //  Return {:ok, base58_encoded_tree}
}
#[rustler::nif(schedule = "DirtyIo")]
fn mint_nft_nif(
    tree: String,
    nft_name: String,
    nft_url: String,
    nft_symbol: String,
    creator_address: String, // bs58 keypair
    creator_share: u8,
    creator_verification_status: bool,
    seller_fee_basis_points: u16,
    primary_sale_happened: bool,
    is_mutable: bool,
    token_program_version: u8, // 1 for original and 2 for Token22
    proof_tree: String,
) -> NifResult<(Atom, LeafSchemaReplica, String)> {
    let runtime = tokio::runtime::Runtime::new().unwrap(); //  Expensive, but okay in DirtyCpu

    let nft_creator =
        get_keypair(creator_address).map_err(|_| Error::Atom("invalid_creator_keypair"))?;

    let meta_data = MetadataArgs {
        name: nft_name,
        uri: nft_url,
        symbol: nft_symbol,
        creators: vec![Creator {
            address: nft_creator.pubkey(), //  Creator is payer
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
        match mint(tree.clone(), meta_data, proof_tree.clone()).await {
            Ok((leaf_schema, proof_tree)) => {
                Ok((atoms::ok(), leaf_schema, proof_tree)) // Return {:ok, leaf_schema, proof_tree}
            }
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
    merkle_tree: String,
    asset: LeafSchemaReplica,
) -> NifResult<(Atom, String, String)> {
    let runtime = tokio::runtime::Runtime::new().unwrap(); //  Creating new runtime is costly

    runtime.block_on(async {
        let reciever_pubkey =
            Pubkey::from_str(&reciever).map_err(|_| Error::Atom("invalid_pubkey"))?; // ✅ Proper error handling

        let (signature, updated_proof_tree) = transfer(reciever_pubkey, tree, merkle_tree, &asset)
            .await
            .map_err(|e| Error::Term(Box::new(format!("failed_to_transfer: {}", e))))?;

        Ok((atoms::ok(), signature, updated_proof_tree)) //  Return {:ok, signature, proof_tree}
    })
}
rustler::init!("Elixir.CnftNifs");
/*
usage
"""
{:ok, payer_bs58} = CnftNifs.getbs58_payer()
{:ok, tree_bs58} = NIFDEMO.create_tree()
{:ok, proof_tree} = NIFDEMO.create_merkle_tree()
{:ok, nft, proof_tree} = NIFDEMO.mint_nft(tree_bs58, "CoolNFT", "https://example.com/nft.png", "CNFT", payer_bs58, 100, true, 500, false, true, 1, proof_tree)
 {:ok, signature, proof_tree} = NIFDEMO.transfer_nft("ap5oPFPVSnxtc8bbvcCeKwy9Xnu5NePhMGzX2hexDVh", tree_bs58, proof_tree, nft)
76r2MK11WgnaW42cEHE8eYQc4nPJgbtVoqvm1zkgMo4w

 """

*/
