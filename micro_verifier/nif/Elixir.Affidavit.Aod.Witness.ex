# SPDX-License-Identifier: MIT OR Apache-2.0
# Scaffold — AFFI-SPEC-26.11.19 PR-8. Copy into a host Mix app's lib/.
# Does not compile inside the affidavit Cargo workspace.

defmodule Affidavit.Aod.Witness do
  @moduledoc """
  Historical Witness over Zero-String Sealed `.aod` fossils.

  Certify, don't decide. This module never maps Region 3 and never burns
  leases. Crypto authority stays in affidavit (`affidavit-micro-verifier` /
  `affidavit-core`); do not add blake3/ml-dsa NIFs in the host app.
  """

  @on_load :load_nif

  @doc false
  def load_nif do
    path = :filename.join(:code.priv_dir(:affidavit_aod), ~c"affidavit_micro_verifier")

    case :erlang.load_nif(path, 0) do
      :ok -> :ok
      {:error, reason} -> {:error, reason}
    end
  end

  @doc "ABI revision; must equal 1 for this scaffold."
  @spec abi_version() :: non_neg_integer()
  def abi_version, do: :erlang.nif_error(:not_loaded)

  @doc """
  Envelope-only Historical Witness.

  Returns `{:ok, attestation}` or `{:error, code}` where `code` is an
  `AodRefusal` machine code (`0xE501..0xE50D`).
  """
  @spec verify(binary()) :: {:ok, map()} | {:error, integer()}
  def verify(_aod_binary), do: :erlang.nif_error(:not_loaded)
end
