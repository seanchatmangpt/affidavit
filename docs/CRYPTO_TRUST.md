# Cryptographic Trust Plane

Affidavit owns cryptographic evidence and standing. It does not own consequence authorization.

The boundary is:

Affidavit proof -> SA2A authority decision -> BRCE/actuator consequence -> Affidavit receipt/evidence.

## Ownership

Affidavit owns the public and durable semantics needed to verify cryptographic evidence:

- key identifiers and public verification material
- algorithm identifiers and crypto profiles
- custodian and device metadata
- key generations and revocation epochs
- deterministic, domain-separated signed material
- signature envelopes
- signature verification
- replay admission keyed by (kid, nonce)
- cryptographic standing returned only after verification
- migration and algorithm-agility vocabulary

Private signing capability does not need to live inside Affidavit. SigningProvider is the custody seam for Secure Enclave, TPM, HSM, KMS, remote-signing, and software implementations. KeyRecord deliberately contains public verification material only.

SA2A and BRCE remain responsible for interpreting verified standing as input to an authorization decision. VerifiedSignature is therefore evidence, not an authority grant.

## Signed material

The v1 profile is affidavit/crypto-standing/v1.

A provider signs the exact bytes:

AFFIDAVIT-CRYPTO-STANDING-v1\0 || canonical_json(SigningMaterial)

SigningMaterial binds:

- profile
- algorithm
- kid
- exact subject digest
- principal
- policy epoch
- revocation epoch
- key generation
- nonce
- not-before
- expiry
- audience

Changing any of those coordinates changes the signed bytes. The JCS serializer is the dedicated serde_jcs RFC 8785 implementation rather than Affidavit's generic sorted-key JSON helper. Signed integer coordinates are refused above 2^53-1 so Swift/Elixir/JavaScript verifiers cannot silently round them. Consumers must independently supply the exact expected subject digest and audience when verifying.

## Key lifecycle

KeyRegistry is deterministic and serializable. A KeyRecord contains no private key. It binds a kid to algorithm, public verification bytes, custodian identity, custody class, generation, revocation epoch, validity window, and active/revoked state.

Revocation epochs are monotonic. Verification refuses a revoked key or any envelope whose generation or revocation epoch differs from the registry.

The registry type itself does not claim durability. A production host must persist it in a store whose durability and compare-and-set guarantees match the deployment's assurance profile.

## Replay

NonceLedger admits each (kid, nonce) once. The nonce is recorded only after every structural check and the cryptographic signature verify.

The in-process type is a reference implementation of the replay rule, not a production durability claim. A C2/C3 deployment must back the same rule with durable atomic state so a restart cannot reopen a consumed nonce.

## Post-quantum profile

The pqc Cargo feature enables the built-in ML-DSA-65 verifier through RustCrypto ml-dsa. The previous post-quantum module used forgeable BLAKE3 placeholders named after Dilithium/Kyber; that implementation has been removed rather than retained as a compatibility path.

ES256 and SLH-DSA-SHA2-128s are present in the algorithm vocabulary so provider and wire formats do not need redesign when those verifier adapters are admitted. The built-in verifier currently refuses them as unavailable; naming an algorithm is not evidence that it executed.

The upstream RustCrypto implementation is a cryptographic dependency, not an Affidavit reimplementation. Its own audit/assurance status remains part of the deployment evidence.

## Consumer contract

A consumer should:

1. Recompute the exact subject/effect digest from its admitted durable state.
2. Construct VerificationContext with that digest, expected audience, current logical time, and minimum policy epoch.
3. Call verify_and_record against the current key registry and replay state.
4. Treat VerifiedSignature as cryptographic standing only.
5. Apply its own authorization policy and consequence boundary.
6. Return the observed result to Affidavit as evidence/receipt material.

This prevents the key registry from becoming an ambient authority service: proof remains separate from permission.

## Failure semantics

The boundary is fail-closed and typed. Examples include unknown key, revoked key, algorithm mismatch, generation mismatch, revocation-epoch mismatch, stale policy epoch, replay, audience mismatch, subject mismatch, invalid signature encoding, invalid signature, and verifier unavailable.

Unsupported algorithms are refused. They never silently fall back to another verifier.

## Verification

The crypto-trust CI lane exercises the root library with the pqc feature, including real ML-DSA-65 sign/verify round trips and negative tests for signature mutation, wrong subject, replay, and revocation. Formatting, Clippy, doctests, and the root library tests remain part of the gate.
