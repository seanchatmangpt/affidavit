# Tutorial: Mounting `affidavit-wasm` inside the BEAM (Elixir/Wasmex)

**Difficulty:** Intermediate  
**Target Runtime:** Elixir OTP Node with `wasmex`  

---

## 1. Overview

`affidavit-wasm` ships a sandboxed WebAssembly artifact (`dist/affidavit_wasm.wasm`) that imports strictly `wasi_snapshot_preview1` symbols. This allows BEAM applications like `smalt` to admit and seal execution receipts in sub-100 microseconds without dirty NIF crashes.

---

## 2. Configuration & Mounting

Place `affidavit_wasm.wasm` and its SHA256 sidecar into your application's `priv/wasm/` directory:

```elixir
config :smalt,
  affidavit_wasm_path: "priv/wasm/affidavit_wasm.wasm",
  affidavit_sha256_path: "priv/wasm/affidavit.sha256"
```

---

## 3. Invoking `affi_admit_and_seal`

In your Elixir GenServer or pipeline module:

```elixir
def admit_and_seal_state(state_delta_bytes, tariff_cents, modality) do
  # Calls the C-ABI export affi_admit_and_seal inside linear memory
  case Smalt.Affidavit.WasmEngine.call_admit_and_seal(state_delta_bytes, tariff_cents, modality, %{}) do
    {:ok, %{ctrt_bytes: ctrt_352b}} ->
      {:ok, ctrt_352b}

    {:error, {:seal_refused, code}} ->
      {:error, code}
  end
end
```

The call executes in constant time and is completely isolated inside the WASM sandbox.
