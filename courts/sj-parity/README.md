# SJ projection parity court (register row 44)

Standalone court crate: it compares affidavit receipts against the REAL
`clnrm-core` 26.5.x `SjReceipt` projection. `clnrm-core` 26.5.x is not on
crates.io, so the court needs a local clnrm checkout and cannot live in the root
manifest without breaking the build on every other host.

```bash
cd courts/sj-parity
# edit the clnrm-sj-core path in Cargo.toml if your checkout is elsewhere
cargo test
```

Without the checkout the court is `UNSUPPORTED(no-clnrm-checkout)`, never a
fake pass.
