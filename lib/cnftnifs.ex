defmodule CnftNifs do
  use Rustler, otp_app: :cnftnifs, crate: :cnftnifs

  def create_tree do
    case create_tree_nif() do
      {:ok, keypair} ->
        IO.puts("Created tree with keypair: #{keypair}")
        {:ok, keypair}

      {:error, reason} ->
        IO.puts("Failed to create tree: #{reason}")
        {:error, reason}
    end
  end

  def create_merkle_tree do
    case create_merkle_tree_nif() do
      {:ok, base58_tree} ->
        IO.puts("Created Merkle Tree (Base64): #{base58_tree}")
        {:ok, base58_tree}

      {:error, reason} ->
        IO.puts("Failed to create tree: #{reason}")
        {:error, reason}
    end
  end

  def getbs58_payer do
    case getbs58_payer_nif() do
      {:ok, payer_bs58} ->
        IO.puts("Payer Keypair (Base58): #{payer_bs58}")
        {:ok, payer_bs58}

      {:error, reason} ->
        IO.puts("Failed to get payer keypair: #{reason}")
        {:error, reason}
    end
  end

  def mint_nft(tree, nft_name, nft_url, nft_symbol, creator_address, creator_share, creator_verification_status, seller_fee_basis_points, primary_sale_happened, is_mutable, token_program_version, proof_tree) do
    case mint_nft_nif(tree, nft_name, nft_url, nft_symbol, creator_address, creator_share, creator_verification_status, seller_fee_basis_points, primary_sale_happened, is_mutable, token_program_version, proof_tree) do
      {:ok, leaf_schema, updated_proof_tree} ->
        IO.puts("Minted NFT successfully!")
        {:ok, leaf_schema, updated_proof_tree}

      {:error, reason} ->
        IO.puts("Failed to mint NFT: #{reason}")
        {:error, reason}
    end
  end

  def transfer_nft(reciever_bs58, tree, merkle_tree, asset) do
    case transfer_nft_nif(reciever_bs58, tree, merkle_tree, asset) do
      {:ok, signature, updated_proof_tree} ->
        IO.puts("NFT transferred successfully! Tx Signature: #{signature}")
        {:ok, signature, updated_proof_tree}

      {:error, reason} ->
        IO.puts("Failed to transfer NFT: #{reason}")
        {:error, reason}
    end
  end

  # Private functions for NIFs
  defp create_tree_nif, do: :erlang.nif_error(:nif_not_loaded)
  defp create_merkle_tree_nif, do: :erlang.nif_error(:nif_not_loaded)
  defp getbs58_payer_nif, do: :erlang.nif_error(:nif_not_loaded)
  defp mint_nft_nif(_, _, _, _, _, _, _, _, _, _, _, _), do: :erlang.nif_error(:nif_not_loaded)
  defp transfer_nft_nif(_, _, _, _), do: :erlang.nif_error(:nif_not_loaded)
end
