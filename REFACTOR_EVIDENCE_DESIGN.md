# Evidence<Receipt, Admitted, AffidavitReceiptChain> Refactor Design

## Executive Summary

This refactor wraps affidavit's `Receipt` type with the wasm4pm-compat `Evidence` wrapper, enforcing the Layer 2/3 seam (ARDPRD §4) where receipts are admitted (passed OCEL + BLAKE3 chain-seal validation atomically) at deserialization time, and output functions only accept `Evidence<_, Admitted, _>`.

**Status:** Types layer is already present (lines 8-57 in `src/types.rs`); CLI/handlers need wiring for the two-phase transition: `Raw → Admitted`.

---

## Current State (as of commit HEAD)

### types.rs (ALREADY COMPLETE)
```rust
use wasm4pm_compat::evidence::Evidence;
use wasm4pm_compat::state::Admitted;

pub struct AffidavitReceiptChain;  // Witness marker
pub type AdmittedReceipt = Evidence<Receipt, Admitted, AffidavitReceiptChain>;
```

- `Receipt` deserialization already re-verifies chain hash (lines 80-116)
- `Receipt::sealed()` private constructor enforces canonical sealing (lines 59-75)

### cli.rs (PARTIAL — `show()` is complete)
```rust
pub fn show(receipt: &str) -> Result<AdmittedReceipt> {
    let r = load_receipt(receipt)?;
    let admission = Admission::<Receipt, AffidavitReceiptChain>::new(r);
    Ok(admission.into_evidence())
}
```

**Already working:** `show` wraps loaded receipt into `Evidence<Receipt, Admitted, AffidavitReceiptChain>`.

### handlers.rs (PARTIAL)
```rust
pub fn show(receipt: String) -> Result<()> {
    let admitted = adapt(affidavit::cli::show(&receipt))?;
    let parsed = admitted.into_inner();  // Extract Receipt from Evidence<_, Admitted, _>
    // ... display logic
}
```

**Already working:** Handler unpacks Evidence and displays receipt.

---

## Refactor Plan (80/20 Leverage)

### Phase 1: `emit()` → Remains Unchanged (No Evidence Wrapper Needed)

`emit` appends to `.affi/working.json` (in-progress state). Events are not yet admitted; wrapping as Evidence would be premature. The receipt becomes Evidence only when **finalized and deserialized**.

**No change required.**

---

### Phase 2: `assemble()` → Returns `Evidence<Receipt, Raw, AffidavitReceiptChain>`

Currently:
```rust
pub fn assemble(out: Option<&str>) -> Result<()> {
    let events = chain::load_working()?;
    let assembler = chain::ChainAssembler::from_events(events)?;
    let receipt = assembler.finalize();  // Receipt (unsealed phase)
    chain::save_receipt(&receipt, &path)?;
    println!("assembled receipt -> {}", path.display());
    Ok(())
}
```

**Change:**
```rust
// Return Evidence<Receipt, Raw, AffidavitReceiptChain> (unsealed phase)
// so output functions can enforce type-level gate: only Admitted accepted
pub fn assemble(out: Option<&str>) -> Result<Evidence<Receipt, Raw, AffidavitReceiptChain>> {
    let events = chain::load_working()?;
    let assembler = chain::ChainAssembler::from_events(events)?;
    let receipt = assembler.finalize();
    
    let address = chain::content_address(&receipt)?;
    let path: PathBuf = match out {
        Some(p) => PathBuf::from(p),
        None => PathBuf::from(format!("{address}.json")),
    };
    
    chain::save_receipt(&receipt, &path)?;
    
    // Wrap as Raw (not yet admitted; assembly preserves BLAKE3 only)
    let raw_evidence = Evidence::<Receipt, Raw, AffidavitReceiptChain>::new(receipt);
    println!("assembled receipt -> {}", path.display());
    println!("content address: {address}");
    Ok(raw_evidence)
}
```

**Handler adaptation:**
```rust
pub fn assemble(out: Option<String>) -> Result<()> {
    // assemble returns Evidence<Receipt, Raw, _>; we only print, no gate applied
    let raw = adapt(affidavit::cli::assemble(out.as_deref()))?;
    // Handler doesn't extract; type-level gate is for output functions only
    Ok(())
}
```

---

### Phase 3: `verify()` → Accept `Evidence<Receipt, Admitted, AffidavitReceiptChain>`

