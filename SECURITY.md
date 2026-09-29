# Security Policy

## Supported Versions

This project uses CalVer (`YY.M.patch`). Only the latest released version
receives security fixes.

| Version        | Supported          |
| -------------- | ------------------ |
| latest release | :white_check_mark: |
| older          | :x:                |

## Reporting a Vulnerability

Please **do not** open a public issue for security problems.

Report privately to **xpointsh@gmail.com** with:

- a description of the issue and its impact,
- steps to reproduce (a minimal proof-of-concept if possible),
- any suggested remediation.

You can expect an acknowledgement within **5 business days**. Once a fix is
available we will coordinate disclosure and credit reporters who wish to be
named.

## Known Non-Issues

- Crates in this ecosystem use local `path` dependencies on sibling repos; a
  build that fails in isolation (without the siblings checked out alongside) is
  expected and is **not** a security issue.

## Cryptographic Posture (v26.9.28 trust plane)

Algorithms and statuses (details and standing evidence in
[`docs/CRYPTO_TRUST_PLANE.md`](docs/CRYPTO_TRUST_PLANE.md)):

| algorithm | status |
| --- | --- |
| ES256 (P-256, SHA-256, RFC 6979 deterministic) | software signing: in use; enclave signing: PARTIAL_ALIVE (see below) |
| ML-DSA-65 (FIPS 204) | software signing: in use |
| SLH-DSA-SHA2-128s (FIPS 205) | software signing: in use |
| hybrid ES256+ML-DSA-65 | in use; both halves must verify |

**Custody model.** Software keys live in process memory and are exportable by
definition; treat any host that holds a software signing key as the key. The
Secure Enclave adapter (feature `secure-enclave`, macOS) holds only a key
reference — id, label, public point — while the private key is generated inside
the enclave (`AccessibleWhenUnlockedThisDeviceOnly` + `DevicePasscode`, never
biometry) and never enters process memory. HSM/TPM custody is **not**
supported: the provider is typed `UNSUPPORTED` and refusing code paths return
typed values, never fabricated signatures. The plane itself never renders or
persists secret bytes; long-lived custody belongs in an enclave or HSM that you
operate.

**Replay and revocation.** Envelopes are `CTP-ENVELOPE-v1`: twelve fields, all
inside the signed bytes, canonicalized with JCS (RFC 8785) and digested with
domain-separated BLAKE3. Replay defense is the `(kid, nonce)` tuple over a
300-second window; every revocation advances a global revocation epoch that
signatures stamp, with a 300-second staleness grace for propagation lag. Only
`VALID` verdicts mint standing receipts; receipts are hash-sealed and refuse
tampering on load.

**What is NOT claimed.**

- No side-channel hardening claims of any kind are made for the software
  implementations (pure-Rust `p256` / `ml-dsa` / `slh-dsa`): constant-time
  behavior is whatever those libraries provide, and is not asserted here.
- Secure Enclave signing is `PARTIAL_ALIVE`: the live known-answer tests are
  `#[ignore]`-gated. Witnessed limitation: unsigned/ad-hoc binaries cannot
  persist enclave keys (macOS refuses with OSStatus -25308 / -34018
  `errSecMissingEntitlement`); a witnessed run requires an entitlement-signed
  host binary.
- There is no transparency log yet; revocation is verifier-local state
  (`RevocationList`), so the plane cannot detect key equivocation across
  verifiers.
- Test coverage caveat: at this tag the unit and end-to-end test courts have
  compile errors (recorded in the trust-plane doc's honest standing table);
  the library compiles and doctests pass, but the courts must be repaired and
  re-run before relying on the plane in production.
