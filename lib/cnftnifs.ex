defmodule CnftNifs do
  use Rustler, otp_app: :cnftnifs, crate: :cnftnifs

  def create_tree() do
    case create_tree_nif() do
      {:ok, tree_bs58} ->
        IO.puts("Created tree with keypair: #{tree_bs58}")
        {:ok, tree_bs58}

      {:error, reason} ->
        IO.puts("Failed to create tree: #{reason}")
        {:error, reason}
    end
  end

  def mint_nft(tree, nft_name, nft_url, nft_symbol, creator_share, creator_verification_status, seller_fee_basis_points, primary_sale_happened, is_mutable, token_program_version) do
    case mint_nft_nif(tree, nft_name, nft_url, nft_symbol, creator_share, creator_verification_status, seller_fee_basis_points, primary_sale_happened, is_mutable, token_program_version) do
      {:ok, asset_id} ->
        IO.puts("Minted NFT successfully!")
        {:ok, asset_id}

      {:error, reason} ->
        IO.puts("Failed to mint NFT: #{reason}")
        {:error, reason}
    end
  end

  def transfer_nft(reciever, tree, asset) do
    case transfer_nft_nif(reciever, tree, asset) do
      {:ok, signature} ->
        IO.puts("NFT transferred successfully! Tx Signature: #{signature}")
        {:ok, signature}

      {:error, reason} ->
        IO.puts("Failed to transfer NFT: #{reason}")
        {:error, reason}
    end
  end

  # Private functions for NIFs
  defp create_tree_nif, do: :erlang.nif_error(:nif_not_loaded)
  defp mint_nft_nif(_, _, _, _, _, _, _, _, _, _), do: :erlang.nif_error(:nif_not_loaded)
  defp transfer_nft_nif(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
end
