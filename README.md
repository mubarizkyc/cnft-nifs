# CnftNifs

## Config 
set keypair path in **cnfts-nifs/native/cnftnifs/constants.rs** to your devent keypair path 
```rust

pub const KEYPAIR_PATH: &str = ""; // Change to your actual path
pub const CREATOR_KEYPAIR_PATH: &str = "";

```
set aura api key path in **cnfts-nifs/native/cnftnifs/utils.rs** ,as we will be using Aura for offchain asset & proof retrivel,you can get it from [here](https://aura-app.metaplex.com/en/login) 
```rust
pub const AURA_URL: &str = "https://devnet-aura.metaplex.com/df9a341a-4158-439c-bbde-28635dfd1cad";

```
## Build & Test
```shell
mix deps.get && cargo build --release && mix compile && mix test
iex -S mix # start iex
```
## Nifs
**Create Tree**

we still need some onchain accounts to keep track of the Merkle Tree and its configuration

we can change Max Depth and Max Buffer Size in constrants.rs,
[How it affects nfts](https://developers.metaplex.com/bubblegum/create-trees)
```shell
{:ok, tree_bs58} = CnftNifs.create_tree()
```
**Mint**

we need previously created tree & Metadata ,currently the nif is not taking optonal Metadata parameters
```rust
#[rustler::nif(schedule = "DirtyIo")]
fn mint_nft_nif(
    tree: String,
    nft_name: String,
    nft_url: String,
    nft_symbol: String,
    creator_share: u8, // must be >99
    creator_verification_status: bool,
    seller_fee_basis_points: u16,
    primary_sale_happened: bool,
    is_mutable: bool,
    token_program_version: u8, // 1 for original and 2 for Token22
) -> NifResult<(Atom, String)> {
```
```elixir
# replace nft name ,url ,symbol and other configs as you like
{:ok, asset_id} = CnftNifs.mint_nft(tree_bs58, "CoolNFT", "https://example.com/nft.png", "CNFT", 100, true, 500, false, true, 1)
```
**Transfer**

for transfer we need recievers pubkey ,the tree we created and the asset_id
```elixir
{:ok, signature} = CnftNifs.transfer_nft("ap5oPFPVSnxtc8bbvcCeKwy9Xnu5NePhMGzX2hexDVh", tree_bs58,asset_id)
```

for offchain storage we are using [Aura](https://aura.metaplex.com/) make sure the [server](https://api.devnet.solana.com) is up befoe using 
