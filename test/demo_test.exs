defmodule CnftNifsTest do
  use ExUnit.Case

  test "mint_nft/10 mints an NFT successfully" do
    {:ok, tree} = CnftNifs.create_tree()

    # Ensure the tree is valid Base58
    assert tree =~ ~r/^[1-9A-HJ-NP-Za-km-z]+$/

    {:ok, asset_id} = CnftNifs.mint_nft(tree, "NFT Name", "https://example.com/nft.png", "NFTSYM", 100, true, 500, true, true, 1)
    IO.puts("Minted asset ID: #{asset_id}")

    assert asset_id =~ ~r/^[1-9A-HJ-NP-Za-km-z]+$/
  end

  test "transfer_nft/3 transfers an NFT successfully" do
    {:ok, tree} = CnftNifs.create_tree()
    {:ok, asset} = CnftNifs.mint_nft(tree, "NFT Name", "https://example.com/nft.png", "NFTSYM", 100, true, 500, true, true, 1)

    # Use a valid receiver address
    receiver = "ap5oPFPVSnxtc8bbvcCeKwy9Xnu5NePhMGzX2hexDVh"

    assert receiver =~ ~r/^[1-9A-HJ-NP-Za-km-z]+$/

    {:ok, signature} = CnftNifs.transfer_nft(receiver, tree, asset)
    IO.puts("Transfer successful: #{signature}")

    assert signature =~ ~r/^[1-9A-HJ-NP-Za-km-z]+$/
  end
end
