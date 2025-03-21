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
mix deps.get && cargo build --release && mix compile 
```
## Start Elixir Shell
```shell
iex -S mix 
```
## Nifs
**Create Tree**
Max Depth and Max Buffer Size can be modified in constrants.rs,
[How it affects nfts](https://developers.metaplex.com/bubblegum/create-trees)
```shell
{:ok, tree_bs58} = CnftNifs.create_tree()
```
**Mint**

we need previously created tree & Metadata for minting nft
```rust
// formtat
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

for transfer we need recievers pubkey &  asset_id
```elixir
{:ok, signature} = CnftNifs.transfer_nft("ap5oPFPVSnxtc8bbvcCeKwy9Xnu5NePhMGzX2hexDVh",asset_id)
```
## Test
after quitting shell session
```shell
mix test
```
for offchain storage we are using [Aura](https://aura.metaplex.com/) make sure the [server](https://api.devnet.solana.com) is up befoe using 
