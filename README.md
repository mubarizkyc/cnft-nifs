# CnftNifs

**TODO: Add description**

## Installation

If [available in Hex](https://hex.pm/docs/publish), the package can be installed
by adding `cnftnifs` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:cnftnifs, "~> 0.1.0"}
  ]
end
```

Documentation can be generated with [ExDoc](https://github.com/elixir-lang/ex_doc)
and published on [HexDocs](https://hexdocs.pm). Once published, the docs can
be found at <https://hexdocs.pm/cnftnis>.
## Config 
set keypair path in cnfts-nifs/native/cnftnifs/constants.rs to your devent keypair path 
```rust

pub const KEYPAIR_PATH: &str = ""; // Change to your actual path
pub const CREATOR_KEYPAIR_PATH: &str = "";

```
## Build & Test
```shell
mix deps.get && cargo build --release && mix compile && mix test
iex -S mix # start iex
```
## Nifs
```shell
{:ok, tree_bs58} = CnftNifs.create_tree()
{:ok, asset_id} = CnftNifs.mint_nft(tree_bs58, "CoolNFT", "https://example.com/nft.png", "CNFT", 100, true, 500, false, true, 1)
{:ok, signature} = CnftNifs.transfer_nft("ap5oPFPVSnxtc8bbvcCeKwy9Xnu5NePhMGzX2hexDVh", tree_bs58,asset_id)
```