Currently:
```rust
pub fn verify(receipt: &str) -> Result<(i32, crate::types::Verdict)> {
    let parsed = load_receipt(receipt)?;  // Receipt
    let verdict = verifier::verify(&parsed);
    let exit_code = if verdict.accepted { 0 } else { 2 };
    Ok((exit_code, verdict))
}
```

**Change:**
```rust
pub fn verify(receipt_evidence: &Evidence<Receipt, Admitted, AffidavitReceiptChain>) 
    -> Result<(i32, crate::types::Verdict)> 
{
    // Extract inner Receipt from admitted evidence
    let parsed = receipt_evidence.inner();
    let verdict = verifier::verify(parsed);
    let exit_code = if verdict.accepted { 0 } else { 2 };
    Ok((exit_code, verdict))
}
```

**BUT:** The CLI dispatch layer (`handlers.rs`) is responsible for loading the receipt and admitting it before calling `verify`. Handler changes below.

---

### Phase 4: Type-Level Gate in Output Functions

**New design principle:** Functions that accept `Evidence<_, Admitted, _>` are certified output gates (Layer 3, ARDPRD §4). Functions that accept `Evidence<_, Raw, _>` are still-assembling phases.

#### New helper: `load_receipt_admitted()`
Add to `cli.rs`:
```rust
use wasm4pm_compat::admission::Admission;

/// Load and admit a receipt at deserialization time.
/// 
/// Wraps deserialized Receipt in Evidence<Receipt, Admitted, AffidavitReceiptChain>.
/// The BLAKE3 chain hash is re-verified during deserialization (in Receipt's custom
/// Deserialize impl), and the Admission wrapper signals that both OCEL + chain
/// constraints have been met atomically.
fn load_receipt_admitted(path: &str) -> Result<Evidence<Receipt, Admitted, AffidavitReceiptChain>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading receipt {path:?}"))?;
    let receipt: Receipt = serde_json::from_str(&text)
        .with_context(|| format!("parsing receipt {path:?}"))?;
    
    // Receipt deserialization already verified chain hash; wrap as admitted
    let admission = Admission::<Receipt, AffidavitReceiptChain>::new(receipt);
    Ok(admission.into_evidence())
}
```

#### Refactor `show()`: Load → Admit → Return
```rust
pub fn show(receipt: &str) -> Result<AdmittedReceipt> {
    let admitted = load_receipt_admitted(receipt)?;
    // Caller (handler) will extract and display
    Ok(admitted)
}
```

No change to current `show()`; it already uses `load_receipt` + `Admission::new`.
**But** update `load_receipt()` calls in `verify()` to use `load_receipt_admitted()`:

```rust
pub fn verify(receipt: &str) -> Result<(i32, crate::types::Verdict)> {
    let admitted = load_receipt_admitted(receipt)?;
    let parsed = admitted.inner();
    let verdict = verifier::verify(parsed);
    let exit_code = if verdict.accepted { 0 } else { 2 };
    Ok((exit_code, verdict))
}
```

---

### Phase 5: handlers.rs Wiring

Update handler signatures to accept Evidence where needed:

#### emit() — No change
```rust
pub fn emit(payload: String, object: Vec<String>, r#type: String) -> Result<()> {
    adapt(affidavit::cli::emit(&r#type, &object, &payload))
}
```

#### assemble() — No change to signature; just adapt return
```rust
pub fn assemble(out: Option<String>) -> Result<()> {
    let _raw_evidence = adapt(affidavit::cli::assemble(out.as_deref()))?;
    // assemble returns Evidence<Receipt, Raw, _>; we don't enforce gate here
    // (gate is for output functions like show/verify)
    Ok(())
}
```

#### verify() — Already works; cli::verify loads + admits before verifying
```rust
pub fn verify(receipt: String) -> Result<()> {
    let (code, verdict) = adapt(affidavit::cli::verify(&receipt))?;
    // ... print verdict ...
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}
```

#### show() — Already works; cli::show loads + admits
```rust
pub fn show(receipt: String) -> Result<()> {
    let admitted = adapt(affidavit::cli::show(&receipt))?;
    let parsed = admitted.into_inner();
    // ... display ...
    Ok(())
}
```

---

## Import Changes

### types.rs (COMPLETE)
Already imports from wasm4pm-compat:
```rust
use wasm4pm_compat::evidence::Evidence;
use wasm4pm_compat::state::Admitted;
```

**Add for Raw state** (used in assemble's return type):
```rust
use wasm4pm_compat::state::Raw;
```

### cli.rs (UPDATE)
```rust
use crate::types::{Blake3Hash, ObjectRef, Receipt, AdmittedReceipt, AffidavitReceiptChain};
use crate::verifier;
use anyhow::{bail, Context, Result};
use std::io::Read;
use std::path::PathBuf;
use wasm4pm_compat::admission::Admission;
use wasm4pm_compat::evidence::Evidence;
use wasm4pm_compat::state::Raw;
```

### verifier.rs (NO CHANGE)
Already codes against `&Receipt`; no Evidence wrapper needed.

### chain.rs (NO CHANGE)
`ChainAssembler::finalize()` returns `Receipt` as is. Wrapping happens one layer up in `cli.rs`.

---

## Summary Table: Changes by File

| File | Change | Rationale |
|------|--------|-----------|
| **types.rs** | Add `use wasm4pm_compat::state::Raw;` | Support `Evidence<_, Raw, _>` in assemble's return type |
| **cli.rs** | Add `load_receipt_admitted()` helper; update `verify()` to use it | Admit receipt at deserialization time; enforce gate at Layer 3 |
| **cli.rs** | Change `assemble()` return type to `Result<Evidence<Receipt, Raw, AffidavitReceiptChain>>` | Preserve unsealed phase; handler doesn't extract |
| **cli.rs** | Update imports to include `Evidence`, `Raw`, `Admission` | Support new Evidence wrapper operations |
| **handlers.rs** | No changes to signatures (already adapted) | Handlers already extract Evidence correctly in show/verify |
| **verifier.rs** | **No change** | Stays pure: codes against `&Receipt` only |
| **chain.rs** | **No change** | Wrapping is one layer up; chain stays unsealed |
| **lib.rs** | Add `AdmittedReceipt` to pub exports (optional) | Convenience alias for public API |

---

## Type Flow Diagram

```
emit()
  → OperationEvent added to .affi/working.json
  → No Evidence wrapper

assemble()
  → ChainAssembler::finalize() → Receipt (sealed, chain verified)
  → Wrap as Evidence<Receipt, Raw, AffidavitReceiptChain>
  → Persisted to disk as JSON
  → Return type: Evidence<_, Raw, _>

load_receipt_admitted()
  → Read receipt JSON from disk
  → Deserialize Receipt (chain hash re-verified in custom Deserialize)
  → Wrap as Evidence<Receipt, Admitted, AffidavitReceiptChain>
  → Return type: Evidence<_, Admitted, _>

show(), verify()
  → Call load_receipt_admitted()
  → Extract inner Receipt from Evidence<_, Admitted, _>
  → Operate on Receipt
  → Return or display

verifier::verify(&Receipt)
  → Pure function; no Evidence
  → Codes against Receipt bytes only
  → Returns Verdict
```

---

## Implementation Order

1. **Update types.rs**: Add `use wasm4pm_compat::state::Raw;`
2. **Add load_receipt_admitted() to cli.rs**: New helper function
3. **Update verify() in cli.rs**: Use `load_receipt_admitted()` and unpack admitted
4. **Update assemble() in cli.rs**: Change return type to `Result<Evidence<Receipt, Raw, AffidavitReceiptChain>>`
5. **Update cli.rs imports**: Add `Evidence`, `Raw`, `Admission`
6. **Update handlers.rs**: Minimal changes (already adapted)
7. **Update lib.rs (optional)**: Export `AdmittedReceipt` for public API

---

## Tests to Verify

1. **emit + assemble round-trip**: Working events → Evidence<_, Raw, _>
2. **Load + admit**: Persisted receipt → Evidence<_, Admitted, _>
3. **Type gate enforcement**: Handler can only unpack admitted evidence
4. **Chain integrity post-admit**: Deserialization re-verification still works
5. **Forged receipt rejection**: JSON tampering still caught at deserialization

---

## Leverage from wasm4pm-compat

- **Evidence<T, S, W>**: Generic wrapper (already used in show())
- **Admitted state**: Signals passage of all admission gates atomically
- **Raw state**: Intermediate unsealed phase (for assemble phase)
- **Admission::new(receipt)**: Wraps value into admitted evidence
- **evidence.inner()**: Unpacks inner value from any state
- **evidence.into_inner()**: Consumes and extracts inner value

No custom witness logic needed; `AffidavitReceiptChain` is a zero-cost marker.
