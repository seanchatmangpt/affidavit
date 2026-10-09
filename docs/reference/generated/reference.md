# affidavit reference

<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-BEGIN: reference body is RIGID                -->
<!-- Every row below is rendered from queries/ast_extract.rq.      -->
<!-- Agents MUST NOT add, edit, reorder, or remove any row or      -->
<!-- table cell. Prose outside the fenced slot below is refused    -->
<!-- by the doc_quality court.                                     -->
<!-- ============================================================= -->

## Modules


### affidavit-core/src/accumulator/mmr.rs

| `MmrError` | enum | MmrError { IndexOutOfBounds { index: u64, total_leaves: u64, }, EmptyMmr, ProofLengthMismatch, PeakCountMismatch } |  |  |  |  |

| `append` | function | append(&mut self, leaf: Digest) -> u64 |  |  |  |  |

| `append_payload` | function | append_payload(&mut self, payload: &[u8]) -> u64 |  |  |  |  |

| `bag_peaks` | function | bag_peaks(peaks: &[Digest]) -> Digest |  |  |  |  |

| `hash_children` | function | hash_children(height: u32, left: &Digest, right: &Digest) -> Digest |  |  |  |  |

| `hash_leaf_payload` | function | hash_leaf_payload(payload: &[u8]) -> Digest |  |  |  |  |

| `mountain_heights` | function | mountain_heights(mut num_leaves: u64) -> Vec<u32> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `num_leaves` | function | num_leaves(&self) -> u64 |  |  |  |  |

| `peaks` | function | peaks(&self) -> Vec<Digest> |  |  |  |  |

| `prove` | function | prove(&self, leaf_index: u64) -> Result<MmrProof, MmrError> |  |  |  |  |

| `root` | function | root(&self) -> Digest |  |  |  |  |

| `verify` | function | verify(&self, leaf_digest: &Digest, expected_root: &Digest) -> bool |  |  |  |  |

| `mmr:v1:bag:` | str_key | DOMAIN_BAG = "mmr:v1:bag:" |  |  |  |  |

| `mmr:v1:leaf:` | str_key | DOMAIN_LEAF = "mmr:v1:leaf:" |  |  |  |  |

| `mmr:v1:node:` | str_key | DOMAIN_NODE = "mmr:v1:node:" |  |  |  |  |

| `MmrAccumulator` | struct | MmrAccumulator { leaves: Vec<Digest>, mountains: Vec<Vec<Vec<Digest>>>, _hasher: PhantomData<H> } |  |  |  |  |

| `MmrProof` | struct | MmrProof { pub leaf_index: u64, pub total_leaves: u64, pub siblings: Vec<Digest>, pub peaks: Vec<Digest> } |  |  |  |  |

| `MountainPeak` | struct | MountainPeak { pub height: u32, pub digest: Digest } |  |  |  |  |


### affidavit-core/src/accumulator/mmr.rs

| `MmrError` | enum | MmrError { IndexOutOfBounds { index: u64, total_leaves: u64, }, EmptyMmr, ProofLengthMismatch, PeakCountMismatch } |  |  |  |  |

| `append` | function | append(&mut self, leaf: Digest) -> u64 |  |  |  |  |

| `append_payload` | function | append_payload(&mut self, payload: &[u8]) -> u64 |  |  |  |  |

| `bag_peaks` | function | bag_peaks(peaks: &[Digest]) -> Digest |  |  |  |  |

| `hash_children` | function | hash_children(height: u32, left: &Digest, right: &Digest) -> Digest |  |  |  |  |

| `hash_leaf_payload` | function | hash_leaf_payload(payload: &[u8]) -> Digest |  |  |  |  |

| `mountain_heights` | function | mountain_heights(mut num_leaves: u64) -> Vec<u32> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `num_leaves` | function | num_leaves(&self) -> u64 |  |  |  |  |

| `peaks` | function | peaks(&self) -> Vec<Digest> |  |  |  |  |

| `prove` | function | prove(&self, leaf_index: u64) -> Result<MmrProof, MmrError> |  |  |  |  |

| `root` | function | root(&self) -> Digest |  |  |  |  |

| `verify` | function | verify(&self, leaf_digest: &Digest, expected_root: &Digest) -> bool |  |  |  |  |

| `mmr:v1:bag:` | str_key | DOMAIN_BAG = "mmr:v1:bag:" |  |  |  |  |

| `mmr:v1:leaf:` | str_key | DOMAIN_LEAF = "mmr:v1:leaf:" |  |  |  |  |

| `mmr:v1:node:` | str_key | DOMAIN_NODE = "mmr:v1:node:" |  |  |  |  |

| `MmrAccumulator` | struct | MmrAccumulator { leaves: Vec<Digest>, mountains: Vec<Vec<Vec<Digest>>>, _hasher: PhantomData<H> } |  |  |  |  |

| `MmrProof` | struct | MmrProof { pub leaf_index: u64, pub total_leaves: u64, pub siblings: Vec<Digest>, pub peaks: Vec<Digest> } |  |  |  |  |

| `MountainPeak` | struct | MountainPeak { pub height: u32, pub digest: Digest } |  |  |  |  |


### affidavit-core/src/accumulator/mod.rs

| `mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, }` | use | mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, } |  |  |  |  |


### affidavit-core/src/accumulator/mod.rs

| `mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, }` | use | mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, } |  |  |  |  |


### affidavit-core/src/chain.rs

| `PROFILE` | const | PROFILE: &str |  |  |  |  |

| `absorb_into` | function | absorb_into(&self, state: &mut H::State) |  |  |  |  |

| `borrow` | function | borrow(&self) -> Event<'_> |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) -> Digest |  |  |  |  |

| `compute_chain_hash` | function | compute_chain_hash(events: &[Event<'_>]) -> Digest |  |  |  |  |

| `event` | function | event(mut self, event_type: &str, event_id: &str, commitment: Digest) -> Self |  |  |  |  |

| `events` | function | events(&self) -> &[OwnedEvent] |  |  |  |  |

| `finalize` | function | finalize(self) -> Receipt |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `profile` | function | profile(&self) -> &str |  |  |  |  |

| `verify` | function | verify(&self) -> Verdict |  |  |  |  |

| `affidavit-core/chain/v1` | str_key | DOMAIN = "affidavit-core/chain/v1" |  |  |  |  |

| `affidavit-core/v1` | str_key | PROFILE = "affidavit-core/v1" |  |  |  |  |

| `ChainBuilder` | struct | ChainBuilder { events: Vec<OwnedEvent>, _hasher: PhantomData<H> } |  |  |  |  |

| `Event` | struct | Event { pub seq: u64, pub event_id: &'a str, pub event_type: &'a str, pub commitment: Digest } |  |  |  |  |

| `OwnedEvent` | struct | OwnedEvent { pub seq: u64, pub event_id: String, pub event_type: String, pub commitment: Digest } |  |  |  |  |

| `Receipt` | struct | Receipt { events: Vec<OwnedEvent>, chain_hash: Digest, profile: &'static str, _seal: Seal } |  |  |  |  |

| `Seal` | struct | Seal |  |  |  |  |

| `owned::{ChainBuilder, OwnedEvent, Receipt, Seal}` | use | owned::{ChainBuilder, OwnedEvent, Receipt, Seal} |  |  |  |  |


### affidavit-core/src/chain.rs

| `PROFILE` | const | PROFILE: &str |  |  |  |  |

| `absorb_into` | function | absorb_into(&self, state: &mut H::State) |  |  |  |  |

| `borrow` | function | borrow(&self) -> Event<'_> |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) -> Digest |  |  |  |  |

| `compute_chain_hash` | function | compute_chain_hash(events: &[Event<'_>]) -> Digest |  |  |  |  |

| `event` | function | event(mut self, event_type: &str, event_id: &str, commitment: Digest) -> Self |  |  |  |  |

| `events` | function | events(&self) -> &[OwnedEvent] |  |  |  |  |

| `finalize` | function | finalize(self) -> Receipt |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `profile` | function | profile(&self) -> &str |  |  |  |  |

| `verify` | function | verify(&self) -> Verdict |  |  |  |  |

| `affidavit-core/chain/v1` | str_key | DOMAIN = "affidavit-core/chain/v1" |  |  |  |  |

| `affidavit-core/v1` | str_key | PROFILE = "affidavit-core/v1" |  |  |  |  |

| `ChainBuilder` | struct | ChainBuilder { events: Vec<OwnedEvent>, _hasher: PhantomData<H> } |  |  |  |  |

| `Event` | struct | Event { pub seq: u64, pub event_id: &'a str, pub event_type: &'a str, pub commitment: Digest } |  |  |  |  |

| `OwnedEvent` | struct | OwnedEvent { pub seq: u64, pub event_id: String, pub event_type: String, pub commitment: Digest } |  |  |  |  |

| `Receipt` | struct | Receipt { events: Vec<OwnedEvent>, chain_hash: Digest, profile: &'static str, _seal: Seal } |  |  |  |  |

| `Seal` | struct | Seal |  |  |  |  |

| `owned::{ChainBuilder, OwnedEvent, Receipt, Seal}` | use | owned::{ChainBuilder, OwnedEvent, Receipt, Seal} |  |  |  |  |


### affidavit-core/src/crypto_verify.rs

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_FIELDS` | const | ENVELOPE_FIELDS: [(&str, u32); 12] |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `Algorithm` | enum | Algorithm { Es256, HybridEs256MlDsa65, MlDsa65, SlhDsa128s } |  |  |  |  |

| `EnvelopeError` | enum | EnvelopeError { Malformed(&'static str), WrongVersion, NonCanonicalNumber(&'static str), BufferTooSmall { needed: usize, given: usize, }, NotYetValid(u64), Expired(u64) } |  |  |  |  |

| `Profile` | enum | Profile { Classical, Hybrid, Pqc } |  |  |  |  |

| `borrow` | function | borrow(&self) -> EnvelopeRef<'_> |  |  |  |  |

| `canonical_len` | function | canonical_len(&self) -> usize |  |  |  |  |

| `check_canonical` | function | check_canonical(&self) -> Result<(), EnvelopeError> |  |  |  |  |

| `from_json` | function | from_json(input: &'a [u8]) -> Result<Self, EnvelopeError> |  |  |  |  |

| `from_wire` | function | from_wire(s: &str) -> Result<Self, EnvelopeError> |  |  |  |  |

| `signing_input` | function | signing_input(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> |  |  |  |  |

| `signing_input_len` | function | signing_input_len(&self) -> usize |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) -> Result<(), EnvelopeError> |  |  |  |  |

| `wire_str` | function | wire_str(self) -> &'static str |  |  |  |  |

| `write_canonical` | function | write_canonical(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> |  |  |  |  |

| `write_signing_input` | function | write_signing_input(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V1_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V2_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `version` | str_key | ENVELOPE_FIELDS = "version" |  |  |  |  |

| `EnvelopeRef` | struct | EnvelopeRef { pub version: &'a str, pub algorithm: Algorithm, pub key_id: &'a str, pub profile: Profile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: &'a str } |  |  |  |  |

| `SignatureEnvelope` | struct | SignatureEnvelope { pub version: alloc::string::String, pub algorithm: Algorithm, pub key_id: alloc::string::String, pub profile: Profile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: alloc::string::String } |  |  |  |  |


### affidavit-core/src/crypto_verify.rs

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_FIELDS` | const | ENVELOPE_FIELDS: [(&str, u32); 12] |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `Algorithm` | enum | Algorithm { Es256, HybridEs256MlDsa65, MlDsa65, SlhDsa128s } |  |  |  |  |

| `EnvelopeError` | enum | EnvelopeError { Malformed(&'static str), WrongVersion, NonCanonicalNumber(&'static str), BufferTooSmall { needed: usize, given: usize, }, NotYetValid(u64), Expired(u64) } |  |  |  |  |

| `Profile` | enum | Profile { Classical, Hybrid, Pqc } |  |  |  |  |

| `borrow` | function | borrow(&self) -> EnvelopeRef<'_> |  |  |  |  |

| `canonical_len` | function | canonical_len(&self) -> usize |  |  |  |  |

| `check_canonical` | function | check_canonical(&self) -> Result<(), EnvelopeError> |  |  |  |  |

| `from_json` | function | from_json(input: &'a [u8]) -> Result<Self, EnvelopeError> |  |  |  |  |

| `from_wire` | function | from_wire(s: &str) -> Result<Self, EnvelopeError> |  |  |  |  |

| `signing_input` | function | signing_input(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> |  |  |  |  |

| `signing_input_len` | function | signing_input_len(&self) -> usize |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) -> Result<(), EnvelopeError> |  |  |  |  |

| `wire_str` | function | wire_str(self) -> &'static str |  |  |  |  |

| `write_canonical` | function | write_canonical(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> |  |  |  |  |

| `write_signing_input` | function | write_signing_input(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V1_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V2_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `version` | str_key | ENVELOPE_FIELDS = "version" |  |  |  |  |

| `EnvelopeRef` | struct | EnvelopeRef { pub version: &'a str, pub algorithm: Algorithm, pub key_id: &'a str, pub profile: Profile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: &'a str } |  |  |  |  |

| `SignatureEnvelope` | struct | SignatureEnvelope { pub version: alloc::string::String, pub algorithm: Algorithm, pub key_id: alloc::string::String, pub profile: Profile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: alloc::string::String } |  |  |  |  |


### affidavit-core/src/digest.rs

| `ZERO` | const | ZERO: Digest |  |  |  |  |

| `as_bytes` | function | as_bytes(&self) -> &[u8; 32] |  |  |  |  |

| `is_zero` | function | is_zero(&self) -> bool |  |  |  |  |

| `Digest` | struct | Digest { pub [u8; 32] } |  |  |  |  |

| `Fnv256` | struct |  |  |  |  |  |

| `ChainHasher` | trait |  |  |  |  |  |


### affidavit-core/src/digest.rs

| `ZERO` | const | ZERO: Digest |  |  |  |  |

| `as_bytes` | function | as_bytes(&self) -> &[u8; 32] |  |  |  |  |

| `is_zero` | function | is_zero(&self) -> bool |  |  |  |  |

| `Digest` | struct | Digest { pub [u8; 32] } |  |  |  |  |

| `Fnv256` | struct |  |  |  |  |  |

| `ChainHasher` | trait |  |  |  |  |  |


### affidavit-core/src/external_evidence.rs

| `AUTHORITY_NONE` | const | AUTHORITY_NONE: &str |  |  |  |  |

| `AUTHZEN_STANDARD` | const | AUTHZEN_STANDARD: &str |  |  |  |  |

| `CONSEQUENCE_EVIDENCE_ONLY` | const | CONSEQUENCE_EVIDENCE_ONLY: &str |  |  |  |  |

| `SPIFFE_PREFIX` | const | SPIFFE_PREFIX: &str |  |  |  |  |

| `EvidenceError` | enum | EvidenceError { EmptyField(&'static str), InvalidSpiffeScheme, MissingTrustDomain, InvalidSpiffeAuthority, SpiffeQueryOrFragment, SpiffeIdMismatch, TrustDomainMismatch, WorkloadNotVerified, JwtNotAdmitted, InvalidPolicyDecisionPoint, PolicyDecisionPointMismatch, PrincipalMismatch, EffectMismatch } |  |  |  |  |

| `SvidType` | enum | SvidType { X509, Jwt } |  |  |  |  |

| `admit_authzen_evidence` | function | admit_authzen_evidence( request: AuthZenRequestRef<'_>, decision: AuthZenDecisionEvidenceRef<'_>, expected_policy_decision_point: &str, expected_principal: &str, expected_effect_digest: &str, ) -> Result<(), EvidenceError> |  |  |  |  |

| `admit_workload_identity` | function | admit_workload_identity( evidence: WorkloadIdentityEvidenceRef<'_>, expected_spiffe_id: &str, expected_trust_domain: &str, allow_jwt: bool, ) -> Result<(), EvidenceError> |  |  |  |  |

| `code` | function | code(self) -> &'static str |  |  |  |  |

| `parse` | function | parse(raw: &'a str) -> Result<Self, EvidenceError> |  |  |  |  |

| `EVIDENCE_ONLY` | str_key | CONSEQUENCE_EVIDENCE_ONLY = "EVIDENCE_ONLY" |  |  |  |  |

| `NONE` | str_key | AUTHORITY_NONE = "NONE" |  |  |  |  |

| `OpenID AuthZEN Authorization API 1.0` | str_key | AUTHZEN_STANDARD = "OpenID AuthZEN Authorization API 1.0" |  |  |  |  |

| `spiffe://` | str_key | SPIFFE_PREFIX = "spiffe://" |  |  |  |  |

| `AuthZenActionRef` | struct | AuthZenActionRef { pub name: &'a str } |  |  |  |  |

| `AuthZenDecisionEvidenceRef` | struct | AuthZenDecisionEvidenceRef { pub decision: bool, pub policy_decision_point: &'a str } |  |  |  |  |

| `AuthZenEntityRef` | struct | AuthZenEntityRef { pub entity_type: &'a str, pub id: &'a str } |  |  |  |  |

| `AuthZenRequestRef` | struct | AuthZenRequestRef { pub subject: AuthZenEntityRef<'a>, pub action: AuthZenActionRef<'a>, pub resource: AuthZenEntityRef<'a> } |  |  |  |  |

| `SpiffeIdRef` | struct | SpiffeIdRef { pub uri: &'a str, pub trust_domain: &'a str, pub path: &'a str } |  |  |  |  |

| `WorkloadIdentityEvidenceRef` | struct | WorkloadIdentityEvidenceRef { pub identity: SpiffeIdRef<'a>, pub svid_type: SvidType, pub bundle_digest: &'a str, pub verified: bool } |  |  |  |  |


### affidavit-core/src/external_evidence.rs

| `AUTHORITY_NONE` | const | AUTHORITY_NONE: &str |  |  |  |  |

| `AUTHZEN_STANDARD` | const | AUTHZEN_STANDARD: &str |  |  |  |  |

| `CONSEQUENCE_EVIDENCE_ONLY` | const | CONSEQUENCE_EVIDENCE_ONLY: &str |  |  |  |  |

| `SPIFFE_PREFIX` | const | SPIFFE_PREFIX: &str |  |  |  |  |

| `EvidenceError` | enum | EvidenceError { EmptyField(&'static str), InvalidSpiffeScheme, MissingTrustDomain, InvalidSpiffeAuthority, SpiffeQueryOrFragment, SpiffeIdMismatch, TrustDomainMismatch, WorkloadNotVerified, JwtNotAdmitted, InvalidPolicyDecisionPoint, PolicyDecisionPointMismatch, PrincipalMismatch, EffectMismatch } |  |  |  |  |

| `SvidType` | enum | SvidType { X509, Jwt } |  |  |  |  |

| `admit_authzen_evidence` | function | admit_authzen_evidence( request: AuthZenRequestRef<'_>, decision: AuthZenDecisionEvidenceRef<'_>, expected_policy_decision_point: &str, expected_principal: &str, expected_effect_digest: &str, ) -> Result<(), EvidenceError> |  |  |  |  |

| `admit_workload_identity` | function | admit_workload_identity( evidence: WorkloadIdentityEvidenceRef<'_>, expected_spiffe_id: &str, expected_trust_domain: &str, allow_jwt: bool, ) -> Result<(), EvidenceError> |  |  |  |  |

| `code` | function | code(self) -> &'static str |  |  |  |  |

| `parse` | function | parse(raw: &'a str) -> Result<Self, EvidenceError> |  |  |  |  |

| `EVIDENCE_ONLY` | str_key | CONSEQUENCE_EVIDENCE_ONLY = "EVIDENCE_ONLY" |  |  |  |  |

| `NONE` | str_key | AUTHORITY_NONE = "NONE" |  |  |  |  |

| `OpenID AuthZEN Authorization API 1.0` | str_key | AUTHZEN_STANDARD = "OpenID AuthZEN Authorization API 1.0" |  |  |  |  |

| `spiffe://` | str_key | SPIFFE_PREFIX = "spiffe://" |  |  |  |  |

| `AuthZenActionRef` | struct | AuthZenActionRef { pub name: &'a str } |  |  |  |  |

| `AuthZenDecisionEvidenceRef` | struct | AuthZenDecisionEvidenceRef { pub decision: bool, pub policy_decision_point: &'a str } |  |  |  |  |

| `AuthZenEntityRef` | struct | AuthZenEntityRef { pub entity_type: &'a str, pub id: &'a str } |  |  |  |  |

| `AuthZenRequestRef` | struct | AuthZenRequestRef { pub subject: AuthZenEntityRef<'a>, pub action: AuthZenActionRef<'a>, pub resource: AuthZenEntityRef<'a> } |  |  |  |  |

| `SpiffeIdRef` | struct | SpiffeIdRef { pub uri: &'a str, pub trust_domain: &'a str, pub path: &'a str } |  |  |  |  |

| `WorkloadIdentityEvidenceRef` | struct | WorkloadIdentityEvidenceRef { pub identity: SpiffeIdRef<'a>, pub svid_type: SvidType, pub bundle_digest: &'a str, pub verified: bool } |  |  |  |  |


### affidavit-core/src/lib.rs

| `accumulator::{MmrAccumulator, MmrError, MmrProof}` | use | accumulator::{MmrAccumulator, MmrError, MmrProof} |  |  |  |  |

| `chain::{ChainBuilder, OwnedEvent, Receipt}` | use | chain::{ChainBuilder, OwnedEvent, Receipt} |  |  |  |  |

| `chain::{compute_chain_hash, Event, PROFILE}` | use | chain::{compute_chain_hash, Event, PROFILE} |  |  |  |  |

| `crypto_verify::SignatureEnvelope` | use | crypto_verify::SignatureEnvelope |  |  |  |  |

| `crypto_verify::{ Algorithm, EnvelopeError, EnvelopeRef, Profile, DOMAIN_TAG, ENVELOPE_FIELDS, ENVELOPE_VERSION, }` | use | crypto_verify::{ Algorithm, EnvelopeError, EnvelopeRef, Profile, DOMAIN_TAG, ENVELOPE_FIELDS, ENVELOPE_VERSION, } |  |  |  |  |

| `digest::{ChainHasher, Digest, Fnv256}` | use | digest::{ChainHasher, Digest, Fnv256} |  |  |  |  |

| `external_evidence::{ admit_authzen_evidence, admit_workload_identity, AuthZenActionRef, AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef, SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD, CONSEQUENCE_EVIDENCE_ONLY, SPIFFE_PREFIX, }` | use | external_evidence::{ admit_authzen_evidence, admit_workload_identity, AuthZenActionRef, AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef, SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD, CONSEQUENCE_EVIDENCE_ONLY, SPIFFE_PREFIX, } |  |  |  |  |

| `mining::{DirectlyFollowsGraph, Trace}` | use | mining::{DirectlyFollowsGraph, Trace} |  |  |  |  |

| `verifier::{verify, RejectReason, Verdict}` | use | verifier::{verify, RejectReason, Verdict} |  |  |  |  |


### affidavit-core/src/lib.rs

| `accumulator::{MmrAccumulator, MmrError, MmrProof}` | use | accumulator::{MmrAccumulator, MmrError, MmrProof} |  |  |  |  |

| `chain::{ChainBuilder, OwnedEvent, Receipt}` | use | chain::{ChainBuilder, OwnedEvent, Receipt} |  |  |  |  |

| `chain::{compute_chain_hash, Event, PROFILE}` | use | chain::{compute_chain_hash, Event, PROFILE} |  |  |  |  |

| `crypto_verify::SignatureEnvelope` | use | crypto_verify::SignatureEnvelope |  |  |  |  |

| `crypto_verify::{ Algorithm, EnvelopeError, EnvelopeRef, Profile, DOMAIN_TAG, ENVELOPE_FIELDS, ENVELOPE_VERSION, }` | use | crypto_verify::{ Algorithm, EnvelopeError, EnvelopeRef, Profile, DOMAIN_TAG, ENVELOPE_FIELDS, ENVELOPE_VERSION, } |  |  |  |  |

| `digest::{ChainHasher, Digest, Fnv256}` | use | digest::{ChainHasher, Digest, Fnv256} |  |  |  |  |

| `external_evidence::{ admit_authzen_evidence, admit_workload_identity, AuthZenActionRef, AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef, SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD, CONSEQUENCE_EVIDENCE_ONLY, SPIFFE_PREFIX, }` | use | external_evidence::{ admit_authzen_evidence, admit_workload_identity, AuthZenActionRef, AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef, SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD, CONSEQUENCE_EVIDENCE_ONLY, SPIFFE_PREFIX, } |  |  |  |  |

| `mining::{DirectlyFollowsGraph, Trace}` | use | mining::{DirectlyFollowsGraph, Trace} |  |  |  |  |

| `verifier::{verify, RejectReason, Verdict}` | use | verifier::{verify, RejectReason, Verdict} |  |  |  |  |


### affidavit-core/src/mining/conformance.rs

| `ConformanceVerdict` | enum | ConformanceVerdict { Conformant, NonConformant } |  |  |  |  |

| `fitness` | function | fitness(&self) -> f64 |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) -> bool |  |  |  |  |

| `replay` | function | replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) -> ConformanceResult |  |  |  |  |

| `verdict` | function | verdict(&self) -> ConformanceVerdict |  |  |  |  |

| `ConformanceResult` | struct | ConformanceResult { pub legal_moves: u64, pub total_moves: u64, pub start_ok: bool, pub end_ok: bool, pub first_violation: Option<usize> } |  |  |  |  |


### affidavit-core/src/mining/conformance.rs

| `ConformanceVerdict` | enum | ConformanceVerdict { Conformant, NonConformant } |  |  |  |  |

| `fitness` | function | fitness(&self) -> f64 |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) -> bool |  |  |  |  |

| `replay` | function | replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) -> ConformanceResult |  |  |  |  |

| `verdict` | function | verdict(&self) -> ConformanceVerdict |  |  |  |  |

| `ConformanceResult` | struct | ConformanceResult { pub legal_moves: u64, pub total_moves: u64, pub start_ok: bool, pub end_ok: bool, pub first_violation: Option<usize> } |  |  |  |  |


### affidavit-core/src/mining/footprint.rs

| `AlphaRelation` | enum | AlphaRelation { Causality, ReverseCausality, Parallel, Choice } |  |  |  |  |

| `activities` | function | activities(&self) -> &[String] |  |  |  |  |

| `from_dfg` | function | from_dfg(dfg: &DirectlyFollowsGraph) -> Self |  |  |  |  |

| `relation` | function | relation(&self, a: &str, b: &str) -> AlphaRelation |  |  |  |  |

| `Footprint` | struct | Footprint { activities: Vec<String>, follows: BTreeSet<(String, String)> } |  |  |  |  |


### affidavit-core/src/mining/footprint.rs

| `AlphaRelation` | enum | AlphaRelation { Causality, ReverseCausality, Parallel, Choice } |  |  |  |  |

| `activities` | function | activities(&self) -> &[String] |  |  |  |  |

| `from_dfg` | function | from_dfg(dfg: &DirectlyFollowsGraph) -> Self |  |  |  |  |

| `relation` | function | relation(&self, a: &str, b: &str) -> AlphaRelation |  |  |  |  |

| `Footprint` | struct | Footprint { activities: Vec<String>, follows: BTreeSet<(String, String)> } |  |  |  |  |


### affidavit-core/src/mining/mod.rs

| `activity_list` | function | activity_list(&self) -> Vec<&str> |  |  |  |  |

| `add_trace` | function | add_trace(&mut self, trace: &Trace<'_>) |  |  |  |  |

| `directly_follows` | function | directly_follows(&self, a: &str, b: &str) -> u64 |  |  |  |  |

| `discover` | function | discover(traces: &[Trace<'_>]) -> Self |  |  |  |  |

| `follows` | function | follows(&self, a: &str, b: &str) -> bool |  |  |  |  |

| `from_activities` | function | from_activities(activities: &[&'a str]) -> Self |  |  |  |  |

| `from_events` | function | from_events(events: &[Event<'a>]) -> Self |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_end` | function | is_end(&self, a: &str) -> bool |  |  |  |  |

| `is_start` | function | is_start(&self, a: &str) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `DirectlyFollowsGraph` | struct | DirectlyFollowsGraph { pub edges: BTreeMap<(String, String), u64>, pub activities: BTreeSet<String>, pub start: BTreeMap<String, u64>, pub end: BTreeMap<String, u64>, pub trace_count: u64 } |  |  |  |  |

| `Trace` | struct | Trace { pub activities: Vec<&'a str> } |  |  |  |  |

| `conformance::{replay, ConformanceResult, ConformanceVerdict}` | use | conformance::{replay, ConformanceResult, ConformanceVerdict} |  |  |  |  |

| `footprint::{AlphaRelation, Footprint}` | use | footprint::{AlphaRelation, Footprint} |  |  |  |  |

| `stats::LogStatistics` | use | stats::LogStatistics |  |  |  |  |


### affidavit-core/src/mining/mod.rs

| `activity_list` | function | activity_list(&self) -> Vec<&str> |  |  |  |  |

| `add_trace` | function | add_trace(&mut self, trace: &Trace<'_>) |  |  |  |  |

| `directly_follows` | function | directly_follows(&self, a: &str, b: &str) -> u64 |  |  |  |  |

| `discover` | function | discover(traces: &[Trace<'_>]) -> Self |  |  |  |  |

| `follows` | function | follows(&self, a: &str, b: &str) -> bool |  |  |  |  |

| `from_activities` | function | from_activities(activities: &[&'a str]) -> Self |  |  |  |  |

| `from_events` | function | from_events(events: &[Event<'a>]) -> Self |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_end` | function | is_end(&self, a: &str) -> bool |  |  |  |  |

| `is_start` | function | is_start(&self, a: &str) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `DirectlyFollowsGraph` | struct | DirectlyFollowsGraph { pub edges: BTreeMap<(String, String), u64>, pub activities: BTreeSet<String>, pub start: BTreeMap<String, u64>, pub end: BTreeMap<String, u64>, pub trace_count: u64 } |  |  |  |  |

| `Trace` | struct | Trace { pub activities: Vec<&'a str> } |  |  |  |  |

| `conformance::{replay, ConformanceResult, ConformanceVerdict}` | use | conformance::{replay, ConformanceResult, ConformanceVerdict} |  |  |  |  |

| `footprint::{AlphaRelation, Footprint}` | use | footprint::{AlphaRelation, Footprint} |  |  |  |  |

| `stats::LogStatistics` | use | stats::LogStatistics |  |  |  |  |


### affidavit-core/src/mining/stats.rs

| `distinct_activities` | function | distinct_activities(&self) -> usize |  |  |  |  |

| `distinct_variants` | function | distinct_variants(&self) -> usize |  |  |  |  |

| `from_traces` | function | from_traces(traces: &[Trace<'_>]) -> Self |  |  |  |  |

| `most_frequent_variant` | function | most_frequent_variant(&self) -> Option<(&[String], u64)> |  |  |  |  |

| `LogStatistics` | struct | LogStatistics { pub event_count: u64, pub trace_count: u64, pub activity_frequency: BTreeMap<String, u64>, pub variants: BTreeMap<Vec<String>, u64> } |  |  |  |  |


### affidavit-core/src/mining/stats.rs

| `distinct_activities` | function | distinct_activities(&self) -> usize |  |  |  |  |

| `distinct_variants` | function | distinct_variants(&self) -> usize |  |  |  |  |

| `from_traces` | function | from_traces(traces: &[Trace<'_>]) -> Self |  |  |  |  |

| `most_frequent_variant` | function | most_frequent_variant(&self) -> Option<(&[String], u64)> |  |  |  |  |

| `LogStatistics` | struct | LogStatistics { pub event_count: u64, pub trace_count: u64, pub activity_frequency: BTreeMap<String, u64>, pub variants: BTreeMap<Vec<String>, u64> } |  |  |  |  |


### affidavit-core/src/verifier.rs

| `RejectReason` | enum | RejectReason { WrongProfile, ChainHashMismatch, SeqNotContiguous { index: usize, found: u64, }, DuplicateEventId, EmptyEventType { index: usize, }, ZeroCommitment { index: usize, } } |  |  |  |  |

| `Verdict` | enum | Verdict { Accept, Reject(RejectReason) } |  |  |  |  |

| `is_accept` | function | is_accept(&self) -> bool |  |  |  |  |

| `reason` | function | reason(&self) -> Option<RejectReason> |  |  |  |  |

| `verify` | function | verify( events: &[Event<'_>], chain_hash: &crate::digest::Digest, profile: &str, ) -> Verdict |  |  |  |  |


### affidavit-core/src/verifier.rs

| `RejectReason` | enum | RejectReason { WrongProfile, ChainHashMismatch, SeqNotContiguous { index: usize, found: u64, }, DuplicateEventId, EmptyEventType { index: usize, }, ZeroCommitment { index: usize, } } |  |  |  |  |

| `Verdict` | enum | Verdict { Accept, Reject(RejectReason) } |  |  |  |  |

| `is_accept` | function | is_accept(&self) -> bool |  |  |  |  |

| `reason` | function | reason(&self) -> Option<RejectReason> |  |  |  |  |

| `verify` | function | verify( events: &[Event<'_>], chain_hash: &crate::digest::Digest, profile: &str, ) -> Verdict |  |  |  |  |


### affidavit-wasm/src/abi.rs

| `call` | function | call(request: &[u8]) -> Vec<u8> |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) -> Vec<u8> |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() -> Vec<u8> |  |  |  |  |

| `AbiError` | struct | AbiError { pub code: &'static str, pub message: String, pub details: Option<Value> } |  |  |  |  |

| `crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES}` | use | crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES} |  |  |  |  |


### affidavit-wasm/src/abi.rs

| `call` | function | call(request: &[u8]) -> Vec<u8> |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) -> Vec<u8> |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() -> Vec<u8> |  |  |  |  |

| `AbiError` | struct | AbiError { pub code: &'static str, pub message: String, pub details: Option<Value> } |  |  |  |  |

| `crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES}` | use | crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES} |  |  |  |  |


### affidavit-wasm/src/abi_meta.rs

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `CRATE_NAME` | const | CRATE_NAME: &str |  |  |  |  |

| `ERROR_CODES` | const | ERROR_CODES: &[&str] |  |  |  |  |

| `EXPORT_PREFIX` | const | EXPORT_PREFIX: &str |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `OPS` | const | OPS: &[&str] |  |  |  |  |

| `af` | str_key | EXPORT_PREFIX = "af" |  |  |  |  |

| `affidavit-wasm` | str_key | CRATE_NAME = "affidavit-wasm" |  |  |  |  |

| `bad_json` | str_key | ERROR_CODES = "bad_json" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |


### affidavit-wasm/src/abi_meta.rs

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `CRATE_NAME` | const | CRATE_NAME: &str |  |  |  |  |

| `ERROR_CODES` | const | ERROR_CODES: &[&str] |  |  |  |  |

| `EXPORT_PREFIX` | const | EXPORT_PREFIX: &str |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `OPS` | const | OPS: &[&str] |  |  |  |  |

| `af` | str_key | EXPORT_PREFIX = "af" |  |  |  |  |

| `affidavit-wasm` | str_key | CRATE_NAME = "affidavit-wasm" |  |  |  |  |

| `bad_json` | str_key | ERROR_CODES = "bad_json" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |


### affidavit-wasm/src/advanced.rs

| `op_jcs_canonicalize` | function | op_jcs_canonicalize(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_range_proof_verify` | function | op_range_proof_verify(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_smt_absence_verify` | function | op_smt_absence_verify(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |


### affidavit-wasm/src/advanced.rs

| `op_jcs_canonicalize` | function | op_jcs_canonicalize(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_range_proof_verify` | function | op_range_proof_verify(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_smt_absence_verify` | function | op_smt_absence_verify(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |


### affidavit-wasm/src/crypto.rs

| `SignatureInputError` | enum | SignatureInputError { Envelope(affidavit_core::crypto_verify::EnvelopeError), BadExpectedHex } |  |  |  |  |

| `code` | function | code(&self) -> &'static str |  |  |  |  |

| `derive_subject_digest` | function | derive_subject_digest(domain_tag: &str, subject: &[u8]) -> [u8; 32] |  |  |  |  |

| `verify_signature_input` | function | verify_signature_input( envelope_json: &[u8], expected_signing_input_hex: &str, ) -> Result<bool, SignatureInputError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f` | str_key | KAT_DIGEST_HEX = "3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `affidavit-paid-delivery/v1` | str_key | KAT_TAG = "affidavit-paid-delivery/v1" |  |  |  |  |

| `paid-delivery:evt-001` | str_key | KAT_SUBJECT = "paid-delivery:evt-001" |  |  |  |  |


### affidavit-wasm/src/crypto.rs

| `SignatureInputError` | enum | SignatureInputError { Envelope(affidavit_core::crypto_verify::EnvelopeError), BadExpectedHex } |  |  |  |  |

| `code` | function | code(&self) -> &'static str |  |  |  |  |

| `derive_subject_digest` | function | derive_subject_digest(domain_tag: &str, subject: &[u8]) -> [u8; 32] |  |  |  |  |

| `verify_signature_input` | function | verify_signature_input( envelope_json: &[u8], expected_signing_input_hex: &str, ) -> Result<bool, SignatureInputError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f` | str_key | KAT_DIGEST_HEX = "3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `affidavit-paid-delivery/v1` | str_key | KAT_TAG = "affidavit-paid-delivery/v1" |  |  |  |  |

| `paid-delivery:evt-001` | str_key | KAT_SUBJECT = "paid-delivery:evt-001" |  |  |  |  |


### affidavit-wasm/src/ffi.rs

| `af_abi_version` | function | af_abi_version() -> u32 |  |  |  |  |

| `af_alloc` | function | af_alloc(len: u32) -> *mut u8 |  |  |  |  |

| `af_call` | function | af_call(ptr: *mut u8, len: u32) -> u64 |  |  |  |  |

| `af_free` | function | af_free(ptr: *mut u8, len: u32) |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) -> *mut u8 |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) -> Vec<u8> |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |


### affidavit-wasm/src/ffi.rs

| `af_abi_version` | function | af_abi_version() -> u32 |  |  |  |  |

| `af_alloc` | function | af_alloc(len: u32) -> *mut u8 |  |  |  |  |

| `af_call` | function | af_call(ptr: *mut u8, len: u32) -> u64 |  |  |  |  |

| `af_free` | function | af_free(ptr: *mut u8, len: u32) |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) -> *mut u8 |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) -> Vec<u8> |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |


### affidavit-wasm/src/receipt.rs

| `FORMAT_VERSION` | const | FORMAT_VERSION: &str |  |  |  |  |

| `GENESIS_SEED` | const | GENESIS_SEED: &str |  |  |  |  |

| `blake3_hex` | function | blake3_hex(bytes: &[u8]) -> String |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) -> Result<String, serde_json::Error> |  |  |  |  |

| `finish` | function | finish(outcomes: Vec<CheckOutcome>) -> Verdict |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) -> Result<String, serde_json::Error> |  |  |  |  |

| `seal` | function | seal(events: Vec<OperationEvent>) -> Result<Receipt, serde_json::Error> |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) -> Verdict |  |  |  |  |

| `affidavit-v` | str_key | GENESIS_SEED = "affidavit-v" |  |  |  |  |

| `core/v1` | str_key | FORMAT_VERSION = "core/v1" |  |  |  |  |

| `CheckOutcome` | struct | CheckOutcome { pub stage: String, pub passed: bool, pub detail: String } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub obj_type: String, pub qualifier: Option<String> } |  |  |  |  |

| `OperationEvent` | struct | OperationEvent { pub id: String, pub seq: u64, pub event_type: String, pub objects: Vec<ObjectRef>, pub payload_commitment: String } |  |  |  |  |

| `Receipt` | struct | Receipt { pub format_version: String, pub events: Vec<OperationEvent>, pub chain_hash: String } |  |  |  |  |

| `Verdict` | struct | Verdict { pub accepted: bool, pub profile: &'static str, pub outcomes: Vec<CheckOutcome>, pub reason: String } |  |  |  |  |


### affidavit-wasm/src/receipt.rs

| `FORMAT_VERSION` | const | FORMAT_VERSION: &str |  |  |  |  |

| `GENESIS_SEED` | const | GENESIS_SEED: &str |  |  |  |  |

| `blake3_hex` | function | blake3_hex(bytes: &[u8]) -> String |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) -> Result<String, serde_json::Error> |  |  |  |  |

| `finish` | function | finish(outcomes: Vec<CheckOutcome>) -> Verdict |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) -> Result<String, serde_json::Error> |  |  |  |  |

| `seal` | function | seal(events: Vec<OperationEvent>) -> Result<Receipt, serde_json::Error> |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) -> Verdict |  |  |  |  |

| `affidavit-v` | str_key | GENESIS_SEED = "affidavit-v" |  |  |  |  |

| `core/v1` | str_key | FORMAT_VERSION = "core/v1" |  |  |  |  |

| `CheckOutcome` | struct | CheckOutcome { pub stage: String, pub passed: bool, pub detail: String } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub obj_type: String, pub qualifier: Option<String> } |  |  |  |  |

| `OperationEvent` | struct | OperationEvent { pub id: String, pub seq: u64, pub event_type: String, pub objects: Vec<ObjectRef>, pub payload_commitment: String } |  |  |  |  |

| `Receipt` | struct | Receipt { pub format_version: String, pub events: Vec<OperationEvent>, pub chain_hash: String } |  |  |  |  |

| `Verdict` | struct | Verdict { pub accepted: bool, pub profile: &'static str, pub outcomes: Vec<CheckOutcome>, pub reason: String } |  |  |  |  |


### affidavit-wasm/src/signature.rs

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c` | str_key | ED_KEY = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c" |  |  |  |  |

| `72` | str_key | ED_INPUT = "72" |  |  |  |  |

| `92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00` | str_key | ED_SIG = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00" |  |  |  |  |


### affidavit-wasm/src/signature.rs

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c` | str_key | ED_KEY = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c" |  |  |  |  |

| `72` | str_key | ED_INPUT = "72" |  |  |  |  |

| `92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00` | str_key | ED_SIG = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00" |  |  |  |  |


### affidavit-wasm/tests/common/mod.rs

| `AFFIDAVIT_WASM` | env_key | std::env::var_os("AFFIDAVIT_WASM") |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `call` | function | call(&mut self, request: Value) -> Value |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, request: &[u8]) -> Vec<u8> |  |  |  |  |

| `memory_pages` | function | memory_pages(&self) -> u64 |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `wasm_path` | function | wasm_path() -> &'static PathBuf |  |  |  |  |

| `Host` | struct | Host { pub store: Store<WasiCtx>, pub memory: Memory, pub alloc: TypedFunc<u32, u32>, pub free: TypedFunc<(u32, u32), ()>, pub call: TypedFunc<(u32, u32), u64>, pub abi_version: TypedFunc<(), u32> } |  |  |  |  |


### affidavit-wasm/tests/common/mod.rs

| `AFFIDAVIT_WASM` | env_key | std::env::var_os("AFFIDAVIT_WASM") |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `call` | function | call(&mut self, request: Value) -> Value |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, request: &[u8]) -> Vec<u8> |  |  |  |  |

| `memory_pages` | function | memory_pages(&self) -> u64 |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `wasm_path` | function | wasm_path() -> &'static PathBuf |  |  |  |  |

| `Host` | struct | Host { pub store: Store<WasiCtx>, pub memory: Memory, pub alloc: TypedFunc<u32, u32>, pub free: TypedFunc<(u32, u32), ()>, pub call: TypedFunc<(u32, u32), u64>, pub abi_version: TypedFunc<(), u32> } |  |  |  |  |


### affidavit-wasm/tests/crypto_abi.rs

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V1_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V2_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |


### affidavit-wasm/tests/crypto_abi.rs

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V1_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V2_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |


### affidavit-wasm/tests/op_differential.rs

| `../registry/op-examples.json` | str_key | EXAMPLES = "../registry/op-examples.json" |  |  |  |  |


### affidavit-wasm/tests/op_differential.rs

| `../registry/op-examples.json` | str_key | EXAMPLES = "../registry/op-examples.json" |  |  |  |  |


### affidavit-wasm/tests/registry_artifacts.rs

| `AFFIDAVIT_WASM` | env_key | std::env::var_os("AFFIDAVIT_WASM") |  |  |  |  |

| `../registry/ARTIFACTS.sha256` | str_key | ARTIFACTS = "../registry/ARTIFACTS.sha256" |  |  |  |  |

| `../registry/capability-registry.json` | str_key | REGISTRY = "../registry/capability-registry.json" |  |  |  |  |


### affidavit-wasm/tests/registry_artifacts.rs

| `AFFIDAVIT_WASM` | env_key | std::env::var_os("AFFIDAVIT_WASM") |  |  |  |  |

| `../registry/ARTIFACTS.sha256` | str_key | ARTIFACTS = "../registry/ARTIFACTS.sha256" |  |  |  |  |

| `../registry/capability-registry.json` | str_key | REGISTRY = "../registry/capability-registry.json" |  |  |  |  |


### affidavit-wasm/tests/wasm_abi.rs

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | KAT_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `fixtures/golden_receipt.json` | str_key | GOLDEN = "fixtures/golden_receipt.json" |  |  |  |  |

| `fixtures/tampered_receipt.json` | str_key | TAMPERED = "fixtures/tampered_receipt.json" |  |  |  |  |


### affidavit-wasm/tests/wasm_abi.rs

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | KAT_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `fixtures/golden_receipt.json` | str_key | GOLDEN = "fixtures/golden_receipt.json" |  |  |  |  |

| `fixtures/tampered_receipt.json` | str_key | TAMPERED = "fixtures/tampered_receipt.json" |  |  |  |  |


### affidavit-web

| `build` | script | next build |  |  |  |  |

| `dev` | script | next dev |  |  |  |  |

| `lint` | script | next lint |  |  |  |  |

| `start` | script | next start |  |  |  |  |


### benches/crypto_trust_bench.rs

| `BLAKE3` | str_key | BENCH_DIGEST_ALGORITHM = "BLAKE3" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | BENCH_ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s` | str_key | BENCH_ALGORITHM_REGISTRY = "ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s" |  |  |  |  |

| `JCS-RFC8785` | str_key | BENCH_CANONICALIZATION = "JCS-RFC8785" |  |  |  |  |

| `affidavit crypto-trust-plane benchmark message` | str_key | BENCH_MESSAGE = "affidavit crypto-trust-plane benchmark message" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | BENCH_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |


### benches/variance.rs

| `create` | str_key | SEQUENTIAL_ACTIVITIES = "create" |  |  |  |  |

| `release` | str_key | INTERLEAVED_ACTIVITIES = "release" |  |  |  |  |


### praxis/crates/chatman-common/src/chain.rs

| `crate::provenance::{ content_address, genesis_seed, fold_event, recompute_chain, is_valid_digest, RollingChain, RollingHash, }` | use | crate::provenance::{ content_address, genesis_seed, fold_event, recompute_chain, is_valid_digest, RollingChain, RollingHash, } |  |  |  |  |


### praxis/crates/chatman-common/src/chain.rs

| `crate::provenance::{ content_address, genesis_seed, fold_event, recompute_chain, is_valid_digest, RollingChain, RollingHash, }` | use | crate::provenance::{ content_address, genesis_seed, fold_event, recompute_chain, is_valid_digest, RollingChain, RollingHash, } |  |  |  |  |


### praxis/crates/chatman-common/src/cli.rs

| `ColorMode` | enum | ColorMode { Auto, Always, Never } |  |  |  |  |

| `OutputFormat` | enum | OutputFormat { Json, Yaml, Text } |  |  |  |  |

| `bold` | function | bold(text: &str) -> String |  |  |  |  |

| `color_enabled` | function | color_enabled(&self) -> bool |  |  |  |  |

| `dim` | function | dim(text: &str) -> String |  |  |  |  |

| `enabled` | function | enabled(&self, is_tty: bool) -> bool |  |  |  |  |

| `green` | function | green(text: &str) -> String |  |  |  |  |

| `init` | function | init(args: &GlobalArgs, service_name: &str) -> Result<()> |  |  |  |  |

| `print_output` | function | print_output(value: &T, format: &OutputFormat, text_fn: F) -> Result<()> |  |  |  |  |

| `red` | function | red(text: &str) -> String |  |  |  |  |

| `yellow` | function | yellow(text: &str) -> String |  |  |  |  |

| `GlobalArgs` | struct | GlobalArgs { pub format: OutputFormat, pub color: ColorMode, pub verbose: u8 } |  |  |  |  |


### praxis/crates/chatman-common/src/cli.rs

| `ColorMode` | enum | ColorMode { Auto, Always, Never } |  |  |  |  |

| `OutputFormat` | enum | OutputFormat { Json, Yaml, Text } |  |  |  |  |

| `bold` | function | bold(text: &str) -> String |  |  |  |  |

| `color_enabled` | function | color_enabled(&self) -> bool |  |  |  |  |

| `dim` | function | dim(text: &str) -> String |  |  |  |  |

| `enabled` | function | enabled(&self, is_tty: bool) -> bool |  |  |  |  |

| `green` | function | green(text: &str) -> String |  |  |  |  |

| `init` | function | init(args: &GlobalArgs, service_name: &str) -> Result<()> |  |  |  |  |

| `print_output` | function | print_output(value: &T, format: &OutputFormat, text_fn: F) -> Result<()> |  |  |  |  |

| `red` | function | red(text: &str) -> String |  |  |  |  |

| `yellow` | function | yellow(text: &str) -> String |  |  |  |  |

| `GlobalArgs` | struct | GlobalArgs { pub format: OutputFormat, pub color: ColorMode, pub verbose: u8 } |  |  |  |  |


### praxis/crates/chatman-common/src/error.rs

| `Error` | enum | Error { Message(String), Io( Json( } |  |  |  |  |

| `msg` | function | msg(m: impl Into<String>) -> Self |  |  |  |  |


### praxis/crates/chatman-common/src/error.rs

| `Error` | enum | Error { Message(String), Io( Json( } |  |  |  |  |

| `msg` | function | msg(m: impl Into<String>) -> Self |  |  |  |  |


### praxis/crates/chatman-common/src/lib.rs

| `chain::RollingChain` | use | chain::RollingChain |  |  |  |  |

| `error::{Error, Result}` | use | error::{Error, Result} |  |  |  |  |


### praxis/crates/chatman-common/src/lib.rs

| `chain::RollingChain` | use | chain::RollingChain |  |  |  |  |

| `error::{Error, Result}` | use | error::{Error, Result} |  |  |  |  |


### praxis/crates/chatman-common/src/provenance.rs

| `content_address` | function | content_address(bytes: &[u8]) -> String |  |  |  |  |

| `current` | function | current(&self) -> &str |  |  |  |  |

| `finalize` | function | finalize(self) -> String |  |  |  |  |

| `fold_event` | function | fold_event(prev_hex: &str, payload: &[u8]) -> String |  |  |  |  |

| `genesis_seed` | function | genesis_seed(domain: &str) -> String |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_valid_digest` | function | is_valid_digest(s: &str) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(domain: &str) -> Self |  |  |  |  |

| `push` | function | push(&mut self, payload: &[u8]) |  |  |  |  |

| `recompute_chain` | function | recompute_chain( domain: &str, payloads: impl IntoIterator<Item = &'a [u8]>, ) -> String |  |  |  |  |

| `update` | function | update(&mut self, bytes: &[u8]) |  |  |  |  |

| `RollingChain` | struct | RollingChain { running: String, count: usize } |  |  |  |  |

| `RollingHash` | struct | RollingHash { hasher: blake3::Hasher } |  |  |  |  |


### praxis/crates/chatman-common/src/provenance.rs

| `content_address` | function | content_address(bytes: &[u8]) -> String |  |  |  |  |

| `current` | function | current(&self) -> &str |  |  |  |  |

| `finalize` | function | finalize(self) -> String |  |  |  |  |

| `fold_event` | function | fold_event(prev_hex: &str, payload: &[u8]) -> String |  |  |  |  |

| `genesis_seed` | function | genesis_seed(domain: &str) -> String |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_valid_digest` | function | is_valid_digest(s: &str) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(domain: &str) -> Self |  |  |  |  |

| `push` | function | push(&mut self, payload: &[u8]) |  |  |  |  |

| `recompute_chain` | function | recompute_chain( domain: &str, payloads: impl IntoIterator<Item = &'a [u8]>, ) -> String |  |  |  |  |

| `update` | function | update(&mut self, bytes: &[u8]) |  |  |  |  |

| `RollingChain` | struct | RollingChain { running: String, count: usize } |  |  |  |  |

| `RollingHash` | struct | RollingHash { hasher: blake3::Hasher } |  |  |  |  |


### praxis/crates/chatman-common/src/telemetry.rs

| `init_otel` | function | init_otel(service_name: &str, endpoint: &str) -> Result<()> |  |  |  |  |

| `init_tracing` | function | init_tracing(service_name: &str) -> Result<()> |  |  |  |  |

| `new` | function | new(service_name: &str) -> Result<Self> |  |  |  |  |

| `TracingGuard` | struct | TracingGuard { _private: () } |  |  |  |  |


### praxis/crates/chatman-common/src/telemetry.rs

| `init_otel` | function | init_otel(service_name: &str, endpoint: &str) -> Result<()> |  |  |  |  |

| `init_tracing` | function | init_tracing(service_name: &str) -> Result<()> |  |  |  |  |

| `new` | function | new(service_name: &str) -> Result<Self> |  |  |  |  |

| `TracingGuard` | struct | TracingGuard { _private: () } |  |  |  |  |


### praxis/crates/chatman-common/src/testkit.rs

| `UPDATE_GOLDEN` | env_key | std::env::var("UPDATE_GOLDEN") |  |  |  |  |

| `UPDATE_SNAPSHOTS` | env_key | std::env::var("UPDATE_SNAPSHOTS") |  |  |  |  |

| `assert_golden` | function | assert_golden(actual: &[u8], path: &Path) -> Result<()> |  |  |  |  |

| `assert_snapshot` | function | assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) |  |  |  |  |

| `build` | function | build(self) -> Result<TempReceipt> |  |  |  |  |

| `builder` | function | builder() -> TempReceiptBuilder |  |  |  |  |

| `chain_hash` | function | chain_hash(mut self, h: impl Into<String>) -> Self |  |  |  |  |

| `deterministic_uuid` | function | deterministic_uuid(seed: &str) -> String |  |  |  |  |

| `dir` | function | dir(&self) -> &Path |  |  |  |  |

| `event` | function | event(mut self, event: serde_json::Value) -> Self |  |  |  |  |

| `filename` | function | filename(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `format_version` | function | format_version(mut self, v: impl Into<String>) -> Self |  |  |  |  |

| `path` | function | path(&self) -> PathBuf |  |  |  |  |

| `profile` | function | profile(mut self, p: impl Into<String>) -> Self |  |  |  |  |

| `TempReceipt` | struct | TempReceipt { dir: tempfile::TempDir, filename: String } |  |  |  |  |

| `TempReceiptBuilder` | struct | TempReceiptBuilder { format_version: String, chain_hash: String, events: Vec<serde_json::Value>, profile: String, filename: String } |  |  |  |  |

| `tempfile` | use | tempfile |  |  |  |  |


### praxis/crates/chatman-common/src/testkit.rs

| `UPDATE_GOLDEN` | env_key | std::env::var("UPDATE_GOLDEN") |  |  |  |  |

| `UPDATE_SNAPSHOTS` | env_key | std::env::var("UPDATE_SNAPSHOTS") |  |  |  |  |

| `assert_golden` | function | assert_golden(actual: &[u8], path: &Path) -> Result<()> |  |  |  |  |

| `assert_snapshot` | function | assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) |  |  |  |  |

| `build` | function | build(self) -> Result<TempReceipt> |  |  |  |  |

| `builder` | function | builder() -> TempReceiptBuilder |  |  |  |  |

| `chain_hash` | function | chain_hash(mut self, h: impl Into<String>) -> Self |  |  |  |  |

| `deterministic_uuid` | function | deterministic_uuid(seed: &str) -> String |  |  |  |  |

| `dir` | function | dir(&self) -> &Path |  |  |  |  |

| `event` | function | event(mut self, event: serde_json::Value) -> Self |  |  |  |  |

| `filename` | function | filename(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `format_version` | function | format_version(mut self, v: impl Into<String>) -> Self |  |  |  |  |

| `path` | function | path(&self) -> PathBuf |  |  |  |  |

| `profile` | function | profile(mut self, p: impl Into<String>) -> Self |  |  |  |  |

| `TempReceipt` | struct | TempReceipt { dir: tempfile::TempDir, filename: String } |  |  |  |  |

| `TempReceiptBuilder` | struct | TempReceiptBuilder { format_version: String, chain_hash: String, events: Vec<serde_json::Value>, profile: String, filename: String } |  |  |  |  |

| `tempfile` | use | tempfile |  |  |  |  |


### praxis/template/src/chain.rs

| `GENESIS_SEED` | const | GENESIS_SEED: &[u8] |  |  |  |  |

| `append` | function | append(&mut self, event_bytes: &[u8]) -> Blake3Hash |  |  |  |  |

| `finalize` | function | finalize(self) -> String |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[impl AsRef<[u8]>]) -> String |  |  |  |  |

| `{{project-name}}-v` | str_key | GENESIS_SEED_STR = "{{project-name}}-v" |  |  |  |  |

| `ChainAssembler` | struct | ChainAssembler { running: Blake3Hash } |  |  |  |  |


### praxis/template/src/chain.rs

| `GENESIS_SEED` | const | GENESIS_SEED: &[u8] |  |  |  |  |

| `append` | function | append(&mut self, event_bytes: &[u8]) -> Blake3Hash |  |  |  |  |

| `finalize` | function | finalize(self) -> String |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[impl AsRef<[u8]>]) -> String |  |  |  |  |

| `{{project-name}}-v` | str_key | GENESIS_SEED_STR = "{{project-name}}-v" |  |  |  |  |

| `ChainAssembler` | struct | ChainAssembler { running: Blake3Hash } |  |  |  |  |


### praxis/template/src/cli.rs

| `ColorMode` | enum | ColorMode { Auto, On, Off } |  |  |  |  |

| `OutputFormat` | enum | OutputFormat { Json, Yaml, Text } |  |  |  |  |

| `NO_COLOR` | env_key | std::env::var("NO_COLOR") |  |  |  |  |

| `enabled` | function | enabled(self) -> bool |  |  |  |  |

| `print_output` | function | print_output(value: &T, format: OutputFormat) -> Result<()> |  |  |  |  |

| `Cli` | struct | Cli { long, global = true, value_enum, default_value_t = OutputFormat::Text, env = "OUTPUT_FORMAT" )] pub format: OutputFormat, long, global = true, value_enum, default_value_t = ColorMode::Auto, env = "COLOR_MODE" )] pub color: ColorMode, pub verbose: bool } |  |  |  |  |


### praxis/template/src/cli.rs

| `ColorMode` | enum | ColorMode { Auto, On, Off } |  |  |  |  |

| `OutputFormat` | enum | OutputFormat { Json, Yaml, Text } |  |  |  |  |

| `NO_COLOR` | env_key | std::env::var("NO_COLOR") |  |  |  |  |

| `enabled` | function | enabled(self) -> bool |  |  |  |  |

| `print_output` | function | print_output(value: &T, format: OutputFormat) -> Result<()> |  |  |  |  |

| `Cli` | struct | Cli { long, global = true, value_enum, default_value_t = OutputFormat::Text, env = "OUTPUT_FORMAT" )] pub format: OutputFormat, long, global = true, value_enum, default_value_t = ColorMode::Auto, env = "COLOR_MODE" )] pub color: ColorMode, pub verbose: bool } |  |  |  |  |


### praxis/template/src/error.rs

| `AppError` | enum | AppError { Io( Serde( Other(String), } |  |  |  |  |


### praxis/template/src/error.rs

| `AppError` | enum | AppError { Io( Serde( Other(String), } |  |  |  |  |


### praxis/template/src/lib.rs

| `error::AppError` | use | error::AppError |  |  |  |  |

| `types::{Blake3Hash, ObjectRef, canonical_bytes}` | use | types::{Blake3Hash, ObjectRef, canonical_bytes} |  |  |  |  |


### praxis/template/src/lib.rs

| `error::AppError` | use | error::AppError |  |  |  |  |

| `types::{Blake3Hash, ObjectRef, canonical_bytes}` | use | types::{Blake3Hash, ObjectRef, canonical_bytes} |  |  |  |  |


### praxis/template/src/types.rs

| `ProfileId` | enum | ProfileId { CoreV1 } |  |  |  |  |

| `as_hex` | function | as_hex(&self) -> &str |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) -> Self |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) -> Self |  |  |  |  |

| `Blake3Hash` | struct | Blake3Hash { pub String } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub type_: String, pub qualifier: Option<String> } |  |  |  |  |


### praxis/template/src/types.rs

| `ProfileId` | enum | ProfileId { CoreV1 } |  |  |  |  |

| `as_hex` | function | as_hex(&self) -> &str |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) -> Self |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) -> Self |  |  |  |  |

| `Blake3Hash` | struct | Blake3Hash { pub String } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub type_: String, pub qualifier: Option<String> } |  |  |  |  |


### src/1000x_auto_remediate_dx.rs

| `new` | function | new(receipt_path: impl AsRef<Path>, source_dir: impl AsRef<Path>) -> Self |  |  |  |  |

| `remediate` | function | remediate(&self) -> Result<String> |  |  |  |  |

| `AutoRemediator` | struct | AutoRemediator { pub receipt_path: PathBuf, pub source_dir: PathBuf } |  |  |  |  |


### src/1000x_autonomous_governance.rs

| `audit_workspace` | function | audit_workspace(&self) -> anyhow::Result<Vec<GovernanceReport>> |  |  |  |  |

| `handle_governance_audit` | function | handle_governance_audit() -> anyhow::Result<String> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `ArchitectureProposal` | struct | ArchitectureProposal { pub rule_reference: String, pub root_cause: String, pub proposed_change: String, pub confidence: f64 } |  |  |  |  |

| `GovernanceAgent` | struct | GovernanceAgent { receipts_dir: PathBuf, rules_path: PathBuf } |  |  |  |  |

| `GovernanceReport` | struct | GovernanceReport { pub receipt_id: String, pub verdict: Verdict, pub proposals: Vec<ArchitectureProposal> } |  |  |  |  |


### src/1000x_chaos_e2e.rs

| `verify_with_chaos` | function | verify_with_chaos(&self, receipt: &Receipt) -> Result<Verdict, String> |  |  |  |  |

| `ChaosVerifier` | struct | ChaosVerifier { pub latency_ms: u64, pub corruption_probability: f64, pub handle_drop_probability: f64 } |  |  |  |  |


### src/1000x_cli_telepathy_qol.rs

| `new` | function | new(root: P) -> Self |  |  |  |  |

| `predict` | function | predict(&self) -> Option<Prediction> |  |  |  |  |

| `run_cli` | function | run_cli() |  |  |  |  |

| `shell_integration` | function | shell_integration() -> &'static str |  |  |  |  |

| `Prediction` | struct | Prediction { pub command: String, pub confidence: f64, pub reason: String, pub category: String } |  |  |  |  |

| `Telepathy` | struct | Telepathy { workspace_root: PathBuf, rules: Vec<Box<dyn TelepathyRule>> } |  |  |  |  |


### src/1000x_distributed_sharding.rs

| `DEFAULT_SHARD_SIZE` | const | DEFAULT_SHARD_SIZE: usize |  |  |  |  |

| `ShardingError` | enum | ShardingError { Dht(String), ChainMismatch { index: usize, expected: String, found: String }, SeqMismatch { index: usize, expected: u64, found: u64 }, BoundaryMismatch { index: usize, next: usize }, Failure(String) } |  |  |  |  |

| `new` | function | new(dht: Arc<dyn KademliaDHT>) -> Self |  |  |  |  |

| `shard_receipt` | function | shard_receipt(receipt: Receipt, shard_size: usize) -> Result<(ReceiptManifest, Vec<ReceiptShard>), crate::error::AffidavitError> |  |  |  |  |

| `verify_distributed` | function | verify_distributed(&self, receipt_id: &Blake3Hash) -> Result<Verdict, ShardingError> |  |  |  |  |

| `DistributedVerifier` | struct | DistributedVerifier { dht: Arc<dyn KademliaDHT> } |  |  |  |  |

| `ReceiptManifest` | struct | ReceiptManifest { pub sharding_version: String, pub receipt_id: Blake3Hash, pub total_events: u64, pub shard_count: usize, pub shard_size: usize, pub final_chain_hash: Blake3Hash } |  |  |  |  |

| `ReceiptShard` | struct | ReceiptShard { pub receipt_id: Blake3Hash, pub shard_index: usize, pub start_seq: u64, pub end_seq: u64, pub prev_chain_hash: Blake3Hash, pub shard_chain_hash: Blake3Hash, pub events: Vec<OperationEvent> } |  |  |  |  |

| `KademliaDHT` | trait |  |  |  |  |  |


### src/1000x_formal_verification_spec.rs

| `State` | enum | State { Init = 0, Decode = 1, CheckFormat = 2, ChainIntegrity = 3, Continuity = 4, VerifyCommitments = 5, EvaluateProfile = 6, EmitVerdict = 7, Terminal = 8 } |  |  |  |  |

| `get` | function | get() -> State |  |  |  |  |

| `init` | function | init() |  |  |  |  |

| `terminate` | function | terminate() |  |  |  |  |

| `transition_to` | function | transition_to(expected_prev: State, next: State) |  |  |  |  |

| `CurrentState` | struct |  |  |  |  |  |


### src/1000x_gpu_verifier.rs

| `WGSL_SHADER` | const | WGSL_SHADER: &str |  |  |  |  |

| `is_accepted` | function | is_accepted(&self) -> bool |  |  |  |  |

| `new` | function | new() -> anyhow::Result<Self> |  |  |  |  |

| `prepare_batch` | function | prepare_batch( receipts: &[crate::types::Receipt], ) -> (Vec<GpuEvent>, Vec<GpuReceiptMetadata>) |  |  |  |  |

| `verify_batch` | function | verify_batch( &self, events: &[GpuEvent], metadata: &[GpuReceiptMetadata], ) -> anyhow::Result<Vec<GpuVerdict>> |  |  |  |  |

| `GpuEvent` | struct | GpuEvent { pub seq: u64, pub type_hash: [u32; 8], pub payload_commitment: [u32; 8], pub id_hash: [u32; 8] } |  |  |  |  |

| `GpuReceiptMetadata` | struct | GpuReceiptMetadata { pub event_start: u32, pub event_count: u32, pub expected_chain_hash: [u32; 8], pub format_version_hash: [u32; 8] } |  |  |  |  |

| `GpuVerdict` | struct | GpuVerdict { pub bitmask: u32 } |  |  |  |  |

| `GpuVerifier` | struct | GpuVerifier { device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, pipeline: wgpu::ComputePipeline, bind_group_layout: wgpu::BindGroupLayout } |  |  |  |  |


### src/1000x_holographic_lsp_dx.rs

| `get_hologram_svg` | function | get_hologram_svg(events: &[OperationEvent]) -> String |  |  |  |  |


### src/1000x_nlp_query_qol.rs

| `new` | function | new() -> Self |  |  |  |  |

| `parse` | function | parse(&self, input: &str) -> OcpqQuery |  |  |  |  |

| `NlpQueryParser` | struct |  |  |  |  |  |


### src/1000x_otel_hyper_spec.rs

| `AGGREGATION_DURATION_NS` | const | AGGREGATION_DURATION_NS: &str |  |  |  |  |

| `ANOMALY_SCORE` | const | ANOMALY_SCORE: &str |  |  |  |  |

| `ATTRIBUTE_COUNT` | const | ATTRIBUTE_COUNT: &str |  |  |  |  |

| `BASIC_BLOCKS_EXECUTED` | const | BASIC_BLOCKS_EXECUTED: &str |  |  |  |  |

| `BLAKE3_BYTES_HASHED` | const | BLAKE3_BYTES_HASHED: &str |  |  |  |  |

| `BLAKE3_CHUNKS_PROCESSED` | const | BLAKE3_CHUNKS_PROCESSED: &str |  |  |  |  |

| `BLAKE3_HASH_DURATION_NS` | const | BLAKE3_HASH_DURATION_NS: &str |  |  |  |  |

| `BLOCK_SIZE` | const | BLOCK_SIZE: &str |  |  |  |  |

| `BRANCH_MISS_RATE` | const | BRANCH_MISS_RATE: &str |  |  |  |  |

| `BRANCH_PREDICTION_MISSES` | const | BRANCH_PREDICTION_MISSES: &str |  |  |  |  |

| `BRANCH_TOTAL` | const | BRANCH_TOTAL: &str |  |  |  |  |

| `BYTES_READ` | const | BYTES_READ: &str |  |  |  |  |

| `BYTES_WRITTEN` | const | BYTES_WRITTEN: &str |  |  |  |  |

| `CACHE_L1_HITS` | const | CACHE_L1_HITS: &str |  |  |  |  |

| `CACHE_L1_MISSES` | const | CACHE_L1_MISSES: &str |  |  |  |  |

| `CACHE_L2_HITS` | const | CACHE_L2_HITS: &str |  |  |  |  |

| `CACHE_L2_MISSES` | const | CACHE_L2_MISSES: &str |  |  |  |  |

| `CERTIFICATES_CHAIN_LENGTH` | const | CERTIFICATES_CHAIN_LENGTH: &str |  |  |  |  |

| `CERTIFICATES_EXPIRY_DAYS` | const | CERTIFICATES_EXPIRY_DAYS: &str |  |  |  |  |

| `CIPHER_MODE` | const | CIPHER_MODE: &str |  |  |  |  |

| `COMPRESSION_RATIO` | const | COMPRESSION_RATIO: &str |  |  |  |  |

| `CONSISTENCY_CHECK_DURATION_NS` | const | CONSISTENCY_CHECK_DURATION_NS: &str |  |  |  |  |

| `CONSTRAINTS_SATISFIED` | const | CONSTRAINTS_SATISFIED: &str |  |  |  |  |

| `CONSTRAINTS_TOTAL` | const | CONSTRAINTS_TOTAL: &str |  |  |  |  |

| `CONSTRAINTS_VIOLATED` | const | CONSTRAINTS_VIOLATED: &str |  |  |  |  |

| `CONTEXT_SWITCHES_INVOLUNTARY` | const | CONTEXT_SWITCHES_INVOLUNTARY: &str |  |  |  |  |

| `CONTEXT_SWITCHES_VOLUNTARY` | const | CONTEXT_SWITCHES_VOLUNTARY: &str |  |  |  |  |

| `CPU_ID` | const | CPU_ID: &str |  |  |  |  |

| `CRYPTO_BLAKE3` | const | CRYPTO_BLAKE3: &str |  |  |  |  |

| `CYCLES_TOTAL` | const | CYCLES_TOTAL: &str |  |  |  |  |

| `CYCLE_DETECTION_DURATION_NS` | const | CYCLE_DETECTION_DURATION_NS: &str |  |  |  |  |

| `DECRYPTION_DURATION_NS` | const | DECRYPTION_DURATION_NS: &str |  |  |  |  |

| `DISCOVERY_ALGORITHM` | const | DISCOVERY_ALGORITHM: &str |  |  |  |  |

| `DISK_LATENCY_NS` | const | DISK_LATENCY_NS: &str |  |  |  |  |

| `DNS_LOOKUP_DURATION_NS` | const | DNS_LOOKUP_DURATION_NS: &str |  |  |  |  |

| `EDGE_COUNT` | const | EDGE_COUNT: &str |  |  |  |  |

| `ENCRYPTION_DURATION_NS` | const | ENCRYPTION_DURATION_NS: &str |  |  |  |  |

| `ENTROPY_SOURCE` | const | ENTROPY_SOURCE: &str |  |  |  |  |

| `EVENT_COUNT` | const | EVENT_COUNT: &str |  |  |  |  |

| `EVENT_TYPE_COUNT` | const | EVENT_TYPE_COUNT: &str |  |  |  |  |

| `EVIDENCE_COUNT` | const | EVIDENCE_COUNT: &str |  |  |  |  |

| `EVIDENCE_SIZE_BYTES` | const | EVIDENCE_SIZE_BYTES: &str |  |  |  |  |

| `EXECUTION_INNER_LOOP` | const | EXECUTION_INNER_LOOP: &str |  |  |  |  |

| `EXPORTS_INVOKED` | const | EXPORTS_INVOKED: &str |  |  |  |  |

| `FILES_OPENED` | const | FILES_OPENED: &str |  |  |  |  |

| `FILTERING_DURATION_NS` | const | FILTERING_DURATION_NS: &str |  |  |  |  |

| `FORMAT_VERSION` | const | FORMAT_VERSION: &str |  |  |  |  |

| `FS_SYNC_COUNT` | const | FS_SYNC_COUNT: &str |  |  |  |  |

| `FUNCTIONS_INVOKED` | const | FUNCTIONS_INVOKED: &str |  |  |  |  |

| `HASH_THROUGHPUT` | const | HASH_THROUGHPUT: &str |  |  |  |  |

| `HYPOTHESIS_COUNT` | const | HYPOTHESIS_COUNT: &str |  |  |  |  |

| `ID_COLLISIONS` | const | ID_COLLISIONS: &str |  |  |  |  |

| `IMPORTS_FAILED` | const | IMPORTS_FAILED: &str |  |  |  |  |

| `IMPORTS_RESOLVED` | const | IMPORTS_RESOLVED: &str |  |  |  |  |

| `INIT_DURATION_NS` | const | INIT_DURATION_NS: &str |  |  |  |  |

| `INSTRUCTIONS_SKIPPED` | const | INSTRUCTIONS_SKIPPED: &str |  |  |  |  |

| `INSTRUCTIONS_TOTAL` | const | INSTRUCTIONS_TOTAL: &str |  |  |  |  |

| `INSTRUCTIONS_VERIFIED` | const | INSTRUCTIONS_VERIFIED: &str |  |  |  |  |

| `INSTRUCTION_RETIRED` | const | INSTRUCTION_RETIRED: &str |  |  |  |  |

| `IO_WAIT_DURATION_NS` | const | IO_WAIT_DURATION_NS: &str |  |  |  |  |

| `ITERATIONS_COUNT` | const | ITERATIONS_COUNT: &str |  |  |  |  |

| `JIT_CODE_SIZE_BYTES` | const | JIT_CODE_SIZE_BYTES: &str |  |  |  |  |

| `JIT_COMPILE_DURATION_NS` | const | JIT_COMPILE_DURATION_NS: &str |  |  |  |  |

| `JOIN_DURATION_NS` | const | JOIN_DURATION_NS: &str |  |  |  |  |

| `KEY_DERIVATION_DURATION_NS` | const | KEY_DERIVATION_DURATION_NS: &str |  |  |  |  |

| `KEY_ROTATION_COUNT` | const | KEY_ROTATION_COUNT: &str |  |  |  |  |

| `KEY_STRENGTH_BITS` | const | KEY_STRENGTH_BITS: &str |  |  |  |  |

| `LOG_SIZE_BYTES` | const | LOG_SIZE_BYTES: &str |  |  |  |  |

| `LOOP_ITERATIONS` | const | LOOP_ITERATIONS: &str |  |  |  |  |

| `MAC_VERIFY_DURATION_NS` | const | MAC_VERIFY_DURATION_NS: &str |  |  |  |  |

| `MAC_VERIFY_SUCCESS` | const | MAC_VERIFY_SUCCESS: &str |  |  |  |  |

| `MAPPING_DURATION_NS` | const | MAPPING_DURATION_NS: &str |  |  |  |  |

| `MEMORY_ALLOCATED` | const | MEMORY_ALLOCATED: &str |  |  |  |  |

| `MEMORY_FREED` | const | MEMORY_FREED: &str |  |  |  |  |

| `MEMORY_PEAK` | const | MEMORY_PEAK: &str |  |  |  |  |

| `MODULE_HASH` | const | MODULE_HASH: &str |  |  |  |  |

| `MODULE_SIZE_BYTES` | const | MODULE_SIZE_BYTES: &str |  |  |  |  |

| `NESTING_MAX_DEPTH` | const | NESTING_MAX_DEPTH: &str |  |  |  |  |

| `NETWORK_PACKETS_RECEIVED` | const | NETWORK_PACKETS_RECEIVED: &str |  |  |  |  |

| `NETWORK_PACKETS_SENT` | const | NETWORK_PACKETS_SENT: &str |  |  |  |  |

| `NODE_COUNT` | const | NODE_COUNT: &str |  |  |  |  |

| `NONCE_VALUE` | const | NONCE_VALUE: &str |  |  |  |  |

| `OBJECT_COUNT` | const | OBJECT_COUNT: &str |  |  |  |  |

| `OBJECT_TYPE_COUNT` | const | OBJECT_TYPE_COUNT: &str |  |  |  |  |

| `OCEL_EVENT_THROUGHPUT` | const | OCEL_EVENT_THROUGHPUT: &str |  |  |  |  |

| `OCEL_PARSING` | const | OCEL_PARSING: &str |  |  |  |  |

| `PADDING_SCHEME` | const | PADDING_SCHEME: &str |  |  |  |  |

| `PAGES_INITIAL` | const | PAGES_INITIAL: &str |  |  |  |  |

| `PAGES_MAXIMUM` | const | PAGES_MAXIMUM: &str |  |  |  |  |

| `PAGE_FAULTS_MAJOR` | const | PAGE_FAULTS_MAJOR: &str |  |  |  |  |

| `PAGE_FAULTS_MINOR` | const | PAGE_FAULTS_MINOR: &str |  |  |  |  |

| `PARSING_DURATION_NS` | const | PARSING_DURATION_NS: &str |  |  |  |  |

| `PATH_LENGTH` | const | PATH_LENGTH: &str |  |  |  |  |

| `PIPELINE_STALL_DURATION_NS` | const | PIPELINE_STALL_DURATION_NS: &str |  |  |  |  |

| `PROCESS_UPTIME_NS` | const | PROCESS_UPTIME_NS: &str |  |  |  |  |

| `PROJECTION_DURATION_NS` | const | PROJECTION_DURATION_NS: &str |  |  |  |  |

| `PROOF_SIZE_BYTES` | const | PROOF_SIZE_BYTES: &str |  |  |  |  |

| `RANDOM_BYTES_REQUESTED` | const | RANDOM_BYTES_REQUESTED: &str |  |  |  |  |

| `REACHABILITY_CHECK_DURATION_NS` | const | REACHABILITY_CHECK_DURATION_NS: &str |  |  |  |  |

| `RECURSION_DEPTH` | const | RECURSION_DEPTH: &str |  |  |  |  |

| `REFUTATION_COUNT` | const | REFUTATION_COUNT: &str |  |  |  |  |

| `REGISTERS_SPILLS` | const | REGISTERS_SPILLS: &str |  |  |  |  |

| `RELATIONSHIP_COUNT` | const | RELATIONSHIP_COUNT: &str |  |  |  |  |

| `REORDER_BUFFER_OCCUPANCY` | const | REORDER_BUFFER_OCCUPANCY: &str |  |  |  |  |

| `RULES_DURATION_NS` | const | RULES_DURATION_NS: &str |  |  |  |  |

| `RULES_EVALUATED` | const | RULES_EVALUATED: &str |  |  |  |  |

| `RUNTIME_ENGINE` | const | RUNTIME_ENGINE: &str |  |  |  |  |

| `SALT_LENGTH` | const | SALT_LENGTH: &str |  |  |  |  |

| `SATURATION_REACHED` | const | SATURATION_REACHED: &str |  |  |  |  |

| `SCHEMA_VALIDATION_DURATION_NS` | const | SCHEMA_VALIDATION_DURATION_NS: &str |  |  |  |  |

| `SEQUENCE_GAP_COUNT` | const | SEQUENCE_GAP_COUNT: &str |  |  |  |  |

| `SERIALIZATION_DURATION_NS` | const | SERIALIZATION_DURATION_NS: &str |  |  |  |  |

| `SIGNATURE_ALGORITHM` | const | SIGNATURE_ALGORITHM: &str |  |  |  |  |

| `SIGNATURE_VERIFY_DURATION_NS` | const | SIGNATURE_VERIFY_DURATION_NS: &str |  |  |  |  |

| `SOCKET_COUNT` | const | SOCKET_COUNT: &str |  |  |  |  |

| `SORTING_DURATION_NS` | const | SORTING_DURATION_NS: &str |  |  |  |  |

| `SPECULATIVE_EXECUTION_DURATION_NS` | const | SPECULATIVE_EXECUTION_DURATION_NS: &str |  |  |  |  |

| `SPECULATIVE_PATH_COUNT` | const | SPECULATIVE_PATH_COUNT: &str |  |  |  |  |

| `SPECULATIVE_RETIRED_COUNT` | const | SPECULATIVE_RETIRED_COUNT: &str |  |  |  |  |

| `SPECULATIVE_SQUASHED_COUNT` | const | SPECULATIVE_SQUASHED_COUNT: &str |  |  |  |  |

| `STACK_DEPTH_CURRENT` | const | STACK_DEPTH_CURRENT: &str |  |  |  |  |

| `STACK_DEPTH_MAX` | const | STACK_DEPTH_MAX: &str |  |  |  |  |

| `SUB_INSTRUCTION_MICRO_OPS` | const | SUB_INSTRUCTION_MICRO_OPS: &str |  |  |  |  |

| `SYSCALLS_COUNT` | const | SYSCALLS_COUNT: &str |  |  |  |  |

| `TABLE_ELEMENTS` | const | TABLE_ELEMENTS: &str |  |  |  |  |

| `TABLE_GROW_COUNT` | const | TABLE_GROW_COUNT: &str |  |  |  |  |

| `THREAD_ID` | const | THREAD_ID: &str |  |  |  |  |

| `TLB_MISSES` | const | TLB_MISSES: &str |  |  |  |  |

| `TRANSFORMATION_COUNT` | const | TRANSFORMATION_COUNT: &str |  |  |  |  |

| `TRAPS_COUNT` | const | TRAPS_COUNT: &str |  |  |  |  |

| `VERDICT_OUTCOME` | const | VERDICT_OUTCOME: &str |  |  |  |  |

| `VERIFICATION_REACHABILITY` | const | VERIFICATION_REACHABILITY: &str |  |  |  |  |

| `VERIFICATION_SUCCESS_RATE` | const | VERIFICATION_SUCCESS_RATE: &str |  |  |  |  |

| `VERIFY_WASM` | const | VERIFY_WASM: &str |  |  |  |  |

| `WASM_MEMORY_USAGE` | const | WASM_MEMORY_USAGE: &str |  |  |  |  |

| `WITNESS_ID` | const | WITNESS_ID: &str |  |  |  |  |

| `WITNESS_LEVEL` | const | WITNESS_LEVEL: &str |  |  |  |  |

| `all_attribute_keys` | function | all_attribute_keys() -> Vec<&'static str> |  |  |  |  |

| `record_crypto_blake3_maximalist` | function | record_crypto_blake3_maximalist( _bytes_hashed: u64, _duration_ns: u64, ) |  |  |  |  |

| `record_wasm_verify_maximalist` | function | record_wasm_verify_maximalist( _module_hash: &str, _instruction_count: u64, _memory_peak: u64, ) |  |  |  |  |

| `affidavit.crypto.blake3.bytes_hashed` | str_key | BLAKE3_BYTES_HASHED = "affidavit.crypto.blake3.bytes_hashed" |  |  |  |  |

| `affidavit.crypto.blake3.chunks_processed` | str_key | BLAKE3_CHUNKS_PROCESSED = "affidavit.crypto.blake3.chunks_processed" |  |  |  |  |

| `affidavit.crypto.blake3.hash_duration_ns` | str_key | BLAKE3_HASH_DURATION_NS = "affidavit.crypto.blake3.hash_duration_ns" |  |  |  |  |

| `affidavit.crypto.block.size` | str_key | BLOCK_SIZE = "affidavit.crypto.block.size" |  |  |  |  |

| `affidavit.crypto.certificates.chain_length` | str_key | CERTIFICATES_CHAIN_LENGTH = "affidavit.crypto.certificates.chain_length" |  |  |  |  |

| `affidavit.crypto.certificates.expiry_days` | str_key | CERTIFICATES_EXPIRY_DAYS = "affidavit.crypto.certificates.expiry_days" |  |  |  |  |

| `affidavit.crypto.cipher.mode` | str_key | CIPHER_MODE = "affidavit.crypto.cipher.mode" |  |  |  |  |

| `affidavit.crypto.decryption_duration_ns` | str_key | DECRYPTION_DURATION_NS = "affidavit.crypto.decryption_duration_ns" |  |  |  |  |

| `affidavit.crypto.encryption.duration_ns` | str_key | ENCRYPTION_DURATION_NS = "affidavit.crypto.encryption.duration_ns" |  |  |  |  |

| `affidavit.crypto.entropy.source` | str_key | ENTROPY_SOURCE = "affidavit.crypto.entropy.source" |  |  |  |  |

| `affidavit.crypto.iterations.count` | str_key | ITERATIONS_COUNT = "affidavit.crypto.iterations.count" |  |  |  |  |

| `affidavit.crypto.key.derivation_duration_ns` | str_key | KEY_DERIVATION_DURATION_NS = "affidavit.crypto.key.derivation_duration_ns" |  |  |  |  |

| `affidavit.crypto.key.rotation_count` | str_key | KEY_ROTATION_COUNT = "affidavit.crypto.key.rotation_count" |  |  |  |  |

| `affidavit.crypto.key.strength_bits` | str_key | KEY_STRENGTH_BITS = "affidavit.crypto.key.strength_bits" |  |  |  |  |

| `affidavit.crypto.mac.verify_duration_ns` | str_key | MAC_VERIFY_DURATION_NS = "affidavit.crypto.mac.verify_duration_ns" |  |  |  |  |

| `affidavit.crypto.mac.verify_success` | str_key | MAC_VERIFY_SUCCESS = "affidavit.crypto.mac.verify_success" |  |  |  |  |

| `affidavit.crypto.nonce.value` | str_key | NONCE_VALUE = "affidavit.crypto.nonce.value" |  |  |  |  |

| `affidavit.crypto.padding.scheme` | str_key | PADDING_SCHEME = "affidavit.crypto.padding.scheme" |  |  |  |  |

| `affidavit.crypto.random.bytes_requested` | str_key | RANDOM_BYTES_REQUESTED = "affidavit.crypto.random.bytes_requested" |  |  |  |  |

| `affidavit.crypto.salt.length` | str_key | SALT_LENGTH = "affidavit.crypto.salt.length" |  |  |  |  |

| `affidavit.crypto.signature.algorithm` | str_key | SIGNATURE_ALGORITHM = "affidavit.crypto.signature.algorithm" |  |  |  |  |

| `affidavit.crypto.signature.verify_duration_ns` | str_key | SIGNATURE_VERIFY_DURATION_NS = "affidavit.crypto.signature.verify_duration_ns" |  |  |  |  |

| `affidavit.execution.basic_blocks.executed` | str_key | BASIC_BLOCKS_EXECUTED = "affidavit.execution.basic_blocks.executed" |  |  |  |  |

| `affidavit.execution.branch.prediction_misses` | str_key | BRANCH_PREDICTION_MISSES = "affidavit.execution.branch.prediction_misses" |  |  |  |  |

| `affidavit.execution.branch.total` | str_key | BRANCH_TOTAL = "affidavit.execution.branch.total" |  |  |  |  |

| `affidavit.execution.cache.l1.hits` | str_key | CACHE_L1_HITS = "affidavit.execution.cache.l1.hits" |  |  |  |  |

| `affidavit.execution.cache.l1.misses` | str_key | CACHE_L1_MISSES = "affidavit.execution.cache.l1.misses" |  |  |  |  |

| `affidavit.execution.cache.l2.hits` | str_key | CACHE_L2_HITS = "affidavit.execution.cache.l2.hits" |  |  |  |  |

| `affidavit.execution.cache.l2.misses` | str_key | CACHE_L2_MISSES = "affidavit.execution.cache.l2.misses" |  |  |  |  |

| `affidavit.execution.context_switches.involuntary` | str_key | CONTEXT_SWITCHES_INVOLUNTARY = "affidavit.execution.context_switches.involuntary" |  |  |  |  |

| `affidavit.execution.context_switches.voluntary` | str_key | CONTEXT_SWITCHES_VOLUNTARY = "affidavit.execution.context_switches.voluntary" |  |  |  |  |

| `affidavit.execution.cpu.id` | str_key | CPU_ID = "affidavit.execution.cpu.id" |  |  |  |  |

| `affidavit.execution.cycles.total` | str_key | CYCLES_TOTAL = "affidavit.execution.cycles.total" |  |  |  |  |

| `affidavit.execution.functions.invoked` | str_key | FUNCTIONS_INVOKED = "affidavit.execution.functions.invoked" |  |  |  |  |

| `affidavit.execution.instructions.retired` | str_key | INSTRUCTION_RETIRED = "affidavit.execution.instructions.retired" |  |  |  |  |

| `affidavit.execution.loop.iterations` | str_key | LOOP_ITERATIONS = "affidavit.execution.loop.iterations" |  |  |  |  |

| `affidavit.execution.page_faults.major` | str_key | PAGE_FAULTS_MAJOR = "affidavit.execution.page_faults.major" |  |  |  |  |

| `affidavit.execution.page_faults.minor` | str_key | PAGE_FAULTS_MINOR = "affidavit.execution.page_faults.minor" |  |  |  |  |

| `affidavit.execution.process.uptime_ns` | str_key | PROCESS_UPTIME_NS = "affidavit.execution.process.uptime_ns" |  |  |  |  |

| `affidavit.execution.recursion.depth` | str_key | RECURSION_DEPTH = "affidavit.execution.recursion.depth" |  |  |  |  |

| `affidavit.execution.registers.spills` | str_key | REGISTERS_SPILLS = "affidavit.execution.registers.spills" |  |  |  |  |

| `affidavit.execution.syscalls.count` | str_key | SYSCALLS_COUNT = "affidavit.execution.syscalls.count" |  |  |  |  |

| `affidavit.execution.thread.id` | str_key | THREAD_ID = "affidavit.execution.thread.id" |  |  |  |  |

| `affidavit.execution.tlb.misses` | str_key | TLB_MISSES = "affidavit.execution.tlb.misses" |  |  |  |  |

| `affidavit.io.bytes_read` | str_key | BYTES_READ = "affidavit.io.bytes_read" |  |  |  |  |

| `affidavit.io.bytes_written` | str_key | BYTES_WRITTEN = "affidavit.io.bytes_written" |  |  |  |  |

| `affidavit.io.disk.latency_ns` | str_key | DISK_LATENCY_NS = "affidavit.io.disk.latency_ns" |  |  |  |  |

| `affidavit.io.dns_lookup_duration_ns` | str_key | DNS_LOOKUP_DURATION_NS = "affidavit.io.dns_lookup_duration_ns" |  |  |  |  |

| `affidavit.io.files_opened` | str_key | FILES_OPENED = "affidavit.io.files_opened" |  |  |  |  |

| `affidavit.io.fs.sync_count` | str_key | FS_SYNC_COUNT = "affidavit.io.fs.sync_count" |  |  |  |  |

| `affidavit.io.network.packets_received` | str_key | NETWORK_PACKETS_RECEIVED = "affidavit.io.network.packets_received" |  |  |  |  |

| `affidavit.io.network.packets_sent` | str_key | NETWORK_PACKETS_SENT = "affidavit.io.network.packets_sent" |  |  |  |  |

| `affidavit.io.socket_count` | str_key | SOCKET_COUNT = "affidavit.io.socket_count" |  |  |  |  |

| `affidavit.io.wait_duration_ns` | str_key | IO_WAIT_DURATION_NS = "affidavit.io.wait_duration_ns" |  |  |  |  |

| `affidavit.metric.branch_miss_rate` | str_key | BRANCH_MISS_RATE = "affidavit.metric.branch_miss_rate" |  |  |  |  |

| `affidavit.metric.hash_throughput` | str_key | HASH_THROUGHPUT = "affidavit.metric.hash_throughput" |  |  |  |  |

| `affidavit.metric.ocel_event_throughput` | str_key | OCEL_EVENT_THROUGHPUT = "affidavit.metric.ocel_event_throughput" |  |  |  |  |

| `affidavit.metric.verification_success_rate` | str_key | VERIFICATION_SUCCESS_RATE = "affidavit.metric.verification_success_rate" |  |  |  |  |

| `affidavit.metric.wasm_memory_usage` | str_key | WASM_MEMORY_USAGE = "affidavit.metric.wasm_memory_usage" |  |  |  |  |

| `affidavit.ocel.aggregation.duration_ns` | str_key | AGGREGATION_DURATION_NS = "affidavit.ocel.aggregation.duration_ns" |  |  |  |  |

| `affidavit.ocel.anomaly_score` | str_key | ANOMALY_SCORE = "affidavit.ocel.anomaly_score" |  |  |  |  |

| `affidavit.ocel.attribute.count` | str_key | ATTRIBUTE_COUNT = "affidavit.ocel.attribute.count" |  |  |  |  |

| `affidavit.ocel.compression.ratio` | str_key | COMPRESSION_RATIO = "affidavit.ocel.compression.ratio" |  |  |  |  |

| `affidavit.ocel.discovery.algorithm` | str_key | DISCOVERY_ALGORITHM = "affidavit.ocel.discovery.algorithm" |  |  |  |  |

| `affidavit.ocel.event.count` | str_key | EVENT_COUNT = "affidavit.ocel.event.count" |  |  |  |  |

| `affidavit.ocel.event.type_count` | str_key | EVENT_TYPE_COUNT = "affidavit.ocel.event.type_count" |  |  |  |  |

| `affidavit.ocel.filtering.duration_ns` | str_key | FILTERING_DURATION_NS = "affidavit.ocel.filtering.duration_ns" |  |  |  |  |

| `affidavit.ocel.id.collisions` | str_key | ID_COLLISIONS = "affidavit.ocel.id.collisions" |  |  |  |  |

| `affidavit.ocel.join.duration_ns` | str_key | JOIN_DURATION_NS = "affidavit.ocel.join.duration_ns" |  |  |  |  |

| `affidavit.ocel.log.size_bytes` | str_key | LOG_SIZE_BYTES = "affidavit.ocel.log.size_bytes" |  |  |  |  |

| `affidavit.ocel.mapping.duration_ns` | str_key | MAPPING_DURATION_NS = "affidavit.ocel.mapping.duration_ns" |  |  |  |  |

| `affidavit.ocel.nesting.max_depth` | str_key | NESTING_MAX_DEPTH = "affidavit.ocel.nesting.max_depth" |  |  |  |  |

| `affidavit.ocel.object.count` | str_key | OBJECT_COUNT = "affidavit.ocel.object.count" |  |  |  |  |

| `affidavit.ocel.object.type_count` | str_key | OBJECT_TYPE_COUNT = "affidavit.ocel.object.type_count" |  |  |  |  |

| `affidavit.ocel.parsing.duration_ns` | str_key | PARSING_DURATION_NS = "affidavit.ocel.parsing.duration_ns" |  |  |  |  |

| `affidavit.ocel.projection.duration_ns` | str_key | PROJECTION_DURATION_NS = "affidavit.ocel.projection.duration_ns" |  |  |  |  |

| `affidavit.ocel.relationship.count` | str_key | RELATIONSHIP_COUNT = "affidavit.ocel.relationship.count" |  |  |  |  |

| `affidavit.ocel.sequence.gap_count` | str_key | SEQUENCE_GAP_COUNT = "affidavit.ocel.sequence.gap_count" |  |  |  |  |

| `affidavit.ocel.serialization.duration_ns` | str_key | SERIALIZATION_DURATION_NS = "affidavit.ocel.serialization.duration_ns" |  |  |  |  |

| `affidavit.ocel.sorting.duration_ns` | str_key | SORTING_DURATION_NS = "affidavit.ocel.sorting.duration_ns" |  |  |  |  |

| `affidavit.ocel.transformation.count` | str_key | TRANSFORMATION_COUNT = "affidavit.ocel.transformation.count" |  |  |  |  |

| `affidavit.span.crypto_blake3` | str_key | CRYPTO_BLAKE3 = "affidavit.span.crypto_blake3" |  |  |  |  |

| `affidavit.span.execution_inner_loop` | str_key | EXECUTION_INNER_LOOP = "affidavit.span.execution_inner_loop" |  |  |  |  |

| `affidavit.span.ocel_parsing` | str_key | OCEL_PARSING = "affidavit.span.ocel_parsing" |  |  |  |  |

| `affidavit.span.verification_reachability` | str_key | VERIFICATION_REACHABILITY = "affidavit.span.verification_reachability" |  |  |  |  |

| `affidavit.span.verify_wasm` | str_key | VERIFY_WASM = "affidavit.span.verify_wasm" |  |  |  |  |

| `affidavit.speculative.duration_ns` | str_key | SPECULATIVE_EXECUTION_DURATION_NS = "affidavit.speculative.duration_ns" |  |  |  |  |

| `affidavit.speculative.micro_ops` | str_key | SUB_INSTRUCTION_MICRO_OPS = "affidavit.speculative.micro_ops" |  |  |  |  |

| `affidavit.speculative.path_count` | str_key | SPECULATIVE_PATH_COUNT = "affidavit.speculative.path_count" |  |  |  |  |

| `affidavit.speculative.pipeline_stall_ns` | str_key | PIPELINE_STALL_DURATION_NS = "affidavit.speculative.pipeline_stall_ns" |  |  |  |  |

| `affidavit.speculative.reorder_buffer_occupancy` | str_key | REORDER_BUFFER_OCCUPANCY = "affidavit.speculative.reorder_buffer_occupancy" |  |  |  |  |

| `affidavit.speculative.retired_count` | str_key | SPECULATIVE_RETIRED_COUNT = "affidavit.speculative.retired_count" |  |  |  |  |

| `affidavit.speculative.squashed_count` | str_key | SPECULATIVE_SQUASHED_COUNT = "affidavit.speculative.squashed_count" |  |  |  |  |

| `affidavit.verification.consistency.check_duration_ns` | str_key | CONSISTENCY_CHECK_DURATION_NS = "affidavit.verification.consistency.check_duration_ns" |  |  |  |  |

| `affidavit.verification.constraints.satisfied` | str_key | CONSTRAINTS_SATISFIED = "affidavit.verification.constraints.satisfied" |  |  |  |  |

| `affidavit.verification.constraints.total` | str_key | CONSTRAINTS_TOTAL = "affidavit.verification.constraints.total" |  |  |  |  |

| `affidavit.verification.constraints.violated` | str_key | CONSTRAINTS_VIOLATED = "affidavit.verification.constraints.violated" |  |  |  |  |

| `affidavit.verification.cycle.detection_duration_ns` | str_key | CYCLE_DETECTION_DURATION_NS = "affidavit.verification.cycle.detection_duration_ns" |  |  |  |  |

| `affidavit.verification.edge.count` | str_key | EDGE_COUNT = "affidavit.verification.edge.count" |  |  |  |  |

| `affidavit.verification.evidence.count` | str_key | EVIDENCE_COUNT = "affidavit.verification.evidence.count" |  |  |  |  |

| `affidavit.verification.evidence.size_bytes` | str_key | EVIDENCE_SIZE_BYTES = "affidavit.verification.evidence.size_bytes" |  |  |  |  |

| `affidavit.verification.format.version` | str_key | FORMAT_VERSION = "affidavit.verification.format.version" |  |  |  |  |

| `affidavit.verification.hypothesis.count` | str_key | HYPOTHESIS_COUNT = "affidavit.verification.hypothesis.count" |  |  |  |  |

| `affidavit.verification.node.count` | str_key | NODE_COUNT = "affidavit.verification.node.count" |  |  |  |  |

| `affidavit.verification.path.length` | str_key | PATH_LENGTH = "affidavit.verification.path.length" |  |  |  |  |

| `affidavit.verification.proof.size_bytes` | str_key | PROOF_SIZE_BYTES = "affidavit.verification.proof.size_bytes" |  |  |  |  |

| `affidavit.verification.reachability.check_duration_ns` | str_key | REACHABILITY_CHECK_DURATION_NS = "affidavit.verification.reachability.check_duration_ns" |  |  |  |  |

| `affidavit.verification.refutation.count` | str_key | REFUTATION_COUNT = "affidavit.verification.refutation.count" |  |  |  |  |

| `affidavit.verification.rules.duration_ns` | str_key | RULES_DURATION_NS = "affidavit.verification.rules.duration_ns" |  |  |  |  |

| `affidavit.verification.rules.evaluated` | str_key | RULES_EVALUATED = "affidavit.verification.rules.evaluated" |  |  |  |  |

| `affidavit.verification.saturation.reached` | str_key | SATURATION_REACHED = "affidavit.verification.saturation.reached" |  |  |  |  |

| `affidavit.verification.schema.validation_duration_ns` | str_key | SCHEMA_VALIDATION_DURATION_NS = "affidavit.verification.schema.validation_duration_ns" |  |  |  |  |

| `affidavit.verification.verdict.outcome` | str_key | VERDICT_OUTCOME = "affidavit.verification.verdict.outcome" |  |  |  |  |

| `affidavit.verification.witness.id` | str_key | WITNESS_ID = "affidavit.verification.witness.id" |  |  |  |  |

| `affidavit.verification.witness.level` | str_key | WITNESS_LEVEL = "affidavit.verification.witness.level" |  |  |  |  |

| `affidavit.wasm.exports.invoked` | str_key | EXPORTS_INVOKED = "affidavit.wasm.exports.invoked" |  |  |  |  |

| `affidavit.wasm.imports.failed` | str_key | IMPORTS_FAILED = "affidavit.wasm.imports.failed" |  |  |  |  |

| `affidavit.wasm.imports.resolved` | str_key | IMPORTS_RESOLVED = "affidavit.wasm.imports.resolved" |  |  |  |  |

| `affidavit.wasm.init.duration_ns` | str_key | INIT_DURATION_NS = "affidavit.wasm.init.duration_ns" |  |  |  |  |

| `affidavit.wasm.instructions.skipped` | str_key | INSTRUCTIONS_SKIPPED = "affidavit.wasm.instructions.skipped" |  |  |  |  |

| `affidavit.wasm.instructions.total` | str_key | INSTRUCTIONS_TOTAL = "affidavit.wasm.instructions.total" |  |  |  |  |

| `affidavit.wasm.instructions.verified` | str_key | INSTRUCTIONS_VERIFIED = "affidavit.wasm.instructions.verified" |  |  |  |  |

| `affidavit.wasm.jit.code_size_bytes` | str_key | JIT_CODE_SIZE_BYTES = "affidavit.wasm.jit.code_size_bytes" |  |  |  |  |

| `affidavit.wasm.jit.compile_duration_ns` | str_key | JIT_COMPILE_DURATION_NS = "affidavit.wasm.jit.compile_duration_ns" |  |  |  |  |

| `affidavit.wasm.memory.allocated` | str_key | MEMORY_ALLOCATED = "affidavit.wasm.memory.allocated" |  |  |  |  |

| `affidavit.wasm.memory.freed` | str_key | MEMORY_FREED = "affidavit.wasm.memory.freed" |  |  |  |  |

| `affidavit.wasm.memory.peak` | str_key | MEMORY_PEAK = "affidavit.wasm.memory.peak" |  |  |  |  |

| `affidavit.wasm.module.hash` | str_key | MODULE_HASH = "affidavit.wasm.module.hash" |  |  |  |  |

| `affidavit.wasm.module.size_bytes` | str_key | MODULE_SIZE_BYTES = "affidavit.wasm.module.size_bytes" |  |  |  |  |

| `affidavit.wasm.pages.initial` | str_key | PAGES_INITIAL = "affidavit.wasm.pages.initial" |  |  |  |  |

| `affidavit.wasm.pages.maximum` | str_key | PAGES_MAXIMUM = "affidavit.wasm.pages.maximum" |  |  |  |  |

| `affidavit.wasm.runtime.engine` | str_key | RUNTIME_ENGINE = "affidavit.wasm.runtime.engine" |  |  |  |  |

| `affidavit.wasm.stack.depth.current` | str_key | STACK_DEPTH_CURRENT = "affidavit.wasm.stack.depth.current" |  |  |  |  |

| `affidavit.wasm.stack.depth.max` | str_key | STACK_DEPTH_MAX = "affidavit.wasm.stack.depth.max" |  |  |  |  |

| `affidavit.wasm.table.elements` | str_key | TABLE_ELEMENTS = "affidavit.wasm.table.elements" |  |  |  |  |

| `affidavit.wasm.table.grow_count` | str_key | TABLE_GROW_COUNT = "affidavit.wasm.table.grow_count" |  |  |  |  |

| `affidavit.wasm.traps.count` | str_key | TRAPS_COUNT = "affidavit.wasm.traps.count" |  |  |  |  |


### src/1000x_receipt_to_wasm_qol.rs

| `compile` | function | compile(&self) -> Vec<u8> |  |  |  |  |

| `new` | function | new(receipt: Receipt) -> Self |  |  |  |  |

| `ReceiptWasmCompiler` | struct | ReceiptWasmCompiler { receipt: Receipt } |  |  |  |  |


### src/1000x_tdd_synthesizer_dx.rs

| `affi_trace_sink.log` | str_key | TRACE_SINK = "affi_trace_sink.log" |  |  |  |  |

| `tests/autogen_tdd_witness.rs` | str_key | TARGET_TEST = "tests/autogen_tdd_witness.rs" |  |  |  |  |


### src/1000x_time_travel_dx.rs

| `new` | function | new(receipt: &'a Receipt, initial_state: S, transition: F) -> Self |  |  |  |  |

| `run_repl` | function | run_repl(&mut self) -> io::Result<()> |  |  |  |  |

| `TimeTravelDebugger` | struct | TimeTravelDebugger { receipt: &'a Receipt, states: Vec<S>, cursor: usize } |  |  |  |  |


### src/admission.rs

| `AffidavitRefusal` | enum | AffidavitRefusal { OcelLawViolation(OcelRefusal), StructuralLawViolation { stage: String, reason: String, } } |  |  |  |  |

| `admit` | function | admit(receipt: Receipt) -> Result<AdmittedReceipt, AffidavitRefusal> |  |  |  |  |


### src/architecture.rs

| `ARCHITECTURE_QUERY_SCHEMA` | const | ARCHITECTURE_QUERY_SCHEMA: &str |  |  |  |  |

| `ARCHITECTURE_RECEIPT_SCHEMA` | const | ARCHITECTURE_RECEIPT_SCHEMA: &str |  |  |  |  |

| `ArchitectureRefusal` | enum | ArchitectureRefusal { MissingExactSubject, MissingEvidence, UnknownPromotion, MutableOrChangedContract, MutableOrChangedSbb, CrossSubjectReuse, DoAuthorityForbidden, ReplayMismatch, MalformedDigest { field: &'static str, }, SchemaMismatch, NotAReplacement, ChainBroken, StandingChainMismatch, NotSupersedable, AlreadySuperseded, OrphanChainLink, ForgedEvidence, Malformed } |  |  |  |  |

| `ArchitectureStanding` | enum | ArchitectureStanding { Unknown, Candidate, Qualified, Refused, Superseded } |  |  |  |  |

| `EvidenceSource` | enum | EvidenceSource { AutofdeLab, Xaas, Runtime } |  |  |  |  |

| `admit` | function | admit( &mut self, receipt: ArchitectureQualificationReceipt, ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `admit_supersession` | function | admit_supersession( &mut self, supersession: Supersession, ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `binding_digest` | function | binding_digest(&self) -> String |  |  |  |  |

| `certify` | function | certify( abb_digest: impl Into<String>, contract_digest: impl Into<String>, sbb_digest: impl Into<String>, exact_subject_digest: impl Into<String>, qualification_evidence_digests: Vec<String>, producer_digest: impl Into<String>, artifact_digests: Vec<String>, standing: ArchitectureStanding, ) -> Result<Self, ArchitectureRefusal> |  |  |  |  |

| `certify_from_evidence` | function | certify_from_evidence( abb_digest: impl Into<String>, contract_digest: impl Into<String>, sbb_digest: impl Into<String>, exact_subject_digest: impl Into<String>, evidence: &[QualificationEvidence], producer_digest: impl Into<String>, artifact_digests: Vec<String>, standing: ArchitectureStanding, ) -> Result<Self, ArchitectureRefusal> |  |  |  |  |

| `current_qualified` | function | current_qualified(&self, abb_digest: &str) -> Vec<&ArchitectureQualificationReceipt> |  |  |  |  |

| `from_json_verified` | function | from_json_verified(json: &str) -> Result<Self, ArchitectureRefusal> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `observe` | function | observe( source: EvidenceSource, producer_digest: impl Into<String>, bytes: &[u8], ) -> Result<Self, ArchitectureRefusal> |  |  |  |  |

| `query_json` | function | query_json(&self, abb_digest: &str) -> String |  |  |  |  |

| `standing_of` | function | standing_of(&self, receipt_digest: &str) -> Option<ArchitectureStanding> |  |  |  |  |

| `supersede` | function | supersede( &self, new_sbb_digest: impl Into<String>, new_subject_digest: impl Into<String>, evidence: Vec<String>, ) -> Result<Supersession, ArchitectureRefusal> |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `verify` | function | verify( &self, prior: &ArchitectureQualificationReceipt, ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_bytes` | function | verify_bytes(&self, bytes: &[u8]) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_chain` | function | verify_chain(&self, prior: &Self) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_evidence` | function | verify_evidence( &self, observed: &[(QualificationEvidence, &[u8])], ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_integrity` | function | verify_integrity(&self) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_replay` | function | verify_replay( &self, expected_abb_digest: &str, expected_contract_digest: &str, expected_sbb_digest: &str, expected_subject_digest: &str, ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `affidavit.architecture-qualification.v2` | str_key | ARCHITECTURE_RECEIPT_SCHEMA = "affidavit.architecture-qualification.v2" |  |  |  |  |

| `affidavit.architecture-standing-query.v1` | str_key | ARCHITECTURE_QUERY_SCHEMA = "affidavit.architecture-standing-query.v1" |  |  |  |  |

| `ArchitectureQualificationReceipt` | struct | ArchitectureQualificationReceipt { pub schema: String, pub abb_digest: String, pub contract_digest: String, pub sbb_digest: String, pub exact_subject_digest: String, pub qualification_evidence_digests: Vec<String>, pub producer_digest: String, pub artifact_digests: Vec<String>, pub standing: ArchitectureStanding, pub confers_do_authority: bool, pub prior_receipt_digest: Option<String>, pub superseded_by_receipt_digest: Option<String>, pub receipt_digest: String } |  |  |  |  |

| `ArchitectureStandingLedger` | struct | ArchitectureStandingLedger { receipts: BTreeMap<String, ArchitectureQualificationReceipt>, retired_by: BTreeMap<String, String> } |  |  |  |  |

| `QualificationEvidence` | struct | QualificationEvidence { pub source: EvidenceSource, pub producer_digest: String, pub content_digest: String } |  |  |  |  |

| `Supersession` | struct | Supersession { pub retired: ArchitectureQualificationReceipt, pub successor: ArchitectureQualificationReceipt } |  |  |  |  |


### src/authority_fence.rs

| `REVOCATION_TOMBSTONE` | const | REVOCATION_TOMBSTONE: StateValue |  |  |  |  |

| `DoWitness` | enum | DoWitness { Revoked { id: StateKey, proof: InclusionProof, }, Admitted(AbsenceWitness) } |  |  |  |  |

| `FastPathRefusal` | enum | FastPathRefusal { Replay, ClockSkew( Revoked, Fence( } |  |  |  |  |

| `FenceError` | enum | FenceError { Tree(String), AlreadyRevoked(String), NotRevoked(String), WitnessStaleOrInvalid, Revoked } |  |  |  |  |

| `confirm` | function | confirm(&self, fence: &AuthorityFence) -> Result<(), FenceError> |  |  |  |  |

| `gate_do` | function | gate_do(witness: &DoWitness, live_root: Option<StateRoot>) -> Result<(), FenceError> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `prepare_do_permit` | function | prepare_do_permit(&mut self, id: &StateKey) -> Result<DoWitness, FenceError> |  |  |  |  |

| `prove_revoked` | function | prove_revoked(&mut self, id: &StateKey) -> Result<DoWitness, FenceError> |  |  |  |  |

| `revoke` | function | revoke(&mut self, id: &StateKey) -> Result<StateRoot, FenceError> |  |  |  |  |

| `root` | function | root(&self) -> Option<StateRoot> |  |  |  |  |

| `routes_to_sibling_slot` | function | routes_to_sibling_slot(queried: &StateKey, neighbor: &StateKey, depth: usize) -> bool |  |  |  |  |

| `unrevoke` | function | unrevoke(&mut self, id: &StateKey) -> Result<Option<StateRoot>, FenceError> |  |  |  |  |

| `verify_fast_path` | function | verify_fast_path( fence: &mut AuthorityFence, replay: &mut ReplayFilter, clock: &mut HlcClock, claim: &ConsequenceClaim, ) -> Result<AdmittedProof, FastPathRefusal> |  |  |  |  |

| `AdmittedProof` | struct | AdmittedProof { pub permit: DoWitness, pub admitted_at: HlcTimestamp, pub gate_root: Option<StateRoot> } |  |  |  |  |

| `AuthorityFence` | struct | AuthorityFence { tree: StateTree } |  |  |  |  |

| `ConsequenceClaim` | struct | ConsequenceClaim { pub consequence_id: [u8; 32], pub authority_id: [u8; 32], pub received: HlcTimestamp } |  |  |  |  |


### src/bench.rs

| `bench_throughput` | function | bench_throughput(iterations: u32) -> Result<()> |  |  |  |  |

| `bench_variance_on_receipt` | function | bench_variance_on_receipt(path: &str, iterations: u32) -> Result<()> |  |  |  |  |

| `bench_variance_suite` | function | bench_variance_suite(_iterations: u32) -> Result<()> |  |  |  |  |

| `run_profile_workload` | function | run_profile_workload(seconds: u64, _receipt_path: Option<&str>) -> Result<()> |  |  |  |  |


### src/bin/affi-shell.rs

| `run` | function | run() -> Result<()> |  |  |  |  |


### src/binary_envelope.rs

| `EnvelopeError` | enum | EnvelopeError { Encode(String), Decode(String) } |  |  |  |  |

| `seal` | function | seal(value: &T) -> Result<Vec<u8>, EnvelopeError> |  |  |  |  |

| `sealed_commitment` | function | sealed_commitment( value: &T, ) -> Result<crate::types::Blake3Hash, EnvelopeError> |  |  |  |  |

| `unseal` | function | unseal(bytes: &[u8]) -> Result<T, EnvelopeError> |  |  |  |  |


### src/bls_aggregate.rs

| `BlsError` | enum | BlsError { CommitteeMismatch { sigs: usize, keys: usize, }, VerificationFailed } |  |  |  |  |

| `aggregate_committee` | function | aggregate_committee( signatures: &[Signature<TinyBLS381>], public_keys: &[PublicKey<TinyBLS381>], ) -> Result<(Signature<TinyBLS381>, PublicKey<TinyBLS381>), BlsError> |  |  |  |  |

| `generate` | function | generate(rng: &mut R) -> Self |  |  |  |  |

| `public` | function | public(&self) -> PublicKey<TinyBLS381> |  |  |  |  |

| `sign` | function | sign(&self, context: &[u8], message: &[u8]) -> Signature<TinyBLS381> |  |  |  |  |

| `verify_committee` | function | verify_committee( context: &[u8], message: &[u8], aggregate_signature: &Signature<TinyBLS381>, aggregate_key: &PublicKey<TinyBLS381>, ) -> bool |  |  |  |  |

| `affidavit:bls:v1:committee` | str_key | CONTEXT = "affidavit:bls:v1:committee" |  |  |  |  |

| `CommitteeKey` | struct | CommitteeKey { pub KeypairVT<TinyBLS381> } |  |  |  |  |


### src/brce.rs

| `ALL` | const | ALL: [Rule; 8] |  |  |  |  |

| `CANONICALIZATION` | const | CANONICALIZATION: &str |  |  |  |  |

| `GENESIS` | const | GENESIS: &str |  |  |  |  |

| `PROFILE_ACTUATION` | const | PROFILE_ACTUATION: &str |  |  |  |  |

| `PROFILE_RECONCILIATION` | const | PROFILE_RECONCILIATION: &str |  |  |  |  |

| `PROFILE_REPLAY` | const | PROFILE_REPLAY: &str |  |  |  |  |

| `Admission` | enum | Admission { Admitted, Refused(String), Blocked(String), Unsupported(String) } |  |  |  |  |

| `BrceError` | enum | BrceError { Io( Malformed { line: usize, reason: String, }, Refused(String), Blocked(String), } |  |  |  |  |

| `Entry` | enum | Entry { Parsed { request: Request, request_digest: String, }, Routed { request_digest: String, route: RouteDecision, route_digest: String, }, Admitted { request_digest: String, admission: Admission, admission_digest: String, }, Constructed { action: ConstructedAction, construct_digest: String, }, Prepared { consequence_id: String, attempt_id: String, construct_digest: String, grant: AuthorityGrant, at: u64, }, Done { consequence_id: String, attempt_id: String, construct_digest: String, result: ActuationResult, executor_identity: String, at: u64, }, Receipted { receipt: BrceReceipt, }, Reconciled { consequence_id: String, prepared_record_digest: String, verdict: ReconciliationVerdict, evidence_digest: String, }, Refusal { stage: String, reason: String, } } |  |  |  |  |

| `Observation` | enum | Observation { Effect(String), NoEffect, Unknown } |  |  |  |  |

| `ReconciliationVerdict` | enum | ReconciliationVerdict { EffectConfirmed, NoEffectConfirmed, ExecutionUnknown, BlockedReconciliation } |  |  |  |  |

| `Rule` | enum | Rule { ChainIntegrity, ZeroUnreceiptedActuation, DoRequiresAuthority, ConstructRequiresAdmission, ReceiptDigestValid, AtMostOnceConsequence, CrashWindowReconciled, UnknownNotPromoted } |  |  |  |  |

| `actuate` | function | actuate( &mut self, admitted: &Admitted, action: &ConstructedAction, grant: &AuthorityGrant, attempt_id: &str, actuator: &mut (impl Actuator + Observer), now: u64, ) -> Result<BrceReceipt, BrceError> |  |  |  |  |

| `admit` | function | admit( &mut self, request: Request, route: RouteDecision, policy: impl Fn(&Request, &RouteDecision) -> Admission, ) -> Result<Admitted, BrceError> |  |  |  |  |

| `admitted` | function | admitted(&self) -> bool |  |  |  |  |

| `append` | function | append(&mut self, entry: Entry) -> Result<&LedgerRecord, BrceError> |  |  |  |  |

| `compute_digest` | function | compute_digest(&self) -> String |  |  |  |  |

| `construct` | function | construct( &mut self, admitted: &Admitted, consequence_id: &str, idempotent: bool, ) -> Result<ConstructedAction, BrceError> |  |  |  |  |

| `construct_digest` | function | construct_digest(&self) -> String |  |  |  |  |

| `court` | function | court(ledger: &BrceLedger, world: &dyn Observer) -> CourtVerdict |  |  |  |  |

| `digest` | function | digest(value: &T) -> String |  |  |  |  |

| `entries` | function | entries(&self) -> Vec<Entry> |  |  |  |  |

| `execute` | function | execute( &mut self, action: &ConstructedAction, attempt_id: &str, actuator: &mut dyn Actuator, now: u64, ) -> Result<ActuationResult, BrceError> |  |  |  |  |

| `from_entries` | function | from_entries(entries: impl IntoIterator<Item = Entry>) -> Self |  |  |  |  |

| `from_records` | function | from_records(records: Vec<LedgerRecord>) -> Self |  |  |  |  |

| `grant_digest` | function | grant_digest(&self) -> String |  |  |  |  |

| `head` | function | head(&self) -> String |  |  |  |  |

| `mutant_suite` | function | mutant_suite(base: &BrceLedger, world: &dyn Observer) -> Vec<MutantOutcome> |  |  |  |  |

| `new` | function | new(root: impl Into<PathBuf>) -> std::io::Result<Self> |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) -> Result<Self, BrceError> |  |  |  |  |

| `pending_consequences` | function | pending_consequences(ledger: &BrceLedger) -> BTreeSet<String> |  |  |  |  |

| `prepare` | function | prepare( &mut self, action: &ConstructedAction, grant: &AuthorityGrant, attempt_id: &str, now: u64, ) -> Result<(), BrceError> |  |  |  |  |

| `receipt` | function | receipt( &mut self, admitted: &Admitted, action: &ConstructedAction, grant: &AuthorityGrant, attempt_id: &str, observer: &dyn Observer, ) -> Result<BrceReceipt, BrceError> |  |  |  |  |

| `reconcile` | function | reconcile( &mut self, observer: &dyn Observer, ) -> Result<Vec<(String, ReconciliationVerdict)>, BrceError> |  |  |  |  |

| `record_digest` | function | record_digest(seq: u64, prev: &str, entry: &Entry) -> String |  |  |  |  |

| `records` | function | records(&self) -> &[LedgerRecord] |  |  |  |  |

| `refused_rules` | function | refused_rules(&self) -> BTreeSet<Rule> |  |  |  |  |

| `replay_digest` | function | replay_digest(ledger: &BrceLedger) -> String |  |  |  |  |

| `root` | function | root(&self) -> &Path |  |  |  |  |

| `seal` | function | seal(mut self) -> Self |  |  |  |  |

| `snapshot` | function | snapshot(world: &dyn Observer) -> Self |  |  |  |  |

| `0000000000000000000000000000000000000000000000000000000000000000` | str_key | GENESIS = "0000000000000000000000000000000000000000000000000000000000000000" |  |  |  |  |

| `affidavit/brce-actuation/v1` | str_key | PROFILE_ACTUATION = "affidavit/brce-actuation/v1" |  |  |  |  |

| `affidavit/brce-reconciliation/v1` | str_key | PROFILE_RECONCILIATION = "affidavit/brce-reconciliation/v1" |  |  |  |  |

| `affidavit/brce-replay/v1` | str_key | PROFILE_REPLAY = "affidavit/brce-replay/v1" |  |  |  |  |

| `serde_json-struct-order+blake3` | str_key | CANONICALIZATION = "serde_json-struct-order+blake3" |  |  |  |  |

| `ActuationEvidence` | struct | ActuationEvidence { pub executor_identity: String, pub started_at: u64, pub completed_at: u64, pub result_class: String, pub result_digest: String } |  |  |  |  |

| `ActuationResult` | struct | ActuationResult { pub result_class: String, pub result_digest: String, pub changed: bool } |  |  |  |  |

| `Admitted` | struct | Admitted { pub request: Request, pub request_digest: String, pub route_digest: String, pub admission_digest: String } |  |  |  |  |

| `AuthorityGrant` | struct | AuthorityGrant { pub grant_id: String, pub issuer: String, pub subject: String, pub operation: String, pub target: String, pub construct_digest: String, pub expires_at: u64, pub maximum_uses: u32 } |  |  |  |  |

| `BrceLedger` | struct | BrceLedger { records: Vec<LedgerRecord>, path: Option<PathBuf> } |  |  |  |  |

| `BrcePipeline` | struct | BrcePipeline { pub ledger: BrceLedger, run_id: String, tool_version: String } |  |  |  |  |

| `BrceReceipt` | struct | BrceReceipt { pub profile: String, pub run_id: String, pub subject: String, pub request_digest: String, pub route_digest: String, pub admission_digest: String, pub construct_digest: String, pub authority_grant_digest: String, pub consequence_id: String, pub attempt_id: String, pub actuation: ActuationEvidence, pub effect: EffectEvidence, pub verification: VerificationEvidence, pub replay: ReplayIdentity, pub canonicalization: String, pub previous_receipt: String, pub receipt_digest: String } |  |  |  |  |

| `ConstructedAction` | struct | ConstructedAction { pub request_digest: String, pub subject: String, pub operation: String, pub target: String, pub parameters: BTreeMap<String, String>, pub consequence_id: String, pub idempotent: bool } |  |  |  |  |

| `CourtRefusal` | struct | CourtRefusal { pub rule: Rule, pub seq: Option<u64>, pub detail: String } |  |  |  |  |

| `CourtVerdict` | struct | CourtVerdict { pub standing: String, pub rules: Vec<Rule>, pub refusals: Vec<CourtRefusal>, pub records: usize, pub receipts: usize, pub consequences_observed: usize, pub ledger_head: String, pub replay_digest: String } |  |  |  |  |

| `EffectEvidence` | struct | EffectEvidence { pub executed: bool, pub changed: bool, pub effect_digest: String } |  |  |  |  |

| `FileActuator` | struct | FileActuator { root: PathBuf } |  |  |  |  |

| `LedgerRecord` | struct | LedgerRecord { pub seq: u64, pub prev: String, pub entry: Entry, pub digest: String } |  |  |  |  |

| `MutantOutcome` | struct | MutantOutcome { pub rule: Rule, pub mutation: String, pub standing: String, pub refused_rules: Vec<Rule>, pub killed: bool } |  |  |  |  |

| `ReplayIdentity` | struct | ReplayIdentity { pub command_or_entrypoint: String, pub tool_identity: String, pub tool_version: String } |  |  |  |  |

| `Request` | struct | Request { pub request_id: String, pub subject: String, pub operation: String, pub target: String, pub parameters: BTreeMap<String, String>, pub requester: String } |  |  |  |  |

| `RouteDecision` | struct | RouteDecision { pub capability: String, pub executor_class: String } |  |  |  |  |

| `StaticWorld` | struct | StaticWorld { pub effects: BTreeMap<String, String> } |  |  |  |  |

| `VerificationEvidence` | struct | VerificationEvidence { pub verifier_identity: String, pub verdict: String, pub evidence_digest: String } |  |  |  |  |

| `Actuator` | trait |  |  |  |  |  |

| `Observer` | trait |  |  |  |  |  |


### src/canonical_jcs.rs

| `JcsError` | enum | JcsError { Serialization(String) } |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &serde_json::Value) -> Result<Vec<u8>, JcsError> |  |  |  |  |

| `canonical_commitment` | function | canonical_commitment( value: &serde_json::Value, ) -> Result<crate::types::Blake3Hash, JcsError> |  |  |  |  |

| `to_jcs` | function | to_jcs(value: &T) -> Result<Vec<u8>, JcsError> |  |  |  |  |


### src/canonical_time.rs

| `CanonicalTimeError` | enum | CanonicalTimeError { Malformed(String), Overflow } |  |  |  |  |

| `canonical_string` | function | canonical_string(ts: Timestamp) -> String |  |  |  |  |

| `canonicalize` | function | canonicalize(s: &str) -> Result<String, CanonicalTimeError> |  |  |  |  |

| `checked_add_ms` | function | checked_add_ms(ts: Timestamp, millis: u64) -> Result<Timestamp, CanonicalTimeError> |  |  |  |  |

| `parse` | function | parse(s: &str) -> Result<Timestamp, CanonicalTimeError> |  |  |  |  |

| `iso8601_timestamp::Timestamp` | use | iso8601_timestamp::Timestamp |  |  |  |  |


### src/catalog.rs

| `format_catalog` | function | format_catalog(fixtures: &[Fixture]) -> String |  |  |  |  |

| `list_fixtures` | function | list_fixtures( db: &FixtureDatabase, name_filter: Option<String>, events_filter: Option<usize>, ) -> Vec<Fixture> |  |  |  |  |


### src/causal_graph.rs

| `CausalError` | enum | CausalError { CycleDetected(String), UnknownNode(String) } |  |  |  |  |

| `add_dependency` | function | add_dependency(&mut self, from: K, to: K) |  |  |  |  |

| `contains` | function | contains(&self, node: K) -> bool |  |  |  |  |

| `edge_count` | function | edge_count(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `topological_order` | function | topological_order(&self) -> Result<Vec<K>, CausalError> |  |  |  |  |

| `verify_acyclic` | function | verify_acyclic(&self) -> Result<(), CausalError> |  |  |  |  |

| `CausalGraph` | struct | CausalGraph { graph: DiGraphMap<K, ()> } |  |  |  |  |


### src/chain.rs

| `FORMAT_VERSION` | const | FORMAT_VERSION: &str |  |  |  |  |

| `GENESIS_SEED` | const | GENESIS_SEED: &[u8] |  |  |  |  |

| `WORKING_PATH` | const | WORKING_PATH: &str |  |  |  |  |

| `ChainError` | enum | ChainError { Encode( Decode( Io { path: String, source: std::io::Error, }, } |  |  |  |  |

| `append` | function | append(&mut self, event: OperationEvent) -> Result<(), ChainError> |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) -> Result<Blake3Hash, ChainError> |  |  |  |  |

| `deserialize_receipt` | function | deserialize_receipt(bytes: &[u8]) -> Result<Receipt, ChainError> |  |  |  |  |

| `events` | function | events(&self) -> &[OperationEvent] |  |  |  |  |

| `finalize` | function | finalize(self) -> Receipt |  |  |  |  |

| `from_events` | function | from_events(events: Vec<OperationEvent>) -> Result<Self, ChainError> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `load_working` | function | load_working() -> Result<Vec<OperationEvent>, ChainError> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) -> Result<Blake3Hash, ChainError> |  |  |  |  |

| `save_receipt` | function | save_receipt(receipt: &Receipt, path: &Path) -> Result<(), ChainError> |  |  |  |  |

| `save_working` | function | save_working(events: &[OperationEvent]) -> Result<(), ChainError> |  |  |  |  |

| `serialize_receipt` | function | serialize_receipt(receipt: &Receipt) -> Result<Vec<u8>, ChainError> |  |  |  |  |

| `.affi/working.json` | str_key | WORKING_PATH = ".affi/working.json" |  |  |  |  |

| `affidavit-v` | str_key | GENESIS_SEED_STR = "affidavit-v" |  |  |  |  |

| `core/v1` | str_key | FORMAT_VERSION = "core/v1" |  |  |  |  |

| `ChainAssembler` | struct | ChainAssembler { events: Vec<OperationEvent>, running: Blake3Hash } |  |  |  |  |


### src/cli.rs

| `assemble` | function | assemble(out: Option<&str>) -> Result<crate::types::AssembleOutput> |  |  |  |  |

| `emit` | function | emit( event_type: &str, objects: &[String], payload: &str, ) -> Result<crate::types::EmitOutput> |  |  |  |  |

| `show` | function | show(receipt: &str) -> Result<Receipt> |  |  |  |  |

| `verify` | function | verify(receipt: &str) -> Result<(i32, crate::types::Verdict)> |  |  |  |  |


### src/crypto_trust_attestation.rs

| `ATTESTATION_FORMAT` | const | ATTESTATION_FORMAT: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `AttestationError` | enum | AttestationError { BadFormat(String), BadSignature, AttributeMismatch(String), Serialization(String) } |  |  |  |  |

| `AttestationKind` | enum | AttestationKind { Self_, Hardware, Import } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `attest_key` | function | attest_key( record: &KeyRecord, kind: AttestationKind, device: Option<(&str, bool, bool)>, signer: &Es256SigningKey, signer_kid: &str, at: u64, ) -> Result<AttestationRecord, AttestationError> |  |  |  |  |

| `verify_attestation` | function | verify_attestation( att: &AttestationRecord, signer_pk: &[u8], ) -> Result<bool, AttestationError> |  |  |  |  |

| `CTP-ATTEST-v1` | str_key | ATTESTATION_FORMAT = "CTP-ATTEST-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `AttestationRecord` | struct | AttestationRecord { pub format: String, pub key_id: String, pub kind: String, pub device_model: Option<String>, pub secure_hardware: bool, pub non_exportable: bool, pub attested_at: u64, pub signature: Vec<u8> } |  |  |  |  |


### src/crypto_trust_canonical.rs

| `CANONICALIZATION` | const | CANONICALIZATION: &str |  |  |  |  |

| `DIGEST_ALGORITHM` | const | DIGEST_ALGORITHM: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `CanonicalError` | enum | CanonicalError { Json(String), NonCanonicalNumber(String) } |  |  |  |  |

| `digest` | function | digest(domain: &str, parts: &[&[u8]]) -> [u8; 32] |  |  |  |  |

| `digest_hex` | function | digest_hex(domain: &str, parts: &[&[u8]]) -> String |  |  |  |  |

| `domain_separated` | function | domain_separated(domain: &str, parts: &[&[u8]]) -> Vec<u8> |  |  |  |  |

| `jcs` | function | jcs(value: &Value) -> Result<String, CanonicalError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `BLAKE3` | str_key | DIGEST_ALGORITHM = "BLAKE3" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `JCS-RFC8785` | str_key | CANONICALIZATION = "JCS-RFC8785" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |


### src/crypto_trust_crl_file.rs

| `CRL_FILE` | const | CRL_FILE: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `CrlFileError` | enum | CrlFileError { WrongFormat(String), Io(String), Serialization(String), Verification(String), Staleness(String) } |  |  |  |  |

| `load_and_apply` | function | load_and_apply( path: Option<&Path>, revocations: &mut RevocationList, issuer_pk: &[u8], current_epoch: u64, max_staleness: u64, now: u64, ) -> Result<usize, CrlFileError> |  |  |  |  |

| `new` | function | new(path: impl Into<PathBuf>) -> CrlFile |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `publish_verified` | function | publish_verified( revocations: &RevocationList, issuer: &Es256SigningKey, issuer_kid: &KeyId, epoch: u64, at: u64, out: Option<&Path>, ) -> Result<PathBuf, CrlFileError> |  |  |  |  |

| `read` | function | read(path: Option<&Path>) -> Result<SignedRevocationList, CrlFileError> |  |  |  |  |

| `write` | function | write( list: &SignedRevocationList, path: Option<&Path>, ) -> Result<PathBuf, CrlFileError> |  |  |  |  |

| `.affi/crl.json` | str_key | CRL_FILE = ".affi/crl.json" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `compromised` | str_key | REASON = "compromised" |  |  |  |  |

| `k-a` | str_key | KID_A = "k-a" |  |  |  |  |

| `k-b` | str_key | KID_B = "k-b" |  |  |  |  |

| `CrlFile` | struct | CrlFile { path: PathBuf } |  |  |  |  |


### src/crypto_trust_doctor.rs

| `DOCTOR_CHECK_DOMAIN` | const | DOCTOR_CHECK_DOMAIN: &str |  |  |  |  |

| `ENVELOPE_LAW_CHECK_ID` | const | ENVELOPE_LAW_CHECK_ID: &str |  |  |  |  |

| `ES256_SELFTEST_CHECK_ID` | const | ES256_SELFTEST_CHECK_ID: &str |  |  |  |  |

| `PQC_SELFTEST_CHECK_ID` | const | PQC_SELFTEST_CHECK_ID: &str |  |  |  |  |

| `STORE_INTEGRITY_CHECK_ID` | const | STORE_INTEGRITY_CHECK_ID: &str |  |  |  |  |

| `envelope_law_finding` | function | envelope_law_finding() -> Finding |  |  |  |  |

| `es256_selftest_finding` | function | es256_selftest_finding() -> Finding |  |  |  |  |

| `pqc_selftest_finding` | function | pqc_selftest_finding() -> Finding |  |  |  |  |

| `run_crypto_checks` | function | run_crypto_checks() -> Vec<Finding> |  |  |  |  |

| `store_integrity_finding` | function | store_integrity_finding(path: &Path) -> Finding |  |  |  |  |

| `affidavit.crypto-trust-plane.v1/doctor` | str_key | DOCTOR_CHECK_DOMAIN = "affidavit.crypto-trust-plane.v1/doctor" |  |  |  |  |

| `crypto-envelope-law` | str_key | ENVELOPE_LAW_CHECK_ID = "crypto-envelope-law" |  |  |  |  |

| `crypto-es256-selftest` | str_key | ES256_SELFTEST_CHECK_ID = "crypto-es256-selftest" |  |  |  |  |

| `crypto-pqc-selftest` | str_key | PQC_SELFTEST_CHECK_ID = "crypto-pqc-selftest" |  |  |  |  |

| `crypto-store-integrity` | str_key | STORE_INTEGRITY_CHECK_ID = "crypto-store-integrity" |  |  |  |  |


### src/crypto_trust_enclave.rs

| `ALGORITHM_NAME` | const | ALGORITHM_NAME: &str |  |  |  |  |

| `ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE` | const | ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE: &str |  |  |  |  |

| `KEY_MATERIAL_EXPORTABLE` | const | KEY_MATERIAL_EXPORTABLE: bool |  |  |  |  |

| `PROVIDER_KIND` | const | PROVIDER_KIND: &str |  |  |  |  |

| `TARGET_OS` | const | TARGET_OS: &str |  |  |  |  |

| `EnclaveError` | enum | EnclaveError { SecurityFramework(String), KeyNotFound(String), UnsupportedPlatform } |  |  |  |  |

| `delete_enclave_key` | function | delete_enclave_key(label: &str) -> Result<(), EnclaveError> |  |  |  |  |

| `enclave_sign` | function | enclave_sign(label: &str, msg: &[u8]) -> Result<Vec<u8>, EnclaveError> |  |  |  |  |

| `enclave_verify` | function | enclave_verify( public_key_sec1: &[u8], msg: &[u8], sig_der: &[u8], ) -> Result<bool, EnclaveError> |  |  |  |  |

| `generate_enclave_key` | function | generate_enclave_key(label: &str) -> Result<EnclaveKeyRef, EnclaveError> |  |  |  |  |

| `ES256` | str_key | ALGORITHM_NAME = "ES256" |  |  |  |  |

| `SECURE_ENCLAVE` | str_key | PROVIDER_KIND = "SECURE_ENCLAVE" |  |  |  |  |

| `Unblock for errSecMissingEntitlement (-34018): Secure Enclave keygen succeeds ` | str_key | ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE = "Unblock for errSecMissingEntitlement (-34018): Secure Enclave keygen succeeds " |  |  |  |  |

| `macos` | str_key | TARGET_OS = "macos" |  |  |  |  |

| `EnclaveKeyRef` | struct | EnclaveKeyRef { pub key_id: KeyId, pub label: String, pub public_key_sec1: Vec<u8> } |  |  |  |  |

| `imp::{delete_enclave_key, enclave_sign, enclave_verify, generate_enclave_key}` | use | imp::{delete_enclave_key, enclave_sign, enclave_verify, generate_enclave_key} |  |  |  |  |


### src/crypto_trust_envelope.rs

| `ENVELOPE_FIELDS` | const | ENVELOPE_FIELDS: [(&str, u32); 12] |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `EnvelopeError` | enum | EnvelopeError { Malformed(String), Expired(u64), NotYetValid(u64), ReplayRejected(String), WrongVersion(String) } |  |  |  |  |

| `envelope_document` | function | envelope_document(&self) -> serde_json::Value |  |  |  |  |

| `from_bytes` | function | from_bytes(b: &[u8]) -> Result<Self, EnvelopeError> |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64, window_seconds: u64) -> usize |  |  |  |  |

| `record` | function | record( &mut self, kid: &str, nonce: [u8; 16], at: u64, window_seconds: u64, ) -> Result<(), EnvelopeError> |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> |  |  |  |  |

| `signing_input` | function | signing_input(&self) -> Vec<u8> |  |  |  |  |

| `signing_input_checked` | function | signing_input_checked(&self) -> Result<Vec<u8>, EnvelopeError> |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) -> Result<Vec<u8>, EnvelopeError> |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) -> Result<(), EnvelopeError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `version` | str_key | ENVELOPE_FIELDS = "version" |  |  |  |  |

| `NonceJournal` | struct | NonceJournal { seen: BTreeMap<String, (String, u64)> } |  |  |  |  |

| `SignatureEnvelope` | struct | SignatureEnvelope { pub version: String, pub algorithm: AlgorithmId, pub key_id: KeyId, pub profile: CryptoProfile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: String } |  |  |  |  |


### src/crypto_trust_es256.rs

| `ALGORITHM_NAME` | const | ALGORITHM_NAME: &str |  |  |  |  |

| `DETERMINISTIC` | const | DETERMINISTIC: bool |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `PUBLIC_KEY_LENGTH` | const | PUBLIC_KEY_LENGTH: usize |  |  |  |  |

| `SPEC_REF` | const | SPEC_REF: &str |  |  |  |  |

| `Es256Error` | enum | Es256Error { P256(String), MalformedPublicKey, MalformedSignature } |  |  |  |  |

| `from_seed` | function | from_seed(seed: &[u8; 32]) -> Result<Self, Es256Error> |  |  |  |  |

| `generate` | function | generate() -> Result<Self, Es256Error> |  |  |  |  |

| `key_id_fingerprint` | function | key_id_fingerprint(&self) -> crate::crypto_trust_keys::KeyFingerprint |  |  |  |  |

| `public_key_sec1` | function | public_key_sec1(&self) -> Vec<u8> |  |  |  |  |

| `sign` | function | sign(&self, msg: &[u8]) -> Vec<u8> |  |  |  |  |

| `verify_es256` | function | verify_es256( public_key_sec1: &[u8], msg: &[u8], sig_der: &[u8], ) -> Result<bool, Es256Error> |  |  |  |  |

| `A6E3C57DD01ABE90086538398355DD4C3B17AA873382B0F24D6129493D8AAD60` | str_key | RFC6979_K = "A6E3C57DD01ABE90086538398355DD4C3B17AA873382B0F24D6129493D8AAD60" |  |  |  |  |

| `C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6721` | str_key | RFC6979_X = "C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6721" |  |  |  |  |

| `EFD48B2AACB6A8FD1140DD9CD45E81D69D2C877B56AAF991C34D0EA84EAF3716` | str_key | RFC6979_R = "EFD48B2AACB6A8FD1140DD9CD45E81D69D2C877B56AAF991C34D0EA84EAF3716" |  |  |  |  |

| `ES256` | str_key | ALGORITHM_NAME = "ES256" |  |  |  |  |

| `F7CB1C942D657C41D436C7A1B6E29F65F3E900DBB9AFF4064DC4AB2F843ACDA8` | str_key | RFC6979_S = "F7CB1C942D657C41D436C7A1B6E29F65F3E900DBB9AFF4064DC4AB2F843ACDA8" |  |  |  |  |

| `FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551` | str_key | P256_ORDER_HEX = "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551" |  |  |  |  |

| `RFC 6979 / SEC 2` | str_key | SPEC_REF = "RFC 6979 / SEC 2" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `sample` | str_key | RFC6979_MESSAGE = "sample" |  |  |  |  |

| `Es256SigningKey` | struct | Es256SigningKey { secret: SigningKey } |  |  |  |  |


### src/crypto_trust_journal.rs

| `JOURNAL_DOMAIN` | const | JOURNAL_DOMAIN: &str |  |  |  |  |

| `JOURNAL_FORMAT` | const | JOURNAL_FORMAT: &str |  |  |  |  |

| `JOURNAL_GENESIS` | const | JOURNAL_GENESIS: &str |  |  |  |  |

| `JournalError` | enum | JournalError { SeqGap { expected: u64, got: u64, }, ChainBroken { at: u64, }, Serialization(String) } |  |  |  |  |

| `append` | function | append(&mut self, draft: JournalEntryDraft) -> Result<JournalEntry, JournalError> |  |  |  |  |

| `by_epoch` | function | by_epoch(&self, epoch: u64) -> Vec<&JournalEntry> |  |  |  |  |

| `by_key` | function | by_key(&self, kid: &str) -> Vec<&JournalEntry> |  |  |  |  |

| `entries` | function | entries(&self) -> &[JournalEntry] |  |  |  |  |

| `from_jsonl` | function | from_jsonl(s: &str) -> Result<Self, JournalError> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `record_receipt` | function | record_receipt( journal: &mut StandingJournal, receipt: &crate::crypto_trust_verify::CryptoStandingReceipt, policy_epoch: u64, revocation_epoch: u64, ) -> Result<JournalEntry, JournalError> |  |  |  |  |

| `to_jsonl` | function | to_jsonl(&self) -> String |  |  |  |  |

| `verify_chain` | function | verify_chain(&self) -> Result<(), JournalError> |  |  |  |  |

| `0000000000000000000000000000000000000000000000000000000000000000` | str_key | JOURNAL_GENESIS = "0000000000000000000000000000000000000000000000000000000000000000" |  |  |  |  |

| `1111111111111111111111111111111111111111111111111111111111111111` | str_key | COMMITMENT = "1111111111111111111111111111111111111111111111111111111111111111" |  |  |  |  |

| `2222222222222222222222222222222222222222222222222222222222222222` | str_key | RECEIPT_HASH = "2222222222222222222222222222222222222222222222222222222222222222" |  |  |  |  |

| `CTP-JOURNAL-v1` | str_key | JOURNAL_FORMAT = "CTP-JOURNAL-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | JOURNAL_DOMAIN = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `afk1_aaaaaaaaaaaaaaaa` | str_key | KID_A = "afk1_aaaaaaaaaaaaaaaa" |  |  |  |  |

| `afk1_bbbbbbbbbbbbbbbb` | str_key | KID_B = "afk1_bbbbbbbbbbbbbbbb" |  |  |  |  |

| `JournalEntry` | struct | JournalEntry { pub seq: u64, pub prev: String, pub key_id: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub standing: String, pub envelope_commitment: String, pub receipt_hash: String, pub entry_hash: String } |  |  |  |  |

| `JournalEntryDraft` | struct | JournalEntryDraft { pub key_id: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub standing: String, pub envelope_commitment: String, pub receipt_hash: String } |  |  |  |  |

| `StandingJournal` | struct | StandingJournal { entries: Vec<JournalEntry> } |  |  |  |  |


### src/crypto_trust_journal_persist.rs

| `PERSIST_DOMAIN_TAG` | const | PERSIST_DOMAIN_TAG: &str |  |  |  |  |

| `PERSIST_FILE` | const | PERSIST_FILE: &str |  |  |  |  |

| `PersistError` | enum | PersistError { Replay { kid: String, }, Io(String), Corruption { line: usize, reason: String, }, Serialization(String), Lock(String) } |  |  |  |  |

| `entries` | function | entries(&self) -> &[NonceRecord] |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `load` | function | load(path: impl Into<PathBuf>) -> Result<Self, PersistError> |  |  |  |  |

| `open_or_create` | function | open_or_create(path: impl Into<PathBuf>) -> Result<Self, PersistError> |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64, window_seconds: u64) -> Result<usize, PersistError> |  |  |  |  |

| `record` | function | record( &mut self, kid: &str, nonce: [u8; 16], at: u64, window_seconds: u64, ) -> Result<(), PersistError> |  |  |  |  |

| `refresh` | function | refresh(&mut self) -> Result<(), PersistError> |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> |  |  |  |  |

| `verify_integrity` | function | verify_integrity(&self) -> Result<(), PersistError> |  |  |  |  |

| `.affi/nonce-journal.jsonl` | str_key | PERSIST_FILE = ".affi/nonce-journal.jsonl" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | PERSIST_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `afk1_aaaaaaaaaaaaaaaa` | str_key | KID_A = "afk1_aaaaaaaaaaaaaaaa" |  |  |  |  |

| `afk1_bbbbbbbbbbbbbbbb` | str_key | KID_B = "afk1_bbbbbbbbbbbbbbbb" |  |  |  |  |

| `NonceLedgerFile` | struct | NonceLedgerFile { path: PathBuf, entries: Vec<NonceRecord> } |  |  |  |  |

| `NonceRecord` | struct | NonceRecord { pub kid: String, pub nonce_hex: String, pub seen_at: u64 } |  |  |  |  |


### src/crypto_trust_jwks.rs

| `JwksError` | enum | JwksError { MalformedPublicKey(String, String), UnsupportedAlgorithm(String), FeatureRequired, DuplicateKid(String) } |  |  |  |  |

| `export_jwk` | function | export_jwk(record: &KeyRecord) -> Result<serde_json::Value, JwksError> |  |  |  |  |

| `export_jwks` | function | export_jwks(records: &[KeyRecord]) -> Result<serde_json::Value, JwksError> |  |  |  |  |

| `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_` | str_key | ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_" |  |  |  |  |


### src/crypto_trust_kat.rs

| `KAT_ALGORITHM_REGISTRY` | const | KAT_ALGORITHM_REGISTRY: &str |  |  |  |  |

| `KAT_DOMAIN_TAG` | const | KAT_DOMAIN_TAG: &str |  |  |  |  |

| `KAT_ENVELOPE_VERSION` | const | KAT_ENVELOPE_VERSION: &str |  |  |  |  |

| `KAT_SCHEMA_VERSION` | const | KAT_SCHEMA_VERSION: &str |  |  |  |  |

| `KatError` | enum | KatError { AlgorithmUnknown(String), VerificationFailed { vector_id: String, }, MalformedVector { vector_id: String, }, Serialization(String) } |  |  |  |  |

| `build_vector` | function | build_vector(index: usize, algorithm: &str) -> Result<KatVector, KatError> |  |  |  |  |

| `export_json` | function | export_json(corpus: &[KatVector]) -> String |  |  |  |  |

| `generate_corpus` | function | generate_corpus() -> Vec<KatVector> |  |  |  |  |

| `import_json` | function | import_json(s: &str) -> Result<Vec<KatVector>, KatError> |  |  |  |  |

| `verify_corpus` | function | verify_corpus(vectors: &[KatVector]) -> Result<KatReport, KatError> |  |  |  |  |

| `verify_vector` | function | verify_vector(vector: &KatVector) -> Result<bool, KatError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | KAT_ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `CTP-KAT-v1` | str_key | KAT_SCHEMA_VERSION = "CTP-KAT-v1" |  |  |  |  |

| `ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s` | str_key | KAT_ALGORITHM_REGISTRY = "ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | KAT_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `KatReport` | struct | KatReport { pub total: usize, pub passed: usize } |  |  |  |  |

| `KatVector` | struct | KatVector { pub vector_id: String, pub algorithm: String, pub seed_hex: String, pub public_key_hex: String, pub message_hex: String, pub signature_hex: String } |  |  |  |  |


### src/crypto_trust_keys.rs

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `AlgorithmId` | enum | AlgorithmId { Es256, HybridEs256MlDsa65, MlDsa65, SlhDsa128s, Ed25519, Es256k } |  |  |  |  |

| `CryptoProfile` | enum | CryptoProfile { Classical, Hybrid, Pqc } |  |  |  |  |

| `KeyOrigin` | enum | KeyOrigin { HardwareAttested { device: String }, Imported { source: String }, Generated } |  |  |  |  |

| `KeyProviderKind` | enum | KeyProviderKind { Hsm, SecureEnclave, Software } |  |  |  |  |

| `PublicKeyMaterial` | enum | PublicKeyMaterial { Es256Sec1(Vec<u8>), Hybrid { es256: Vec<u8>, mldsa65: Vec<u8> }, MlDsa65(Vec<u8>), SlhDsa128s(Vec<u8>), Ed25519(Vec<u8>), Es256kSec1(Vec<u8>) } |  |  |  |  |

| `RegistryError` | enum | RegistryError { Duplicate(KeyId), DuplicateFingerprint(String), Unknown(KeyId) } |  |  |  |  |

| `algorithm` | function | algorithm(&self) -> AlgorithmId |  |  |  |  |

| `all` | function | all() -> &'static [AlgorithmId] |  |  |  |  |

| `as_hex` | function | as_hex(&self) -> String |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(&self) -> Vec<u8> |  |  |  |  |

| `encoded_len` | function | encoded_len(&self) -> usize |  |  |  |  |

| `fingerprint_public_key` | function | fingerprint_public_key( algorithm: AlgorithmId, public_key: &PublicKeyMaterial, ) -> KeyFingerprint |  |  |  |  |

| `from_fingerprint` | function | from_fingerprint(fingerprint: &KeyFingerprint) -> KeyId |  |  |  |  |

| `key_material_exportable` | function | key_material_exportable(self) -> bool |  |  |  |  |

| `kind` | function | kind(self) -> &'static str |  |  |  |  |

| `new` | function | new() -> InMemoryKeyRegistry |  |  |  |  |

| `profile` | function | profile(self) -> CryptoProfile |  |  |  |  |

| `public_key_len` | function | public_key_len(self) -> Option<usize> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `CustodianIdentity` | struct | CustodianIdentity { pub subject: String, pub device: Option<String>, pub org: Option<String> } |  |  |  |  |

| `InMemoryKeyRegistry` | struct | InMemoryKeyRegistry { records: BTreeMap<KeyId, KeyRecord>, fingerprints: BTreeSet<KeyFingerprint> } |  |  |  |  |

| `KeyFingerprint` | struct | KeyFingerprint { pub [u8; 32] } |  |  |  |  |

| `KeyId` | struct | KeyId { pub String } |  |  |  |  |

| `KeyRecord` | struct | KeyRecord { pub id: KeyId, pub algorithm: AlgorithmId, pub fingerprint: KeyFingerprint, pub custodian: CustodianIdentity, pub origin: KeyOrigin, pub public_key: PublicKeyMaterial, pub created_epoch: u64 } |  |  |  |  |

| `KeyRegistry` | trait |  |  |  |  |  |


### src/crypto_trust_lifecycle.rs

| `MAX_REVOCATION_STALENESS_SECONDS` | const | MAX_REVOCATION_STALENESS_SECONDS: u64 |  |  |  |  |

| `NONCE_WINDOW_SECONDS` | const | NONCE_WINDOW_SECONDS: u64 |  |  |  |  |

| `REPLAY_KEY` | const | REPLAY_KEY: &str |  |  |  |  |

| `LifecycleRefusal` | enum | LifecycleRefusal { Revoked(String, u64, String), NoActiveEpoch(String), EpochExpired { kid: String, index: u64, max: u64 }, EpochsExhausted(String), AlreadyRetired(u64), NonMonotonic(u64, u64) } |  |  |  |  |

| `active_at` | function | active_at(&self, now: u64) -> bool |  |  |  |  |

| `active_epoch` | function | active_epoch(&self, kid: &str) -> Option<&KeyEpoch> |  |  |  |  |

| `admit_opening` | function | admit_opening(&self, kid: &str, history: &[KeyEpoch]) -> Result<(), LifecycleRefusal> |  |  |  |  |

| `current_epoch` | function | current_epoch(&self) -> u64 |  |  |  |  |

| `epochs` | function | epochs(&self, kid: &str) -> &[KeyEpoch] |  |  |  |  |

| `is_revoked` | function | is_revoked(&self, kid: &str) -> bool |  |  |  |  |

| `new` | function | new(policy: RotationPolicy) -> Self |  |  |  |  |

| `open_epoch` | function | open_epoch(&mut self, kid: &str, at: u64) -> Result<u64, LifecycleRefusal> |  |  |  |  |

| `policy` | function | policy(&self) -> &RotationPolicy |  |  |  |  |

| `require_active_epoch` | function | require_active_epoch(&self, kid: &str, now: u64) -> Result<&KeyEpoch, LifecycleRefusal> |  |  |  |  |

| `retire_epoch` | function | retire_epoch(&mut self, kid: &str, index: u64, at: u64) |  |  |  |  |

| `revoke` | function | revoke(&mut self, kid: &str, at: u64, reason: String) |  |  |  |  |

| `revoked_at` | function | revoked_at(&self, kid: &str) -> Option<u64> |  |  |  |  |

| `signature_epoch_live` | function | signature_epoch_live( &self, kid: &str, sig_revocation_epoch: u64, now: u64, ) -> Result<(), LifecycleRefusal> |  |  |  |  |

| `validate_epoch` | function | validate_epoch( &self, kid: &str, epoch: &KeyEpoch, now: u64, ) -> Result<(), LifecycleRefusal> |  |  |  |  |

| `validate_signature_key` | function | validate_signature_key(&self, kid: &str, now: u64) -> Result<(), LifecycleRefusal> |  |  |  |  |

| `compromised` | str_key | REASON = "compromised" |  |  |  |  |

| `k-1` | str_key | KID = "k-1" |  |  |  |  |

| `kid,nonce` | str_key | REPLAY_KEY = "kid,nonce" |  |  |  |  |

| `KeyEpoch` | struct | KeyEpoch { pub index: u64, pub activated_at: u64, pub retired_at: Option<u64> } |  |  |  |  |

| `LifecycleLedger` | struct | LifecycleLedger { epochs: BTreeMap<String, Vec<KeyEpoch>>, policy: RotationPolicy } |  |  |  |  |

| `RevocationList` | struct | RevocationList { records: BTreeMap<String, RevocationRecord> } |  |  |  |  |

| `RevocationRecord` | struct | RevocationRecord { pub revoked_at: u64, pub reason: String } |  |  |  |  |

| `RotationPolicy` | struct | RotationPolicy { pub max_epochs_in_flight: usize, pub max_age_seconds: u64 } |  |  |  |  |


### src/crypto_trust_log.rs

| `HEAD_SIGNING_DOMAIN` | const | HEAD_SIGNING_DOMAIN: &str |  |  |  |  |

| `OPS_FILE` | const | OPS_FILE: &str |  |  |  |  |

| `OPS_FORMAT` | const | OPS_FORMAT: &str |  |  |  |  |

| `LogOpsError` | enum | LogOpsError { CommitmentMismatch, Journal(String), Transparency(String), Signing(String), Serialization(String) } |  |  |  |  |

| `audit` | function | audit(&self) -> Result<AuditReport, LogOpsError> |  |  |  |  |

| `from_jsonl` | function | from_jsonl(s: &str) -> Result<Self, LogOpsError> |  |  |  |  |

| `heads` | function | heads(&self) -> &[SignedTreeHead] |  |  |  |  |

| `journal` | function | journal(&self) -> &StandingJournal |  |  |  |  |

| `leaf` | function | leaf(&self, index: u64) -> Option<&[u8; 32]> |  |  |  |  |

| `log` | function | log(&self) -> &TransparencyLog |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `prove_inclusion` | function | prove_inclusion(&self, leaf: u64) -> Result<(InclusionProof, [u8; 32]), LogOpsError> |  |  |  |  |

| `publish_head` | function | publish_head( &self, signer: &Es256SigningKey, kid: &str, now: u64, ) -> Result<SignedTreeHead, LogOpsError> |  |  |  |  |

| `record` | function | record( &mut self, receipt: &CryptoStandingReceipt, commitment: [u8; 32], signer: &Es256SigningKey, signer_kid: &str, now: u64, ) -> Result<(JournalSeq, LeafIndex), LogOpsError> |  |  |  |  |

| `to_jsonl` | function | to_jsonl(&self) -> String |  |  |  |  |

| `verify_head` | function | verify_head(head: &SignedTreeHead, public_key_sec1: &[u8]) -> Result<bool, LogOpsError> |  |  |  |  |

| `.affi/transparency.jsonl` | str_key | OPS_FILE = ".affi/transparency.jsonl" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `CTP-LOGOPS-v1` | str_key | OPS_FORMAT = "CTP-LOGOPS-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | HEAD_SIGNING_DOMAIN = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `afk1_transparency-log-ops` | str_key | SIGNER_KID = "afk1_transparency-log-ops" |  |  |  |  |

| `AuditReport` | struct | AuditReport { pub journal_entries: u64, pub log_leaves: u64, pub consistent: bool } |  |  |  |  |

| `SignedTreeHead` | struct | SignedTreeHead { pub tree_size: u64, pub head_hex: String, pub timestamp: u64, pub kid: String, pub signature: Vec<u8> } |  |  |  |  |

| `TrustLogOps` | struct | TrustLogOps { journal: StandingJournal, log: TransparencyLog, heads: Vec<SignedTreeHead> } |  |  |  |  |


### src/crypto_trust_nonce_store.rs

| `DEFAULT_WINDOW_SECONDS` | const | DEFAULT_WINDOW_SECONDS: u64 |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `JOURNAL_FORMAT` | const | JOURNAL_FORMAT: &str |  |  |  |  |

| `REPLAY_KEY` | const | REPLAY_KEY: &str |  |  |  |  |

| `STORE_FILE` | const | STORE_FILE: &str |  |  |  |  |

| `NonceStoreError` | enum | NonceStoreError { Io( WrongFormat { expected: String, found: String, }, Corrupt { line: usize, reason: String, }, ReplayRejected(String), Lock(String), } |  |  |  |  |

| `create` | function | create(path: P) -> Result<Self, NonceStoreError> |  |  |  |  |

| `default_journal_path` | function | default_journal_path() -> PathBuf |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `open` | function | open(path: P, window_seconds: u64) -> Result<Self, NonceStoreError> |  |  |  |  |

| `open_default` | function | open_default() -> Result<Self, NonceStoreError> |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64) -> Result<usize, NonceStoreError> |  |  |  |  |

| `record` | function | record(&mut self, kid: &str, nonce: &[u8; 16], at: u64) -> Result<(), NonceStoreError> |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> |  |  |  |  |

| `.affi/keys.json` | str_key | STORE_FILE = ".affi/keys.json" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `CTP-NONCE-JOURNAL-v1` | str_key | JOURNAL_FORMAT = "CTP-NONCE-JOURNAL-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `kid,nonce` | str_key | REPLAY_KEY = "kid,nonce" |  |  |  |  |

| `DiskNonceJournal` | struct | DiskNonceJournal { path: PathBuf, entries: BTreeMap<String, (String, u64)>, window_seconds: u64 } |  |  |  |  |


### src/crypto_trust_pqc.rs

| `HYBRID_SPEC_REF` | const | HYBRID_SPEC_REF: &str |  |  |  |  |

| `ML_DSA_65_PUBLIC_KEY_LEN` | const | ML_DSA_65_PUBLIC_KEY_LEN: usize |  |  |  |  |

| `ML_DSA_65_SEED_LEN` | const | ML_DSA_65_SEED_LEN: usize |  |  |  |  |

| `ML_DSA_65_SIGNATURE_LEN` | const | ML_DSA_65_SIGNATURE_LEN: usize |  |  |  |  |

| `ML_DSA_65_SPEC_REF` | const | ML_DSA_65_SPEC_REF: &str |  |  |  |  |

| `PQC_DOMAIN_TAG` | const | PQC_DOMAIN_TAG: &str |  |  |  |  |

| `SLH_DSA_128S_SEED_LEN` | const | SLH_DSA_128S_SEED_LEN: usize |  |  |  |  |

| `SLH_DSA_128S_SPEC_REF` | const | SLH_DSA_128S_SPEC_REF: &str |  |  |  |  |

| `PqcError` | enum | PqcError { MlDsa(String), SlhDsa(String), Es256Half( MalformedPublicKey, MalformedSignature, } |  |  |  |  |

| `hybrid_sign` | function | hybrid_sign(secret: &HybridSecret, msg: &[u8]) -> Result<HybridSignature, PqcError> |  |  |  |  |

| `hybrid_verify` | function | hybrid_verify( es256_pk: &[u8], mldsa65_pk: &[u8], msg: &[u8], sig: &HybridSignature, ) -> Result<bool, PqcError> |  |  |  |  |

| `ml_dsa65_from_seed` | function | ml_dsa65_from_seed(seed: &[u8; ML_DSA_65_SEED_LEN]) -> MlDsa65KeyPair |  |  |  |  |

| `ml_dsa65_generate` | function | ml_dsa65_generate() -> Result<MlDsa65KeyPair, PqcError> |  |  |  |  |

| `ml_dsa65_sign` | function | ml_dsa65_sign( seed: &[u8; ML_DSA_65_SEED_LEN], msg: &[u8], rnd: &[u8; 32], ) -> Result<Vec<u8>, PqcError> |  |  |  |  |

| `ml_dsa65_verify` | function | ml_dsa65_verify(public: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, PqcError> |  |  |  |  |

| `slh_dsa128s_from_seed` | function | slh_dsa128s_from_seed(seeds: &[u8; SLH_DSA_128S_SEED_LEN]) -> SlhDsa128sKeyPair |  |  |  |  |

| `slh_dsa128s_generate` | function | slh_dsa128s_generate() -> Result<SlhDsa128sKeyPair, PqcError> |  |  |  |  |

| `slh_dsa128s_sign` | function | slh_dsa128s_sign( seeds: &[u8; SLH_DSA_128S_SEED_LEN], msg: &[u8], ) -> Result<Vec<u8>, PqcError> |  |  |  |  |

| `slh_dsa128s_verify` | function | slh_dsa128s_verify(public: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, PqcError> |  |  |  |  |

| `FIPS 204` | str_key | ML_DSA_65_SPEC_REF = "FIPS 204" |  |  |  |  |

| `FIPS 205` | str_key | SLH_DSA_128S_SPEC_REF = "FIPS 205" |  |  |  |  |

| `affidavit pqc lane: ML-DSA-65 / SLH-DSA-SHA2-128s / hybrid` | str_key | MSG = "affidavit pqc lane: ML-DSA-65 / SLH-DSA-SHA2-128s / hybrid" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | PQC_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `draft-ietf-lamps-pq-composite-sig` | str_key | HYBRID_SPEC_REF = "draft-ietf-lamps-pq-composite-sig" |  |  |  |  |

| `HybridSecret` | struct | HybridSecret { pub es256: Es256SigningKey, pub mldsa65_seed: [u8; ML_DSA_65_SEED_LEN] } |  |  |  |  |

| `HybridSignature` | struct | HybridSignature { pub es256_der: Vec<u8>, pub mldsa65: Vec<u8> } |  |  |  |  |

| `MlDsa65KeyPair` | struct | MlDsa65KeyPair { pub seed: [u8; ML_DSA_65_SEED_LEN], pub public: Vec<u8> } |  |  |  |  |

| `SlhDsa128sKeyPair` | struct | SlhDsa128sKeyPair { pub seeds: [u8; SLH_DSA_128S_SEED_LEN], pub public: Vec<u8> } |  |  |  |  |


### src/crypto_trust_provider.rs

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `ProviderRefusal` | enum | ProviderRefusal { KeyMismatch { envelope: String, provider: String, }, AlgorithmMismatch { envelope: &'static str, provider: &'static str, }, MalformedEnvelope(String), EmptySignature, Provider(String) } |  |  |  |  |

| `from_seed` | function | from_seed(id: KeyId, seed: &[u8; 32]) -> Result<Self, ProviderRefusal> |  |  |  |  |

| `generate` | function | generate(id: KeyId) -> Result<Self, ProviderRefusal> |  |  |  |  |

| `key_record` | function | key_record(&self, custodian: CustodianIdentity, created_epoch: u64) -> KeyRecord |  |  |  |  |

| `sign_with_provider` | function | sign_with_provider( provider: &P, envelope: SignatureEnvelope, ) -> Result<DetachedSignature, ProviderRefusal> |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `DetachedSignature` | struct | DetachedSignature { pub envelope: SignatureEnvelope, pub signature: Vec<u8> } |  |  |  |  |

| `SoftwareEs256Provider` | struct | SoftwareEs256Provider { id: KeyId, key: Es256SigningKey } |  |  |  |  |

| `SigningProvider` | trait |  |  |  |  |  |


### src/crypto_trust_quorum.rs

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `QUORUM_DEFAULT_K` | const | QUORUM_DEFAULT_K: usize |  |  |  |  |

| `QUORUM_MAX_N` | const | QUORUM_MAX_N: usize |  |  |  |  |

| `QuorumError` | enum | QuorumError { KExceedsN { k: usize, n: usize }, InsufficientShares { got: usize, needed: usize }, DuplicateSigner(String), InvalidShare(String), AlgorithmRefused(crate::crypto_trust_keys::AlgorithmId), Registry(String) } |  |  |  |  |

| `allowed_algorithms` | function | allowed_algorithms(&self) -> &BTreeSet<AlgorithmId> |  |  |  |  |

| `new` | function | new(registry: &'a R) -> QuorumEngine<'a, R> |  |  |  |  |

| `verify_quorum` | function | verify_quorum( &self, signing_input: &[u8], shares: &[SignatureShare], k: usize, ) -> Result<QuorumVerdict, QuorumError> |  |  |  |  |

| `with_allowed_algorithms` | function | with_allowed_algorithms( mut self, algs: impl IntoIterator<Item = AlgorithmId>, ) -> QuorumEngine<'a, R> |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `QuorumEngine` | struct | QuorumEngine { registry: &'a R, allowed_algorithms: BTreeSet<AlgorithmId> } |  |  |  |  |

| `QuorumVerdict` | struct | QuorumVerdict { pub satisfied: bool, pub valid_shares: usize, pub distinct_signers: usize } |  |  |  |  |

| `SignatureShare` | struct | SignatureShare { pub key_id: crate::crypto_trust_keys::KeyId, pub algorithm: crate::crypto_trust_keys::AlgorithmId, pub signature: Vec<u8> } |  |  |  |  |


### src/crypto_trust_revocation.rs

| `CRL_FORMAT` | const | CRL_FORMAT: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `THIS_EPOCH` | const | THIS_EPOCH: u64 |  |  |  |  |

| `RevocationPubError` | enum | RevocationPubError { BadFormat(String), BadSignature, "stale publication: epoch {published} vs current {current}, max staleness {max_staleness}" )] StaleEpoch { published: u64, current: u64, max_staleness: u64, }, Serialization(String), Lifecycle(String) } |  |  |  |  |

| `apply_to` | function | apply_to( revocations: &mut RevocationList, crl: &SignedRevocationList, issuer_pk_sec1: &[u8], current_epoch: u64, max_staleness: u64, now: u64, ) -> Result<usize, RevocationPubError> |  |  |  |  |

| `publish` | function | publish( revocations: &RevocationList, issuer: &Es256SigningKey, issuer_kid: &KeyId, epoch: u64, at: u64, ) -> Result<SignedRevocationList, RevocationPubError> |  |  |  |  |

| `verify_publication` | function | verify_publication( crl: &SignedRevocationList, issuer_pk_sec1: &[u8], ) -> Result<bool, RevocationPubError> |  |  |  |  |

| `CTP-CRL-v1` | str_key | CRL_FORMAT = "CTP-CRL-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `compromised` | str_key | REASON = "compromised" |  |  |  |  |

| `k-a` | str_key | KID_A = "k-a" |  |  |  |  |

| `k-b` | str_key | KID_B = "k-b" |  |  |  |  |

| `RevocationRecordMirror` | struct | RevocationRecordMirror { pub kid: String, pub revoked_at: u64, pub reason: String } |  |  |  |  |

| `SignedRevocationList` | struct | SignedRevocationList { pub format: String, pub issuer_kid: String, pub epoch: u64, pub published_at: u64, pub revoked: Vec<RevocationRecordMirror>, pub signature: Vec<u8> } |  |  |  |  |


### src/crypto_trust_rotation.rs

| `MIGRATIONS` | const | MIGRATIONS: &[(&str, &str, &str)] |  |  |  |  |

| `ROTATION_DIGEST_LABEL` | const | ROTATION_DIGEST_LABEL: &[u8] |  |  |  |  |

| `ROTATION_DOMAIN_TAG` | const | ROTATION_DOMAIN_TAG: &str |  |  |  |  |

| `RotationError` | enum | RotationError { MigrationRefused(CryptoProfile, CryptoProfile), SuccessorSignatureInvalid, RecordTampered, Provider(String), Serialization(String) } |  |  |  |  |

| `admission_allowed` | function | admission_allowed(from: CryptoProfile, to: CryptoProfile) -> bool |  |  |  |  |

| `assert_not_downgrade` | function | assert_not_downgrade(from: CryptoProfile, to: CryptoProfile) -> Result<(), RotationError> |  |  |  |  |

| `assert_not_downgrade_under` | function | assert_not_downgrade_under( from: CryptoProfile, to: CryptoProfile, policy: &MigrationPolicy, ) -> Result<(), RotationError> |  |  |  |  |

| `rotate_es256_to_hybrid` | function | rotate_es256_to_hybrid( old: &Es256SigningKey, hybrid_secret: &HybridSecret, at: u64, ) -> Result<RotationRecord, RotationError> |  |  |  |  |

| `verify_rotation` | function | verify_rotation( record: &RotationRecord, hybrid_pk_es256: &[u8], hybrid_pk_mldsa65: &[u8], ) -> Result<bool, RotationError> |  |  |  |  |

| `CLASSICAL` | str_key | MIGRATIONS = "CLASSICAL" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | ROTATION_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `ctp.rotation.record.v1` | str_key | ROTATION_DIGEST_LABEL = "ctp.rotation.record.v1" |  |  |  |  |

| `MigrationPolicy` | struct | MigrationPolicy { pub strict_upgrades_only: bool } |  |  |  |  |

| `RotationCeremony` | struct |  |  |  |  |  |

| `RotationRecord` | struct | RotationRecord { pub old_key_id: String, pub new_key_id: String, pub from_profile: String, pub to_profile: String, pub rotated_at: u64, pub successor_signature: Vec<u8> } |  |  |  |  |


### src/crypto_trust_rotation_store.rs

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ROTATION_STORE_FILE` | const | ROTATION_STORE_FILE: &str |  |  |  |  |

| `STORE_FORMAT` | const | STORE_FORMAT: &str |  |  |  |  |

| `RotationStoreError` | enum | RotationStoreError { Io(String), Serialization { line: u64, message: String }, WrongFormat { line: u64, found: String }, Tampered { line: u64, kid: String }, Verification { line: u64, message: String } } |  |  |  |  |

| `append` | function | append( &self, record: &RotationRecord, successor_public_es256: &[u8], successor_public_mldsa65: &[u8], ) -> Result<u64, RotationStoreError> |  |  |  |  |

| `latest_for_predecessor` | function | latest_for_predecessor( &self, kid: &str, ) -> Result<Option<VerifiedRotation>, RotationStoreError> |  |  |  |  |

| `latest_for_successor` | function | latest_for_successor( &self, kid: &str, ) -> Result<Option<VerifiedRotation>, RotationStoreError> |  |  |  |  |

| `load` | function | load(&self) -> Result<Vec<VerifiedRotation>, RotationStoreError> |  |  |  |  |

| `new` | function | new(path: impl Into<PathBuf>) -> RotationStore |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `.affi/rotations.jsonl` | str_key | ROTATION_STORE_FILE = ".affi/rotations.jsonl" |  |  |  |  |

| `CTP-ROTSTORE-v1` | str_key | STORE_FORMAT = "CTP-ROTSTORE-v1" |  |  |  |  |

| `CTP_ROTSTORE_XPROC_STORE` | str_key | XPROC_ENV = "CTP_ROTSTORE_XPROC_STORE" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `RotationStore` | struct | RotationStore { path: PathBuf } |  |  |  |  |

| `RotationStoreEnvelope` | struct | RotationStoreEnvelope { pub format: String, pub record: RotationRecord, pub successor_public_es256: Vec<u8>, pub successor_public_mldsa65: Vec<u8> } |  |  |  |  |

| `VerifiedRotation` | struct | VerifiedRotation { pub line: u64, pub record: RotationRecord, pub successor_public_es256: Vec<u8>, pub successor_public_mldsa65: Vec<u8> } |  |  |  |  |


### src/crypto_trust_sa2a.rs

| `ENVELOPE_FIELD_NAMES` | const | ENVELOPE_FIELD_NAMES: [&str; 12] |  |  |  |  |

| `SA2A_APPROVAL_TAG` | const | SA2A_APPROVAL_TAG: &str |  |  |  |  |

| `Sa2aWireError` | enum | Sa2aWireError { WrongVersion(String), UnknownAlgorithm(String), MalformedHex(String), Malformed(String) } |  |  |  |  |

| `approval_to_envelope` | function | approval_to_envelope(a: &Sa2aApproval) -> Result<SignatureEnvelope, Sa2aWireError> |  |  |  |  |

| `envelope_sa2a_signing_input` | function | envelope_sa2a_signing_input( env: &SignatureEnvelope, principal: &str, ) -> Result<Vec<u8>, Sa2aWireError> |  |  |  |  |

| `envelope_to_approval` | function | envelope_to_approval( env: &SignatureEnvelope, principal: &str, ) -> Result<Sa2aApproval, Sa2aWireError> |  |  |  |  |

| `sa2a_signing_input` | function | sa2a_signing_input(a: &Sa2aApproval) -> Vec<u8> |  |  |  |  |

| `sa2a_signing_input_checked` | function | sa2a_signing_input_checked(a: &Sa2aApproval) -> Result<Vec<u8>, Sa2aWireError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `SA2A-C2-APPROVAL-v1` | str_key | SA2A_APPROVAL_TAG = "SA2A-C2-APPROVAL-v1" |  |  |  |  |

| `subject-a` | str_key | PRINCIPAL = "subject-a" |  |  |  |  |

| `version` | str_key | ENVELOPE_FIELD_NAMES = "version" |  |  |  |  |

| `Sa2aApproval` | struct | Sa2aApproval { pub v: String, pub alg: String, pub kid: String, pub effect_digest: String, pub principal: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: String, pub not_before: u64, pub expires: u64, pub audience: String } |  |  |  |  |


### src/crypto_trust_seal.rs

| `DIGEST_ALGORITHM` | const | DIGEST_ALGORITHM: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `SEALED_RECEIPT_FORMAT` | const | SEALED_RECEIPT_FORMAT: &str |  |  |  |  |

| `SealError` | enum | SealError { SubjectMismatch, Chain(String), Verification(String), Serialization(String) } |  |  |  |  |

| `seal_receipt` | function | seal_receipt( receipt: &Receipt, envelope: SignatureEnvelope, signature: Vec<u8>, ) -> Result<SealedReceipt, SealError> |  |  |  |  |

| `subject_digest_of` | function | subject_digest_of(receipt: &Receipt) -> Result<[u8; 32], SealError> |  |  |  |  |

| `verify_sealed` | function | verify_sealed( sealed: &SealedReceipt, engine: &VerificationEngine, ) -> Result<CryptographicVerdict, SealError> |  |  |  |  |

| `BLAKE3` | str_key | DIGEST_ALGORITHM = "BLAKE3" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `PQ-SEAL-v1` | str_key | SEALED_RECEIPT_FORMAT = "PQ-SEAL-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `SealedReceipt` | struct | SealedReceipt { pub base: Receipt, pub envelope: SignatureEnvelope, pub signature: Vec<u8> } |  |  |  |  |


### src/crypto_trust_store.rs

| `DIGEST_ALGORITHM` | const | DIGEST_ALGORITHM: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `STORE_FILE` | const | STORE_FILE: &str |  |  |  |  |

| `STORE_FORMAT` | const | STORE_FORMAT: &str |  |  |  |  |

| `StoreError` | enum | StoreError { ChecksumMismatch { claimed: String, recomputed: String, }, WrongFormat(String), Io(String), Canonical(String), Registry( Serialization( } |  |  |  |  |

| `checksum_for` | function | checksum_for(records: &[KeyRecord]) -> Result<String, StoreError> |  |  |  |  |

| `load` | function | load(&self) -> Result<KeyStoreFile, StoreError> |  |  |  |  |

| `open` | function | open(path: P) -> Result<FileKeyStore, StoreError> |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `records` | function | records(&self) -> Result<Vec<KeyRecord>, StoreError> |  |  |  |  |

| `register_checked` | function | register_checked(&mut self, record: KeyRecord) -> Result<(), StoreError> |  |  |  |  |

| `.affi/keys.json` | str_key | STORE_FILE = ".affi/keys.json" |  |  |  |  |

| `BLAKE3` | str_key | DIGEST_ALGORITHM = "BLAKE3" |  |  |  |  |

| `CTP-STORE-v1` | str_key | STORE_FORMAT = "CTP-STORE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `FileKeyStore` | struct | FileKeyStore { path: PathBuf, records: BTreeMap<KeyId, KeyRecord> } |  |  |  |  |

| `KeyStoreFile` | struct | KeyStoreFile { pub format: String, pub records: Vec<KeyRecord>, pub checksum: String } |  |  |  |  |


### src/crypto_trust_transparency.rs

| `CONSISTENCY_DOMAIN` | const | CONSISTENCY_DOMAIN: &str |  |  |  |  |

| `INCLUSION_DOMAIN` | const | INCLUSION_DOMAIN: &str |  |  |  |  |

| `LOG_FORMAT` | const | LOG_FORMAT: &str |  |  |  |  |

| `MERKLE_DOMAIN` | const | MERKLE_DOMAIN: &str |  |  |  |  |

| `TransparencyError` | enum | TransparencyError { LeafOutOfRange(u64), SizeOutOfRange(u64, u64) } |  |  |  |  |

| `append` | function | append(&mut self, commitment: [u8; 32]) -> u64 |  |  |  |  |

| `consistency_proof` | function | consistency_proof(&self, first: u64) -> Result<ConsistencyProof, TransparencyError> |  |  |  |  |

| `head` | function | head(&self) -> [u8; 32] |  |  |  |  |

| `inclusion_proof` | function | inclusion_proof(&self, idx: u64) -> Result<InclusionProof, TransparencyError> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `leaf` | function | leaf(&self, index: u64) -> Option<&[u8; 32]> |  |  |  |  |

| `len` | function | len(&self) -> u64 |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `verify_consistency` | function | verify_consistency( proof: &ConsistencyProof, first_head: &[u8; 32], second_head: &[u8; 32], ) -> bool |  |  |  |  |

| `verify_inclusion` | function | verify_inclusion(leaf: &[u8; 32], proof: &InclusionProof, head: &[u8; 32]) -> bool |  |  |  |  |

| `CTP-TRANS-v1` | str_key | LOG_FORMAT = "CTP-TRANS-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1/consistency` | str_key | CONSISTENCY_DOMAIN = "affidavit.crypto-trust-plane.v1/consistency" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1/inclusion` | str_key | INCLUSION_DOMAIN = "affidavit.crypto-trust-plane.v1/inclusion" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1/merkle` | str_key | MERKLE_DOMAIN = "affidavit.crypto-trust-plane.v1/merkle" |  |  |  |  |

| `ConsistencyProof` | struct | ConsistencyProof { pub first: u64, pub second: u64, pub path: Vec<[u8; 32]> } |  |  |  |  |

| `InclusionProof` | struct | InclusionProof { pub leaf_index: u64, pub tree_size: u64, pub path: Vec<[u8; 32]> } |  |  |  |  |

| `TransparencyLog` | struct | TransparencyLog { leaves: Vec<[u8; 32]> } |  |  |  |  |


### src/crypto_trust_verify.rs

| `CRYPTO_STANDING_PROFILE` | const | CRYPTO_STANDING_PROFILE: &str |  |  |  |  |

| `DEFAULT_PROFILE` | const | DEFAULT_PROFILE: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `MAX_REVOCATION_STALENESS_SECONDS` | const | MAX_REVOCATION_STALENESS_SECONDS: u64 |  |  |  |  |

| `MIN_PROFILE` | const | MIN_PROFILE: &str |  |  |  |  |

| `CryptographicStanding` | enum | CryptographicStanding { Valid, Invalid, Expired, Revoked, ReplayRejected, UnknownKey, ProfileRefused, Malformed } |  |  |  |  |

| `StandingReceiptError` | enum | StandingReceiptError { ReceiptHashMismatch { claimed: String, recomputed: String }, BadSignature { signer_kid: String }, Serialization(String) } |  |  |  |  |

| `VerifyRefusal` | enum | VerifyRefusal { MalformedEnvelope(String), UnknownKey(String), KeyRevoked(String), StaleRevocationEpoch(String), Expired(u64), NotYetValid(u64), ReplayRejected(String), ProfileRefused(crate::crypto_trust_keys::AlgorithmId), AudienceRefused(String), ProfileFloor(CryptoProfile), Provider(String), InvalidSignature(String), SubjectMismatch(String) } |  |  |  |  |

| `all` | function | all() -> &'static [CryptographicStanding] |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `certify_signed` | function | certify_signed( &self, env: &SignatureEnvelope, signature: &[u8], subject: &str, signing: &Es256SigningKey, ) -> Result<CryptoStandingReceipt, VerifyRefusal> |  |  |  |  |

| `from_graph_defaults` | function | from_graph_defaults() -> Self |  |  |  |  |

| `new` | function | new( registry: InMemoryKeyRegistry, revocations: RevocationList, nonces: NonceJournal, policy: TrustPolicy, ) -> Self |  |  |  |  |

| `nonce_seen_at` | function | nonce_seen_at(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> |  |  |  |  |

| `policy` | function | policy(&self) -> &TrustPolicy |  |  |  |  |

| `prune_nonces` | function | prune_nonces(&mut self) -> usize |  |  |  |  |

| `register_key` | function | register_key(&mut self, record: KeyRecord) -> Result<(), RegistryError> |  |  |  |  |

| `registry` | function | registry(&self) -> &InMemoryKeyRegistry |  |  |  |  |

| `revocations` | function | revocations(&self) -> &RevocationList |  |  |  |  |

| `revoke_key` | function | revoke_key(&mut self, kid: &str, at: u64, reason: String) |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), StandingReceiptError> |  |  |  |  |

| `verify_envelope` | function | verify_envelope( &self, env: &SignatureEnvelope, signature: &[u8], ) -> Result<CryptographicVerdict, VerifyRefusal> |  |  |  |  |

| `with_now` | function | with_now(mut self, now: u64) -> Self |  |  |  |  |

| `CLASSICAL` | str_key | MIN_PROFILE = "CLASSICAL" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `HYBRID` | str_key | DEFAULT_PROFILE = "HYBRID" |  |  |  |  |

| `VALID` | str_key | EXPECTED = "VALID" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `affidavit/crypto-standing/v1` | str_key | CRYPTO_STANDING_PROFILE = "affidavit/crypto-standing/v1" |  |  |  |  |

| `CryptoStandingReceipt` | struct | CryptoStandingReceipt { pub profile: String, pub subject: String, pub envelope_commitment: String, pub key_id: String, pub algorithm: String, pub standing: CryptographicStanding, pub observed_at: u64, pub receipt_hash: String, pub signer_kid: String, pub signer_public_key: Vec<u8>, pub signature: Vec<u8>, _seal: () } |  |  |  |  |

| `CryptographicVerdict` | struct | CryptographicVerdict { pub standing: CryptographicStanding, pub key_id: Option<KeyId>, pub subject_digest: [u8; 32] } |  |  |  |  |

| `TrustPolicy` | struct | TrustPolicy { pub allowed_audiences: std::collections::BTreeSet<String>, pub min_profile: CryptoProfile, pub allowed: std::collections::BTreeSet<AlgorithmId>, pub max_revocation_staleness_seconds: u64, pub now: u64 } |  |  |  |  |

| `VerificationEngine` | struct | VerificationEngine { registry: InMemoryKeyRegistry, revocations: RevocationList, nonces: RefCell<NonceJournal>, policy: TrustPolicy } |  |  |  |  |


### src/crypto_trust_witness.rs

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `MIN_WITNESSES` | const | MIN_WITNESSES: usize |  |  |  |  |

| `WitnessError` | enum | WitnessError { InsufficientWitnesses { got: usize, needed: usize }, DuplicateWitness(String), UnknownWitness(String), MalformedCosignature(String), Serialization(String) } |  |  |  |  |

| `collect` | function | collect( &self, witnesses: &[(&str, &Es256SigningKey)], ) -> Result<Vec<WitnessSignature>, WitnessError> |  |  |  |  |

| `head` | function | head(&self) -> &SignedTreeHead |  |  |  |  |

| `new` | function | new(head: SignedTreeHead) -> Self |  |  |  |  |

| `preimage` | function | preimage(&self) -> Result<[u8; 32], WitnessError> |  |  |  |  |

| `verify_cosigned` | function | verify_cosigned( head: &SignedTreeHead, cosigs: &[WitnessSignature], min: usize, pks: &[(&str, &[u8])], ) -> Result<CosignVerdict, WitnessError> |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `afk1_transparency-log-ops` | str_key | SIGNER_KID = "afk1_transparency-log-ops" |  |  |  |  |

| `CosignVerdict` | struct | CosignVerdict { pub cosigned: bool, pub valid_witnesses: usize, pub distinct_witnesses: usize } |  |  |  |  |

| `WitnessCosign` | struct | WitnessCosign { head: SignedTreeHead } |  |  |  |  |

| `WitnessSignature` | struct | WitnessSignature { pub kid: String, pub signature: Vec<u8> } |  |  |  |  |


### src/dfcm.rs

| `DFCM_AUTHORITY_CEILING` | const | DFCM_AUTHORITY_CEILING: &str |  |  |  |  |

| `DFCM_CLAIM_CEILING` | const | DFCM_CLAIM_CEILING: &str |  |  |  |  |

| `DFCM_PROFILE` | const | DFCM_PROFILE: &str |  |  |  |  |

| `ClosureGap` | enum | ClosureGap { MissingSubject { repository: String, expected_candidate: String, }, ExactSubjectMoved { repository: String, expected_candidate: String, observed_candidates: Vec<String>, }, MissingEvidence { subject: ExactSubject, evidence: EvidenceKey, }, StandingMismatch { subject: ExactSubject, required: String, observed: String, }, MissingStandingReceipt { subject: ExactSubject, } } |  |  |  |  |

| `DfcmRefusal` | enum | DfcmRefusal { EmptyField(&'static str), NoObligations, ZeroFrontierBudget, DuplicateObligation(String), NoProofPaths(String), DuplicatePath(String, String), EmptyPath(String, String), DuplicateSubjectRequirement(ExactSubject), DuplicateEvidenceRequirement(ExactSubject, EvidenceKey), DuplicateObservation(ExactSubject), DuplicateEvidenceWitness(ExactSubject, EvidenceKey), MalformedBlake3(&'static str), StandingReceiptInvalid(ExactSubject, String), StandingReceiptSubjectMismatch(ExactSubject, ExactSubject), StandingReceiptStandingMismatch(ExactSubject, String, String), FrontierBudgetExceeded { budget: usize, required: usize }, NonCanonicalOrder(&'static str), WrongProfile, WrongClaimCeiling, WrongAuthorityCeiling, ObligationEvaluationMismatch, FrontierMismatch, ClosureMismatch, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `EvidenceKind` | enum | EvidenceKind { ImplementedSource, ObservedExecution, ExactHeadCourt, HostedCi, RuntimeStanding, Merge, Publication } |  |  |  |  |

| `certify_dfcm` | function | certify_dfcm( mut release_profile: DfcmProfile, mut observations: Vec<SubjectObservation>, ) -> Result<DfcmReceipt, DfcmRefusal> |  |  |  |  |

| `v26_9_18_profile` | function | v26_9_18_profile(s: V26_9_18Subjects) -> DfcmProfile |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), DfcmRefusal> |  |  |  |  |

| `EVIDENCE_ONLY_NO_SELECT_CONSTRUCT_OR_DO_AUTHORITY` | str_key | DFCM_AUTHORITY_CEILING = "EVIDENCE_ONLY_NO_SELECT_CONSTRUCT_OR_DO_AUTHORITY" |  |  |  |  |

| `EXACT_SUBJECT_RELEASE_CLOSURE_ONLY_NO_RUNTIME_MERGE_PUBLICATION_OR_DEPLOYMENT_INFERENCE` | str_key | DFCM_CLAIM_CEILING = "EXACT_SUBJECT_RELEASE_CLOSURE_ONLY_NO_RUNTIME_MERGE_PUBLICATION_OR_DEPLOYMENT_INFERENCE" |  |  |  |  |

| `affidavit/dfcm/v1` | str_key | DFCM_PROFILE = "affidavit/dfcm/v1" |  |  |  |  |

| `DfcmProfile` | struct | DfcmProfile { pub release: String, pub certifier: ExactSubject, pub frontier_budget: usize, pub obligations: Vec<Obligation> } |  |  |  |  |

| `DfcmReceipt` | struct | DfcmReceipt { pub profile: String, pub claim_ceiling: String, pub authority_ceiling: String, pub release_profile: DfcmProfile, pub observations: Vec<SubjectObservation>, pub obligations: Vec<ObligationEvaluation>, pub minimal_frontiers: Vec<MinimalFrontier>, pub closed: bool, pub receipt_hash: Blake3Hash } |  |  |  |  |

| `EvidenceKey` | struct | EvidenceKey { pub kind: EvidenceKind, pub court: String } |  |  |  |  |

| `EvidenceWitness` | struct | EvidenceWitness { pub key: EvidenceKey, pub commitment: Blake3Hash } |  |  |  |  |

| `ExactSubject` | struct | ExactSubject { pub repository: String, pub candidate: String } |  |  |  |  |

| `MinimalFrontier` | struct | MinimalFrontier { pub gaps: Vec<ClosureGap> } |  |  |  |  |

| `Obligation` | struct | Obligation { pub id: String, pub description: String, pub paths: Vec<ProofPath> } |  |  |  |  |

| `ObligationEvaluation` | struct | ObligationEvaluation { pub obligation: String, pub satisfied: bool, pub paths: Vec<PathEvaluation> } |  |  |  |  |

| `PathEvaluation` | struct | PathEvaluation { pub name: String, pub satisfied: bool, pub gaps: Vec<ClosureGap> } |  |  |  |  |

| `ProofPath` | struct | ProofPath { pub name: String, pub subjects: Vec<SubjectRequirement> } |  |  |  |  |

| `SubjectObservation` | struct | SubjectObservation { pub subject: ExactSubject, pub standing: Standing, pub evidence: Vec<EvidenceWitness>, pub standing_receipt: Option<StandingReceipt> } |  |  |  |  |

| `SubjectRequirement` | struct | SubjectRequirement { pub subject: ExactSubject, pub required_evidence: Vec<EvidenceKey>, pub required_standing: Option<Standing>, pub require_standing_receipt: bool } |  |  |  |  |

| `V26_9_18Subjects` | struct | V26_9_18Subjects { pub affidavit: ExactSubject, pub ggen: ExactSubject, pub bcinr: ExactSubject, pub ash_r2rml: ExactSubject, pub ggen_igniter: ExactSubject, pub wasm4pm: ExactSubject, pub unrdf: ExactSubject } |  |  |  |  |


### src/diag.rs

| `INTERNAL` | const | INTERNAL: i32 |  |  |  |  |

| `IO_ERROR` | const | IO_ERROR: i32 |  |  |  |  |

| `OK` | const | OK: i32 |  |  |  |  |

| `REJECT` | const | REJECT: i32 |  |  |  |  |

| `SLA_BREACH` | const | SLA_BREACH: i32 |  |  |  |  |

| `USAGE_ERROR` | const | USAGE_ERROR: i32 |  |  |  |  |

| `ErrorCode` | enum | ErrorCode { ChainHashMismatch = 1001, GenesisHashMismatch = 1002, SeqGap = 1003, DuplicateEventId = 1004, InvalidCommitment = 1005, TamperedReceipt = 1006, UnknownFormatVersion = 1100, MalformedReceipt = 1101, MissingEventType = 1102, UnknownProfile = 1200, ProfileViolation = 1201, ReceiptNotFound = 1300, ReceiptUnreadable = 1301, WorkingDirMissing = 1302, InvalidObjectId = 1400, InvalidEventType = 1401 } |  |  |  |  |

| `code` | function | code(self) -> u16 |  |  |  |  |

| `exit_code` | function | exit_code(self) -> i32 |  |  |  |  |

| `from_error` | function | from_error(code: ErrorCode, err: &dyn std::error::Error) -> Self |  |  |  |  |

| `hint` | function | hint(self) -> Option<&'static str> |  |  |  |  |

| `message` | function | message(self) -> &'static str |  |  |  |  |

| `new` | function | new(code: ErrorCode, message: impl Into<String>) -> Self |  |  |  |  |

| `with_hint` | function | with_hint(mut self, hint: impl Into<String>) -> Self |  |  |  |  |

| `with_span` | function | with_span(mut self, file: impl Into<String>, line: Option<u32>) -> Self |  |  |  |  |

| `Diag` | struct | Diag { pub code: u16, pub message: String, pub hint: Option<String>, pub span: Option<Span> } |  |  |  |  |

| `Span` | struct | Span { pub file: String, pub line: Option<u32> } |  |  |  |  |


### src/diff.rs

| `diff_json_receipts` | function | diff_json_receipts(old_json: &str, new_json: &str) -> anyhow::Result<DiffResult> |  |  |  |  |

| `diff_receipts` | function | diff_receipts(old: &Receipt, new: &Receipt) -> DiffResult |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `DiffEntry` | struct | DiffEntry { pub seq: u64, pub event_type: String, pub commitment_prefix: String } |  |  |  |  |

| `DiffResult` | struct | DiffResult { pub added: Vec<DiffEntry>, pub removed: Vec<DiffEntry>, pub modified: Vec<ModifiedEntry> } |  |  |  |  |

| `ModifiedEntry` | struct | ModifiedEntry { pub seq: u64, pub old: DiffEntry, pub new: DiffEntry } |  |  |  |  |


### src/discovery.rs

| `ACTIVITY_KEY` | const | ACTIVITY_KEY: &str |  |  |  |  |

| `conformance_metrics` | function | conformance_metrics(receipt: &Receipt) -> (f64, f64) |  |  |  |  |

| `discover_dfg_summary` | function | discover_dfg_summary(receipt: &Receipt) -> (usize, usize, usize, usize) |  |  |  |  |

| `discover_from_admitted` | function | discover_from_admitted(admitted: &crate::types::AdmittedReceipt) -> String |  |  |  |  |

| `discover_process_tree` | function | discover_process_tree(receipt: &Receipt) -> String |  |  |  |  |

| `project_to_event_log` | function | project_to_event_log(receipt: &Receipt) -> EventLog |  |  |  |  |

| `quality_metrics` | function | quality_metrics(receipt: &Receipt) -> (f64, f64, f64) |  |  |  |  |

| `quality_metrics_from_admitted` | function | quality_metrics_from_admitted(admitted: &crate::types::AdmittedReceipt) -> (f64, f64, f64) |  |  |  |  |

| `concept:name` | str_key | ACTIVITY_KEY = "concept:name" |  |  |  |  |


### src/doctor_check.rs

| `DOCTOR_CHECKS` | const | DOCTOR_CHECKS: [&'static dyn DoctorCheck] |  |  |  |  |

| `FindingStatus` | enum | FindingStatus { Ok, Warn, Fail } |  |  |  |  |

| `auto_fixable` | function | auto_fixable(mut self) -> Self |  |  |  |  |

| `fail` | function | fail( id: &'static str, message: impl Into<String>, remediation: impl Into<String>, ) -> Self |  |  |  |  |

| `label` | function | label(&self) -> &'static str |  |  |  |  |

| `ok` | function | ok(id: &'static str, message: impl Into<String>) -> Self |  |  |  |  |

| `run_all` | function | run_all() -> Vec<Finding> |  |  |  |  |

| `warn` | function | warn( id: &'static str, message: impl Into<String>, remediation: impl Into<String>, ) -> Self |  |  |  |  |

| `Finding` | struct | Finding { pub id: &'static str, pub status: FindingStatus, pub message: String, pub remediation: Option<String>, pub auto_fixable: bool } |  |  |  |  |

| `DoctorCheck` | trait |  |  |  |  |  |


### src/ecosystem.rs

| `ECOSYSTEM_AUTHORITY_CEILING` | const | ECOSYSTEM_AUTHORITY_CEILING: &str |  |  |  |  |

| `ECOSYSTEM_CLAIM_CEILING` | const | ECOSYSTEM_CLAIM_CEILING: &str |  |  |  |  |

| `ECOSYSTEM_PROFILE` | const | ECOSYSTEM_PROFILE: &str |  |  |  |  |

| `EcosystemRefusal` | enum | EcosystemRefusal { NoRequirements, ZeroAliveRequirement(EcosystemRole), DuplicateRequirement(EcosystemRole), DuplicateMember { role: EcosystemRole, subject: String, candidate: String, }, MemberInvalid { subject: String, reason: String, }, NonCanonicalOrder(&'static str), EmptyField(&'static str), MalformedBlake3(&'static str), WrongProfile, WrongClaimCeiling, WrongAuthorityCeiling, CoverageMismatch, StandingMismatch, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `EcosystemRole` | enum | EcosystemRole { SemanticAuthority, Manufacture, PlanningControl, WorldExecution, FormalProof, RuntimeExecution, ProcessEvidence, Configuration, SupplyChain, Verification, Replay } |  |  |  |  |

| `certify_ecosystem` | function | certify_ecosystem( admitted: &AdmittedReceipt, mut observation: EcosystemObservation, ) -> Result<EcosystemReceipt, EcosystemRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), EcosystemRefusal> |  |  |  |  |

| `EVIDENCE_ONLY_NO_AMBIENT_ACTUATION_AUTHORITY` | str_key | ECOSYSTEM_AUTHORITY_CEILING = "EVIDENCE_ONLY_NO_AMBIENT_ACTUATION_AUTHORITY" |  |  |  |  |

| `VERIFIED_STANDING_COMPOSITION_ONLY_NO_TRANSITIVE_TRUTH_OR_ACTUATION_CLAIM` | str_key | ECOSYSTEM_CLAIM_CEILING = "VERIFIED_STANDING_COMPOSITION_ONLY_NO_TRANSITIVE_TRUTH_OR_ACTUATION_CLAIM" |  |  |  |  |

| `affidavit/ecosystem/v1` | str_key | ECOSYSTEM_PROFILE = "affidavit/ecosystem/v1" |  |  |  |  |

| `EcosystemMember` | struct | EcosystemMember { pub role: EcosystemRole, pub standing_receipt: StandingReceipt } |  |  |  |  |

| `EcosystemObservation` | struct | EcosystemObservation { pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub requirements: Vec<RoleRequirement>, pub members: Vec<EcosystemMember>, pub previous_receipt: Option<Blake3Hash> } |  |  |  |  |

| `EcosystemReceipt` | struct | EcosystemReceipt { pub profile: String, pub claim_ceiling: String, pub authority_ceiling: String, pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub admitted_receipt_hash: Blake3Hash, pub requirements: Vec<RoleRequirement>, pub members: Vec<EcosystemMember>, pub coverage: Vec<RoleCoverage>, pub standing: Standing, pub previous_receipt: Option<Blake3Hash>, pub receipt_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `RoleCoverage` | struct | RoleCoverage { pub role: EcosystemRole, pub required_alive: usize, pub total: usize, pub alive: usize, pub partial_alive: usize, pub unknown: usize, pub blocked: usize, pub build_broken: usize, pub unsupported: usize, pub satisfied: bool } |  |  |  |  |

| `RoleRequirement` | struct | RoleRequirement { pub role: EcosystemRole, pub minimum_alive: usize } |  |  |  |  |


### src/ed25519_witness.rs

| `Ed25519WitnessError` | enum | Ed25519WitnessError { MalformedPublicKey, MalformedSignature, VerificationFailed } |  |  |  |  |

| `from_rng` | function | from_rng(rng: &mut R) -> Self |  |  |  |  |

| `generate` | function | generate() -> Self |  |  |  |  |

| `public` | function | public(&self) -> [u8; 32] |  |  |  |  |

| `sign` | function | sign(&self, message: &[u8]) -> [u8; 64] |  |  |  |  |

| `verify_witness` | function | verify_witness( public_key: &[u8; 32], message: &[u8], signature: &[u8; 64], ) -> Result<(), Ed25519WitnessError> |  |  |  |  |

| `WitnessKeyPair` | struct | WitnessKeyPair { signing: SigningKey } |  |  |  |  |


### src/errc.rs

| `ERRC_CLAIM_CEILING` | const | ERRC_CLAIM_CEILING: &str |  |  |  |  |

| `ERRC_PROFILE` | const | ERRC_PROFILE: &str |  |  |  |  |

| `ERRC_SOURCE_ARTIFACT` | const | ERRC_SOURCE_ARTIFACT: &str |  |  |  |  |

| `ERRC_SOURCE_COMMIT` | const | ERRC_SOURCE_COMMIT: &str |  |  |  |  |

| `ERRC_SOURCE_REPOSITORY` | const | ERRC_SOURCE_REPOSITORY: &str |  |  |  |  |

| `ErrcQuadrant` | enum | ErrcQuadrant { Eliminate, Reduce, Raise, Create } |  |  |  |  |

| `ErrcRefusal` | enum | ErrcRefusal { EmptyField(&'static str), MalformedBlake3(&'static str), NoClaims, NoPreservationFence, DuplicateClaimId(String), DuplicateInvariantId(String), DuplicateFactorCoordinate { target: String, metric: String, unit: String, }, DirectionViolation { claim_id: String, quadrant: ErrcQuadrant, baseline: u64, candidate: u64, }, NonCanonicalOrder(&'static str), WrongProfile, WrongSource, WrongClaimCeiling, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `certify_errc` | function | certify_errc( admitted: &AdmittedReceipt, mut observation: ErrcObservation, ) -> Result<ErrcReceipt, ErrcRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), ErrcRefusal> |  |  |  |  |

| `60d38265b8d1d94c43f04ca6bdb8537184e510a8` | str_key | ERRC_SOURCE_COMMIT = "60d38265b8d1d94c43f04ca6bdb8537184e510a8" |  |  |  |  |

| `DECLARED_DIRECTIONAL_TRANSFORMATION_ONLY_NO_CAUSAL_OR_OPTIMALITY_CLAIM` | str_key | ERRC_CLAIM_CEILING = "DECLARED_DIRECTIONAL_TRANSFORMATION_ONLY_NO_CAUSAL_OR_OPTIMALITY_CLAIM" |  |  |  |  |

| `affidavit/errc/v1` | str_key | ERRC_PROFILE = "affidavit/errc/v1" |  |  |  |  |

| `scripts/ci_errc.py` | str_key | ERRC_SOURCE_ARTIFACT = "scripts/ci_errc.py" |  |  |  |  |

| `seanchatmangpt/ggen-legacy` | str_key | ERRC_SOURCE_REPOSITORY = "seanchatmangpt/ggen-legacy" |  |  |  |  |

| `ErrcClaim` | struct | ErrcClaim { pub id: String, pub target: String, pub quadrant: ErrcQuadrant, pub measure: ErrcMeasure, pub evidence_commitment: Blake3Hash } |  |  |  |  |

| `ErrcMeasure` | struct | ErrcMeasure { pub metric: String, pub unit: String, pub baseline: u64, pub candidate: u64 } |  |  |  |  |

| `ErrcObservation` | struct | ErrcObservation { pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub claims: Vec<ErrcClaim>, pub preserved_invariants: Vec<PreservedInvariant>, pub replay: ReplayEvidence, pub previous_receipt: Option<Blake3Hash> } |  |  |  |  |

| `ErrcReceipt` | struct | ErrcReceipt { pub profile: String, pub source: ErrcSource, pub claim_ceiling: String, pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub admitted_receipt_hash: Blake3Hash, pub claims: Vec<ErrcClaim>, pub preserved_invariants: Vec<PreservedInvariant>, pub quadrant_counts: QuadrantCounts, pub replay: ReplayEvidence, pub previous_receipt: Option<Blake3Hash>, pub receipt_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `ErrcSource` | struct | ErrcSource { pub repository: String, pub commit: String, pub artifact: String } |  |  |  |  |

| `PreservedInvariant` | struct | PreservedInvariant { pub id: String, pub statement: String, pub evidence_commitment: Blake3Hash } |  |  |  |  |

| `QuadrantCounts` | struct | QuadrantCounts { pub eliminate: u64, pub reduce: u64, pub raise: u64, pub create: u64 } |  |  |  |  |


### src/errc_claim_assurance.rs

| `ERRC_CLAIM_ASSURANCE_CEILING` | const | ERRC_CLAIM_ASSURANCE_CEILING: &str |  |  |  |  |

| `ERRC_CLAIM_ASSURANCE_PROFILE` | const | ERRC_CLAIM_ASSURANCE_PROFILE: &str |  |  |  |  |

| `ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT` | const | ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT: &str |  |  |  |  |

| `ErrcClaimAssuranceRefusal` | enum | ErrcClaimAssuranceRefusal { ParentInvalid(ErrcRefusal), EmptyWitnessField { claim_id: String, field: &'static str, }, MalformedEvidenceCommitment(String), NoExclusions(String), EmptyExclusion(String), DuplicateExclusion { claim_id: String, exclusion: String, }, NoClaims, MissingWitness(String), UnexpectedWitness(String), DuplicateWitness(String), NonCanonicalOrder(&'static str), WrongProfile, WrongSource, WrongClaimCeiling, ParentHashMismatch, ClaimSetMismatch, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `certify_errc_claim_assurance` | function | certify_errc_claim_assurance( parent: &ErrcReceipt, mut witnesses: Vec<ErrcClaimWitness>, ) -> Result<ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), ErrcClaimAssuranceRefusal> |  |  |  |  |

| `verify_against` | function | verify_against(&self, parent: &ErrcReceipt) -> Result<(), ErrcClaimAssuranceRefusal> |  |  |  |  |

| `CLAIM_WITNESS_COMPLETENESS_AND_BINDING_ONLY_NO_TRUTH_CAUSALITY_PRODUCTION_COMPLIANCE_OPTIMALITY_OR_ACTUATION_CLAIM` | str_key | ERRC_CLAIM_ASSURANCE_CEILING = "CLAIM_WITNESS_COMPLETENESS_AND_BINDING_ONLY_NO_TRUTH_CAUSALITY_PRODUCTION_COMPLIANCE_OPTIMALITY_OR_ACTUATION_CLAIM" |  |  |  |  |

| `affidavit/errc-claim-assurance/v1` | str_key | ERRC_CLAIM_ASSURANCE_PROFILE = "affidavit/errc-claim-assurance/v1" |  |  |  |  |

| `governance/claims-register.md` | str_key | ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT = "governance/claims-register.md" |  |  |  |  |

| `ErrcClaimAssuranceReceipt` | struct | ErrcClaimAssuranceReceipt { pub profile: String, pub source: ErrcSource, pub claim_ceiling: String, pub errc_receipt_hash: Blake3Hash, pub claim_ids: Vec<String>, pub witnesses: Vec<ErrcClaimWitness>, pub receipt_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `ErrcClaimWitness` | struct | ErrcClaimWitness { pub claim_id: String, pub verifier: String, pub evidence_locator: String, pub observed_result: String, pub evidence_commitment: Blake3Hash, pub exclusions: Vec<String> } |  |  |  |  |


### src/error.rs

| `AffidavitError` | enum | AffidavitError { Io( Json( Parse(String), Validation(String), AdmissionRefused(String), VerificationFailed(String), Execution(String), WorkingReceipt(String), ContentAddressing(String), Discovery(String), Lsp(String), Ocel( Chain( Pqc( Mining( Sharding( Prediction( Slo( } |  |  |  |  |

| `ChainError` | enum | ChainError { Encode( Decode( Io { path: String, source: std::io::Error, }, } |  |  |  |  |

| `MiningError` | enum | MiningError { Him(String), Admission(String), OcelConversion(String) } |  |  |  |  |

| `OcelError` | enum | OcelError { EmptyEventType, EmptyEventId, EmptyObjectId(usize), EmptyObjectType(usize), MalformedObjectRef(String) } |  |  |  |  |

| `PqcError` | enum | PqcError { Signing(String), Verification(String), Encapsulation(String), Chain( } |  |  |  |  |

| `PredictionError` | enum | PredictionError { Wasm4pm(String), Admission(String), InvalidTopK(usize) } |  |  |  |  |

| `ShardingError` | enum | ShardingError { Dht(String), ChainMismatch { index: usize, expected: String, found: String, }, SeqMismatch { index: usize, expected: u64, found: u64, }, BoundaryMismatch { index: usize, next: usize, }, Failure(String) } |  |  |  |  |

| `SloViolation` | enum | SloViolation { LatencyP99 { observed_ms: f64, threshold_ms: f64, }, ErrorRate { observed_pct: f64, threshold_pct: f64, }, "availability SLO breach: observed {observed_pct:.3}% < threshold {threshold_pct:.3}%" )] Availability { observed_pct: f64, threshold_pct: f64, } } |  |  |  |  |


### src/event_builder.rs

| `build` | function | build(self, counter: &mut SeqCounter) -> Result<OperationEvent, crate::ocel::OcelError> |  |  |  |  |

| `new` | function | new(event_type: impl Into<String>) -> Self |  |  |  |  |

| `object` | function | object(mut self, id: impl Into<String>, object_type: impl Into<String>) -> Self |  |  |  |  |

| `payload` | function | payload(mut self, payload: impl Into<Vec<u8>>) -> Self |  |  |  |  |

| `payload_str` | function | payload_str(mut self, payload: impl Into<String>) -> Self |  |  |  |  |

| `qualified_object` | function | qualified_object( mut self, id: impl Into<String>, object_type: impl Into<String>, qualifier: impl Into<String>, ) -> Self |  |  |  |  |

| `EventBuilder` | struct | EventBuilder { event_type: String, objects: Vec<ObjectRef>, payload: Vec<u8> } |  |  |  |  |


### src/execution_manifest.rs

| `ManifestRefusal` | enum | ManifestRefusal { ManifestDigestMismatch, SubjectIdentityChanged, AuthorityBindingChanged, PolicyBindingChanged, OntologyBindingChanged, ToolSurfaceChanged, IntentBindingChanged, InvalidManifest(&'static str) } |  |  |  |  |

| `binding` | function | binding(&self) -> Result<ExecutionBinding, ManifestRefusal> |  |  |  |  |

| `digest` | function | digest(&self) -> Result<String, ManifestRefusal> |  |  |  |  |

| `requalification_reason` | function | requalification_reason( before: &ExecutionManifest, after: &ExecutionManifest, ) -> Result<Option<ManifestRefusal>, ManifestRefusal> |  |  |  |  |

| `validate` | function | validate(&self) -> Result<(), ManifestRefusal> |  |  |  |  |

| `verify_binding` | function | verify_binding( manifest: &ExecutionManifest, binding: &ExecutionBinding, ) -> Result<(), ManifestRefusal> |  |  |  |  |

| `ExecutionBinding` | struct | ExecutionBinding { pub execution_manifest_digest: String, pub exact_subject: String, pub subject_digest: String, pub intent_digest: String, pub authority_grant_digest: Option<String> } |  |  |  |  |

| `ExecutionManifest` | struct | ExecutionManifest { pub exact_subject: String, pub subject_digest: String, pub repository: Option<String>, pub base_sha: Option<String>, pub ontology_digest: Option<String>, pub policy_digest: Option<String>, pub capability_manifest_digest: Option<String>, pub tool_manifest_digest: Option<String>, pub planner_identity: Option<String>, pub planner_version: Option<String>, pub generator_identity: Option<String>, pub generator_version: Option<String>, pub runtime_identity: Option<String>, pub runtime_version: Option<String>, pub dependency_lock_digest: Option<String>, pub authority_grant_digest: Option<String>, pub intent_digest: String, pub environment_constraints: BTreeMap<String, String> } |  |  |  |  |


### src/federation.rs

| `STANDING_CAPABILITY` | const | STANDING_CAPABILITY: &str |  |  |  |  |

| `STANDING_CAPABILITY_DIGEST` | const | STANDING_CAPABILITY_DIGEST: &str |  |  |  |  |

| `accepted` | function | accepted(&self) -> bool |  |  |  |  |

| `cli_standing_authority` | function | cli_standing_authority(scope: &str) -> AuthorityEnvelope<RustTypestateLaw> |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify(receipt: &str, observation: &str) -> CourtOutcome |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(receipt: &str) -> CourtOutcome |  |  |  |  |

| `errc_assure` | function | errc_assure(parent: &str, witnesses: &str) -> CourtOutcome |  |  |  |  |

| `errc_certify` | function | errc_certify(receipt: &str, observation: &str) -> CourtOutcome |  |  |  |  |

| `errc_verify` | function | errc_verify(receipt: &str) -> CourtOutcome |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance(receipt: &str, parent: Option<&str>) -> CourtOutcome |  |  |  |  |

| `reason` | function | reason(&self) -> &str |  |  |  |  |

| `standing_certify` | function | standing_certify(receipt: &str, observation: &str, scope: &str) -> CourtOutcome |  |  |  |  |

| `standing_verify` | function | standing_verify(receipt: &str) -> CourtOutcome |  |  |  |  |

| `affidavit.certify-standing` | str_key | STANDING_CAPABILITY = "affidavit.certify-standing" |  |  |  |  |

| `blake3:affidavit-standing-v2` | str_key | STANDING_CAPABILITY_DIGEST = "blake3:affidavit-standing-v2" |  |  |  |  |

| `CourtOutcome` | struct | CourtOutcome { pub code: i32, pub report: serde_json::Value } |  |  |  |  |


### src/fixture_db.rs

| `all` | function | all(&self) -> Vec<Fixture> |  |  |  |  |

| `delete_by_name` | function | delete_by_name(&mut self, name: &str) -> Result<bool> |  |  |  |  |

| `get_by_chain_hash` | function | get_by_chain_hash(&self, chain_hash: &str) -> Option<Fixture> |  |  |  |  |

| `get_by_name` | function | get_by_name(&self, name: &str) -> Option<Fixture> |  |  |  |  |

| `insert` | function | insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) -> Result<Fixture> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) -> Result<Self> |  |  |  |  |

| `reindex` | function | reindex(&mut self) -> Result<()> |  |  |  |  |

| `save` | function | save(&self) -> Result<()> |  |  |  |  |

| `search` | function | search(&self, query: &FixtureQuery) -> Vec<Fixture> |  |  |  |  |

| `Fixture` | struct | Fixture { pub id: String, pub name: String, pub tags: Vec<String>, pub event_count: usize, pub event_types: Vec<String>, pub chain_hash: String, pub inserted_at: String, pub receipt: Receipt } |  |  |  |  |

| `FixtureDatabase` | struct | FixtureDatabase { path: PathBuf, db: JsonDb, index_by_name: BTreeMap<String, usize>, index_by_event_count: BTreeMap<usize, Vec<usize>>, index_by_chain_hash: BTreeMap<String, usize> } |  |  |  |  |

| `FixtureQuery` | struct | FixtureQuery { pub name_contains: Option<String>, pub tag: Option<String>, pub min_events: Option<usize>, pub max_events: Option<usize>, pub event_type: Option<String>, pub limit: Option<usize> } |  |  |  |  |


### src/gall.rs

| `CrownRefusal` | enum | CrownRefusal { InvalidCompositionDigest, InvalidReceiptDigest(String), InvalidPredecessorField { field: &'static str, value: String }, DuplicatePredecessorWorkOrder(String), PredecessorReceiptMismatch, PredecessorNotAlive(String), WrongGateSet(Vec<u8>), DuplicateGate(u8), GateSubjectMismatch { gate: u8 }, VacuousPass { gate: u8 }, MissingFalsifier { gate: u8 }, Gate12NotPositive } |  |  |  |  |

| `CrownStanding` | enum | CrownStanding { Alive, PartialAlive, Refused } |  |  |  |  |

| `GateStatus` | enum | GateStatus { Pass, Open, Refused, Blocked } |  |  |  |  |

| `certify_gall_crown` | function | certify_gall_crown(manifest: &CrownManifest) -> Result<CrownReceipt, CrownRefusal> |  |  |  |  |

| `CrownManifest` | struct | CrownManifest { pub schema: String, pub composition_digest: String, pub predecessor_receipts: Vec<String>, pub predecessor_witnesses: Vec<PredecessorWitness>, pub gates: Vec<GateWitness> } |  |  |  |  |

| `CrownReceipt` | struct | CrownReceipt { pub schema: String, pub composition_digest: String, pub predecessor_receipts: Vec<String>, pub predecessor_witnesses: Vec<PredecessorWitness>, pub gates: Vec<GateWitness>, pub standing: CrownStanding, pub evidence_ceiling: String, pub receipt_digest: String } |  |  |  |  |

| `GateWitness` | struct | GateWitness { pub gate: u8, pub name: String, pub status: GateStatus, pub subject_digest: String, pub evidence_digest: String, pub positive_observed: bool, pub falsifier_required: bool, pub falsifier_attempted: bool } |  |  |  |  |

| `PredecessorWitness` | struct | PredecessorWitness { pub receipt_iri: String, pub receipt_digest: String, pub work_order_iri: String, pub graph_digest: String, pub repository_identity: String, pub head_sha: String, pub standing: String } |  |  |  |  |


### src/generation.rs

| `DEFAULT_SNIPPETS` | const | DEFAULT_SNIPPETS: &str |  |  |  |  |

| `TEST_FN_TEMPLATE` | const | TEST_FN_TEMPLATE: &str |  |  |  |  |

| `TEST_MODULE_TEMPLATE` | const | TEST_MODULE_TEMPLATE: &str |  |  |  |  |

| `filter_by_tag` | function | filter_by_tag(&self, tag: &str) -> Vec<&Snippet> |  |  |  |  |

| `find_by_name` | function | find_by_name(&self, pattern: &str) -> Vec<&Snippet> |  |  |  |  |

| `format_snippet` | function | format_snippet(&self, snippet: &Snippet) -> String |  |  |  |  |

| `from_json` | function | from_json(json: &str) -> Result<Self> |  |  |  |  |

| `generate_test_function` | function | generate_test_function(&self, pattern_name: &str, events: Vec<serde_json::Value>, expected_verdict: &str, expected_failure_stage: Option<&str> ) -> Result<String> |  |  |  |  |

| `generate_test_module` | function | generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) -> Result<String> |  |  |  |  |

| `main` | function | main() -> Result<()> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `CodegenEngine` | struct | CodegenEngine { tera: Tera } |  |  |  |  |

| `Snippet` | struct | Snippet { pub name: String, pub tags: Vec<String>, pub description: String, pub language: String, pub imports: Vec<String>, pub code: String } |  |  |  |  |

| `SnippetRegistry` | struct | SnippetRegistry { pub snippets: Vec<Snippet> } |  |  |  |  |


### src/handlers.rs

| `CheckStatus` | enum | CheckStatus { Ok, Warn, Fail } |  |  |  |  |

| `anomaly_detect` | function | anomaly_detect( receipts_path: String, sensitivity: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `assemble` | function | assemble(out: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `assemble_and_notarize` | function | assemble_and_notarize( notary_provider: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `assemble_with_signature` | function | assemble_with_signature( signing_method: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `attest` | function | attest( receipt: String, attestation_type: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `audit` | function | audit() -> Result<()> |  |  |  |  |

| `bus_factor` | function | bus_factor(receipts_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `catalog` | function | catalog(filter_name: Option<String>, filter_events: Option<usize>) -> Result<()> |  |  |  |  |

| `causality_chain` | function | causality_chain( start_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `conformance` | function | conformance(receipt: String) -> Result<()> |  |  |  |  |

| `coverage_analysis` | function | coverage_analysis( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `dependency_matrix` | function | dependency_matrix( receipts_path: String, output_matrix: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `diagnose` | function | diagnose(receipt: String) -> Result<()> |  |  |  |  |

| `diff` | function | diff(receipt_a: String, receipt_b: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `doctor` | function | doctor(receipts: Option<String>, fix: bool) -> Result<()> |  |  |  |  |

| `dora_metrics` | function | dora_metrics( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify( receipt: String, observation: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit` | function | emit(r -> Result<()> |  |  |  |  |

| `emit_batch` | function | emit_batch(batch_file: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_cicd` | function | emit_from_cicd(provider: String, job_status: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_cloud` | function | emit_from_cloud( provider: String, resource_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_github` | function | emit_from_github(repo: String, event_type: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_gitlab` | function | emit_from_gitlab(repo: String, event_type: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_monitoring` | function | emit_from_monitoring( provider: String, alert_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_quality` | function | emit_from_quality(working_dir: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_security` | function | emit_from_security( provider: String, vuln_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_ocel_quality_measurement` | function | emit_ocel_quality_measurement( working_dir: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_ocel_quality_violation` | function | emit_ocel_quality_violation( working_dir: Option<String>, baseline_commits: Option<u32>, format: Option<String>, rules: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_violation_causal_chain` | function | emit_violation_causal_chain( receipt_path: String, metric_filter: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `envelope_export` | function | envelope_export(sealed_file: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `envelope_list` | function | envelope_list(store: Option<String>) -> Result<()> |  |  |  |  |

| `envelope_sign` | function | envelope_sign(receipt: String, key_file: String, out: Option<String>) -> Result<()> |  |  |  |  |

| `envelope_verify` | function | envelope_verify( sealed_file: String, store: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_assure` | function | errc_assure( parent: String, witnesses: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_certify` | function | errc_certify( receipt: String, observation: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_verify` | function | errc_verify(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance( receipt: String, parent: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `evidence_crl_apply` | function | evidence_crl_apply(file: String, store: Option<String>) -> Result<()> |  |  |  |  |

| `evidence_crl_publish` | function | evidence_crl_publish( kid: String, epoch: u64, store: Option<String>, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `evidence_heads` | function | evidence_heads(journal_file: Option<String>) -> Result<()> |  |  |  |  |

| `evidence_journal` | function | evidence_journal(subject: String, out: Option<String>) -> Result<()> |  |  |  |  |

| `explain_incident` | function | explain_incident( incident_desc: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `find_blast_radius` | function | find_blast_radius( change_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `fix_receipt` | function | fix_receipt( receipt: String, action: Option<String>, dry_run: bool, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `gdpr_proof` | function | gdpr_proof( receipts_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `graph` | function | graph(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `guide_search` | function | guide_search(keyword: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `hipaa` | function | hipaa(receipts_path: String, out: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `inspect` | function | inspect(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `install_git_hook` | function | install_git_hook(threshold: Option<String>) -> Result<()> |  |  |  |  |

| `keys_generate` | function | keys_generate(algorithm: String, custodian: String, out: Option<String>) -> Result<()> |  |  |  |  |

| `keys_import` | function | keys_import( algorithm: String, public_key_hex: String, custodian: String, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `keys_list` | function | keys_list(store: Option<String>) -> Result<()> |  |  |  |  |

| `keys_revoke` | function | keys_revoke(kid: String, reason: String, store: Option<String>) -> Result<()> |  |  |  |  |

| `keys_rotate` | function | keys_rotate(kid: String, store: Option<String>, out: Option<String>) -> Result<()> |  |  |  |  |

| `license_compliance` | function | license_compliance( receipts_path: String, license_policy: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `model` | function | model(receipt: String) -> Result<()> |  |  |  |  |

| `monitor` | function | monitor( watch: Option<String>, _metrics: Option<String>, _rules: Option<String>, baseline_commits: Option<u32>, interval: Option<u64>, output: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `notarize` | function | notarize(receipt: String, out: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `orphaned_code` | function | orphaned_code( receipts_path: String, days: Option<u32>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `pci_dss` | function | pci_dss(receipts_path: String, out: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `policy_enforce` | function | policy_enforce( receipts_path: String, policy_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `portfolio_health` | function | portfolio_health( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `predict` | function | predict( receipts_path: String, prediction_type: String, _model: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `profile` | function | profile(receipt: Option<String>, duration: Option<u64>) -> Result<()> |  |  |  |  |

| `query` | function | query(q: String, receipts_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `receipt_throughput` | function | receipt_throughput(iterations: Option<u32>) -> Result<()> |  |  |  |  |

| `replay` | function | replay(receipt: String) -> Result<()> |  |  |  |  |

| `root_cause` | function | root_cause( effect_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_attest` | function | sbom_attest( sbom_path: String, receipt: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_blast_radius` | function | sbom_blast_radius( sbom_path: String, component: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_compliance` | function | sbom_compliance( sbom_path: String, framework: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_emit` | function | sbom_emit(sbom_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `sbom_ntia` | function | sbom_ntia(sbom_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `sbom_scan` | function | sbom_scan( sbom_path: String, advisories_path: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `search` | function | search(pattern: String, receipts_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `security_debt` | function | security_debt( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `send_violation_webhook` | function | send_violation_webhook( violation: &crate::quality::QualityViolation, webhook_url: &str, ) -> anyhow::Result<()> |  |  |  |  |

| `show` | function | show(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `sign` | function | sign( receipt: String, key_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `soc2_audit` | function | soc2_audit( receipts_path: String, soc2_type: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `standing_certify` | function | standing_certify( receipt: String, observation: String, scope: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `standing_verify` | function | standing_verify(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `stats` | function | stats(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `team_velocity` | function | team_velocity( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `tech_debt` | function | tech_debt( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `test` | function | test() -> Result<()> |  |  |  |  |

| `timeline` | function | timeline( receipts_path: String, start_time: Option<String>, end_time: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `trend_analysis` | function | trend_analysis( receipts_path: String, metric: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `variance` | function | variance(receipt: Option<String>, iterations: Option<u32>) -> Result<()> |  |  |  |  |

| `verify` | function | verify( receipt: String, format: Option<String>, _profile: Option<String>, _strict: Option<bool>, ) -> Result<()> |  |  |  |  |

| `verify_compliance` | function | verify_compliance(receipt: String, framework: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `verify_family` | function | verify_family(receipts_dir: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `verify_sla` | function | verify_sla(receipt: String, sla_file: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `visualize` | function | visualize(format: String, receipt: String) -> Result<()> |  |  |  |  |

| `why` | function | why(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `.affi/crl.json` | str_key | EVIDENCE_CRL_FILE = ".affi/crl.json" |  |  |  |  |

| `.affi/standing-journal.jsonl` | str_key | EVIDENCE_JOURNAL_FILE = ".affi/standing-journal.jsonl" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `AFFI_NOTARY_KEY` | str_key | ENV_NOTARY_KEY = "AFFI_NOTARY_KEY" |  |  |  |  |

| `AFFI_SIGNING_KEY_PATH` | str_key | ENV_SIGNING_KEY_PATH = "AFFI_SIGNING_KEY_PATH" |  |  |  |  |

| `CTP-REVOCATIONS-v1` | str_key | REVOCATIONS_SIDECAR_FORMAT = "CTP-REVOCATIONS-v1" |  |  |  |  |

| `affidavit-crypto-trust` | str_key | SIGNING_METHOD_TRUST_PLANE = "affidavit-crypto-trust" |  |  |  |  |

| `affidavit-notary-local` | str_key | NOTARY_AUDIENCE = "affidavit-notary-local" |  |  |  |  |

| `affidavit.cli` | str_key | ENVELOPE_SIGN_AUDIENCE = "affidavit.cli" |  |  |  |  |

| `linear-3` | str_key | BUILTIN_FIXTURES = "linear-3" |  |  |  |  |

| `local-key-file-unattributed` | str_key | KEY_FILE_CUSTODIAN = "local-key-file-unattributed" |  |  |  |  |

| `DoctorFinding` | struct | DoctorFinding { pub check: String, pub status: CheckStatus, pub message: String, pub remediation: Option<String>, pub auto_fixable: bool } |  |  |  |  |

| `RevocationSidecarEntry` | struct | RevocationSidecarEntry { pub kid: String, pub revoked_at: u64, pub reason: String } |  |  |  |  |


### src/hlc.rs

| `HlcError` | enum | HlcError { ClockSkewExceeded { received_ms: u64, skew_ms: u64, } } |  |  |  |  |

| `concurrent` | function | concurrent(a: HlcTimestamp, b: HlcTimestamp) -> bool |  |  |  |  |

| `happened_before` | function | happened_before(a: HlcTimestamp, b: HlcTimestamp) -> bool |  |  |  |  |

| `peek` | function | peek(&self) -> HlcTimestamp |  |  |  |  |

| `receive` | function | receive(&mut self, received: HlcTimestamp) -> Result<HlcTimestamp, HlcError> |  |  |  |  |

| `send` | function | send(&mut self) -> HlcTimestamp |  |  |  |  |

| `with_skew_bound` | function | with_skew_bound(max_skew_ms: u64) -> Self |  |  |  |  |

| `HlcClock` | struct | HlcClock { last: HlcTimestamp, max_skew_ms: u64 } |  |  |  |  |

| `HlcTimestamp` | struct | HlcTimestamp { pub physical_ms: u64, pub logical: u32 } |  |  |  |  |


### src/lib.rs

| `run` | function | run() -> clap_noun_verb::Result<()> |  |  |  |  |

| `architecture::{ ArchitectureQualificationReceipt, ArchitectureRefusal, ArchitectureStanding, ArchitectureStandingLedger, EvidenceSource, QualificationEvidence, Supersession, ARCHITECTURE_QUERY_SCHEMA, ARCHITECTURE_RECEIPT_SCHEMA, }` | use | architecture::{ ArchitectureQualificationReceipt, ArchitectureRefusal, ArchitectureStanding, ArchitectureStandingLedger, EvidenceSource, QualificationEvidence, Supersession, ARCHITECTURE_QUERY_SCHEMA, ARCHITECTURE_RECEIPT_SCHEMA, } |  |  |  |  |

| `ecosystem::{ certify_ecosystem, EcosystemMember, EcosystemObservation, EcosystemReceipt, EcosystemRefusal, EcosystemRole, RoleCoverage, RoleRequirement, ECOSYSTEM_AUTHORITY_CEILING, ECOSYSTEM_CLAIM_CEILING, ECOSYSTEM_PROFILE, }` | use | ecosystem::{ certify_ecosystem, EcosystemMember, EcosystemObservation, EcosystemReceipt, EcosystemRefusal, EcosystemRole, RoleCoverage, RoleRequirement, ECOSYSTEM_AUTHORITY_CEILING, ECOSYSTEM_CLAIM_CEILING, ECOSYSTEM_PROFILE, } |  |  |  |  |

| `errc::{ certify_errc, ErrcClaim, ErrcMeasure, ErrcObservation, ErrcQuadrant, ErrcReceipt, ErrcRefusal, ErrcSource, PreservedInvariant, QuadrantCounts, ERRC_CLAIM_CEILING, ERRC_PROFILE, ERRC_SOURCE_ARTIFACT, ERRC_SOURCE_COMMIT, ERRC_SOURCE_REPOSITORY, }` | use | errc::{ certify_errc, ErrcClaim, ErrcMeasure, ErrcObservation, ErrcQuadrant, ErrcReceipt, ErrcRefusal, ErrcSource, PreservedInvariant, QuadrantCounts, ERRC_CLAIM_CEILING, ERRC_PROFILE, ERRC_SOURCE_ARTIFACT, ERRC_SOURCE_COMMIT, ERRC_SOURCE_REPOSITORY, } |  |  |  |  |

| `errc_claim_assurance::{ certify_errc_claim_assurance, ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal, ErrcClaimWitness, ERRC_CLAIM_ASSURANCE_CEILING, ERRC_CLAIM_ASSURANCE_PROFILE, ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT, }` | use | errc_claim_assurance::{ certify_errc_claim_assurance, ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal, ErrcClaimWitness, ERRC_CLAIM_ASSURANCE_CEILING, ERRC_CLAIM_ASSURANCE_PROFILE, ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT, } |  |  |  |  |

| `error::AffidavitError` | use | error::AffidavitError |  |  |  |  |

| `federation::{CourtOutcome, STANDING_CAPABILITY, STANDING_CAPABILITY_DIGEST}` | use | federation::{CourtOutcome, STANDING_CAPABILITY, STANDING_CAPABILITY_DIGEST} |  |  |  |  |

| `standing::{ certify_standing, AuthorityBinding, ExecutionEvidence, ReplayEvidence, Standing, StandingObservation, StandingReceipt, StandingRefusal, SubjectIdentity, VerificationEvidence, STANDING_PROFILE, }` | use | standing::{ certify_standing, AuthorityBinding, ExecutionEvidence, ReplayEvidence, Standing, StandingObservation, StandingReceipt, StandingRefusal, SubjectIdentity, VerificationEvidence, STANDING_PROFILE, } |  |  |  |  |

| `types::{ canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt, Verdict, }` | use | types::{ canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt, Verdict, } |  |  |  |  |


### src/lsp/diagnostics.rs

| `DIAGNOSTIC_SOURCE` | const | DIAGNOSTIC_SOURCE: &str |  |  |  |  |

| `verdict_to_diagnostics` | function | verdict_to_diagnostics(verdict: &crate::types::Verdict) -> Vec<Diagnostic> |  |  |  |  |

| `affidavit` | str_key | DIAGNOSTIC_SOURCE = "affidavit" |  |  |  |  |


### src/lsp/goto_definition.rs

| `goto_definition_for_event_type` | function | goto_definition_for_event_type(event_type: &str) -> Option<Location> |  |  |  |  |


### src/lsp/hover.rs

| `hover_for_event_id` | function | hover_for_event_id(event_id: &str, receipt: &Receipt) -> Option<Hover> |  |  |  |  |


### src/lsp/mod.rs

| `diagnostics::*` | use | diagnostics::* |  |  |  |  |

| `goto_definition::*` | use | goto_definition::* |  |  |  |  |

| `hover::*` | use | hover::* |  |  |  |  |


### src/metrics.rs

| `SLO_AVAILABILITY_PCT` | const | SLO_AVAILABILITY_PCT: f64 |  |  |  |  |

| `SLO_ERROR_RATE_PCT` | const | SLO_ERROR_RATE_PCT: f64 |  |  |  |  |

| `SLO_LATENCY_P99_MS` | const | SLO_LATENCY_P99_MS: f64 |  |  |  |  |

| `SloViolation` | enum | SloViolation { LatencyP99 { observed_ms: f64, threshold_ms: f64 }, ErrorRate { observed_pct: f64, threshold_pct: f64 }, Availability { observed_pct: f64, threshold_pct: f64 } } |  |  |  |  |

| `check_slo` | function | check_slo(&self) -> anyhow::Result<(), SloViolation> |  |  |  |  |

| `compute_sli` | function | compute_sli(&self) -> anyhow::Result<ServiceLevelIndicators> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `new_noop` | function | new_noop() -> Self |  |  |  |  |

| `record_receipt_verified` | function | record_receipt_verified(&self, accepted: bool) |  |  |  |  |

| `record_stage_error` | function | record_stage_error(&self, _stage: &str) |  |  |  |  |

| `record_stage_latency` | function | record_stage_latency(&self, _stage: &str, duration: Duration) |  |  |  |  |

| `render` | function | render(&self) -> String |  |  |  |  |

| `with_window` | function | with_window(window_duration: Duration) -> Self |  |  |  |  |

| `MetricsCollector` | struct | MetricsCollector { inner: Mutex<CollectorState>, window_duration: Duration } |  |  |  |  |

| `PrometheusExporter` | struct | PrometheusExporter { collector: &'a MetricsCollector } |  |  |  |  |

| `ServiceLevelIndicators` | struct | ServiceLevelIndicators { pub latency_p99_ms: f64, pub error_rate_pct: f64, pub availability_pct: f64 } |  |  |  |  |


### src/mining.rs

| `MiningError` | enum | MiningError { Him(String), Admission(String), OcelConversion(String), Alignment(String) } |  |  |  |  |

| `alignment_fitness_score` | function | alignment_fitness_score( admitted: &AdmittedReceipt, model: &PetriNet, ) -> Result<AlignmentReport, MiningError> |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) -> Result<PetriNet, MiningError> |  |  |  |  |

| `interpret_score` | function | interpret_score(fitness: f64) -> &'static str |  |  |  |  |

| `predict_next` | function | predict_next( admitted: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, MiningError> |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) -> Result<OCEL, MiningError> |  |  |  |  |

| `ActivityPrediction` | struct | ActivityPrediction { pub activity: String, pub confidence: f64 } |  |  |  |  |

| `AlignmentReport` | struct | AlignmentReport { pub fitness: f64, pub activity_coverage: f64, pub simplicity: f64, pub interpretation: String, pub unfit_events: u32, pub model_moves: u32 } |  |  |  |  |

| `PredictionReport` | struct | PredictionReport { pub predictions: Vec<ActivityPrediction>, pub context_length: usize, pub model_type: String } |  |  |  |  |


### src/model_mining.rs

| `MiningError` | enum | MiningError { Him(String), Admission(String), OcelConversion(String) } |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) -> Result<PetriNet, MiningError> |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) -> Result<OCEL, MiningError> |  |  |  |  |


### src/mutate.rs

| `MutationKind` | enum | MutationKind { EventDrop, EventReorder, TypeChange, PayloadFlip } |  |  |  |  |

| `all_operators` | function | all_operators() -> Vec<Box<dyn MutationOperator>> |  |  |  |  |

| `AppliedMutation` | struct | AppliedMutation { pub kind: MutationKind, pub target_seq: u64, pub mutated_receipt: Receipt } |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |


### src/mutation.rs

| `MutationKind` | enum | MutationKind { EventDrop, EventReorder, TypeChange, PayloadFlip } |  |  |  |  |

| `all_operators` | function | all_operators() -> Vec<Box<dyn MutationOperator>> |  |  |  |  |

| `AppliedMutation` | struct | AppliedMutation { pub kind: MutationKind, pub target_seq: u64, pub mutated_receipt: Receipt } |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |


### src/ocel.rs

| `build_event` | function | build_event( event_type: impl Into<String>, objects: Vec<ObjectRef>, payload: &[u8], counter: &mut SeqCounter, ) -> std::result::Result<OperationEvent, OcelError> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `next_seq` | function | next_seq(&mut self) -> u64 |  |  |  |  |

| `object_ref` | function | object_ref(id: impl Into<String>, obj_type: impl Into<String>) -> ObjectRef |  |  |  |  |

| `parse_object_ref` | function | parse_object_ref(spec: &str) -> std::result::Result<ObjectRef, OcelError> |  |  |  |  |

| `peek` | function | peek(&self) -> u64 |  |  |  |  |

| `qualified_object_ref` | function | qualified_object_ref( id: impl Into<String>, obj_type: impl Into<String>, qualifier: impl Into<String>, ) -> ObjectRef |  |  |  |  |

| `starting_at` | function | starting_at(value: u64) -> Self |  |  |  |  |

| `validate_event` | function | validate_event(event: &OperationEvent) -> std::result::Result<(), OcelError> |  |  |  |  |

| `SeqCounter` | struct | SeqCounter { value: u64 } |  |  |  |  |

| `crate::error::OcelError` | use | crate::error::OcelError |  |  |  |  |


### src/output.rs

| `Format` | enum | Format { Human, Json, Yaml } |  |  |  |  |

| `data` | function | data(&mut self, value: &T) -> io::Result<()> |  |  |  |  |

| `diag` | function | diag(&mut self, diag: &crate::diag::Diag) -> io::Result<()> |  |  |  |  |

| `from_str` | function | from_str(s: &str) -> Self |  |  |  |  |

| `info` | function | info(&mut self, msg: &str) -> io::Result<()> |  |  |  |  |

| `json` | function | json(&mut self, value: &serde_json::Value) -> io::Result<()> |  |  |  |  |

| `line` | function | line(args: std::fmt::Arguments<'_>) |  |  |  |  |

| `new` | function | new(format: Format) -> Self |  |  |  |  |

| `no_color` | function | no_color(mut self) -> Self |  |  |  |  |

| `print_` | function | print_(args: std::fmt::Arguments<'_>) |  |  |  |  |

| `quiet` | function | quiet(mut self) -> Self |  |  |  |  |

| `verbose` | function | verbose(mut self) -> Self |  |  |  |  |

| `warn` | function | warn(&mut self, msg: &str) -> io::Result<()> |  |  |  |  |

| `with_format` | function | with_format(format: Format) -> Self |  |  |  |  |

| `with_sinks` | function | with_sinks( format: Format, stdout: Box<dyn Write + Send>, stderr: Box<dyn Write + Send>, ) -> Self |  |  |  |  |

| `Out` | struct | Out { format: Format, color: bool, quiet: bool, verbose: bool, stdout: Box<dyn Write + Send>, stderr: Box<dyn Write + Send> } |  |  |  |  |


### src/policy_cedar.rs

| `GateVerdict` | enum | GateVerdict { Admitted, Refused(String) } |  |  |  |  |

| `PolicyError` | enum | PolicyError { Parse(String), Request(String), NoDecision } |  |  |  |  |

| `evaluate` | function | evaluate( &self, principal: &str, action: &str, resource: &str, ) -> Result<GateVerdict, PolicyError> |  |  |  |  |

| `from_policies` | function | from_policies(policies: &str) -> Result<Self, PolicyError> |  |  |  |  |

| `PolicyGate` | struct | PolicyGate { authorizer: Authorizer, policies: PolicySet, entities: Entities } |  |  |  |  |


### src/portable_protocol.rs

| `verify_foreign_receipt` | function | verify_foreign_receipt( consequence_digest: &str, authority: ForeignAuthority<'_>, observation: ForeignObservation<'_>, receipt: ForeignReceipt<'_>, ) -> Result<(), &'static str> |  |  |  |  |

| `ForeignAuthority` | struct | ForeignAuthority { pub decision_id: &'a str, pub consequence_digest: &'a str } |  |  |  |  |

| `ForeignObservation` | struct | ForeignObservation { pub consequence_digest: &'a str, pub observation_digest: &'a str } |  |  |  |  |

| `ForeignReceipt` | struct | ForeignReceipt { pub authority_decision_id: &'a str, pub consequence_digest: &'a str, pub observation_digest: &'a str, pub replay_key: &'a str } |  |  |  |  |


### src/predict_maximalist.rs

| `PredictionError` | enum | PredictionError { InvalidTopK(usize), Wasm4pm(String) } |  |  |  |  |

| `predict_next` | function | predict_next( admitted: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, PredictionError> |  |  |  |  |

| `predict_next_with_model` | function | predict_next_with_model( model: &AdmittedReceipt, current_trace: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, PredictionError> |  |  |  |  |

| `ActivityPrediction` | struct | ActivityPrediction { pub activity: String, pub confidence: f64 } |  |  |  |  |

| `PredictionReport` | struct | PredictionReport { pub predictions: Vec<ActivityPrediction>, pub context_length: usize, pub model_type: String } |  |  |  |  |


### src/quality.rs

| `Notification` | enum | Notification { FileChanged(PathBuf), IntervalElapsed, Error(String) } |  |  |  |  |

| `QualityViolation` | enum | QualityViolation { Rule1Sigma { metric: String, value: f64, threshold: f64, z_score: f64, severity: String, }, Rule9InRow { metric: String, consecutive: usize }, RuleTrend { metric: String, direction: String, count: usize, }, RuleAlternating { metric: String, oscillations: usize }, Rule2of3Beyond2Sigma { metric: String, count: usize, threshold: f64, }, Rule4of5Beyond1Sigma { metric: String, count: usize, threshold: f64, }, Rule15InRowWithin1Sigma { metric: String, count: usize, threshold: f64, severity: String, } } |  |  |  |  |

| `add_measurement` | function | add_measurement(&mut self, metric_name: &str, value: f64) |  |  |  |  |

| `description` | function | description(&self) -> String |  |  |  |  |

| `measure_code_quality` | function | measure_code_quality(src_path: &str) -> anyhow::Result<CodeQualityMetrics> |  |  |  |  |

| `metric` | function | metric(&self) -> &str |  |  |  |  |

| `new` | function | new(baseline_mean: f64, baseline_stddev: f64, window_size: usize) -> Self |  |  |  |  |

| `run_watch_loop` | function | run_watch_loop(&mut self) -> Result<()> |  |  |  |  |

| `run_watch_loop_async` | function | run_watch_loop_async( path: &str, interval_secs: u64, ) -> Result<tokio::task::JoinHandle<Result<()>>> |  |  |  |  |

| `severity` | function | severity(&self) -> &str |  |  |  |  |

| `CodeQualityMetrics` | struct | CodeQualityMetrics { pub stub_ratio: f64, pub type_coverage: f64, pub churn: usize, pub comment_ratio: f64, pub cyclomatic_complexity: f64, pub maintainability_index: f64, pub cognitive_complexity: f64, pub clippy_warnings: usize, pub rustfmt_violations: usize, pub cargo_deny_issues: usize, pub cargo_audit_vulnerabilities: usize, pub test_coverage: f64, pub doc_coverage: f64, pub timestamp: u64 } |  |  |  |  |

| `FileWatcher` | struct | FileWatcher { pub path: PathBuf, rx: Receiver<Notification>, last_measure_time: std::time::Instant, debounce_delay_ms: u64 } |  |  |  |  |

| `WesternElectricAnalyzer` | struct | WesternElectricAnalyzer { pub baseline_mean: f64, pub baseline_stddev: f64, pub rolling_window: VecDeque<f64>, pub window_size: usize, pub control_limits: (f64, f64), pub violations: Vec<QualityViolation> } |  |  |  |  |


### src/quality_correlation.rs

| `amplify_severity_for_correlated_violations` | function | amplify_severity_for_correlated_violations( violations: &[QualityViolation], correlations: &[MetricCorrelation], ) -> HashMap<String, f64> |  |  |  |  |

| `analyze_correlations` | function | analyze_correlations( history: &[CodeQualityMetrics], violations: &[QualityViolation], ) -> CorrelationAnalysis |  |  |  |  |

| `compute_metric_correlations` | function | compute_metric_correlations(history: &[CodeQualityMetrics]) -> Vec<MetricCorrelation> |  |  |  |  |

| `detect_simultaneous_violations` | function | detect_simultaneous_violations( violations: &[QualityViolation], ) -> Vec<SimultaneousViolation> |  |  |  |  |

| `direction` | function | direction(&self) -> &str |  |  |  |  |

| `infer_root_cause` | function | infer_root_cause( metrics: &[CodeQualityMetrics], violation: &QualityViolation, ) -> RootCauseHypothesis |  |  |  |  |

| `is_actionable` | function | is_actionable(&self) -> bool |  |  |  |  |

| `is_compound` | function | is_compound(&self) -> bool |  |  |  |  |

| `is_significant` | function | is_significant(&self) -> bool |  |  |  |  |

| `max_severity` | function | max_severity(&self) -> &str |  |  |  |  |

| `new` | function | new( metric_names: Vec<String>, severities: Vec<String>, timestamps: Vec<u64>, time_window_secs: u64, ) -> Self |  |  |  |  |

| `strength` | function | strength(&self) -> &str |  |  |  |  |

| `CorrelationAnalysis` | struct | CorrelationAnalysis { pub metric_correlations: Vec<MetricCorrelation>, pub simultaneous_violations: Vec<SimultaneousViolation>, pub root_causes: Vec<RootCauseHypothesis>, pub amplified_severities: HashMap<String, f64>, pub timestamp: u64 } |  |  |  |  |

| `MetricCorrelation` | struct | MetricCorrelation { pub metric_a: String, pub metric_b: String, pub pearson_coefficient: f64, pub sample_count: usize, pub standard_error: f64 } |  |  |  |  |

| `RootCauseHypothesis` | struct | RootCauseHypothesis { pub causal_metric: String, pub affected_metric: String, pub correlation: f64, pub confidence: f64, pub lag_seconds: u64, pub evidence: String } |  |  |  |  |

| `SimultaneousViolation` | struct | SimultaneousViolation { pub timestamps: Vec<u64>, pub metric_names: Vec<String>, pub severities: Vec<String>, pub time_window_secs: u64, pub violation_count: usize } |  |  |  |  |


### src/quality_extended.rs

| `RuleVariant` | enum | RuleVariant { Rule1SigmaAt1(String), Rule1SigmaAt2(String), Rule1SigmaAt3(String), Rule1SigmaAtCustom(String, f64), RuleConsecutiveWindow6(String), RuleConsecutiveWindow9(String), RuleConsecutiveWindow15(String), RuleConsecutiveWindow20(String), RuleConsecutiveWindow30(String), RuleTrendWindow6(String), RuleTrendWindow9(String), RuleTrendWindow15(String), RuleTrendWindow20(String), RuleAlternating(String), Rule2of3Beyond2Sigma(String), Rule2of3Beyond1Sigma(String), Rule3of3Beyond2Sigma(String), Rule4of5Beyond1Sigma(String), Rule5of5Beyond1Sigma(String), Rule3of5Beyond1Sigma(String), Rule15InRowWithin1Sigma(String), Rule20InRowWithin1Sigma(String), Rule10InRowWithin1Sigma(String) } |  |  |  |  |

| `add_custom_threshold` | function | add_custom_threshold(mut self, rule_name: String, threshold: f64) -> Self |  |  |  |  |

| `add_measurement` | function | add_measurement(&mut self, metric_name: &str, value: f64) |  |  |  |  |

| `compute` | function | compute(violations: &[QualityViolation]) -> Self |  |  |  |  |

| `compute_aggregate_severity` | function | compute_aggregate_severity(violations: &[QualityViolation]) -> AggregatedSeverity |  |  |  |  |

| `description` | function | description(&self) -> String |  |  |  |  |

| `detect_all_rule_variants` | function | detect_all_rule_variants( metrics: &[f64], config: &WesternElectricConfig, ) -> Vec<RuleVariant> |  |  |  |  |

| `detect_rule_storms` | function | detect_rule_storms(violations: &[QualityViolation]) -> Vec<RuleStorm> |  |  |  |  |

| `finalize` | function | finalize(&mut self) -> AggregatedSeverity |  |  |  |  |

| `metric` | function | metric(&self) -> &str |  |  |  |  |

| `new` | function | new(baseline_mean: f64, baseline_stddev: f64) -> Self |  |  |  |  |

| `severity` | function | severity(&self) -> &'static str |  |  |  |  |

| `with_enabled_rules` | function | with_enabled_rules( mut self, rule1: bool, rule2: bool, rule3: bool, rule4: bool, rule5: bool, rule6: bool, rule7: bool, ) -> Self |  |  |  |  |

| `with_sigmas` | function | with_sigmas(mut self, primary: f64, secondary: f64, tertiary: f64) -> Self |  |  |  |  |

| `AggregatedSeverity` | struct | AggregatedSeverity { pub total_violations: usize, pub critical_count: usize, pub high_count: usize, pub medium_count: usize, pub low_count: usize, pub rule_storm_count: usize, pub worst_severity: String, pub affected_metrics: Vec<String>, pub summary: String } |  |  |  |  |

| `EnhancedWesternElectricAnalyzer` | struct | EnhancedWesternElectricAnalyzer { pub config: WesternElectricConfig, pub rolling_windows: HashMap<usize, VecDeque<f64>>, pub detected_rules: Vec<RuleVariant>, pub rule_storms: Vec<RuleStorm>, pub severity_report: Option<AggregatedSeverity> } |  |  |  |  |

| `RuleStorm` | struct | RuleStorm { pub metric: String, pub rules: Vec<RuleVariant>, pub aggregate_severity: String, pub rule_count: usize, pub is_severe: bool, pub summary: String } |  |  |  |  |

| `WesternElectricConfig` | struct | WesternElectricConfig { pub baseline_mean: f64, pub baseline_stddev: f64, pub primary_sigma: f64, pub secondary_sigma: f64, pub tertiary_sigma: f64, pub custom_thresholds: HashMap<String, f64>, pub window_sizes: Vec<usize>, pub rule1_enabled: bool, pub rule2_enabled: bool, pub rule3_enabled: bool, pub rule4_enabled: bool, pub rule5_enabled: bool, pub rule6_enabled: bool, pub rule7_enabled: bool, pub min_consecutive_violations: usize, pub rule_storm_threshold: usize } |  |  |  |  |


### src/quality_object_level.rs

| `ObjectViolation` | enum | ObjectViolation { FileViolation { file_path: String, violation_type: String, value: f64, threshold: f64, severity: String, }, ModuleViolation { module_name: String, violation_type: String, mean_value: f64, stddev_value: f64, severity: String, }, PackageViolation { package_name: String, violation_type: String, current_score: f64, threshold: f64, severity: String, }, APIBreakingChange { from_package: String, to_package: String, description: String, severity: String, }, DependencyViolation { package_name: String, dependency_name: String, reason: String, severity: String, } } |  |  |  |  |

| `aggregate_module_metrics` | function | aggregate_module_metrics(files: &[FileQualityMetrics]) -> ModuleQualityMetrics |  |  |  |  |

| `compute_package_health` | function | compute_package_health(modules: &[ModuleQualityMetrics]) -> PackageHealthScore |  |  |  |  |

| `description` | function | description(&self) -> String |  |  |  |  |

| `detect_object_level_violations` | function | detect_object_level_violations( object_metric: &FileQualityMetrics, baseline: f64, stddev: f64, ) -> Vec<ObjectViolation> |  |  |  |  |

| `maintainability_index` | function | maintainability_index(&self) -> f64 |  |  |  |  |

| `measure_file_quality` | function | measure_file_quality(path: &str) -> anyhow::Result<FileQualityMetrics> |  |  |  |  |

| `new` | function | new(path: String) -> Self |  |  |  |  |

| `severity` | function | severity(&self) -> &str |  |  |  |  |

| `update_health_score` | function | update_health_score(&mut self) |  |  |  |  |

| `FileQualityMetrics` | struct | FileQualityMetrics { pub path: String, pub stub_ratio: f64, pub cyclomatic_complexity: f64, pub max_cyclomatic_complexity: f64, pub test_coverage: f64, pub doc_coverage: f64, pub loc: usize, pub comment_lines: usize, pub public_items: usize, pub documented_public_items: usize, pub cognitive_complexity: f64, pub type_coverage: f64 } |  |  |  |  |

| `ModuleQualityMetrics` | struct | ModuleQualityMetrics { pub module_name: String, pub file_count: usize, pub mean_stub_ratio: f64, pub stddev_stub_ratio: f64, pub mean_cyclomatic_complexity: f64, pub stddev_cyclomatic_complexity: f64, pub max_cyclomatic_complexity: f64, pub mean_test_coverage: f64, pub stddev_test_coverage: f64, pub mean_doc_coverage: f64, pub stddev_doc_coverage: f64, pub total_loc: usize, pub total_public_items: usize, pub total_documented_items: usize, pub mean_type_coverage: f64 } |  |  |  |  |

| `PackageHealthScore` | struct | PackageHealthScore { pub package_name: String, pub health_score: f64, pub test_coverage: f64, pub doc_coverage: f64, pub stub_ratio: f64, pub mean_complexity: f64, pub violation_count: usize, pub api_breaking_changes: usize, pub dependency_violations: usize } |  |  |  |  |


### src/quality_ocel.rs

| `add_event` | function | add_event(&mut self, event: OcelQualityEvent) |  |  |  |  |

| `build_causal_chain` | function | build_causal_chain( violation_event: &OcelQualityEvent, event_log: &[OcelQualityEvent], ) -> Result<ViolationCausalChain, OcelError> |  |  |  |  |

| `correlate_violations_across_objects` | function | correlate_violations_across_objects( log: &OcelQualityLog, ) -> Result<Vec<ObjectCorrelation>, OcelError> |  |  |  |  |

| `events_by_type` | function | events_by_type(&self, quality_event_type: &str) -> Vec<&OcelQualityEvent> |  |  |  |  |

| `from_metrics` | function | from_metrics( object_id: impl Into<String>, object_type: impl Into<String>, metrics: &CodeQualityMetrics, ) -> Self |  |  |  |  |

| `measure_to_ocel_event` | function | measure_to_ocel_event( event_id: &str, seq: u64, metrics: &CodeQualityMetrics, objects: &[ObjectRef], ) -> Result<OcelQualityEvent, OcelError> |  |  |  |  |

| `measurements` | function | measurements(&self) -> Vec<&OcelQualityEvent> |  |  |  |  |

| `new` | function | new(log_id: impl Into<String>, timestamp: u64) -> Self |  |  |  |  |

| `violation_to_ocel_event` | function | violation_to_ocel_event( event_id: &str, seq: u64, violation: &QualityViolation, triggered_by_event_id: &str, objects: &[ObjectRef], ) -> Result<OcelQualityEvent, OcelError> |  |  |  |  |

| `violations` | function | violations(&self) -> Vec<&OcelQualityEvent> |  |  |  |  |

| `ObjectCorrelation` | struct | ObjectCorrelation { pub metric: String, pub severity: String, pub object_ids: Vec<String>, pub object_count: usize, pub event_ids: Vec<String>, pub first_detected_seq: u64, pub last_detected_seq: u64 } |  |  |  |  |

| `ObjectQualityRecord` | struct | ObjectQualityRecord { pub object_id: String, pub object_type: String, pub stub_ratio: f64, pub type_coverage: f64, pub cyclomatic_complexity: f64, pub cognitive_complexity: f64, pub test_coverage: f64, pub doc_coverage: f64, pub clippy_warnings: usize, pub churn: usize, pub measured_at: u64 } |  |  |  |  |

| `OcelQualityEvent` | struct | OcelQualityEvent { pub event: OperationEvent, pub quality_event_type: String, pub triggered_by_event_id: Option<String>, pub quality_payload: serde_json::Value, pub severity: Option<String> } |  |  |  |  |

| `OcelQualityLog` | struct | OcelQualityLog { pub log_id: String, pub events: Vec<OcelQualityEvent>, pub object_records: BTreeMap<String, ObjectQualityRecord>, pub causal_chains: BTreeMap<String, ViolationCausalChain>, pub metric_correlations: BTreeMap<String, Vec<String>>, pub created_at: u64, pub updated_at: u64 } |  |  |  |  |

| `ViolationCausalChain` | struct | ViolationCausalChain { pub root_measurement_event_id: String, pub violation_event_id: String, pub remediation_event_id: Option<String>, pub measured_value: f64, pub threshold: f64, pub violation: QualityViolation, pub remediation_action: Option<String>, pub remediation_status: String, pub event_sequence: Vec<String> } |  |  |  |  |


### src/quantized_payoff.rs

| `DimensionError` | enum | DimensionError { RowCount { expected: usize, got: usize, }, ColCount { row: usize, expected: usize, got: usize, } } |  |  |  |  |

| `MatrixError` | enum | MatrixError { Cell { row: usize, col: usize, source: QuantizationRefusal, }, Dimensions( } |  |  |  |  |

| `QuantizationRefusal` | enum | QuantizationRefusal { NotFinite { score: f64, }, OutOfRange { score: f64, } } |  |  |  |  |

| `bits` | function | bits(&self) -> &[Vec<Q16F16Bits>] |  |  |  |  |

| `from_fractions` | function | from_fractions( n_nodes: usize, lenses: usize, rows: &[Vec<(u64, u64)>], ) -> Result<Self, DimensionError> |  |  |  |  |

| `from_scores` | function | from_scores( n_nodes: usize, lenses: usize, rows: &[Vec<f64>], ) -> Result<Self, MatrixError> |  |  |  |  |

| `lenses` | function | lenses(&self) -> usize |  |  |  |  |

| `nodes` | function | nodes(&self) -> usize |  |  |  |  |

| `quantize` | function | quantize(score: f64) -> Result<Q16F16Bits, QuantizationRefusal> |  |  |  |  |

| `quantize_fraction` | function | quantize_fraction(num: u64, den: u64) -> Q16F16Bits |  |  |  |  |

| `zeroed` | function | zeroed(n_nodes: usize, lenses: usize) -> Self |  |  |  |  |

| `PayoffMatrix` | struct | PayoffMatrix { matrix: Vec<Vec<Q16F16Bits>>, lenses: usize } |  |  |  |  |


### src/receipts_certified.rs

| `CERTIFIED_RECEIPT_AUDIENCE` | const | CERTIFIED_RECEIPT_AUDIENCE: &str |  |  |  |  |

| `PAID_DELIVERY_DOMAIN` | const | PAID_DELIVERY_DOMAIN: &str |  |  |  |  |

| `VALIDITY_WINDOW_SECONDS` | const | VALIDITY_WINDOW_SECONDS: u64 |  |  |  |  |

| `build_canonical_subject` | function | build_canonical_subject(payload_hash_hex: &str, subject: &str) -> String |  |  |  |  |

| `certify_paid_delivery_payload` | function | certify_paid_delivery_payload( payload_hash_hex: &str, subject: &str, signing: &Es256SigningKey, ) -> Result<CertifiedReceiptEnvelope, VerifyRefusal> |  |  |  |  |

| `verify_certified_paid_delivery` | function | verify_certified_paid_delivery( certified: &CertifiedReceiptEnvelope, payload_hash_hex: &str, subject: &str, ) -> Result<(), VerifyRefusal> |  |  |  |  |

| `affidavit-paid-delivery/v1` | str_key | PAID_DELIVERY_DOMAIN = "affidavit-paid-delivery/v1" |  |  |  |  |

| `affidavit.cli` | str_key | CERTIFIED_RECEIPT_AUDIENCE = "affidavit.cli" |  |  |  |  |

| `CertifiedReceiptEnvelope` | struct | CertifiedReceiptEnvelope { pub envelope: SignatureEnvelope, pub receipt: CryptoStandingReceipt, pub signature_hex: String, pub verifying_key_sec1_hex: String } |  |  |  |  |


### src/registry.rs

| `REGISTRY` | const | REGISTRY: &[VerbEntry] |  |  |  |  |

| `VerbGroup` | enum | VerbGroup { Core, Diagnostics, Analysis, Ingestion, Compliance, Attestation, Sbom, Insights, Engineering, Tooling, Federation } |  |  |  |  |

| `by_group` | function | by_group(group: VerbGroup) -> Vec<&'static VerbEntry> |  |  |  |  |

| `description` | function | description(self) -> &'static str |  |  |  |  |

| `did_you_mean` | function | did_you_mean(input: &str) -> Vec<&'static VerbEntry> |  |  |  |  |

| `label` | function | label(self) -> &'static str |  |  |  |  |

| `lookup` | function | lookup(verb: &str, noun: &str) -> Option<&'static VerbEntry> |  |  |  |  |

| `new` | function | new( verb: &'static str, noun: &'static str, group: VerbGroup, summary: &'static str, keywords: &'static [&'static str], ) -> Self |  |  |  |  |

| `search` | function | search(query: &str) -> Vec<&'static VerbEntry> |  |  |  |  |

| `verb_count` | function | verb_count() -> usize |  |  |  |  |

| `with_example` | function | with_example(mut self, example: &'static str) -> Self |  |  |  |  |

| `emit` | str_key | REGISTRY = "emit" |  |  |  |  |

| `VerbEntry` | struct | VerbEntry { pub verb: &'static str, pub noun: &'static str, pub group: VerbGroup, pub summary: &'static str, pub keywords: &'static [&'static str], pub example: Option<&'static str> } |  |  |  |  |


### src/replay_filter.rs

| `ReplayFilterError` | enum | ReplayFilterError { Full(usize) } |  |  |  |  |

| `capacity` | function | capacity(&self) -> usize |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(capacity: usize) -> Self |  |  |  |  |

| `probably_seen` | function | probably_seen(&self, id: &[u8; 32]) -> bool |  |  |  |  |

| `retract` | function | retract(&mut self, id: &[u8; 32]) -> bool |  |  |  |  |

| `witness` | function | witness(&mut self, id: &[u8; 32]) -> Result<bool, ReplayFilterError> |  |  |  |  |

| `ReplayFilter` | struct | ReplayFilter { filter: CuckooFilter<std::collections::hash_map::DefaultHasher>, capacity: usize } |  |  |  |  |


### src/sbom.rs

| `ComponentType` | enum | ComponentType { Application, Library, Framework, Container, OperatingSystem, Device, Firmware, File, Platform, DeviceDriver, MachineLearningModel, Data } |  |  |  |  |

| `SbomError` | enum | SbomError { Parse(String), UnrecognizedFormat(String), MissingField(String) } |  |  |  |  |

| `SbomFormat` | enum | SbomFormat { Spdx23, Spdx30, CycloneDx15, CycloneDx16, SwidTag } |  |  |  |  |

| `canonicalize` | function | canonicalize(&mut self) |  |  |  |  |

| `component` | function | component(&self, bom_ref: &str) -> Option<&Component> |  |  |  |  |

| `content_address` | function | content_address(&self) -> Blake3Hash |  |  |  |  |

| `detect_format` | function | detect_format(doc: &serde_json::Value) -> Result<SbomFormat, SbomError> |  |  |  |  |

| `expr` | function | expr(expression: impl Into<String>) -> Self |  |  |  |  |

| `family` | function | family(&self) -> &'static str |  |  |  |  |

| `has_unique_identifier` | function | has_unique_identifier(&self) -> bool |  |  |  |  |

| `id` | function | id(spdx_id: impl Into<String>) -> Self |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) -> bool |  |  |  |  |

| `label` | function | label(&self) -> String |  |  |  |  |

| `library` | function | library( bom_ref: impl Into<String>, name: impl Into<String>, version: impl Into<String>, ) -> Self |  |  |  |  |

| `license_labels` | function | license_labels(&self) -> Vec<String> |  |  |  |  |

| `missing` | function | missing(&self) -> Vec<&'static str> |  |  |  |  |

| `new` | function | new(algorithm: impl Into<String>, value: impl Into<String>) -> Self |  |  |  |  |

| `ntia_minimum_elements` | function | ntia_minimum_elements(&self) -> NtiaMinimumElements |  |  |  |  |

| `parse` | function | parse(s: &str) -> ComponentType |  |  |  |  |

| `parse_cyclonedx` | function | parse_cyclonedx(doc: &serde_json::Value) -> Result<Sbom, SbomError> |  |  |  |  |

| `parse_sbom_json` | function | parse_sbom_json(json: &str) -> Result<Sbom, SbomError> |  |  |  |  |

| `parse_spdx` | function | parse_spdx(doc: &serde_json::Value) -> Result<Sbom, SbomError> |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `transitive_dependencies` | function | transitive_dependencies(&self, root: &str) -> Vec<String> |  |  |  |  |

| `Component` | struct | Component { pub bom_ref: String, pub name: String, pub version: String, pub component_type: ComponentType, pub purl: Option<String>, pub cpe: Option<String>, pub supplier: Option<Supplier>, pub author: Option<String>, pub licenses: Vec<License>, pub hashes: Vec<Hash>, pub description: Option<String>, pub scope: Option<String> } |  |  |  |  |

| `Dependency` | struct | Dependency { pub dependent: String, pub depends_on: Vec<String> } |  |  |  |  |

| `Hash` | struct | Hash { pub algorithm: String, pub value: String } |  |  |  |  |

| `License` | struct | License { pub spdx_id: Option<String>, pub expression: Option<String>, pub name: Option<String>, pub url: Option<String> } |  |  |  |  |

| `NtiaMinimumElements` | struct | NtiaMinimumElements { pub supplier_name: bool, pub component_name: bool, pub version: bool, pub unique_identifiers: bool, pub dependency_relationship: bool, pub author: bool, pub timestamp: bool } |  |  |  |  |

| `Sbom` | struct | Sbom { pub format: SbomFormat, pub spec_version: String, pub serial_number: Option<String>, pub version: u32, pub metadata: SbomMetadata, pub components: Vec<Component>, pub dependencies: Vec<Dependency> } |  |  |  |  |

| `SbomMetadata` | struct | SbomMetadata { pub author: Option<String>, pub supplier: Option<Supplier>, pub tools: Vec<Tool>, pub primary_component: Option<String>, pub timestamp: u64 } |  |  |  |  |

| `Supplier` | struct | Supplier { pub name: String, pub url: Option<String>, pub contact: Option<String> } |  |  |  |  |

| `Tool` | struct | Tool { pub vendor: Option<String>, pub name: String, pub version: Option<String> } |  |  |  |  |


### src/sbom_artifacts.rs

| `add_cleanup_action` | function | add_cleanup_action(mut self, action: F) -> Self |  |  |  |  |

| `bundle` | function | bundle(&self) -> Option<&SbomForensicsBundle> |  |  |  |  |

| `cleanup` | function | cleanup(mut self) |  |  |  |  |

| `create_bundle` | function | create_bundle( &self, sbom_id: &str, sbom_format: &str, description: Option<String>, ) -> SbomForensicsBundle |  |  |  |  |

| `load_bundle` | function | load_bundle(&self, path: PathBuf) -> Result<SbomForensicsBundle> |  |  |  |  |

| `new` | function | new() -> Result<Self> |  |  |  |  |

| `redact_sensitive` | function | redact_sensitive(&self, value: &str) -> String |  |  |  |  |

| `save_bundle` | function | save_bundle(&self, bundle: &SbomForensicsBundle, path: PathBuf) -> Result<()> |  |  |  |  |

| `take_bundle` | function | take_bundle(mut self) -> Option<SbomForensicsBundle> |  |  |  |  |

| `with_path` | function | with_path(mut self, path: PathBuf) -> Self |  |  |  |  |

| `with_redaction` | function | with_redaction(mut self, redact: bool) -> Self |  |  |  |  |

| `AttestationData` | struct | AttestationData { pub slsa_version: String, pub builder: Option<String>, pub timestamp: u64, pub environment_hash: String } |  |  |  |  |

| `BundleMetadata` | struct | BundleMetadata { pub version: String, pub created_at: u64, pub sbom_id: String, pub bundle_id: String, pub sbom_format: String, pub description: Option<String> } |  |  |  |  |

| `ComplianceRecord` | struct | ComplianceRecord { pub framework: String, pub passed: bool, pub score: f64, pub failures: Vec<String> } |  |  |  |  |

| `ComponentNode` | struct | ComponentNode { pub purl: String, pub name: String, pub version: String, pub supplier: Option<String>, pub licenses: Vec<String> } |  |  |  |  |

| `RiskPropagationRecord` | struct | RiskPropagationRecord { pub component: String, pub root_cve: String, pub propagated_to: Vec<String>, pub blast_radius: usize } |  |  |  |  |

| `SbomArtifactCollector` | struct | SbomArtifactCollector { work_dir: PathBuf, redact_sensitive: bool } |  |  |  |  |

| `SbomArtifactGuard` | struct | SbomArtifactGuard { bundle: Option<SbomForensicsBundle>, path: Option<PathBuf>, cleanup_actions: Vec<Box<dyn FnOnce() + Send + Sync>> } |  |  |  |  |

| `SbomForensicsBundle` | struct | SbomForensicsBundle { pub metadata: BundleMetadata, pub graph: Option<SupplyChainGraph>, pub vulnerabilities: Vec<VulnerabilityRecord>, pub compliance: Vec<ComplianceRecord>, pub risk_propagation: Vec<RiskPropagationRecord>, pub attestation: Option<AttestationData>, pub redactions: HashMap<String, String> } |  |  |  |  |

| `SupplyChainGraph` | struct | SupplyChainGraph { pub component_count: usize, pub edge_count: usize, pub adjacency: HashMap<String, Vec<String>>, pub components: HashMap<String, ComponentNode> } |  |  |  |  |

| `VulnerabilityRecord` | struct | VulnerabilityRecord { pub cve_id: String, pub affected_component: String, pub severity: String, pub cvss_score: Option<f64>, pub vex_status: Option<String> } |  |  |  |  |


### src/sbom_compliance.rs

| `ComplianceError` | enum | ComplianceError { EmptySbom } |  |  |  |  |

| `Framework` | enum | Framework { Ntia, ExecutiveOrder14028, Slsa, InToto, Cisa, Cscrm, Iso27001, Soc2, Vex } |  |  |  |  |

| `SlsaLevel` | enum | SlsaLevel { L0, L1, L2, L3, L4 } |  |  |  |  |

| `assess_all` | function | assess_all(sbom: &Sbom) -> Result<Vec<ComplianceResult>, ComplianceError> |  |  |  |  |

| `assess_slsa` | function | assess_slsa(sbom: &Sbom) -> Result<(SlsaLevel, ComplianceResult), ComplianceError> |  |  |  |  |

| `check_cisa` | function | check_cisa(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_cscrm` | function | check_cscrm(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_eo_14028` | function | check_eo_14028(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_in_toto` | function | check_in_toto(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_iso_27001` | function | check_iso_27001(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_ntia` | function | check_ntia(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_slsa` | function | check_slsa(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_soc2` | function | check_soc2(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `display_name` | function | display_name(&self) -> &'static str |  |  |  |  |

| `rank` | function | rank(&self) -> u8 |  |  |  |  |

| `requirement_count` | function | requirement_count(&self) -> usize |  |  |  |  |

| `score` | function | score(&self) -> f64 |  |  |  |  |

| `supported_frameworks` | function | supported_frameworks() -> &'static [&'static str] |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `vex_readiness` | function | vex_readiness(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `ComplianceResult` | struct | ComplianceResult { pub framework: String, pub level: Option<String>, pub passed: bool, pub satisfied: Vec<String>, pub failed: Vec<String>, pub notes: Vec<String> } |  |  |  |  |


### src/sbom_ocel.rs

| `EVENT_ATTEST` | const | EVENT_ATTEST: &str |  |  |  |  |

| `EVENT_COMPONENT_CATALOGUED` | const | EVENT_COMPONENT_CATALOGUED: &str |  |  |  |  |

| `EVENT_DEPENDENCY_RESOLVED` | const | EVENT_DEPENDENCY_RESOLVED: &str |  |  |  |  |

| `EVENT_GENERATE` | const | EVENT_GENERATE: &str |  |  |  |  |

| `EVENT_IMPORT` | const | EVENT_IMPORT: &str |  |  |  |  |

| `EVENT_LICENSE_DETECTED` | const | EVENT_LICENSE_DETECTED: &str |  |  |  |  |

| `EVENT_SUPPLIER_ATTESTED` | const | EVENT_SUPPLIER_ATTESTED: &str |  |  |  |  |

| `OBJECT_DOCUMENT` | const | OBJECT_DOCUMENT: &str |  |  |  |  |

| `SbomOcelError` | enum | SbomOcelError { EmptyDocument, UnknownComponent(String), Ocel( } |  |  |  |  |

| `build_sbom_causal_chain` | function | build_sbom_causal_chain( events: &[SbomOcelEvent], sbom: &Sbom, root_bom_ref: &str, ) -> Result<SbomCausalChain, SbomOcelError> |  |  |  |  |

| `component_object_type` | function | component_object_type(component: &Component) -> String |  |  |  |  |

| `correlate_components_by_license` | function | correlate_components_by_license( events: &[SbomOcelEvent], sbom: &Sbom, ) -> Vec<ObjectCorrelation> |  |  |  |  |

| `license_object_type` | function | license_object_type(label: &str) -> String |  |  |  |  |

| `sbom_to_ocel_events` | function | sbom_to_ocel_events( sbom: &Sbom, counter: &mut SeqCounter, ) -> Result<Vec<SbomOcelEvent>, SbomOcelError> |  |  |  |  |

| `supplier_object_type` | function | supplier_object_type(name: &str) -> String |  |  |  |  |

| `supported_event_types` | function | supported_event_types() -> &'static [&'static str] |  |  |  |  |

| `vulnerability_object_type` | function | vulnerability_object_type(id: &str) -> String |  |  |  |  |

| `sbom-document` | str_key | OBJECT_DOCUMENT = "sbom-document" |  |  |  |  |

| `sbom:attest` | str_key | EVENT_ATTEST = "sbom:attest" |  |  |  |  |

| `sbom:component-catalogued` | str_key | EVENT_COMPONENT_CATALOGUED = "sbom:component-catalogued" |  |  |  |  |

| `sbom:dependency-resolved` | str_key | EVENT_DEPENDENCY_RESOLVED = "sbom:dependency-resolved" |  |  |  |  |

| `sbom:generate` | str_key | EVENT_GENERATE = "sbom:generate" |  |  |  |  |

| `sbom:import` | str_key | EVENT_IMPORT = "sbom:import" |  |  |  |  |

| `sbom:license-detected` | str_key | EVENT_LICENSE_DETECTED = "sbom:license-detected" |  |  |  |  |

| `sbom:supplier-attested` | str_key | EVENT_SUPPLIER_ATTESTED = "sbom:supplier-attested" |  |  |  |  |

| `ObjectCorrelation` | struct | ObjectCorrelation { pub object_type: String, pub object_ids: Vec<String>, pub event_ids: Vec<String> } |  |  |  |  |

| `SbomCausalChain` | struct | SbomCausalChain { pub document_event_id: String, pub component_event_id: String, pub dependency_event_ids: Vec<String>, pub root_component: String, pub transitive_component_count: usize } |  |  |  |  |

| `SbomOcelEvent` | struct | SbomOcelEvent { pub event: OperationEvent, pub sbom_event_type: String, pub payload: serde_json::Value } |  |  |  |  |


### src/sbom_supply_chain.rs

| `UNKNOWN_SUPPLIER` | const | UNKNOWN_SUPPLIER: &str |  |  |  |  |

| `SupplyChainError` | enum | SupplyChainError { UnknownComponent(String), EmptyGraph } |  |  |  |  |

| `attest_provenance` | function | attest_provenance(sbom: &Sbom, receipt_ref: Option<&str>) -> ProvenanceAttestation |  |  |  |  |

| `blast_radius` | function | blast_radius( graph: &DependencyGraph, bom_ref: &str, ) -> Result<BlastRadius, SupplyChainError> |  |  |  |  |

| `build_report` | function | build_report(sbom: &Sbom, spof_threshold: usize) -> SupplyChainReport |  |  |  |  |

| `contains` | function | contains(&self, bom_ref: &str) -> bool |  |  |  |  |

| `depth` | function | depth(&self, root: &str) -> usize |  |  |  |  |

| `direct_dependencies` | function | direct_dependencies(&self, bom_ref: &str) -> Vec<String> |  |  |  |  |

| `direct_dependents` | function | direct_dependents(&self, bom_ref: &str) -> Vec<String> |  |  |  |  |

| `edge_count` | function | edge_count(&self) -> usize |  |  |  |  |

| `from_sbom` | function | from_sbom(sbom: &Sbom) -> Self |  |  |  |  |

| `is_cyclic` | function | is_cyclic(&self) -> bool |  |  |  |  |

| `node_count` | function | node_count(&self) -> usize |  |  |  |  |

| `nodes` | function | nodes(&self) -> Vec<String> |  |  |  |  |

| `single_points_of_failure` | function | single_points_of_failure( graph: &DependencyGraph, _sbom: &Sbom, threshold: usize, ) -> Vec<String> |  |  |  |  |

| `supplier_concentration` | function | supplier_concentration(sbom: &Sbom) -> Vec<SupplierConcentration> |  |  |  |  |

| `transitive_dependencies` | function | transitive_dependencies(&self, bom_ref: &str) -> Vec<String> |  |  |  |  |

| `transitive_dependents` | function | transitive_dependents(&self, bom_ref: &str) -> Vec<String> |  |  |  |  |

| `UNKNOWN` | str_key | UNKNOWN_SUPPLIER = "UNKNOWN" |  |  |  |  |

| `BlastRadius` | struct | BlastRadius { pub component: String, pub directly_impacted: usize, pub transitively_impacted: usize, pub impacted: Vec<String> } |  |  |  |  |

| `DependencyGraph` | struct | DependencyGraph { nodes: BTreeSet<String>, forward: BTreeMap<String, BTreeSet<String>>, reverse: BTreeMap<String, BTreeSet<String>> } |  |  |  |  |

| `ProvenanceAttestation` | struct | ProvenanceAttestation { pub sbom_address: String, pub primary_component: Option<String>, pub builder: Option<String>, pub attested_supplier: Option<String>, pub dependency_edges: usize, pub generated_from_receipt: Option<String> } |  |  |  |  |

| `SupplierConcentration` | struct | SupplierConcentration { pub supplier: String, pub component_count: usize, pub share: f64, pub components: Vec<String> } |  |  |  |  |

| `SupplyChainReport` | struct | SupplyChainReport { pub component_count: usize, pub edge_count: usize, pub max_depth: usize, pub is_cyclic: bool, pub supplier_count: usize, pub top_supplier_share: f64, pub spof_count: usize } |  |  |  |  |


### src/sbom_vulnerability.rs

| `Severity` | enum | Severity { None, Low, Medium, High, Critical } |  |  |  |  |

| `VexStatus` | enum | VexStatus { NotAffected, Affected, Fixed, UnderInvestigation } |  |  |  |  |

| `VulnerabilityError` | enum | VulnerabilityError { InvalidCvssScore(f64), UnknownComponent(String) } |  |  |  |  |

| `apply_vex` | function | apply_vex(matches: &[VulnerabilityMatch], vex: &[VexStatement]) -> Vec<VulnerabilityMatch> |  |  |  |  |

| `build_report` | function | build_report( sbom: &Sbom, vulns: &[Vulnerability], vex: &[VexStatement], ) -> VulnerabilityReport |  |  |  |  |

| `cvss_band` | function | cvss_band(score: f64) -> &'static str |  |  |  |  |

| `from_cvss` | function | from_cvss(score: f64) -> Severity |  |  |  |  |

| `from_score` | function | from_score(base_score: f64) -> Self |  |  |  |  |

| `match_vulnerabilities` | function | match_vulnerabilities(sbom: &Sbom, vulns: &[Vulnerability]) -> Vec<VulnerabilityMatch> |  |  |  |  |

| `new` | function | new(id: impl Into<String>, base_score: f64) -> Self |  |  |  |  |

| `propagate_risk` | function | propagate_risk( sbom: &Sbom, matches: &[VulnerabilityMatch], ) -> Vec<(String, String, Severity)> |  |  |  |  |

| `severity` | function | severity(&self) -> Severity |  |  |  |  |

| `suppresses` | function | suppresses(&self) -> bool |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `validate` | function | validate(&self) -> Result<(), VulnerabilityError> |  |  |  |  |

| `with_vector` | function | with_vector(base_score: f64, vector: impl Into<String>) -> Self |  |  |  |  |

| `CvssVector` | struct | CvssVector { pub base_score: f64, pub vector: Option<String> } |  |  |  |  |

| `VexStatement` | struct | VexStatement { pub vuln_id: String, pub component_bom_ref: String, pub status: VexStatus, pub justification: Option<String> } |  |  |  |  |

| `Vulnerability` | struct | Vulnerability { pub id: String, pub description: Option<String>, pub cvss: CvssVector, pub affected_purls: Vec<String>, pub affected_cpes: Vec<String>, pub fixed_versions: Vec<String> } |  |  |  |  |

| `VulnerabilityMatch` | struct | VulnerabilityMatch { pub vuln_id: String, pub component_bom_ref: String, pub severity: Severity, pub matched_by: String } |  |  |  |  |

| `VulnerabilityReport` | struct | VulnerabilityReport { pub total_components: usize, pub total_matches: usize, pub by_severity: BTreeMap<String, usize>, pub max_severity: Severity, pub exploitable_after_vex: usize } |  |  |  |  |


### src/secp256k1_witness.rs

| `Secp256k1WitnessError` | enum | Secp256k1WitnessError { MalformedSchnorrKey, SchnorrVerificationFailed, MalformedEcdsaKey, EcdsaVerificationFailed } |  |  |  |  |

| `WitnessSigningError` | enum | WitnessSigningError { InvalidSeed } |  |  |  |  |

| `from_seed` | function | from_seed(seed: &[u8; 32]) -> Result<Self, WitnessSigningError> |  |  |  |  |

| `public_key_sec1` | function | public_key_sec1(&self) -> [u8; 33] |  |  |  |  |

| `sign` | function | sign(&self, msg: &[u8]) -> [u8; 64] |  |  |  |  |

| `verify_bip340` | function | verify_bip340( x_only_public_key: &[u8; 32], message: &[u8], signature: &[u8; 64], ) -> Result<(), Secp256k1WitnessError> |  |  |  |  |

| `verify_bip340_raw` | function | verify_bip340_raw( x_only_public_key: &[u8; 32], message: &[u8; 32], signature: &[u8; 64], ) -> Result<(), Secp256k1WitnessError> |  |  |  |  |

| `verify_ecdsa` | function | verify_ecdsa( compressed_sec1_public_key: &[u8; 33], message: &[u8], signature: &[u8], ) -> Result<(), Secp256k1WitnessError> |  |  |  |  |

| `WitnessSigningKey` | struct | WitnessSigningKey { signing: EcdsaSigningKey } |  |  |  |  |


### src/seq_bitmap.rs

| `SeqBitmapError` | enum | SeqBitmapError { Gap(u32), Duplicate(u32) } |  |  |  |  |

| `count` | function | count(&self) -> u64 |  |  |  |  |

| `has` | function | has(&self, seq: u32) -> bool |  |  |  |  |

| `max` | function | max(&self) -> Option<u32> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `record` | function | record(&mut self, seq: u32) -> Result<(), SeqBitmapError> |  |  |  |  |

| `verify_contiguous` | function | verify_contiguous(&self) -> Result<(), SeqBitmapError> |  |  |  |  |

| `SeqContiguityCertifier` | struct | SeqContiguityCertifier { bitmap: RoaringBitmap } |  |  |  |  |


### src/sj_record.rs

| `CAMPAIGN_DOMAIN` | const | CAMPAIGN_DOMAIN: &str |  |  |  |  |

| `EVENT_COMMIT` | const | EVENT_COMMIT: &str |  |  |  |  |

| `EVENT_COURT` | const | EVENT_COURT: &str |  |  |  |  |

| `EVENT_RESIDUE` | const | EVENT_RESIDUE: &str |  |  |  |  |

| `AuthorityCeiling` | enum | AuthorityCeiling { Observe, Select, Construct, Do } |  |  |  |  |

| `BrokenTerm` | enum | BrokenTerm { MuOnO, AdmissionVacuous, MuUnlawful, RMissingIdentity, RMissingAuthority, RMissingConsequence, RMissingReplay, RMissingStanding, RNotFedBack } |  |  |  |  |

| `SjRefusal` | enum | SjRefusal { EmptyKey { key: &'static str, }, BadCommitSha { key: &'static str, value: String, }, BadSha { key: &'static str, value: String, }, BadStanding(String), MissingBrokenTerm(String), EmptyReplay, BadCommand { index: usize, reason: &'static str, }, Canonical(String), Chain(String), ChainTamper(String), DigestMismatch { expected: String, claimed: String, }, ChainHeadMismatch, EventClaimMismatch, Decode(String) } |  |  |  |  |

| `StandingValue` | enum | StandingValue { Unknown, PartialAlive, Alive, Blocked( BuildBroken, Unsupported( Refused(String), } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `canonical_json` | function | canonical_json(&self) -> Result<String, SjRefusal> |  |  |  |  |

| `chain_events` | function | chain_events(&self) -> Vec<OperationEvent> |  |  |  |  |

| `finalize` | function | finalize(self) -> Result<SjRecord, SjRefusal> |  |  |  |  |

| `from_json` | function | from_json(bytes: &str) -> Result<Self, SjRefusal> |  |  |  |  |

| `new` | function | new(draft: SjCampaignDraft) -> Result<Self, SjRefusal> |  |  |  |  |

| `requires_broken_term` | function | requires_broken_term(&self) -> bool |  |  |  |  |

| `subject_digest` | function | subject_digest(&self) -> Result<[u8; 32], SjRefusal> |  |  |  |  |

| `subject_digest_hex` | function | subject_digest_hex(&self) -> Result<String, SjRefusal> |  |  |  |  |

| `to_json` | function | to_json(&self) -> Result<String, SjRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), SjRefusal> |  |  |  |  |

| `affidavit-campaign/v1` | str_key | CAMPAIGN_DOMAIN = "affidavit-campaign/v1" |  |  |  |  |

| `campaign-commit` | str_key | EVENT_COMMIT = "campaign-commit" |  |  |  |  |

| `campaign-court-witness` | str_key | EVENT_COURT = "campaign-court-witness" |  |  |  |  |

| `campaign-residue` | str_key | EVENT_RESIDUE = "campaign-residue" |  |  |  |  |

| `Authority` | struct | Authority { pub ceiling: AuthorityCeiling, pub grant: String, pub actor: String } |  |  |  |  |

| `CommitRecord` | struct | CommitRecord { pub sha: String, pub summary: String, pub court_results: Vec<String> } |  |  |  |  |

| `Consequence` | struct | Consequence { pub commits: Vec<String>, pub files_changed: Vec<String>, pub remote_effects: Vec<String> } |  |  |  |  |

| `Identity` | struct | Identity { pub subject: String, pub repo: String, pub subject_sha: String, pub base_sha: String, pub subject_digest: SubjectDigest } |  |  |  |  |

| `OriginAuthority` | struct | OriginAuthority { pub ceiling: Option<AuthorityCeiling>, pub grant: String, pub actor: String } |  |  |  |  |

| `Provider` | struct | Provider { pub name: String, pub transport: Option<String> } |  |  |  |  |

| `Replay` | struct | Replay { pub commands: Vec<ReplayCommand>, pub durable_location: Option<String> } |  |  |  |  |

| `ReplayBinding` | struct | ReplayBinding { pub event_ids: Vec<String>, pub chain_head_hash: String, pub predecessor_work_order_ids: Vec<String> } |  |  |  |  |

| `ReplayCommand` | struct | ReplayCommand { pub cmd: String, pub exit: i32, pub cwd: String, pub summary: Option<String>, pub output_sha256: Option<String> } |  |  |  |  |

| `SjCampaign` | struct | SjCampaign { draft: SjCampaignDraft } |  |  |  |  |

| `SjCampaignDraft` | struct | SjCampaignDraft { pub work_order_id: String, pub origin_ceiling: Option<AuthorityCeiling>, pub origin_grant: String, pub origin_actor: String, pub provider_name: String, pub provider_execution_id: String, pub subject: String, pub repo: String, pub subject_sha: String, pub base_sha: String, pub commits: Vec<CommitRecord>, pub residue_declaration: String, pub files_changed: Vec<String>, pub remote_effects: Vec<String>, pub replay_commands: Vec<ReplayCommand>, pub durable_location: Option<String>, pub standing: StandingValue, pub derived_from: String, pub broken_term: Option<BrokenTerm>, pub predecessor_work_order_ids: Vec<String>, pub authority: Authority } |  |  |  |  |

| `SjRecord` | struct | SjRecord { pub base: Receipt, pub document: SjRecordDocument } |  |  |  |  |

| `SjRecordDocument` | struct | SjRecordDocument { pub work_order_id: String, pub origin_authority: OriginAuthority, pub provider: Provider, pub provider_execution_id: String, pub identity: Identity, pub authority: Authority, pub consequence: Consequence, pub replay: Replay, pub replay_binding: ReplayBinding, pub standing: Standing } |  |  |  |  |

| `Standing` | struct | Standing { pub value: StandingValue, pub derived_from: String, pub broken_term: Option<BrokenTerm> } |  |  |  |  |

| `SubjectDigest` | struct | SubjectDigest { pub algorithm: String, pub value: String } |  |  |  |  |


### src/smt.rs

| `SmtError` | enum | SmtError { Tree(String), NoProof } |  |  |  |  |

| `common_prefix_len` | function | common_prefix_len( a: &BitSlice<u8, bitvec::prelude::Msb0>, b: &BitSlice<u8, bitvec::prelude::Msb0>, ) -> usize |  |  |  |  |

| `get` | function | get(&mut self, key: &StateKey) -> Result<Option<StateValue>, SmtError> |  |  |  |  |

| `insert` | function | insert(&mut self, key: &StateKey, value: &StateValue) -> Result<StateRoot, SmtError> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `prove_absence` | function | prove_absence(&mut self, key: &StateKey) -> Result<AbsenceWitness, SmtError> |  |  |  |  |

| `prove_inclusion` | function | prove_inclusion(&mut self, key: &StateKey) -> Result<InclusionProof, SmtError> |  |  |  |  |

| `remove` | function | remove(&mut self, key: &StateKey) -> Result<Option<StateRoot>, SmtError> |  |  |  |  |

| `root` | function | root(&self) -> Option<StateRoot> |  |  |  |  |

| `verify_absence` | function | verify_absence( root: &StateRoot, neighbor_value: &StateValue, witness: &AbsenceWitness, ) -> bool |  |  |  |  |

| `verify_inclusion` | function | verify_inclusion( root: &StateRoot, expected_value: &StateValue, proof: &InclusionProof, ) -> bool |  |  |  |  |

| `AbsenceWitness` | struct | AbsenceWitness { pub queried_key: StateKey, pub neighbor_key: StateKey, pub neighbor_proof: InclusionProof, pub claimed_prefix_bits: usize } |  |  |  |  |

| `InclusionProof` | struct | InclusionProof { pub key: StateKey, pub steps: Vec<(bool, Vec<u8>)> } |  |  |  |  |

| `StateTree` | struct | StateTree { tree: Monotree<monotree::database::MemoryDB, MonotreeBlake3>, root: Option<monotree::Hash>, keys: std::collections::BTreeMap<StateKey, StateValue> } |  |  |  |  |

| `monotree::HASH_LEN` | use | monotree::HASH_LEN |  |  |  |  |


### src/standing.rs

| `STANDING_PROFILE` | const | STANDING_PROFILE: &str |  |  |  |  |

| `Standing` | enum | Standing { Unknown, PartialAlive, Alive, Blocked, BuildBroken, Unsupported } |  |  |  |  |

| `StandingRefusal` | enum | StandingRefusal { AuthorityEnvelopeRejected(Vec<AuthorityRefusal>), UnsupportedAuthorityConstraint, EmptyField(&'static str), MalformedBlake3(&'static str), AliveMissingExecution, AliveExecutionFailed(i32), AliveMissingVerification, AliveVerificationFailed(i32), AliveMissingReplay, WrongProfile, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `certify_standing` | function | certify_standing( admitted: &AdmittedReceipt, authority: &AuthorityEnvelope<W>, observation: StandingObservation, ) -> Result<StandingReceipt, StandingRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), StandingRefusal> |  |  |  |  |

| `affidavit/standing/v2` | str_key | STANDING_PROFILE = "affidavit/standing/v2" |  |  |  |  |

| `AuthorityBinding` | struct | AuthorityBinding { pub witness_key: String, pub capability: String, pub capability_digest: String, pub scope: String, pub constraints: Vec<String>, pub data_minimization_note: String, pub fairness_attestation_ref: String } |  |  |  |  |

| `ExecutionEvidence` | struct | ExecutionEvidence { pub command: String, pub exit_code: i32, pub result_commitment: Blake3Hash } |  |  |  |  |

| `ReplayEvidence` | struct | ReplayEvidence { pub command: String, pub environment_commitment: Blake3Hash } |  |  |  |  |

| `StandingObservation` | struct | StandingObservation { pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub standing: Standing, pub execution: Option<ExecutionEvidence>, pub verification: Option<VerificationEvidence>, pub replay: Option<ReplayEvidence>, pub previous_receipt: Option<Blake3Hash> } |  |  |  |  |

| `StandingReceipt` | struct | StandingReceipt { pub profile: String, pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub admitted_receipt_hash: Blake3Hash, pub authority: AuthorityBinding, pub standing: Standing, pub execution: Option<ExecutionEvidence>, pub verification: Option<VerificationEvidence>, pub replay: Option<ReplayEvidence>, pub previous_receipt: Option<Blake3Hash>, pub receipt_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `SubjectIdentity` | struct | SubjectIdentity { pub subject: String, pub base: String, pub tree: String, pub candidate: String } |  |  |  |  |

| `VerificationEvidence` | struct | VerificationEvidence { pub command: String, pub exit_code: i32, pub report_commitment: Blake3Hash } |  |  |  |  |


### src/threshold_quorum.rs

| `QuorumError` | enum | QuorumError { Protocol(String), VerificationFailed, MalformedKeyMaterial, BelowThreshold { expected: usize, got: usize, } } |  |  |  |  |

| `aggregate_signature` | function | aggregate_signature( package: &SigningPackage, shares: &[(ParticipantId, frost::round2::SignatureShare)], public_key_package: &PublicKeyPackage, threshold: u16, ) -> Result<[u8; 64], QuorumError> |  |  |  |  |

| `generate_quorum` | function | generate_quorum( total: u16, threshold: u16, rng: &mut R, ) -> Result<DealtQuorum, QuorumError> |  |  |  |  |

| `round1_commit` | function | round1_commit( secret_share: &SecretShare, rng: &mut R, ) -> Result<(SigningNonces, frost::round1::SigningCommitments), QuorumError> |  |  |  |  |

| `round2_sign` | function | round2_sign( secret_share: &SecretShare, nonces: &SigningNonces, package: &SigningPackage, ) -> Result<frost::round2::SignatureShare, QuorumError> |  |  |  |  |

| `signing_package` | function | signing_package( commitments: &[(ParticipantId, frost::round1::SigningCommitments)], message: &[u8], threshold: u16, ) -> Result<SigningPackage, QuorumError> |  |  |  |  |

| `verify_quorum` | function | verify_quorum( group_public_key: &GroupPublicKey, message: &[u8], signature_bytes: &[u8; 64], ) -> Result<(), QuorumError> |  |  |  |  |

| `DealtQuorum` | struct | DealtQuorum { pub shares: BTreeMap<ParticipantId, SecretShare>, pub public_key_package: PublicKeyPackage, pub group_key: GroupPublicKey, pub threshold: u16 } |  |  |  |  |

| `GroupPublicKey` | struct | GroupPublicKey { pub [u8; 32] } |  |  |  |  |

| `ParticipantId` | struct | ParticipantId { pub u16 } |  |  |  |  |


### src/tracing.rs

| `AFFI_TRACE_SINK` | env_key | std::env::var("AFFI_TRACE_SINK") |  |  |  |  |

| `captured_spans` | function | captured_spans() -> Vec<SpanRecord> |  |  |  |  |

| `clear_spans` | function | clear_spans() |  |  |  |  |

| `trace_assemble` | function | trace_assemble(event_count: usize, f: F) -> T |  |  |  |  |

| `trace_emit` | function | trace_emit(event_type: &str, _object_count: usize, f: F) -> T |  |  |  |  |

| `trace_show` | function | trace_show(receipt_path: &str, f: F) -> T |  |  |  |  |

| `trace_verify` | function | trace_verify(receipt_path: &str, f: F) -> T |  |  |  |  |

| `SpanRecord` | struct | SpanRecord { pub operation: String, pub target: String } |  |  |  |  |


### src/types.rs

| `ProfileId` | enum | ProfileId { CoreV1 } |  |  |  |  |

| `as_hex` | function | as_hex(&self) -> &str |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8]) -> Self |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) -> Self |  |  |  |  |

| `AffidavitReceiptChain` | struct |  |  |  |  |  |

| `AssembleOutput` | struct | AssembleOutput { pub receipt_path: String, pub content_address: String, pub event_count: usize } |  |  |  |  |

| `Blake3Hash` | struct | Blake3Hash { pub String } |  |  |  |  |

| `CheckOutcome` | struct | CheckOutcome { pub stage: String, pub passed: bool, pub detail: String } |  |  |  |  |

| `EmitOutput` | struct | EmitOutput { pub event_id: String, pub seq: u64, pub event_type: String, pub commitment: String } |  |  |  |  |

| `EventSummary` | struct | EventSummary { pub seq: u64, pub id: String, pub event_type: String, pub object_count: usize, pub commitment: String } |  |  |  |  |

| `InspectionReport` | struct | InspectionReport { pub event_count: usize, pub format_version: String, pub chain_hash: String, pub chain_integrity_valid: bool, pub event_types: std::collections::BTreeMap<String, usize>, pub object_types: std::collections::BTreeMap<String, usize>, pub events: Vec<EventSummary> } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub obj_type: String, pub qualifier: Option<String> } |  |  |  |  |

| `OperationEvent` | struct | OperationEvent { pub id: String, pub seq: u64, pub event_type: String, pub objects: Vec<ObjectRef>, pub payload_commitment: Blake3Hash } |  |  |  |  |

| `QualityMeasurement` | struct | QualityMeasurement { pub timestamp: u64, pub stubs: QualityMetricValue, pub types: QualityMetricValue, pub churn: QualityMetricValue, pub comments: QualityMetricValue, pub complexity: QualityMetricValue, pub clippy_warnings: QualityMetricValue, pub rustfmt_violations: QualityMetricValue, pub cargo_deny_issues: QualityMetricValue, pub cargo_audit_vulnerabilities: QualityMetricValue, pub test_coverage: QualityMetricValue, pub doc_coverage: QualityMetricValue } |  |  |  |  |

| `QualityMetricValue` | struct | QualityMetricValue { pub value: f64, pub description: String } |  |  |  |  |

| `QualityViolationEvent` | struct | QualityViolationEvent { pub rule: String, pub metric: String, pub value: f64, pub threshold: f64, pub z_score: f64, pub severity: String, pub description: String } |  |  |  |  |

| `Receipt` | struct | Receipt { pub format_version: String, pub events: Vec<OperationEvent>, pub chain_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `StatsOutput` | struct | StatsOutput { pub event_count: usize, pub chain_depth: usize, pub chain_hash: String, pub event_type_histogram: std::collections::BTreeMap<String, usize>, pub object_type_histogram: std::collections::BTreeMap<String, usize> } |  |  |  |  |

| `Verdict` | struct | Verdict { pub accepted: bool, pub profile: ProfileId, pub outcomes: Vec<CheckOutcome>, pub reason: String } |  |  |  |  |


### src/verbs/anomaly_detect.rs

| `anomaly_detect` | function | anomaly_detect( receipts_path: String, sensitivity: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/assemble.rs

| `assemble` | function | assemble( out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/assemble_and_notarize.rs

| `assemble_and_notarize` | function | assemble_and_notarize( notary_provider: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/assemble_with_signature.rs

| `assemble_with_signature` | function | assemble_with_signature( signing_method: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/assure.rs

| `assure` | function | assure( parent: String, witnesses: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/attest.rs

| `attest` | function | attest( receipt: String, attestation_type: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/audit.rs

| `audit` | function | audit( ) -> Result<()> |  |  |  |  |


### src/verbs/bus_factor.rs

| `bus_factor` | function | bus_factor( receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/catalog.rs

| `catalog` | function | catalog( filter_name: Option<String>, filter_events: Option<usize>, ) -> Result<()> |  |  |  |  |


### src/verbs/causality_chain.rs

| `causality_chain` | function | causality_chain( start_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/conformance.rs

| `conformance` | function | conformance( receipt: String, ) -> Result<()> |  |  |  |  |


### src/verbs/coverage_analysis.rs

| `coverage_analysis` | function | coverage_analysis( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/dependency_matrix.rs

| `dependency_matrix` | function | dependency_matrix( receipts_path: String, output_matrix: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/diagnose.rs

| `diagnose` | function | diagnose( receipt: String, ) -> Result<()> |  |  |  |  |


### src/verbs/diff.rs

| `diff` | function | diff( receipt_a: String, receipt_b: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/doctor.rs

| `doctor` | function | doctor( receipts: Option<String>, fix: bool, ) -> Result<()> |  |  |  |  |


### src/verbs/dora_metrics.rs

| `dora_metrics` | function | dora_metrics( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/ecosystem_certify.rs

| `ecosystem_certify` | function | ecosystem_certify( receipt: String, observation: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/ecosystem_verify.rs

| `ecosystem_verify` | function | ecosystem_verify( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit.rs

| `emit` | function | emit( event_type: String, object: String, payload: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit_batch.rs

| `emit_batch` | function | emit_batch( batch_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit_from_cicd.rs

| `emit_from_cicd` | function | emit_from_cicd( provider: String, job_status: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit_from_cloud.rs

| `emit_from_cloud` | function | emit_from_cloud( provider: String, resource_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit_from_github.rs

| `emit_from_github` | function | emit_from_github( repo: String, event_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit_from_gitlab.rs

| `emit_from_gitlab` | function | emit_from_gitlab( repo: String, event_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit_from_monitoring.rs

| `emit_from_monitoring` | function | emit_from_monitoring( provider: String, alert_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit_from_sbom.rs

| `emit_from_sbom` | function | emit_from_sbom( sbom_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/emit_from_security.rs

| `emit_from_security` | function | emit_from_security( provider: String, vuln_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/envelope_export.rs

| `envelope_export` | function | envelope_export( sealed_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/envelope_list.rs

| `envelope_list` | function | envelope_list( store: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/envelope_sign.rs

| `envelope_sign` | function | envelope_sign( receipt: String, key_file: String, out: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/envelope_verify.rs

| `envelope_verify` | function | envelope_verify( sealed_file: String, store: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/errc_assure.rs

| `errc_assure` | function | errc_assure( parent: String, witnesses: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/errc_certify.rs

| `errc_certify` | function | errc_certify( receipt: String, observation: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/errc_verify.rs

| `errc_verify` | function | errc_verify( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/errc_verify_assurance.rs

| `errc_verify_assurance` | function | errc_verify_assurance( receipt: String, parent: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/evidence_crl_apply.rs

| `evidence_crl_apply` | function | evidence_crl_apply( file: String, store: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/evidence_crl_publish.rs

| `evidence_crl_publish` | function | evidence_crl_publish( kid: String, epoch: u64, store: Option<String>, out: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/evidence_heads.rs

| `evidence_heads` | function | evidence_heads( journal_file: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/evidence_journal.rs

| `evidence_journal` | function | evidence_journal( subject: String, out: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/explain_incident.rs

| `explain_incident` | function | explain_incident( incident_desc: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/find_blast_radius.rs

| `find_blast_radius` | function | find_blast_radius( change_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/fix.rs

| `fix` | function | fix( receipt: String, action: Option<String>, dry_run: Option<bool>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/fix_receipt.rs

| `fix_receipt` | function | fix_receipt( receipt: String, action: Option<String>, dry_run: bool, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/gdpr_proof.rs

| `gdpr_proof` | function | gdpr_proof( receipts_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/graph.rs

| `graph` | function | graph( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/guide_search.rs

| `guide_search` | function | guide_search( keyword: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/hipaa.rs

| `hipaa` | function | hipaa( receipts_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/inspect.rs

| `inspect` | function | inspect( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/install_git_hook.rs

| `install_git_hook` | function | install_git_hook( threshold: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/keys_generate.rs

| `keys_generate` | function | keys_generate( algorithm: String, custodian: String, out: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/keys_import.rs

| `keys_import` | function | keys_import( algorithm: String, public_key_hex: String, custodian: String, out: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/keys_list.rs

| `keys_list` | function | keys_list( store: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/keys_revoke.rs

| `keys_revoke` | function | keys_revoke( kid: String, reason: String, store: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/keys_rotate.rs

| `keys_rotate` | function | keys_rotate( kid: String, store: Option<String>, out: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/license_compliance.rs

| `license_compliance` | function | license_compliance( receipts_path: String, license_policy: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/model.rs

| `model` | function | model( receipt: String, ) -> Result<()> |  |  |  |  |


### src/verbs/monitor.rs

| `monitor` | function | monitor( watch: Option<String>, metrics: Option<String>, rules: Option<String>, baseline_commits: Option<u32>, interval: Option<u64>, output: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/notarize.rs

| `notarize` | function | notarize( receipt: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/orphaned_code.rs

| `orphaned_code` | function | orphaned_code( receipts_path: String, days: Option<u32>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/pci_dss.rs

| `pci_dss` | function | pci_dss( receipts_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/policy_enforce.rs

| `policy_enforce` | function | policy_enforce( receipts_path: String, policy_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/portfolio_health.rs

| `portfolio_health` | function | portfolio_health( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/predict.rs

| `predict` | function | predict( receipts_path: String, prediction_type: String, model: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/profile.rs

| `profile` | function | profile( receipt: Option<String>, duration: Option<u64>, ) -> Result<()> |  |  |  |  |


### src/verbs/query.rs

| `query` | function | query( q: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/receipt_throughput.rs

| `receipt_throughput` | function | receipt_throughput( iterations: Option<u32>, ) -> Result<()> |  |  |  |  |


### src/verbs/replay.rs

| `replay` | function | replay( receipt: String, ) -> Result<()> |  |  |  |  |


### src/verbs/root_cause.rs

| `root_cause` | function | root_cause( effect_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/sbom_attest.rs

| `sbom_attest` | function | sbom_attest( sbom_path: String, receipt: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/sbom_blast_radius.rs

| `sbom_blast_radius` | function | sbom_blast_radius( sbom_path: String, component: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/sbom_compliance.rs

| `sbom_compliance` | function | sbom_compliance( sbom_path: String, framework: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/sbom_emit.rs

| `sbom_emit` | function | sbom_emit( sbom_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/sbom_ntia.rs

| `sbom_ntia` | function | sbom_ntia( sbom_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/sbom_scan.rs

| `sbom_scan` | function | sbom_scan( sbom_path: String, advisories_path: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/search.rs

| `search` | function | search( pattern: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/security_debt.rs

| `security_debt` | function | security_debt( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/show.rs

| `show` | function | show( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/sign.rs

| `sign` | function | sign( receipt: String, key_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/soc2_audit.rs

| `soc2_audit` | function | soc2_audit( receipts_path: String, soc2_type: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/standing_certify.rs

| `standing_certify` | function | standing_certify( receipt: String, observation: String, scope: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/standing_verify.rs

| `standing_verify` | function | standing_verify( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/stats.rs

| `stats` | function | stats( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/team_velocity.rs

| `team_velocity` | function | team_velocity( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/tech_debt.rs

| `tech_debt` | function | tech_debt( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/test.rs

| `test` | function | test( ) -> Result<()> |  |  |  |  |


### src/verbs/timeline.rs

| `timeline` | function | timeline( receipts_path: String, start_time: Option<String>, end_time: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/trend_analysis.rs

| `trend_analysis` | function | trend_analysis( receipts_path: String, metric: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/variance.rs

| `variance` | function | variance( receipt: Option<String>, iterations: Option<u32>, ) -> Result<()> |  |  |  |  |


### src/verbs/verify.rs

| `verify` | function | verify( receipt: String, format: Option<String>, profile: Option<String>, strict: Option<bool>, ) -> Result<()> |  |  |  |  |


### src/verbs/verify_assurance.rs

| `verify_assurance` | function | verify_assurance( receipt: String, parent: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/verify_compliance.rs

| `verify_compliance` | function | verify_compliance( receipt: String, framework: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/verify_family.rs

| `verify_family` | function | verify_family( receipts_dir: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/verify_sla.rs

| `verify_sla` | function | verify_sla( receipt: String, sla_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verbs/visualize.rs

| `visualize` | function | visualize( format: String, receipt: String, ) -> Result<()> |  |  |  |  |


### src/verbs/why.rs

| `why` | function | why( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |


### src/verifier.rs

| `verify` | function | verify(receipt: &Receipt) -> Verdict |  |  |  |  |


### src/visualize.rs

| `build_graph` | function | build_graph(receipt: &Receipt) -> ReceiptGraph |  |  |  |  |

| `to_dot` | function | to_dot(graph: &ReceiptGraph) -> String |  |  |  |  |

| `to_json` | function | to_json(graph: &ReceiptGraph) -> anyhow::Result<String> |  |  |  |  |

| `GraphEdge` | struct | GraphEdge { pub from: String, pub to: String, pub weight: usize } |  |  |  |  |

| `GraphNode` | struct | GraphNode { pub id: String, pub label: String, pub event_count: usize } |  |  |  |  |

| `ReceiptGraph` | struct | ReceiptGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> } |  |  |  |  |


### src/wasm_court.rs

| `WasmCourtError` | enum | WasmCourtError { Compile(String), Instantiation(String), Export(String), Trap { message: String, }, OutOfFuel { budget: u64, } } |  |  |  |  |

| `new` | function | new() -> Result<Self, WasmCourtError> |  |  |  |  |

| `run_bounded` | function | run_bounded( &self, wasm_bytes: &[u8], export: &str, fuel_budget: u64, ) -> Result<u64, WasmCourtError> |  |  |  |  |

| `WasmCourt` | struct | WasmCourt { engine: Engine } |  |  |  |  |


### src/zero_copy.rs

| `ArchiveError` | enum | ArchiveError { Invalid(String), RangeCheck } |  |  |  |  |

| `freeze` | function | freeze(archive: &ReceiptArchive) -> Result<Vec<u8>, ArchiveError> |  |  |  |  |

| `thaw` | function | thaw(bytes: &[u8]) -> Result<&Archived<ReceiptArchive>, ArchiveError> |  |  |  |  |

| `unfreeze` | function | unfreeze(bytes: &[u8]) -> Result<ReceiptArchive, ArchiveError> |  |  |  |  |

| `validate_archived_range` | function | validate_archived_range( bytes: &[u8], min_seq: u64, max_seq: u64, ) -> Result<(), ArchiveError> |  |  |  |  |

| `ReceiptArchive` | struct | ReceiptArchive { pub chain_id_hex: String, pub entries: Vec<SeqEntry> } |  |  |  |  |

| `SeqEntry` | struct | SeqEntry { pub seq: u64, pub commitment_hex: String } |  |  |  |  |


### src/zk_range.rs

| `RangeProofError` | enum | RangeProofError { Generation(String), MalformedEncoding, VerificationFailed } |  |  |  |  |

| `prove_range` | function | prove_range( value: u64, bits: usize, label: &'static [u8], rng: &mut R, ) -> Result<RangeWitness, RangeProofError> |  |  |  |  |

| `verify_range` | function | verify_range( commitment: &[u8; 32], proof_bytes: &[u8], bits: usize, label: &'static [u8], ) -> Result<(), RangeProofError> |  |  |  |  |

| `affidavit:zk-range:v1:effect-budget` | str_key | LABEL = "affidavit:zk-range:v1:effect-budget" |  |  |  |  |

| `RangeWitness` | struct | RangeWitness { pub commitment: [u8; 32], pub blinding: [u8; 32], pub proof: Vec<u8>, pub bits: usize } |  |  |  |  |


### stubs/clnrm-core/src/lib.rs

| `generate_digest` | function | generate_digest(data: &[u8]) -> Digest |  |  |  |  |

| `verify_digest` | function | verify_digest(data: &[u8], expected: &Digest) -> bool |  |  |  |  |

| `Digest` | struct | Digest { [u8; 32] } |  |  |  |  |


### stubs/clnrm-core/src/lib.rs

| `generate_digest` | function | generate_digest(data: &[u8]) -> Digest |  |  |  |  |

| `verify_digest` | function | verify_digest(data: &[u8], expected: &Digest) -> bool |  |  |  |  |

| `Digest` | struct | Digest { [u8; 32] } |  |  |  |  |


### stubs/wasm4pm-compat/src/lib.rs

| `MAX_POWL_DEPTH` | const | MAX_POWL_DEPTH: usize |  |  |  |  |

| `ArcDirection` | enum | ArcDirection { PlaceToTransition, TransitionToPlace } |  |  |  |  |

| `ArcDirectionConst` | enum | ArcDirectionConst { PlaceToTransition, TransitionToPlace } |  |  |  |  |

| `BpmnEvent` | enum | BpmnEvent { Start, Intermediate, End, Boundary } |  |  |  |  |

| `BpmnGateway` | enum | BpmnGateway { Exclusive, Parallel, Inclusive, EventBased, Complex } |  |  |  |  |

| `BpmnNodeKind` | enum | BpmnNodeKind { Event(BpmnEvent), Task(BpmnTask), Gateway(BpmnGateway) } |  |  |  |  |

| `BpmnRefusal` | enum | BpmnRefusal { DanglingEdge, MissingStart, MissingStartEvent, MissingEndEvent, EmptyProcess, DuplicateNodeId, LaneNodeNotDeclared } |  |  |  |  |

| `CausalConsistency` | enum | CausalConsistency { Unknown, Consistent, HasCycles, HasContradictions } |  |  |  |  |

| `CausalNetRefusal` | enum | CausalNetRefusal { MissingActivity, InvalidDependencyScore, DisconnectedGraph } |  |  |  |  |

| `CompatDiagnostic` | enum | CompatDiagnostic { MissingWitness, MissingRoundTripFixture, RawEvidenceExportedAsAdmitted, LossyProjectionWithoutPolicy, HiddenFlattening, MissingRefusalPath, MissingReceiptShape, UnreachablePrimitive, MigrationRecommended } |  |  |  |  |

| `ComplianceKind` | enum | ComplianceKind { Monitoring, Audit, Certification } |  |  |  |  |

| `ConformanceRefusal` | enum | ConformanceRefusal { EmptyModel, TooManyDeviations, InvalidProfile, MissingLog, MissingModel, MissingDeviationPath, FitnessUnavailable, PrecisionUnavailable, F1Unavailable, GeneralizationUnavailable, SimplicityUnavailable } |  |  |  |  |

| `ConformanceVerdict` | enum | ConformanceVerdict { PerfectAlignment, FitnessDeficit, DeadlockEncountered } |  |  |  |  |

| `CorrelationSchema` | enum | CorrelationSchema { ByCase, ByObject, ByTimestamp, ByAttribute } |  |  |  |  |

| `CubeDimensionKind` | enum | CubeDimensionKind { Activity, Resource, Time, DataAttribute, ObjectType, CaseAttribute } |  |  |  |  |

| `DeclareRefusal` | enum | DeclareRefusal { EmptyActivity, MissingActivation, BinaryRequiresTarget, UnsupportedTemplate, MissingTarget, InvalidTemplateArity, EmptyObjectScope, SynchronizationViolation } |  |  |  |  |

| `DeclareScope` | enum | DeclareScope { SingleObjectScope(String), MultiObjectScope(Vec<String>), SynchronizedObjectScope(Vec<String>), CrossObjectScope(String, String), GlobalScope } |  |  |  |  |

| `DeclareTemplate` | enum | DeclareTemplate { Existence, Absence, Init, Existence2, Existence3, Absence2, Absence3, RespondedExistence, CoExistence, Response, Precedence, Succession, AlternateResponse, AlternatePrecedence, AlternateSuccession, ChainResponse, ChainPrecedence, ChainSuccession, NotSuccession, NotChainSuccession, NotCoExistence, ExclusiveChoice } |  |  |  |  |

| `DfgRefusal` | enum | DfgRefusal { EmptyGraph, DanglingEdge, IsolatedNode } |  |  |  |  |

| `DiagnosticKind` | enum | DiagnosticKind { Warning, Error, Info } |  |  |  |  |

| `DiagnosticSeverity` | enum | DiagnosticSeverity { Error, Warning, Info } |  |  |  |  |

| `EventLogRefusal` | enum | EventLogRefusal { EmptyLog, EmptyTrace, MissingActivity, NonMonotonicTrace } |  |  |  |  |

| `EventPredicateKind` | enum | EventPredicateKind { ActivityEquals, AttributeEquals, TimestampInRange } |  |  |  |  |

| `EvidenceMode` | enum | EvidenceMode { Raw, Parsed, Admitted, Refused, Projected, Exportable, Witnessed, Receipted } |  |  |  |  |

| `FilterShape` | enum | FilterShape { Activity, Timeframe, Variant, Attribute, ObjectType } |  |  |  |  |

| `FormatKind` | enum | FormatKind { OcelJson, OcelXml, OcelSqlite, XesXml, BpmnXml, PetriPnml, PowlJson } |  |  |  |  |

| `InstanceCreationKind` | enum | InstanceCreationKind { Static, Dynamic } |  |  |  |  |

| `InteropRefusal` | enum | InteropRefusal { UnsupportedShape, MissingGrounding, SchemaConflict, UngroundedArtifact, FlatClaimOverObjectCentric, DimensionShapeMismatch } |  |  |  |  |

| `KernelRefusal` | enum | KernelRefusal { ZeroSize, TooDense } |  |  |  |  |

| `LifecycleRefusal` | enum | LifecycleRefusal { EmptyLifecycle, InvalidTransition, OrphanEvent } |  |  |  |  |

| `LossFunction` | enum | LossFunction { MeanSquared, CrossEntropy, Hinge } |  |  |  |  |

| `LossPolicy` | enum | LossPolicy { RefuseLoss, AllowLossWithReport, AllowLossSilent, AllowNamedProjection } |  |  |  |  |

| `LossRefusal` | enum | LossRefusal { InvalidParameters, NumericalInstability } |  |  |  |  |

| `OCELAttributeValue` | enum | OCELAttributeValue { Integer(i64), Float(f64), Boolean(bool), String(std::string::String), Time(std::string::String), Null } |  |  |  |  |

| `ObjectCentricity` | enum | ObjectCentricity { CaseCentric, ObjectCentric, Mixed } |  |  |  |  |

| `ObjectLifecyclePhase` | enum | ObjectLifecyclePhase { Created, Active, Modified, Archived, Deleted } |  |  |  |  |

| `ObjectPredicateKind` | enum | ObjectPredicateKind { AttributeEquals, TypeEquals } |  |  |  |  |

| `ObjectTypeCardinality` | enum | ObjectTypeCardinality { One, ZeroOrOne, OneOrMany, ZeroOrMany } |  |  |  |  |

| `OcDeclareRefusal` | enum | OcDeclareRefusal { EmptyObjectTypeList, SynchronizationRequiresMultipleTypes, ScopeMismatch } |  |  |  |  |

| `OcelAttributeValue` | enum | OcelAttributeValue { Integer(i64), Float(f64), Boolean(bool), String(std::string::String), TimestampNs(u64), List(Vec<OcelAttributeValue>), Map(Vec<(std::string::String, OcelAttributeValue)>), Null } |  |  |  |  |

| `OcelRefusal` | enum | OcelRefusal { EmptyEventObjectLinks, DanglingEventObjectLink } |  |  |  |  |

| `OcpqRefusal` | enum | OcpqRefusal { EmptyFilter, UnsupportedScope } |  |  |  |  |

| `OcpqScopeKind` | enum | OcpqScopeKind { Open, Closed, SingleType } |  |  |  |  |

| `PerspectiveRefusal` | enum | PerspectiveRefusal { MissingPerspective, ConflictingPerspectives } |  |  |  |  |

| `PetriNetRefusal` | enum | PetriNetRefusal { IsolatedPlace, InvalidStructure, EmptyNet } |  |  |  |  |

| `PetriRefusal` | enum | PetriRefusal { IsolatedPlace, IsolatedTransition, MissingInitialMarking, InvalidWeight, MissingFinalMarking, UnsafeNet, InvalidInstanceBounds, ObjectTypeNotPreserved, DeadTransition } |  |  |  |  |

| `Pm4pyShape` | enum | Pm4pyShape { EventLog, ObjectCentricLog, PetriNet, ProcessTree, Bpmn, DirectlyFollowsGraph, Declare } |  |  |  |  |

| `Powl8Op` | enum | Powl8Op { NoOp = 0, Sequence = 1, Choice = 2, Parallel = 3, PartialOrder = 4, Loop = 5, Silent = 6, Or = 7, ChoiceGraph = 8 } |  |  |  |  |

| `Powl8OpError` | enum | Powl8OpError { InvalidDiscriminant } |  |  |  |  |

| `PowlNodeKind` | enum | PowlNodeKind { Atom(String), Silent, PartialOrder(Vec<PowlNodeId>), Choice(Vec<PowlNodeId>), Loop { body: PowlNodeId, redo: Option<PowlNodeId> }, ChoiceGraph { nodes: Vec<PowlNodeId>, edges: Vec<(usize, usize)> } } |  |  |  |  |

| `PowlProjectionState` | enum | PowlProjectionState { Unknown, ProcessTreeProjectable, ExceedsProcessTree, RefusedProjection } |  |  |  |  |

| `PowlRefusal` | enum | PowlRefusal { CyclicPartialOrder, InvalidLoop, InvalidChoiceArity { declared: usize, required_min: usize }, ChoiceGraphDisconnected } |  |  |  |  |

| `PredicateKind` | enum | PredicateKind { Event(String), Object(String), Relation(String), Temporal(String), Cardinality { min: usize, max: usize }, Nested, ChildSetBound { branch_label: String, min: usize, max: usize }, E2ORelation { event_var: String, object_var: String, qualifier: Option<String> }, O2ORelation { source_var: String, target_var: String, qualifier: Option<String> }, TimeBetweenEvents { from_var: String, to_var: String } } |  |  |  |  |

| `PredictionHorizon` | enum | PredictionHorizon { FullCase, Events(usize), TimeUnits(u64) } |  |  |  |  |

| `PredictionRefusal` | enum | PredictionRefusal { InsufficientData, InvalidModel, ConvergenceFailure, MissingPrefix, MissingTarget, EmptyPrefix, TargetUnsupported, NonPrefixTrace, ConstraintNotNamed } |  |  |  |  |

| `PredictionTarget` | enum | PredictionTarget { NextActivity, OutcomeLabel, RemainingTime, DriftSignal, Risk, ComplianceConstraint } |  |  |  |  |

| `ProcessPerspective` | enum | ProcessPerspective { ControlFlow, Data, Resource, Time } |  |  |  |  |

| `ProcessShapeKind` | enum | ProcessShapeKind { Event, Trace, EventLog, EventStream, XesLog, OcelLog, DirectlyFollowsGraph, ObjectCentricDfg, PetriNet, WorkflowNet, ObjectCentricPetriNet, ProcessTree, Powl, DeclareModel, ObjectCentricDeclareModel, LogSkeleton, OcpqQuery, Alignment, TokenReplay, ConformanceVerdict, PredictionProblem, Receipt } |  |  |  |  |

| `ProcessTreeNode` | enum | ProcessTreeNode { Activity(String), Operator { operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId> } } |  |  |  |  |

| `ProcessTreeOperator` | enum | ProcessTreeOperator { Sequence, Xor, Parallel, Or, Loop, Silent } |  |  |  |  |

| `ProcessTreeRefusal` | enum | ProcessTreeRefusal { EmptyTree, OrphanNode, InvalidLoopArity, UnsupportedOperator, MissingRoot, DanglingNodeReference, TauLeafWithChildren, BelowMinimumArity, InvalidArity, CycleDetected } |  |  |  |  |

| `QualityDimension` | enum | QualityDimension { Fitness, Precision, F1, Generalization, Simplicity } |  |  |  |  |

| `QualityMetricKind` | enum | QualityMetricKind { Fitness, Precision, F1, Generalization, Simplicity } |  |  |  |  |

| `ReceiptRefusal` | enum | ReceiptRefusal { EmptyChain, HashMismatch, InvalidDigest, MissingSubject, MissingWitness, MissingDigest, MissingReplayHint, BrokenChainLink(usize) } |  |  |  |  |

| `ReceiptVerdict` | enum | ReceiptVerdict { Admitted, Refused(ReceiptRefusal) } |  |  |  |  |

| `RelationLaw` | enum | RelationLaw { EventToObject, ObjectToObject, ObjectToEvent } |  |  |  |  |

| `RelationPredicateKind` | enum | RelationPredicateKind { E2O, O2O, TimeBetweenEvents } |  |  |  |  |

| `ReplayHintKind` | enum | ReplayHintKind { FromStart, FromSeq(u64), Latest } |  |  |  |  |

| `SoundnessState` | enum | SoundnessState { Unknown, Claimed, Witnessed } |  |  |  |  |

| `StandaloneChoiceGraphNode` | enum | StandaloneChoiceGraphNode { Start, End, Activity(String), SubModel(usize) } |  |  |  |  |

| `SummaryShape` | enum | SummaryShape { Counts, TraceVariants, ActivityDistribution, TimingProfile, ObjectTypeDistribution } |  |  |  |  |

| `TemporalOrder` | enum | TemporalOrder { Before, After, Concurrent, Unknown } |  |  |  |  |

| `TemporalRefusal` | enum | TemporalRefusal { ZeroBound, ConflictingConstraints } |  |  |  |  |

| `TemporalRelation` | enum | TemporalRelation { Before, After, During, Concurrent } |  |  |  |  |

| `WitnessFamily` | enum | WitnessFamily { Standard, ApiGrammar, Implementation, Paper } |  |  |  |  |

| `WorkflowPattern` | enum | WorkflowPattern { Sequence, ParallelSplit, Synchronization, ExclusiveChoice, SimpleMerge, MultiChoice, StructuredSynchronizingMerge, MultiMerge, StructuredDiscriminator, ArbitraryCycles, ImplicitTermination, MultipleInstancesWithoutSync, MultipleInstancesWithDesignTimeKnowledge, DeferredChoice, InterleavedParallelRouting, CancelActivity, CancelCase } |  |  |  |  |

| `activity` | function | activity(&self) -> &str |  |  |  |  |

| `add_arc` | function | add_arc(&mut self, a: Arc) |  |  |  |  |

| `add_place` | function | add_place(&mut self, p: Place) |  |  |  |  |

| `add_transition` | function | add_transition(&mut self, t: Transition) |  |  |  |  |

| `admit_flat` | function | admit_flat(&self) -> Result<(), InteropRefusal> |  |  |  |  |

| `admit_shape` | function | admit_shape(&self) -> Result<(), ProcessTreeRefusal> |  |  |  |  |

| `admits` | function | admits(&self, count: usize) -> bool |  |  |  |  |

| `arcs` | function | arcs(&self) -> &[Arc] |  |  |  |  |

| `arity` | function | arity(&self) -> u8 |  |  |  |  |

| `as_f64` | function | as_f64(&self) -> f64 |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `assert_epsilon_close` | function | assert_epsilon_close(actual: f64, expected: f64) |  |  |  |  |

| `at_ns` | function | at_ns(mut self, ts: u64) -> Self |  |  |  |  |

| `attribute` | function | attribute(&self) -> &str |  |  |  |  |

| `attributes` | function | attributes(&self) -> &[OcelAttribute] |  |  |  |  |

| `binary` | function | binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) -> Self |  |  |  |  |

| `boolean` | function | boolean(key: &str, value: bool) -> Self |  |  |  |  |

| `by` | function | by(mut self, resource: &str) -> Self |  |  |  |  |

| `case_id` | function | case_id(&self) -> &str |  |  |  |  |

| `category` | function | category(&self) -> &str |  |  |  |  |

| `check_filter_shape` | function | check_filter_shape(shape: Pm4pyShape, filter: FilterShape) -> Result<(), InteropRefusal> |  |  |  |  |

| `claim_sound` | function | claim_sound(self) -> WfNet<SoundnessClaimed> |  |  |  |  |

| `conformance_rate` | function | conformance_rate(&self) -> f64 |  |  |  |  |

| `count` | function | count(&self) -> usize |  |  |  |  |

| `count_objects_of_type` | function | count_objects_of_type(&self, t: &str) -> usize |  |  |  |  |

| `declare_separable` | function | declare_separable(net: WfNetConst<S>) -> Self |  |  |  |  |

| `default` | function | default() -> Self |  |  |  |  |

| `den` | function | den(&self) -> u64 |  |  |  |  |

| `direction` | function | direction(&self) -> ArcDirection |  |  |  |  |

| `duration_ns` | function | duration_ns(&self) -> Option<u64> |  |  |  |  |

| `e2o` | function | e2o(&self, event_id: &str) -> Vec<(&str, &str)> |  |  |  |  |

| `edges` | function | edges(&self) -> &[BpmnEdge] |  |  |  |  |

| `empty` | function | empty() -> Self |  |  |  |  |

| `eval` | function | eval(&self, event_id: &str) -> Option<HashMap<std::string::String, OCELAttributeValue>> |  |  |  |  |

| `event` | function | event(id: &str, e: BpmnEvent) -> Self |  |  |  |  |

| `event_count` | function | event_count(&self) -> usize |  |  |  |  |

| `event_id` | function | event_id(&self) -> &str |  |  |  |  |

| `event_object_links` | function | event_object_links(&self) -> &[EventObjectLink] |  |  |  |  |

| `event_set` | function | event_set(&self) -> &Vec<OCELEvent> |  |  |  |  |

| `events` | function | events(&self) -> &[OcelEvent] |  |  |  |  |

| `expression` | function | expression(&self) -> &str |  |  |  |  |

| `extend_with` | function | extend_with(&mut self, env: ReceiptEnvelope) -> Result<(), ReceiptRefusal> |  |  |  |  |

| `final_marking` | function | final_marking(&self) -> Option<&Marking> |  |  |  |  |

| `float` | function | float(key: &str, value: f64) -> Self |  |  |  |  |

| `fnv1a_64` | function | fnv1a_64(bytes: &[u8]) -> u64 |  |  |  |  |

| `frequency` | function | frequency(&self) -> DfgWeight |  |  |  |  |

| `from_events` | function | from_events(events: impl IntoIterator<Item = Event>) -> Self |  |  |  |  |

| `from_owned` | function | from_owned(s: String) -> Self |  |  |  |  |

| `from_static` | function | from_static(s: &'static str) -> Self |  |  |  |  |

| `from_traces` | function | from_traces(traces: impl IntoIterator<Item = Trace>) -> Self |  |  |  |  |

| `gateway` | function | gateway(id: &str, g: BpmnGateway) -> Self |  |  |  |  |

| `get` | function | get(&self) -> f64 |  |  |  |  |

| `get_by_hash` | function | get_by_hash(&self, hash: u64) -> Option<&V> |  |  |  |  |

| `get_by_key` | function | get_by_key(&self, key: &str) -> Option<&V> |  |  |  |  |

| `id` | function | id(&self) -> &str |  |  |  |  |

| `initial_marking` | function | initial_marking(&self) -> &Marking |  |  |  |  |

| `inner` | function | inner(&self) -> &T |  |  |  |  |

| `insert` | function | insert(&mut self, hash: u64, key: String, value: V) |  |  |  |  |

| `integer` | function | integer(key: &str, value: i64) -> Self |  |  |  |  |

| `internal` | function | internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) -> Self |  |  |  |  |

| `into_admitted` | function | into_admitted(self) -> Evidence<T, state::Admitted, C> |  |  |  |  |

| `into_evidence` | function | into_evidence(self) -> Evidence<T, state::Admitted, W> |  |  |  |  |

| `into_exportable` | function | into_exportable(self) -> Evidence<T, state::Exportable, C> |  |  |  |  |

| `into_inner` | function | into_inner(self) -> T |  |  |  |  |

| `into_lost` | function | into_lost(self) -> Dropped |  |  |  |  |

| `into_parsed` | function | into_parsed(self) -> Evidence<T, state::Parsed, C> |  |  |  |  |

| `into_projected` | function | into_projected(self) -> Evidence<T, state::Projected, C> |  |  |  |  |

| `into_reason` | function | into_reason(self) -> R |  |  |  |  |

| `into_receipted` | function | into_receipted(self) -> Evidence<T, state::Receipted, C> |  |  |  |  |

| `is_chain` | function | is_chain(&self) -> bool |  |  |  |  |

| `is_consistent` | function | is_consistent(&self) -> bool |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_grounded` | function | is_grounded(&self) -> bool |  |  |  |  |

| `is_lossless` | function | is_lossless(&self) -> bool |  |  |  |  |

| `is_negative` | function | is_negative(&self) -> bool |  |  |  |  |

| `is_object_centric` | function | is_object_centric(&self) -> bool |  |  |  |  |

| `is_perfect` | function | is_perfect(&self) -> bool |  |  |  |  |

| `is_silent` | function | is_silent(&self) -> bool |  |  |  |  |

| `is_synchronized` | function | is_synchronized(&self) -> bool |  |  |  |  |

| `is_well_shaped` | function | is_well_shaped(&self) -> bool |  |  |  |  |

| `iter` | function | iter(&self) -> impl Iterator<Item = &ReceiptEnvelope> |  |  |  |  |

| `kind` | function | kind(&self) -> QualityMetricKind |  |  |  |  |

| `label` | function | label(&self) -> &str |  |  |  |  |

| `lanes` | function | lanes(&self) -> &[BpmnLane] |  |  |  |  |

| `leaf` | function | leaf(id: ProcessTreeNodeId, label: &str) -> Self |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `length` | function | length(&self) -> usize |  |  |  |  |

| `lifecycle` | function | lifecycle(&self) -> Option<&str> |  |  |  |  |

| `mark` | function | mark(&mut self, place_id: &str, tokens: usize) |  |  |  |  |

| `max` | function | max(&self) -> usize |  |  |  |  |

| `members` | function | members(&self) -> &[String] |  |  |  |  |

| `min` | function | min(&self) -> usize |  |  |  |  |

| `name` | function | name(&self) -> &str |  |  |  |  |

| `net` | function | net(&self) -> &PetriNet |  |  |  |  |

| `new` | function | new(value: T) -> Self |  |  |  |  |

| `node_count` | function | node_count(&self) -> usize |  |  |  |  |

| `node_ids` | function | node_ids(&self) -> &[String] |  |  |  |  |

| `nodes` | function | nodes(&self) -> &[BpmnNode] |  |  |  |  |

| `num` | function | num(&self) -> u64 |  |  |  |  |

| `o2o` | function | o2o(&self, object_id: &str) -> Vec<(&str, &str)> |  |  |  |  |

| `object_changes` | function | object_changes(&self) -> &[ObjectChange] |  |  |  |  |

| `object_id` | function | object_id(&self) -> &str |  |  |  |  |

| `object_object_links` | function | object_object_links(&self) -> &[ObjectObjectLink] |  |  |  |  |

| `object_set` | function | object_set(&self) -> &Vec<OCELObject> |  |  |  |  |

| `object_type` | function | object_type(&self) -> &str |  |  |  |  |

| `object_types` | function | object_types(&self) -> impl Iterator<Item = &str> |  |  |  |  |

| `objects` | function | objects(&self) -> &[Object] |  |  |  |  |

| `place_id` | function | place_id(&self) -> &str |  |  |  |  |

| `place_to_transition` | function | place_to_transition(place: &str, transition: &str) -> Self |  |  |  |  |

| `places` | function | places(&self) -> &[Place] |  |  |  |  |

| `predecessors` | function | predecessors(&self, idx: usize) -> Vec<usize> |  |  |  |  |

| `process` | function | process(&self) -> &BpmnProcess |  |  |  |  |

| `projection` | function | projection(&self) -> ProjectionNameOwned |  |  |  |  |

| `push` | function | push(&mut self, event: Event) |  |  |  |  |

| `qualified` | function | qualified(mut self, q: &str) -> Self |  |  |  |  |

| `qualifier` | function | qualifier(&self) -> Option<&str> |  |  |  |  |

| `raw` | function | raw(value: T) -> Self |  |  |  |  |

| `resource` | function | resource(&self) -> Option<&str> |  |  |  |  |

| `root` | function | root(&self) -> Option<ProcessTreeNodeId> |  |  |  |  |

| `schema` | function | schema(&self) -> &'static str |  |  |  |  |

| `scope` | function | scope(&self) -> &ObjectScopeConst |  |  |  |  |

| `silent` | function | silent(id: &str) -> Self |  |  |  |  |

| `soundness_state` | function | soundness_state(&self) -> SoundnessState |  |  |  |  |

| `source` | function | source(&self) -> &str |  |  |  |  |

| `source_id` | function | source_id(&self) -> &str |  |  |  |  |

| `steps` | function | steps(&self) -> &[NamedLoss] |  |  |  |  |

| `string` | function | string(key: &str, value: &str) -> Self |  |  |  |  |

| `successors` | function | successors(&self, idx: usize) -> Vec<usize> |  |  |  |  |

| `summary` | function | summary(&self, category: &str) -> NamedLoss |  |  |  |  |

| `synchronized` | function | synchronized(constraint: DeclareConstraint, object_types: Vec<String>) -> Self |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `target` | function | target(&self) -> &str |  |  |  |  |

| `target_id` | function | target_id(&self) -> &str |  |  |  |  |

| `task` | function | task(id: &str, t: BpmnTask) -> Self |  |  |  |  |

| `timestamp_ns` | function | timestamp_ns(&self) -> Option<u64> |  |  |  |  |

| `tip` | function | tip(&self) -> &ReceiptEnvelope |  |  |  |  |

| `tokens` | function | tokens(&self) -> &HashMap<String, usize> |  |  |  |  |

| `tokens_on` | function | tokens_on(&self, place_id: &str) -> usize |  |  |  |  |

| `trace_count` | function | trace_count(&self) -> usize |  |  |  |  |

| `traces` | function | traces(&self) -> &[Trace] |  |  |  |  |

| `transition_id` | function | transition_id(&self) -> &str |  |  |  |  |

| `transition_to_place` | function | transition_to_place(transition: &str, place: &str) -> Self |  |  |  |  |

| `transitions` | function | transitions(&self) -> &[Transition] |  |  |  |  |

| `try_from_parts` | function | try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) -> Result<Self, ReceiptRefusal> |  |  |  |  |

| `try_new` | function | try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) -> Result<Self, ReceiptRefusal> |  |  |  |  |

| `unary` | function | unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) -> Self |  |  |  |  |

| `validate` | function | validate(&self) -> Result<(), OcelRefusal> |  |  |  |  |

| `value` | function | value(&self) -> &str |  |  |  |  |

| `verdict` | function | verdict(&self) -> CausalConsistency |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), ReceiptRefusal> |  |  |  |  |

| `weight` | function | weight(&self) -> W |  |  |  |  |

| `with_attribute` | function | with_attribute(mut self, a: OcelAttribute) -> Self |  |  |  |  |

| `with_duration_ns` | function | with_duration_ns(mut self, d: u64) -> Self |  |  |  |  |

| `with_generalization` | function | with_generalization(mut self, v: f64) -> Self |  |  |  |  |

| `with_lifecycle` | function | with_lifecycle(mut self, lc: &str) -> Self |  |  |  |  |

| `with_precision` | function | with_precision(mut self, v: f64) -> Self |  |  |  |  |

| `with_simplicity` | function | with_simplicity(mut self, v: f64) -> Self |  |  |  |  |

| `with_type_dfg` | function | with_type_dfg(mut self, obj_type: &str, dfg: Dfg) -> Self |  |  |  |  |

| `witness_soundness` | function | witness_soundness(self, _proof: SoundnessProof) -> WfNetConst<{ SoundnessState::Witnessed }> |  |  |  |  |

| `Activity` | struct | Activity { pub String } |  |  |  |  |

| `Admission` | struct | Admission { pub value: T, _witness: PhantomData<W> } |  |  |  |  |

| `Admitted` | struct | Admitted |  |  |  |  |

| `Arc` | struct | Arc { pub source: String, pub target: String, pub weight: u32, pub object_type: Option<(String, bool)>, _dir: ArcDirection } |  |  |  |  |

| `ArtifactGrounding` | struct | ArtifactGrounding { pub shape: Pm4pyShape, pub evidence_ref: String, _evidence: PhantomData<E> } |  |  |  |  |

| `Between01` | struct | Between01 { _priv: () } |  |  |  |  |

| `BipartiteArcConst` | struct | BipartiteArcConst { _place_id: String, _transition_id: String, _weight: W } |  |  |  |  |

| `BpmnEdge` | struct | BpmnEdge { source: String, target: String } |  |  |  |  |

| `BpmnLane` | struct | BpmnLane { id: String, name: String, node_ids: Vec<String> } |  |  |  |  |

| `BpmnNode` | struct | BpmnNode { id: String, kind: BpmnNodeKind } |  |  |  |  |

| `BpmnPool` | struct | BpmnPool { id: String, name: String, process: BpmnProcess, lanes: Vec<BpmnLane> } |  |  |  |  |

| `BpmnProcess` | struct | BpmnProcess { nodes: Vec<BpmnNode>, edges: Vec<BpmnEdge> } |  |  |  |  |

| `BpmnTask` | struct | BpmnTask { pub label: String } |  |  |  |  |

| `CancellationRegion` | struct | CancellationRegion { _members: Vec<String> } |  |  |  |  |

| `CausalBinding` | struct | CausalBinding { pub source_tasks: Vec<String>, pub target_tasks: Vec<String> } |  |  |  |  |

| `CausalChain` | struct |  |  |  |  |  |

| `CausalLink` | struct | CausalLink { PhantomData<(From, To)> } |  |  |  |  |

| `CausalNet` | struct | CausalNet { pub nodes: Vec<String>, pub dependency_measures: Vec<(String, String, f64)>, pub inputs: Vec<CausalBinding>, pub outputs: Vec<CausalBinding> } |  |  |  |  |

| `CausallyOrderedEvidence` | struct | CausallyOrderedEvidence { pub inner: T } |  |  |  |  |

| `ChoiceGraph` | struct | ChoiceGraph { nodes: Vec<StandaloneChoiceGraphNode>, edges: Vec<(usize, usize)> } |  |  |  |  |

| `ConditionCell` | struct | ConditionCell { _priv: () } |  |  |  |  |

| `ConformanceResult` | struct | ConformanceResult { pub fitness: f64, pub precision: Option<f64>, pub generalization: Option<f64>, pub simplicity: Option<f64>, pub total_traces: usize, pub fitting_traces: usize, pub deviating_traces: usize } |  |  |  |  |

| `ConformanceVerdict` | struct | ConformanceVerdict { pub fitness: Option<Fitness>, pub deviations: Vec<Deviation> } |  |  |  |  |

| `ConsistencyVerified` | struct | ConsistencyVerified { pub inner: T, verdict: CausalConsistency } |  |  |  |  |

| `ControlFlowPerspective` | struct |  |  |  |  |  |

| `CorrelatedLog` | struct | CorrelatedLog { PhantomData<(A, B)> } |  |  |  |  |

| `CorrelationKey` | struct |  |  |  |  |  |

| `DataPerspective` | struct |  |  |  |  |  |

| `DeclareConstraint` | struct | DeclareConstraint { pub template: DeclareTemplate, pub activation: Activity, pub target: Option<Activity>, pub scope: DeclareScope } |  |  |  |  |

| `DenseKernel` | struct | DenseKernel { pub size: usize } |  |  |  |  |

| `DependencyMeasure` | struct | DependencyMeasure { pub f64 } |  |  |  |  |

| `Deviation` | struct | Deviation { pub position: usize, pub label: String } |  |  |  |  |

| `Dfg` | struct | Dfg { nodes: Vec<DfgNode>, edges: Vec<DfgEdge> } |  |  |  |  |

| `DfgEdge` | struct | DfgEdge { source: String, target: String, count: u64 } |  |  |  |  |

| `DfgEdgeFull` | struct | DfgEdgeFull { pub source: String, pub target: String, _freq: u64, _dur: Option<u64> } |  |  |  |  |

| `DfgNode` | struct | DfgNode { pub String } |  |  |  |  |

| `DfgWeight` | struct | DfgWeight { pub u64 } |  |  |  |  |

| `DiagnosticReport` | struct | DiagnosticReport { pub kind: DiagnosticKind, pub message: String, pub stage: String } |  |  |  |  |

| `Digest` | struct | Digest { pub String } |  |  |  |  |

| `Event` | struct | Event { pub activity: String, pub attributes: HashMap<String, String>, _ts: Option<u64>, _resource: Option<String>, _lifecycle: Option<String> } |  |  |  |  |

| `EventLog` | struct | EventLog { pub traces: Vec<Trace> } |  |  |  |  |

| `EventObjectLink` | struct | EventObjectLink { pub event_id: String, pub object_id: String, _qualifier: Option<String> } |  |  |  |  |

| `EventStream` | struct | EventStream { events: Vec<Event> } |  |  |  |  |

| `EventTypeName` | struct | EventTypeName { inner: Cow<'static, str>, _kind: PhantomData<K> } |  |  |  |  |

| `Evidence` | struct | Evidence { pub value: T, _state: PhantomData<S>, _chain: PhantomData<C> } |  |  |  |  |

| `Exportable` | struct | Exportable |  |  |  |  |

| `F1` | struct | F1 { pub f64 } |  |  |  |  |

| `Fitness` | struct | Fitness { pub f64 } |  |  |  |  |

| `Generalization` | struct | Generalization { pub f64 } |  |  |  |  |

| `InitialFinalMarkingPair` | struct | InitialFinalMarkingPair { _initial: Marking, _final: Marking } |  |  |  |  |

| `InputBinding` | struct | InputBinding { pub &'a str, pub &'a str } |  |  |  |  |

| `LifecycleEvent` | struct | LifecycleEvent { pub event_type: String, pub object_id: String } |  |  |  |  |

| `LossChain` | struct | LossChain { steps: Vec<NamedLoss> } |  |  |  |  |

| `LossReport` | struct | LossReport { pub projection: ProjectionName, pub policy: LossPolicy, _dropped: Dropped, _ph: PhantomData<(A, B)> } |  |  |  |  |

| `Marking` | struct | Marking { pub HashMap<String, usize> } |  |  |  |  |

| `Metric` | struct | Metric { _priv: () } |  |  |  |  |

| `MultiPerspectiveEvidence` | struct | MultiPerspectiveEvidence { pub inner: T, _perspective: PhantomData<P> } |  |  |  |  |

| `MultiPerspectiveLog` | struct | MultiPerspectiveLog { pub traces: Vec<String> } |  |  |  |  |

| `MultipleInstanceSpec` | struct | MultipleInstanceSpec { pub min: usize, pub max: Option<usize>, pub threshold: Option<usize>, pub creation: InstanceCreationKind } |  |  |  |  |

| `MultipleInstanceSpecConst` | struct | MultipleInstanceSpecConst { _priv: () } |  |  |  |  |

| `NamedLoss` | struct | NamedLoss { pub projection_str: String, pub _category: String } |  |  |  |  |

| `OCEL` | struct | OCEL { pub events: Vec<OCELEvent>, pub objects: Vec<OCELObject> } |  |  |  |  |

| `OCELEvent` | struct | OCELEvent { pub id: std::string::String, pub event_type: std::string::String, pub relationships: Vec<OCELRelationship>, pub attributes: Vec<OCELEventAttribute> } |  |  |  |  |

| `OCELEventAttribute` | struct | OCELEventAttribute { pub name: std::string::String, pub value: OCELAttributeValue } |  |  |  |  |

| `OCELObject` | struct | OCELObject { pub id: std::string::String, pub object_type: std::string::String, pub attributes: Vec<OCELEventAttribute>, pub relationships: Vec<OCELRelationship> } |  |  |  |  |

| `OCELRelationship` | struct | OCELRelationship { pub object_id: std::string::String, pub qualifier: std::string::String } |  |  |  |  |

| `OCELType` | struct | OCELType { pub name: std::string::String, pub attributes: Vec<OCELTypeAttribute> } |  |  |  |  |

| `OCELTypeAttribute` | struct | OCELTypeAttribute { pub name: std::string::String, pub value_type: std::string::String } |  |  |  |  |

| `Object` | struct | Object { pub id: String, pub obj_type: String, _attrs: Vec<OcelAttribute> } |  |  |  |  |

| `ObjectCentricDfg` | struct | ObjectCentricDfg { HashMap<String, Dfg> } |  |  |  |  |

| `ObjectCentricPetriNet` | struct | ObjectCentricPetriNet { _net: PetriNet, _object_types: Vec<String> } |  |  |  |  |

| `ObjectChange` | struct | ObjectChange { _object_id: String, _attribute: String, _value: String, _ts: Option<u64> } |  |  |  |  |

| `ObjectLifecycle` | struct | ObjectLifecycle { pub events: Vec<LifecycleEvent> } |  |  |  |  |

| `ObjectObjectLink` | struct | ObjectObjectLink { pub source_id: String, pub target_id: String, _qualifier: Option<String> } |  |  |  |  |

| `ObjectScope` | struct | ObjectScope { pub object_types: Vec<String> } |  |  |  |  |

| `ObjectScopeConst` | struct | ObjectScopeConst { pub object_types: Vec<String> } |  |  |  |  |

| `ObjectTypeCardinality` | struct | ObjectTypeCardinality { pub min_count: Option<usize>, pub max_count: Option<usize>, pub created_by: Vec<std::string::String>, pub terminated_by: Vec<std::string::String> } |  |  |  |  |

| `ObjectTypeName` | struct | ObjectTypeName { inner: Cow<'static, str>, _kind: PhantomData<K> } |  |  |  |  |

| `OcDeclareConstraint` | struct | OcDeclareConstraint { pub constraint: DeclareConstraint, pub object_types: Vec<String>, _synchronized: bool } |  |  |  |  |

| `OcelAttribute` | struct | OcelAttribute { pub key: std::string::String, pub value: OcelAttributeValue } |  |  |  |  |

| `OcelEvent` | struct | OcelEvent { pub id: String, pub event_type: String, _ts: Option<u64>, _attrs: Vec<OcelAttribute> } |  |  |  |  |

| `OcelLog` | struct | OcelLog { objects: Vec<Object>, events: Vec<OcelEvent>, event_object_links: Vec<EventObjectLink>, object_object_links: Vec<ObjectObjectLink>, object_changes: Vec<ObjectChange> } |  |  |  |  |

| `OcpqQuery` | struct | OcpqQuery { pub scope: ObjectScope, pub predicates: Vec<Predicate>, pub sub_queries: Vec<OcpqQuery> } |  |  |  |  |

| `OcpqQueryConst` | struct | OcpqQueryConst { _scope: ObjectScopeConst } |  |  |  |  |

| `OrderEdge` | struct | OrderEdge { pub from: PowlNodeId, pub to: PowlNodeId } |  |  |  |  |

| `OutputBinding` | struct | OutputBinding { pub &'a str, pub &'a str } |  |  |  |  |

| `PackedKeyTable` | struct | PackedKeyTable { data: HashMap<u64, (String, V)> } |  |  |  |  |

| `ParityComparer` | struct |  |  |  |  |  |

| `Parsed` | struct | Parsed |  |  |  |  |

| `PerspectiveCombination` | struct | PerspectiveCombination { PhantomData<(A, B)> } |  |  |  |  |

| `PetriNet` | struct | PetriNet { pub places: Vec<Place>, pub transitions: Vec<Transition>, pub arcs: Vec<Arc>, pub initial_marking: Marking, pub final_marking: Marking } |  |  |  |  |

| `Place` | struct | Place { pub id: String } |  |  |  |  |

| `PlaceNodeMarker` | struct |  |  |  |  |  |

| `PlaceToTransitionArc` | struct | PlaceToTransitionArc { W, PhantomData<(P, T)> } |  |  |  |  |

| `Powl` | struct | Powl { pub nodes: Vec<PowlNode>, pub edges: Vec<OrderEdge>, pub root: Option<PowlNodeId> } |  |  |  |  |

| `PowlChoiceNode` | struct | PowlChoiceNode { branches: Vec<PowlNodeId> } |  |  |  |  |

| `PowlComposition` | struct | PowlComposition { pub inner: Inner } |  |  |  |  |

| `PowlNode` | struct | PowlNode { pub id: PowlNodeId, pub kind: PowlNodeKind } |  |  |  |  |

| `PowlNodeId` | struct | PowlNodeId { pub u64 } |  |  |  |  |

| `Precision` | struct | Precision { pub f64 } |  |  |  |  |

| `Predicate` | struct | Predicate { pub kind: PredicateKind, _data: std::marker::PhantomData<T> } |  |  |  |  |

| `ProcessCube` | struct | ProcessCube { pub dimensions: Vec<CubeDimensionKind> } |  |  |  |  |

| `ProcessSlice` | struct | ProcessSlice { pub dimension: CubeDimensionKind, pub value: String } |  |  |  |  |

| `ProcessTree` | struct | ProcessTree { pub nodes: Vec<ProcessTreeNode>, pub root: Option<ProcessTreeNodeId> } |  |  |  |  |

| `ProcessTreeNodeId` | struct | ProcessTreeNodeId { pub usize } |  |  |  |  |

| `Projected` | struct | Projected |  |  |  |  |

| `ProjectionName` | struct | ProjectionName { pub &'static str } |  |  |  |  |

| `ProjectionNameOwned` | struct | ProjectionNameOwned { pub String } |  |  |  |  |

| `QualityProfile` | struct | QualityProfile { pub fitness: Between01<FN, FD>, pub precision: Between01<PN, PD>, pub f1: Between01<F1N, F1D>, pub generalization: Between01<GN, GD>, pub simplicity: Between01<SN, SD> } |  |  |  |  |

| `Raw` | struct | Raw |  |  |  |  |

| `ReceiptChain` | struct | ReceiptChain { pub run_id: String, pub envelopes: Vec<ReceiptEnvelope> } |  |  |  |  |

| `ReceiptChainConst` | struct | ReceiptChainConst { pub run_id: String, pub links: Vec<ReceiptEnvelope> } |  |  |  |  |

| `ReceiptEnvelope` | struct | ReceiptEnvelope { pub subject: String, pub witness: String, pub digest: Digest, pub replay_hint: ReplayHint } |  |  |  |  |

| `Receipted` | struct | Receipted |  |  |  |  |

| `Refusal` | struct | Refusal { pub reason: R, _witness: PhantomData<W> } |  |  |  |  |

| `Refused` | struct | Refused |  |  |  |  |

| `ReplayHint` | struct | ReplayHint { pub String } |  |  |  |  |

| `Require` | struct |  |  |  |  |  |

| `ResourcePerspective` | struct |  |  |  |  |  |

| `RuntimeMarking` | struct | RuntimeMarking { table: &'a PackedKeyTable<usize> } |  |  |  |  |

| `SeparableWfNet` | struct | SeparableWfNet { pub net: WfNetConst<S> } |  |  |  |  |

| `Simplicity` | struct | Simplicity { pub f64 } |  |  |  |  |

| `SoundnessClaimed` | struct |  |  |  |  |  |

| `SoundnessProof` | struct | SoundnessProof |  |  |  |  |

| `SoundnessUnknown` | struct |  |  |  |  |  |

| `StableId` | struct | StableId { pub String } |  |  |  |  |

| `TemporalConstraint` | struct | TemporalConstraint { pub relation: TemporalRelation, pub bound_ms: u64 } |  |  |  |  |

| `TimePerspective` | struct |  |  |  |  |  |

| `Trace` | struct | Trace { pub events: Vec<Event>, pub case_id: String } |  |  |  |  |

| `Transition` | struct | Transition { pub id: String, pub label: String } |  |  |  |  |

| `TransitionNodeMarker` | struct |  |  |  |  |  |

| `TransitionToPlaceArc` | struct | TransitionToPlaceArc { W, PhantomData<(T, P)> } |  |  |  |  |

| `TypedEventPredicate` | struct | TypedEventPredicate { _expr: String } |  |  |  |  |

| `TypedId` | struct | TypedId { pub id: String, _type: PhantomData<T> } |  |  |  |  |

| `TypedLoopNode` | struct | TypedLoopNode { pub children: Children } |  |  |  |  |

| `TypedObjectPredicate` | struct | TypedObjectPredicate { _expr: String } |  |  |  |  |

| `TypedPowlLoopNode` | struct | TypedPowlLoopNode { pub children: Children } |  |  |  |  |

| `TypedRelationPredicate` | struct | TypedRelationPredicate { _expr: String } |  |  |  |  |

| `UnknownVerifier` | struct |  |  |  |  |  |

| `WfNet` | struct | WfNet { _net: PetriNet, _final: Marking, _state: PhantomData<S> } |  |  |  |  |

| `WfNetConst` | struct | WfNetConst { _priv: () } |  |  |  |  |

| `Witnessed` | struct | Witnessed |  |  |  |  |

| `IsTrue` | trait |  |  |  |  |  |

| `IsValidArc` | trait |  |  |  |  |  |

| `VerifyCausalConsistency` | trait |  |  |  |  |  |

| `Witness` | trait |  |  |  |  |  |

| `crate::models::PackedKeyTable` | use | crate::models::PackedKeyTable |  |  |  |  |


### stubs/wasm4pm-compat/src/lib.rs

| `MAX_POWL_DEPTH` | const | MAX_POWL_DEPTH: usize |  |  |  |  |

| `ArcDirection` | enum | ArcDirection { PlaceToTransition, TransitionToPlace } |  |  |  |  |

| `ArcDirectionConst` | enum | ArcDirectionConst { PlaceToTransition, TransitionToPlace } |  |  |  |  |

| `BpmnEvent` | enum | BpmnEvent { Start, Intermediate, End, Boundary } |  |  |  |  |

| `BpmnGateway` | enum | BpmnGateway { Exclusive, Parallel, Inclusive, EventBased, Complex } |  |  |  |  |

| `BpmnNodeKind` | enum | BpmnNodeKind { Event(BpmnEvent), Task(BpmnTask), Gateway(BpmnGateway) } |  |  |  |  |

| `BpmnRefusal` | enum | BpmnRefusal { DanglingEdge, MissingStart, MissingStartEvent, MissingEndEvent, EmptyProcess, DuplicateNodeId, LaneNodeNotDeclared } |  |  |  |  |

| `CausalConsistency` | enum | CausalConsistency { Unknown, Consistent, HasCycles, HasContradictions } |  |  |  |  |

| `CausalNetRefusal` | enum | CausalNetRefusal { MissingActivity, InvalidDependencyScore, DisconnectedGraph } |  |  |  |  |

| `CompatDiagnostic` | enum | CompatDiagnostic { MissingWitness, MissingRoundTripFixture, RawEvidenceExportedAsAdmitted, LossyProjectionWithoutPolicy, HiddenFlattening, MissingRefusalPath, MissingReceiptShape, UnreachablePrimitive, MigrationRecommended } |  |  |  |  |

| `ComplianceKind` | enum | ComplianceKind { Monitoring, Audit, Certification } |  |  |  |  |

| `ConformanceRefusal` | enum | ConformanceRefusal { EmptyModel, TooManyDeviations, InvalidProfile, MissingLog, MissingModel, MissingDeviationPath, FitnessUnavailable, PrecisionUnavailable, F1Unavailable, GeneralizationUnavailable, SimplicityUnavailable } |  |  |  |  |

| `ConformanceVerdict` | enum | ConformanceVerdict { PerfectAlignment, FitnessDeficit, DeadlockEncountered } |  |  |  |  |

| `CorrelationSchema` | enum | CorrelationSchema { ByCase, ByObject, ByTimestamp, ByAttribute } |  |  |  |  |

| `CubeDimensionKind` | enum | CubeDimensionKind { Activity, Resource, Time, DataAttribute, ObjectType, CaseAttribute } |  |  |  |  |

| `DeclareRefusal` | enum | DeclareRefusal { EmptyActivity, MissingActivation, BinaryRequiresTarget, UnsupportedTemplate, MissingTarget, InvalidTemplateArity, EmptyObjectScope, SynchronizationViolation } |  |  |  |  |

| `DeclareScope` | enum | DeclareScope { SingleObjectScope(String), MultiObjectScope(Vec<String>), SynchronizedObjectScope(Vec<String>), CrossObjectScope(String, String), GlobalScope } |  |  |  |  |

| `DeclareTemplate` | enum | DeclareTemplate { Existence, Absence, Init, Existence2, Existence3, Absence2, Absence3, RespondedExistence, CoExistence, Response, Precedence, Succession, AlternateResponse, AlternatePrecedence, AlternateSuccession, ChainResponse, ChainPrecedence, ChainSuccession, NotSuccession, NotChainSuccession, NotCoExistence, ExclusiveChoice } |  |  |  |  |

| `DfgRefusal` | enum | DfgRefusal { EmptyGraph, DanglingEdge, IsolatedNode } |  |  |  |  |

| `DiagnosticKind` | enum | DiagnosticKind { Warning, Error, Info } |  |  |  |  |

| `DiagnosticSeverity` | enum | DiagnosticSeverity { Error, Warning, Info } |  |  |  |  |

| `EventLogRefusal` | enum | EventLogRefusal { EmptyLog, EmptyTrace, MissingActivity, NonMonotonicTrace } |  |  |  |  |

| `EventPredicateKind` | enum | EventPredicateKind { ActivityEquals, AttributeEquals, TimestampInRange } |  |  |  |  |

| `EvidenceMode` | enum | EvidenceMode { Raw, Parsed, Admitted, Refused, Projected, Exportable, Witnessed, Receipted } |  |  |  |  |

| `FilterShape` | enum | FilterShape { Activity, Timeframe, Variant, Attribute, ObjectType } |  |  |  |  |

| `FormatKind` | enum | FormatKind { OcelJson, OcelXml, OcelSqlite, XesXml, BpmnXml, PetriPnml, PowlJson } |  |  |  |  |

| `InstanceCreationKind` | enum | InstanceCreationKind { Static, Dynamic } |  |  |  |  |

| `InteropRefusal` | enum | InteropRefusal { UnsupportedShape, MissingGrounding, SchemaConflict, UngroundedArtifact, FlatClaimOverObjectCentric, DimensionShapeMismatch } |  |  |  |  |

| `KernelRefusal` | enum | KernelRefusal { ZeroSize, TooDense } |  |  |  |  |

| `LifecycleRefusal` | enum | LifecycleRefusal { EmptyLifecycle, InvalidTransition, OrphanEvent } |  |  |  |  |

| `LossFunction` | enum | LossFunction { MeanSquared, CrossEntropy, Hinge } |  |  |  |  |

| `LossPolicy` | enum | LossPolicy { RefuseLoss, AllowLossWithReport, AllowLossSilent, AllowNamedProjection } |  |  |  |  |

| `LossRefusal` | enum | LossRefusal { InvalidParameters, NumericalInstability } |  |  |  |  |

| `OCELAttributeValue` | enum | OCELAttributeValue { Integer(i64), Float(f64), Boolean(bool), String(std::string::String), Time(std::string::String), Null } |  |  |  |  |

| `ObjectCentricity` | enum | ObjectCentricity { CaseCentric, ObjectCentric, Mixed } |  |  |  |  |

| `ObjectLifecyclePhase` | enum | ObjectLifecyclePhase { Created, Active, Modified, Archived, Deleted } |  |  |  |  |

| `ObjectPredicateKind` | enum | ObjectPredicateKind { AttributeEquals, TypeEquals } |  |  |  |  |

| `ObjectTypeCardinality` | enum | ObjectTypeCardinality { One, ZeroOrOne, OneOrMany, ZeroOrMany } |  |  |  |  |

| `OcDeclareRefusal` | enum | OcDeclareRefusal { EmptyObjectTypeList, SynchronizationRequiresMultipleTypes, ScopeMismatch } |  |  |  |  |

| `OcelAttributeValue` | enum | OcelAttributeValue { Integer(i64), Float(f64), Boolean(bool), String(std::string::String), TimestampNs(u64), List(Vec<OcelAttributeValue>), Map(Vec<(std::string::String, OcelAttributeValue)>), Null } |  |  |  |  |

| `OcelRefusal` | enum | OcelRefusal { EmptyEventObjectLinks, DanglingEventObjectLink } |  |  |  |  |

| `OcpqRefusal` | enum | OcpqRefusal { EmptyFilter, UnsupportedScope } |  |  |  |  |

| `OcpqScopeKind` | enum | OcpqScopeKind { Open, Closed, SingleType } |  |  |  |  |

| `PerspectiveRefusal` | enum | PerspectiveRefusal { MissingPerspective, ConflictingPerspectives } |  |  |  |  |

| `PetriNetRefusal` | enum | PetriNetRefusal { IsolatedPlace, InvalidStructure, EmptyNet } |  |  |  |  |

| `PetriRefusal` | enum | PetriRefusal { IsolatedPlace, IsolatedTransition, MissingInitialMarking, InvalidWeight, MissingFinalMarking, UnsafeNet, InvalidInstanceBounds, ObjectTypeNotPreserved, DeadTransition } |  |  |  |  |

| `Pm4pyShape` | enum | Pm4pyShape { EventLog, ObjectCentricLog, PetriNet, ProcessTree, Bpmn, DirectlyFollowsGraph, Declare } |  |  |  |  |

| `Powl8Op` | enum | Powl8Op { NoOp = 0, Sequence = 1, Choice = 2, Parallel = 3, PartialOrder = 4, Loop = 5, Silent = 6, Or = 7, ChoiceGraph = 8 } |  |  |  |  |

| `Powl8OpError` | enum | Powl8OpError { InvalidDiscriminant } |  |  |  |  |

| `PowlNodeKind` | enum | PowlNodeKind { Atom(String), Silent, PartialOrder(Vec<PowlNodeId>), Choice(Vec<PowlNodeId>), Loop { body: PowlNodeId, redo: Option<PowlNodeId> }, ChoiceGraph { nodes: Vec<PowlNodeId>, edges: Vec<(usize, usize)> } } |  |  |  |  |

| `PowlProjectionState` | enum | PowlProjectionState { Unknown, ProcessTreeProjectable, ExceedsProcessTree, RefusedProjection } |  |  |  |  |

| `PowlRefusal` | enum | PowlRefusal { CyclicPartialOrder, InvalidLoop, InvalidChoiceArity { declared: usize, required_min: usize }, ChoiceGraphDisconnected } |  |  |  |  |

| `PredicateKind` | enum | PredicateKind { Event(String), Object(String), Relation(String), Temporal(String), Cardinality { min: usize, max: usize }, Nested, ChildSetBound { branch_label: String, min: usize, max: usize }, E2ORelation { event_var: String, object_var: String, qualifier: Option<String> }, O2ORelation { source_var: String, target_var: String, qualifier: Option<String> }, TimeBetweenEvents { from_var: String, to_var: String } } |  |  |  |  |

| `PredictionHorizon` | enum | PredictionHorizon { FullCase, Events(usize), TimeUnits(u64) } |  |  |  |  |

| `PredictionRefusal` | enum | PredictionRefusal { InsufficientData, InvalidModel, ConvergenceFailure, MissingPrefix, MissingTarget, EmptyPrefix, TargetUnsupported, NonPrefixTrace, ConstraintNotNamed } |  |  |  |  |

| `PredictionTarget` | enum | PredictionTarget { NextActivity, OutcomeLabel, RemainingTime, DriftSignal, Risk, ComplianceConstraint } |  |  |  |  |

| `ProcessPerspective` | enum | ProcessPerspective { ControlFlow, Data, Resource, Time } |  |  |  |  |

| `ProcessShapeKind` | enum | ProcessShapeKind { Event, Trace, EventLog, EventStream, XesLog, OcelLog, DirectlyFollowsGraph, ObjectCentricDfg, PetriNet, WorkflowNet, ObjectCentricPetriNet, ProcessTree, Powl, DeclareModel, ObjectCentricDeclareModel, LogSkeleton, OcpqQuery, Alignment, TokenReplay, ConformanceVerdict, PredictionProblem, Receipt } |  |  |  |  |

| `ProcessTreeNode` | enum | ProcessTreeNode { Activity(String), Operator { operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId> } } |  |  |  |  |

| `ProcessTreeOperator` | enum | ProcessTreeOperator { Sequence, Xor, Parallel, Or, Loop, Silent } |  |  |  |  |

| `ProcessTreeRefusal` | enum | ProcessTreeRefusal { EmptyTree, OrphanNode, InvalidLoopArity, UnsupportedOperator, MissingRoot, DanglingNodeReference, TauLeafWithChildren, BelowMinimumArity, InvalidArity, CycleDetected } |  |  |  |  |

| `QualityDimension` | enum | QualityDimension { Fitness, Precision, F1, Generalization, Simplicity } |  |  |  |  |

| `QualityMetricKind` | enum | QualityMetricKind { Fitness, Precision, F1, Generalization, Simplicity } |  |  |  |  |

| `ReceiptRefusal` | enum | ReceiptRefusal { EmptyChain, HashMismatch, InvalidDigest, MissingSubject, MissingWitness, MissingDigest, MissingReplayHint, BrokenChainLink(usize) } |  |  |  |  |

| `ReceiptVerdict` | enum | ReceiptVerdict { Admitted, Refused(ReceiptRefusal) } |  |  |  |  |

| `RelationLaw` | enum | RelationLaw { EventToObject, ObjectToObject, ObjectToEvent } |  |  |  |  |

| `RelationPredicateKind` | enum | RelationPredicateKind { E2O, O2O, TimeBetweenEvents } |  |  |  |  |

| `ReplayHintKind` | enum | ReplayHintKind { FromStart, FromSeq(u64), Latest } |  |  |  |  |

| `SoundnessState` | enum | SoundnessState { Unknown, Claimed, Witnessed } |  |  |  |  |

| `StandaloneChoiceGraphNode` | enum | StandaloneChoiceGraphNode { Start, End, Activity(String), SubModel(usize) } |  |  |  |  |

| `SummaryShape` | enum | SummaryShape { Counts, TraceVariants, ActivityDistribution, TimingProfile, ObjectTypeDistribution } |  |  |  |  |

| `TemporalOrder` | enum | TemporalOrder { Before, After, Concurrent, Unknown } |  |  |  |  |

| `TemporalRefusal` | enum | TemporalRefusal { ZeroBound, ConflictingConstraints } |  |  |  |  |

| `TemporalRelation` | enum | TemporalRelation { Before, After, During, Concurrent } |  |  |  |  |

| `WitnessFamily` | enum | WitnessFamily { Standard, ApiGrammar, Implementation, Paper } |  |  |  |  |

| `WorkflowPattern` | enum | WorkflowPattern { Sequence, ParallelSplit, Synchronization, ExclusiveChoice, SimpleMerge, MultiChoice, StructuredSynchronizingMerge, MultiMerge, StructuredDiscriminator, ArbitraryCycles, ImplicitTermination, MultipleInstancesWithoutSync, MultipleInstancesWithDesignTimeKnowledge, DeferredChoice, InterleavedParallelRouting, CancelActivity, CancelCase } |  |  |  |  |

| `activity` | function | activity(&self) -> &str |  |  |  |  |

| `add_arc` | function | add_arc(&mut self, a: Arc) |  |  |  |  |

| `add_place` | function | add_place(&mut self, p: Place) |  |  |  |  |

| `add_transition` | function | add_transition(&mut self, t: Transition) |  |  |  |  |

| `admit_flat` | function | admit_flat(&self) -> Result<(), InteropRefusal> |  |  |  |  |

| `admit_shape` | function | admit_shape(&self) -> Result<(), ProcessTreeRefusal> |  |  |  |  |

| `admits` | function | admits(&self, count: usize) -> bool |  |  |  |  |

| `arcs` | function | arcs(&self) -> &[Arc] |  |  |  |  |

| `arity` | function | arity(&self) -> u8 |  |  |  |  |

| `as_f64` | function | as_f64(&self) -> f64 |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `assert_epsilon_close` | function | assert_epsilon_close(actual: f64, expected: f64) |  |  |  |  |

| `at_ns` | function | at_ns(mut self, ts: u64) -> Self |  |  |  |  |

| `attribute` | function | attribute(&self) -> &str |  |  |  |  |

| `attributes` | function | attributes(&self) -> &[OcelAttribute] |  |  |  |  |

| `binary` | function | binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) -> Self |  |  |  |  |

| `boolean` | function | boolean(key: &str, value: bool) -> Self |  |  |  |  |

| `by` | function | by(mut self, resource: &str) -> Self |  |  |  |  |

| `case_id` | function | case_id(&self) -> &str |  |  |  |  |

| `category` | function | category(&self) -> &str |  |  |  |  |

| `check_filter_shape` | function | check_filter_shape(shape: Pm4pyShape, filter: FilterShape) -> Result<(), InteropRefusal> |  |  |  |  |

| `claim_sound` | function | claim_sound(self) -> WfNet<SoundnessClaimed> |  |  |  |  |

| `conformance_rate` | function | conformance_rate(&self) -> f64 |  |  |  |  |

| `count` | function | count(&self) -> usize |  |  |  |  |

| `count_objects_of_type` | function | count_objects_of_type(&self, t: &str) -> usize |  |  |  |  |

| `declare_separable` | function | declare_separable(net: WfNetConst<S>) -> Self |  |  |  |  |

| `default` | function | default() -> Self |  |  |  |  |

| `den` | function | den(&self) -> u64 |  |  |  |  |

| `direction` | function | direction(&self) -> ArcDirection |  |  |  |  |

| `duration_ns` | function | duration_ns(&self) -> Option<u64> |  |  |  |  |

| `e2o` | function | e2o(&self, event_id: &str) -> Vec<(&str, &str)> |  |  |  |  |

| `edges` | function | edges(&self) -> &[BpmnEdge] |  |  |  |  |

| `empty` | function | empty() -> Self |  |  |  |  |

| `eval` | function | eval(&self, event_id: &str) -> Option<HashMap<std::string::String, OCELAttributeValue>> |  |  |  |  |

| `event` | function | event(id: &str, e: BpmnEvent) -> Self |  |  |  |  |

| `event_count` | function | event_count(&self) -> usize |  |  |  |  |

| `event_id` | function | event_id(&self) -> &str |  |  |  |  |

| `event_object_links` | function | event_object_links(&self) -> &[EventObjectLink] |  |  |  |  |

| `event_set` | function | event_set(&self) -> &Vec<OCELEvent> |  |  |  |  |

| `events` | function | events(&self) -> &[OcelEvent] |  |  |  |  |

| `expression` | function | expression(&self) -> &str |  |  |  |  |

| `extend_with` | function | extend_with(&mut self, env: ReceiptEnvelope) -> Result<(), ReceiptRefusal> |  |  |  |  |

| `final_marking` | function | final_marking(&self) -> Option<&Marking> |  |  |  |  |

| `float` | function | float(key: &str, value: f64) -> Self |  |  |  |  |

| `fnv1a_64` | function | fnv1a_64(bytes: &[u8]) -> u64 |  |  |  |  |

| `frequency` | function | frequency(&self) -> DfgWeight |  |  |  |  |

| `from_events` | function | from_events(events: impl IntoIterator<Item = Event>) -> Self |  |  |  |  |

| `from_owned` | function | from_owned(s: String) -> Self |  |  |  |  |

| `from_static` | function | from_static(s: &'static str) -> Self |  |  |  |  |

| `from_traces` | function | from_traces(traces: impl IntoIterator<Item = Trace>) -> Self |  |  |  |  |

| `gateway` | function | gateway(id: &str, g: BpmnGateway) -> Self |  |  |  |  |

| `get` | function | get(&self) -> f64 |  |  |  |  |

| `get_by_hash` | function | get_by_hash(&self, hash: u64) -> Option<&V> |  |  |  |  |

| `get_by_key` | function | get_by_key(&self, key: &str) -> Option<&V> |  |  |  |  |

| `id` | function | id(&self) -> &str |  |  |  |  |

| `initial_marking` | function | initial_marking(&self) -> &Marking |  |  |  |  |

| `inner` | function | inner(&self) -> &T |  |  |  |  |

| `insert` | function | insert(&mut self, hash: u64, key: String, value: V) |  |  |  |  |

| `integer` | function | integer(key: &str, value: i64) -> Self |  |  |  |  |

| `internal` | function | internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) -> Self |  |  |  |  |

| `into_admitted` | function | into_admitted(self) -> Evidence<T, state::Admitted, C> |  |  |  |  |

| `into_evidence` | function | into_evidence(self) -> Evidence<T, state::Admitted, W> |  |  |  |  |

| `into_exportable` | function | into_exportable(self) -> Evidence<T, state::Exportable, C> |  |  |  |  |

| `into_inner` | function | into_inner(self) -> T |  |  |  |  |

| `into_lost` | function | into_lost(self) -> Dropped |  |  |  |  |

| `into_parsed` | function | into_parsed(self) -> Evidence<T, state::Parsed, C> |  |  |  |  |

| `into_projected` | function | into_projected(self) -> Evidence<T, state::Projected, C> |  |  |  |  |

| `into_reason` | function | into_reason(self) -> R |  |  |  |  |

| `into_receipted` | function | into_receipted(self) -> Evidence<T, state::Receipted, C> |  |  |  |  |

| `is_chain` | function | is_chain(&self) -> bool |  |  |  |  |

| `is_consistent` | function | is_consistent(&self) -> bool |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_grounded` | function | is_grounded(&self) -> bool |  |  |  |  |

| `is_lossless` | function | is_lossless(&self) -> bool |  |  |  |  |

| `is_negative` | function | is_negative(&self) -> bool |  |  |  |  |

| `is_object_centric` | function | is_object_centric(&self) -> bool |  |  |  |  |

| `is_perfect` | function | is_perfect(&self) -> bool |  |  |  |  |

| `is_silent` | function | is_silent(&self) -> bool |  |  |  |  |

| `is_synchronized` | function | is_synchronized(&self) -> bool |  |  |  |  |

| `is_well_shaped` | function | is_well_shaped(&self) -> bool |  |  |  |  |

| `iter` | function | iter(&self) -> impl Iterator<Item = &ReceiptEnvelope> |  |  |  |  |

| `kind` | function | kind(&self) -> QualityMetricKind |  |  |  |  |

| `label` | function | label(&self) -> &str |  |  |  |  |

| `lanes` | function | lanes(&self) -> &[BpmnLane] |  |  |  |  |

| `leaf` | function | leaf(id: ProcessTreeNodeId, label: &str) -> Self |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `length` | function | length(&self) -> usize |  |  |  |  |

| `lifecycle` | function | lifecycle(&self) -> Option<&str> |  |  |  |  |

| `mark` | function | mark(&mut self, place_id: &str, tokens: usize) |  |  |  |  |

| `max` | function | max(&self) -> usize |  |  |  |  |

| `members` | function | members(&self) -> &[String] |  |  |  |  |

| `min` | function | min(&self) -> usize |  |  |  |  |

| `name` | function | name(&self) -> &str |  |  |  |  |

| `net` | function | net(&self) -> &PetriNet |  |  |  |  |

| `new` | function | new(value: T) -> Self |  |  |  |  |

| `node_count` | function | node_count(&self) -> usize |  |  |  |  |

| `node_ids` | function | node_ids(&self) -> &[String] |  |  |  |  |

| `nodes` | function | nodes(&self) -> &[BpmnNode] |  |  |  |  |

| `num` | function | num(&self) -> u64 |  |  |  |  |

| `o2o` | function | o2o(&self, object_id: &str) -> Vec<(&str, &str)> |  |  |  |  |

| `object_changes` | function | object_changes(&self) -> &[ObjectChange] |  |  |  |  |

| `object_id` | function | object_id(&self) -> &str |  |  |  |  |

| `object_object_links` | function | object_object_links(&self) -> &[ObjectObjectLink] |  |  |  |  |

| `object_set` | function | object_set(&self) -> &Vec<OCELObject> |  |  |  |  |

| `object_type` | function | object_type(&self) -> &str |  |  |  |  |

| `object_types` | function | object_types(&self) -> impl Iterator<Item = &str> |  |  |  |  |

| `objects` | function | objects(&self) -> &[Object] |  |  |  |  |

| `place_id` | function | place_id(&self) -> &str |  |  |  |  |

| `place_to_transition` | function | place_to_transition(place: &str, transition: &str) -> Self |  |  |  |  |

| `places` | function | places(&self) -> &[Place] |  |  |  |  |

| `predecessors` | function | predecessors(&self, idx: usize) -> Vec<usize> |  |  |  |  |

| `process` | function | process(&self) -> &BpmnProcess |  |  |  |  |

| `projection` | function | projection(&self) -> ProjectionNameOwned |  |  |  |  |

| `push` | function | push(&mut self, event: Event) |  |  |  |  |

| `qualified` | function | qualified(mut self, q: &str) -> Self |  |  |  |  |

| `qualifier` | function | qualifier(&self) -> Option<&str> |  |  |  |  |

| `raw` | function | raw(value: T) -> Self |  |  |  |  |

| `resource` | function | resource(&self) -> Option<&str> |  |  |  |  |

| `root` | function | root(&self) -> Option<ProcessTreeNodeId> |  |  |  |  |

| `schema` | function | schema(&self) -> &'static str |  |  |  |  |

| `scope` | function | scope(&self) -> &ObjectScopeConst |  |  |  |  |

| `silent` | function | silent(id: &str) -> Self |  |  |  |  |

| `soundness_state` | function | soundness_state(&self) -> SoundnessState |  |  |  |  |

| `source` | function | source(&self) -> &str |  |  |  |  |

| `source_id` | function | source_id(&self) -> &str |  |  |  |  |

| `steps` | function | steps(&self) -> &[NamedLoss] |  |  |  |  |

| `string` | function | string(key: &str, value: &str) -> Self |  |  |  |  |

| `successors` | function | successors(&self, idx: usize) -> Vec<usize> |  |  |  |  |

| `summary` | function | summary(&self, category: &str) -> NamedLoss |  |  |  |  |

| `synchronized` | function | synchronized(constraint: DeclareConstraint, object_types: Vec<String>) -> Self |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `target` | function | target(&self) -> &str |  |  |  |  |

| `target_id` | function | target_id(&self) -> &str |  |  |  |  |

| `task` | function | task(id: &str, t: BpmnTask) -> Self |  |  |  |  |

| `timestamp_ns` | function | timestamp_ns(&self) -> Option<u64> |  |  |  |  |

| `tip` | function | tip(&self) -> &ReceiptEnvelope |  |  |  |  |

| `tokens` | function | tokens(&self) -> &HashMap<String, usize> |  |  |  |  |

| `tokens_on` | function | tokens_on(&self, place_id: &str) -> usize |  |  |  |  |

| `trace_count` | function | trace_count(&self) -> usize |  |  |  |  |

| `traces` | function | traces(&self) -> &[Trace] |  |  |  |  |

| `transition_id` | function | transition_id(&self) -> &str |  |  |  |  |

| `transition_to_place` | function | transition_to_place(transition: &str, place: &str) -> Self |  |  |  |  |

| `transitions` | function | transitions(&self) -> &[Transition] |  |  |  |  |

| `try_from_parts` | function | try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) -> Result<Self, ReceiptRefusal> |  |  |  |  |

| `try_new` | function | try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) -> Result<Self, ReceiptRefusal> |  |  |  |  |

| `unary` | function | unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) -> Self |  |  |  |  |

| `validate` | function | validate(&self) -> Result<(), OcelRefusal> |  |  |  |  |

| `value` | function | value(&self) -> &str |  |  |  |  |

| `verdict` | function | verdict(&self) -> CausalConsistency |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), ReceiptRefusal> |  |  |  |  |

| `weight` | function | weight(&self) -> W |  |  |  |  |

| `with_attribute` | function | with_attribute(mut self, a: OcelAttribute) -> Self |  |  |  |  |

| `with_duration_ns` | function | with_duration_ns(mut self, d: u64) -> Self |  |  |  |  |

| `with_generalization` | function | with_generalization(mut self, v: f64) -> Self |  |  |  |  |

| `with_lifecycle` | function | with_lifecycle(mut self, lc: &str) -> Self |  |  |  |  |

| `with_precision` | function | with_precision(mut self, v: f64) -> Self |  |  |  |  |

| `with_simplicity` | function | with_simplicity(mut self, v: f64) -> Self |  |  |  |  |

| `with_type_dfg` | function | with_type_dfg(mut self, obj_type: &str, dfg: Dfg) -> Self |  |  |  |  |

| `witness_soundness` | function | witness_soundness(self, _proof: SoundnessProof) -> WfNetConst<{ SoundnessState::Witnessed }> |  |  |  |  |

| `Activity` | struct | Activity { pub String } |  |  |  |  |

| `Admission` | struct | Admission { pub value: T, _witness: PhantomData<W> } |  |  |  |  |

| `Admitted` | struct | Admitted |  |  |  |  |

| `Arc` | struct | Arc { pub source: String, pub target: String, pub weight: u32, pub object_type: Option<(String, bool)>, _dir: ArcDirection } |  |  |  |  |

| `ArtifactGrounding` | struct | ArtifactGrounding { pub shape: Pm4pyShape, pub evidence_ref: String, _evidence: PhantomData<E> } |  |  |  |  |

| `Between01` | struct | Between01 { _priv: () } |  |  |  |  |

| `BipartiteArcConst` | struct | BipartiteArcConst { _place_id: String, _transition_id: String, _weight: W } |  |  |  |  |

| `BpmnEdge` | struct | BpmnEdge { source: String, target: String } |  |  |  |  |

| `BpmnLane` | struct | BpmnLane { id: String, name: String, node_ids: Vec<String> } |  |  |  |  |

| `BpmnNode` | struct | BpmnNode { id: String, kind: BpmnNodeKind } |  |  |  |  |

| `BpmnPool` | struct | BpmnPool { id: String, name: String, process: BpmnProcess, lanes: Vec<BpmnLane> } |  |  |  |  |

| `BpmnProcess` | struct | BpmnProcess { nodes: Vec<BpmnNode>, edges: Vec<BpmnEdge> } |  |  |  |  |

| `BpmnTask` | struct | BpmnTask { pub label: String } |  |  |  |  |

| `CancellationRegion` | struct | CancellationRegion { _members: Vec<String> } |  |  |  |  |

| `CausalBinding` | struct | CausalBinding { pub source_tasks: Vec<String>, pub target_tasks: Vec<String> } |  |  |  |  |

| `CausalChain` | struct |  |  |  |  |  |

| `CausalLink` | struct | CausalLink { PhantomData<(From, To)> } |  |  |  |  |

| `CausalNet` | struct | CausalNet { pub nodes: Vec<String>, pub dependency_measures: Vec<(String, String, f64)>, pub inputs: Vec<CausalBinding>, pub outputs: Vec<CausalBinding> } |  |  |  |  |

| `CausallyOrderedEvidence` | struct | CausallyOrderedEvidence { pub inner: T } |  |  |  |  |

| `ChoiceGraph` | struct | ChoiceGraph { nodes: Vec<StandaloneChoiceGraphNode>, edges: Vec<(usize, usize)> } |  |  |  |  |

| `ConditionCell` | struct | ConditionCell { _priv: () } |  |  |  |  |

| `ConformanceResult` | struct | ConformanceResult { pub fitness: f64, pub precision: Option<f64>, pub generalization: Option<f64>, pub simplicity: Option<f64>, pub total_traces: usize, pub fitting_traces: usize, pub deviating_traces: usize } |  |  |  |  |

| `ConformanceVerdict` | struct | ConformanceVerdict { pub fitness: Option<Fitness>, pub deviations: Vec<Deviation> } |  |  |  |  |

| `ConsistencyVerified` | struct | ConsistencyVerified { pub inner: T, verdict: CausalConsistency } |  |  |  |  |

| `ControlFlowPerspective` | struct |  |  |  |  |  |

| `CorrelatedLog` | struct | CorrelatedLog { PhantomData<(A, B)> } |  |  |  |  |

| `CorrelationKey` | struct |  |  |  |  |  |

| `DataPerspective` | struct |  |  |  |  |  |

| `DeclareConstraint` | struct | DeclareConstraint { pub template: DeclareTemplate, pub activation: Activity, pub target: Option<Activity>, pub scope: DeclareScope } |  |  |  |  |

| `DenseKernel` | struct | DenseKernel { pub size: usize } |  |  |  |  |

| `DependencyMeasure` | struct | DependencyMeasure { pub f64 } |  |  |  |  |

| `Deviation` | struct | Deviation { pub position: usize, pub label: String } |  |  |  |  |

| `Dfg` | struct | Dfg { nodes: Vec<DfgNode>, edges: Vec<DfgEdge> } |  |  |  |  |

| `DfgEdge` | struct | DfgEdge { source: String, target: String, count: u64 } |  |  |  |  |

| `DfgEdgeFull` | struct | DfgEdgeFull { pub source: String, pub target: String, _freq: u64, _dur: Option<u64> } |  |  |  |  |

| `DfgNode` | struct | DfgNode { pub String } |  |  |  |  |

| `DfgWeight` | struct | DfgWeight { pub u64 } |  |  |  |  |

| `DiagnosticReport` | struct | DiagnosticReport { pub kind: DiagnosticKind, pub message: String, pub stage: String } |  |  |  |  |

| `Digest` | struct | Digest { pub String } |  |  |  |  |

| `Event` | struct | Event { pub activity: String, pub attributes: HashMap<String, String>, _ts: Option<u64>, _resource: Option<String>, _lifecycle: Option<String> } |  |  |  |  |

| `EventLog` | struct | EventLog { pub traces: Vec<Trace> } |  |  |  |  |

| `EventObjectLink` | struct | EventObjectLink { pub event_id: String, pub object_id: String, _qualifier: Option<String> } |  |  |  |  |

| `EventStream` | struct | EventStream { events: Vec<Event> } |  |  |  |  |

| `EventTypeName` | struct | EventTypeName { inner: Cow<'static, str>, _kind: PhantomData<K> } |  |  |  |  |

| `Evidence` | struct | Evidence { pub value: T, _state: PhantomData<S>, _chain: PhantomData<C> } |  |  |  |  |

| `Exportable` | struct | Exportable |  |  |  |  |

| `F1` | struct | F1 { pub f64 } |  |  |  |  |

| `Fitness` | struct | Fitness { pub f64 } |  |  |  |  |

| `Generalization` | struct | Generalization { pub f64 } |  |  |  |  |

| `InitialFinalMarkingPair` | struct | InitialFinalMarkingPair { _initial: Marking, _final: Marking } |  |  |  |  |

| `InputBinding` | struct | InputBinding { pub &'a str, pub &'a str } |  |  |  |  |

| `LifecycleEvent` | struct | LifecycleEvent { pub event_type: String, pub object_id: String } |  |  |  |  |

| `LossChain` | struct | LossChain { steps: Vec<NamedLoss> } |  |  |  |  |

| `LossReport` | struct | LossReport { pub projection: ProjectionName, pub policy: LossPolicy, _dropped: Dropped, _ph: PhantomData<(A, B)> } |  |  |  |  |

| `Marking` | struct | Marking { pub HashMap<String, usize> } |  |  |  |  |

| `Metric` | struct | Metric { _priv: () } |  |  |  |  |

| `MultiPerspectiveEvidence` | struct | MultiPerspectiveEvidence { pub inner: T, _perspective: PhantomData<P> } |  |  |  |  |

| `MultiPerspectiveLog` | struct | MultiPerspectiveLog { pub traces: Vec<String> } |  |  |  |  |

| `MultipleInstanceSpec` | struct | MultipleInstanceSpec { pub min: usize, pub max: Option<usize>, pub threshold: Option<usize>, pub creation: InstanceCreationKind } |  |  |  |  |

| `MultipleInstanceSpecConst` | struct | MultipleInstanceSpecConst { _priv: () } |  |  |  |  |

| `NamedLoss` | struct | NamedLoss { pub projection_str: String, pub _category: String } |  |  |  |  |

| `OCEL` | struct | OCEL { pub events: Vec<OCELEvent>, pub objects: Vec<OCELObject> } |  |  |  |  |

| `OCELEvent` | struct | OCELEvent { pub id: std::string::String, pub event_type: std::string::String, pub relationships: Vec<OCELRelationship>, pub attributes: Vec<OCELEventAttribute> } |  |  |  |  |

| `OCELEventAttribute` | struct | OCELEventAttribute { pub name: std::string::String, pub value: OCELAttributeValue } |  |  |  |  |

| `OCELObject` | struct | OCELObject { pub id: std::string::String, pub object_type: std::string::String, pub attributes: Vec<OCELEventAttribute>, pub relationships: Vec<OCELRelationship> } |  |  |  |  |

| `OCELRelationship` | struct | OCELRelationship { pub object_id: std::string::String, pub qualifier: std::string::String } |  |  |  |  |

| `OCELType` | struct | OCELType { pub name: std::string::String, pub attributes: Vec<OCELTypeAttribute> } |  |  |  |  |

| `OCELTypeAttribute` | struct | OCELTypeAttribute { pub name: std::string::String, pub value_type: std::string::String } |  |  |  |  |

| `Object` | struct | Object { pub id: String, pub obj_type: String, _attrs: Vec<OcelAttribute> } |  |  |  |  |

| `ObjectCentricDfg` | struct | ObjectCentricDfg { HashMap<String, Dfg> } |  |  |  |  |

| `ObjectCentricPetriNet` | struct | ObjectCentricPetriNet { _net: PetriNet, _object_types: Vec<String> } |  |  |  |  |

| `ObjectChange` | struct | ObjectChange { _object_id: String, _attribute: String, _value: String, _ts: Option<u64> } |  |  |  |  |

| `ObjectLifecycle` | struct | ObjectLifecycle { pub events: Vec<LifecycleEvent> } |  |  |  |  |

| `ObjectObjectLink` | struct | ObjectObjectLink { pub source_id: String, pub target_id: String, _qualifier: Option<String> } |  |  |  |  |

| `ObjectScope` | struct | ObjectScope { pub object_types: Vec<String> } |  |  |  |  |

| `ObjectScopeConst` | struct | ObjectScopeConst { pub object_types: Vec<String> } |  |  |  |  |

| `ObjectTypeCardinality` | struct | ObjectTypeCardinality { pub min_count: Option<usize>, pub max_count: Option<usize>, pub created_by: Vec<std::string::String>, pub terminated_by: Vec<std::string::String> } |  |  |  |  |

| `ObjectTypeName` | struct | ObjectTypeName { inner: Cow<'static, str>, _kind: PhantomData<K> } |  |  |  |  |

| `OcDeclareConstraint` | struct | OcDeclareConstraint { pub constraint: DeclareConstraint, pub object_types: Vec<String>, _synchronized: bool } |  |  |  |  |

| `OcelAttribute` | struct | OcelAttribute { pub key: std::string::String, pub value: OcelAttributeValue } |  |  |  |  |

| `OcelEvent` | struct | OcelEvent { pub id: String, pub event_type: String, _ts: Option<u64>, _attrs: Vec<OcelAttribute> } |  |  |  |  |

| `OcelLog` | struct | OcelLog { objects: Vec<Object>, events: Vec<OcelEvent>, event_object_links: Vec<EventObjectLink>, object_object_links: Vec<ObjectObjectLink>, object_changes: Vec<ObjectChange> } |  |  |  |  |

| `OcpqQuery` | struct | OcpqQuery { pub scope: ObjectScope, pub predicates: Vec<Predicate>, pub sub_queries: Vec<OcpqQuery> } |  |  |  |  |

| `OcpqQueryConst` | struct | OcpqQueryConst { _scope: ObjectScopeConst } |  |  |  |  |

| `OrderEdge` | struct | OrderEdge { pub from: PowlNodeId, pub to: PowlNodeId } |  |  |  |  |

| `OutputBinding` | struct | OutputBinding { pub &'a str, pub &'a str } |  |  |  |  |

| `PackedKeyTable` | struct | PackedKeyTable { data: HashMap<u64, (String, V)> } |  |  |  |  |

| `ParityComparer` | struct |  |  |  |  |  |

| `Parsed` | struct | Parsed |  |  |  |  |

| `PerspectiveCombination` | struct | PerspectiveCombination { PhantomData<(A, B)> } |  |  |  |  |

| `PetriNet` | struct | PetriNet { pub places: Vec<Place>, pub transitions: Vec<Transition>, pub arcs: Vec<Arc>, pub initial_marking: Marking, pub final_marking: Marking } |  |  |  |  |

| `Place` | struct | Place { pub id: String } |  |  |  |  |

| `PlaceNodeMarker` | struct |  |  |  |  |  |

| `PlaceToTransitionArc` | struct | PlaceToTransitionArc { W, PhantomData<(P, T)> } |  |  |  |  |

| `Powl` | struct | Powl { pub nodes: Vec<PowlNode>, pub edges: Vec<OrderEdge>, pub root: Option<PowlNodeId> } |  |  |  |  |

| `PowlChoiceNode` | struct | PowlChoiceNode { branches: Vec<PowlNodeId> } |  |  |  |  |

| `PowlComposition` | struct | PowlComposition { pub inner: Inner } |  |  |  |  |

| `PowlNode` | struct | PowlNode { pub id: PowlNodeId, pub kind: PowlNodeKind } |  |  |  |  |

| `PowlNodeId` | struct | PowlNodeId { pub u64 } |  |  |  |  |

| `Precision` | struct | Precision { pub f64 } |  |  |  |  |

| `Predicate` | struct | Predicate { pub kind: PredicateKind, _data: std::marker::PhantomData<T> } |  |  |  |  |

| `ProcessCube` | struct | ProcessCube { pub dimensions: Vec<CubeDimensionKind> } |  |  |  |  |

| `ProcessSlice` | struct | ProcessSlice { pub dimension: CubeDimensionKind, pub value: String } |  |  |  |  |

| `ProcessTree` | struct | ProcessTree { pub nodes: Vec<ProcessTreeNode>, pub root: Option<ProcessTreeNodeId> } |  |  |  |  |

| `ProcessTreeNodeId` | struct | ProcessTreeNodeId { pub usize } |  |  |  |  |

| `Projected` | struct | Projected |  |  |  |  |

| `ProjectionName` | struct | ProjectionName { pub &'static str } |  |  |  |  |

| `ProjectionNameOwned` | struct | ProjectionNameOwned { pub String } |  |  |  |  |

| `QualityProfile` | struct | QualityProfile { pub fitness: Between01<FN, FD>, pub precision: Between01<PN, PD>, pub f1: Between01<F1N, F1D>, pub generalization: Between01<GN, GD>, pub simplicity: Between01<SN, SD> } |  |  |  |  |

| `Raw` | struct | Raw |  |  |  |  |

| `ReceiptChain` | struct | ReceiptChain { pub run_id: String, pub envelopes: Vec<ReceiptEnvelope> } |  |  |  |  |

| `ReceiptChainConst` | struct | ReceiptChainConst { pub run_id: String, pub links: Vec<ReceiptEnvelope> } |  |  |  |  |

| `ReceiptEnvelope` | struct | ReceiptEnvelope { pub subject: String, pub witness: String, pub digest: Digest, pub replay_hint: ReplayHint } |  |  |  |  |

| `Receipted` | struct | Receipted |  |  |  |  |

| `Refusal` | struct | Refusal { pub reason: R, _witness: PhantomData<W> } |  |  |  |  |

| `Refused` | struct | Refused |  |  |  |  |

| `ReplayHint` | struct | ReplayHint { pub String } |  |  |  |  |

| `Require` | struct |  |  |  |  |  |

| `ResourcePerspective` | struct |  |  |  |  |  |

| `RuntimeMarking` | struct | RuntimeMarking { table: &'a PackedKeyTable<usize> } |  |  |  |  |

| `SeparableWfNet` | struct | SeparableWfNet { pub net: WfNetConst<S> } |  |  |  |  |

| `Simplicity` | struct | Simplicity { pub f64 } |  |  |  |  |

| `SoundnessClaimed` | struct |  |  |  |  |  |

| `SoundnessProof` | struct | SoundnessProof |  |  |  |  |

| `SoundnessUnknown` | struct |  |  |  |  |  |

| `StableId` | struct | StableId { pub String } |  |  |  |  |

| `TemporalConstraint` | struct | TemporalConstraint { pub relation: TemporalRelation, pub bound_ms: u64 } |  |  |  |  |

| `TimePerspective` | struct |  |  |  |  |  |

| `Trace` | struct | Trace { pub events: Vec<Event>, pub case_id: String } |  |  |  |  |

| `Transition` | struct | Transition { pub id: String, pub label: String } |  |  |  |  |

| `TransitionNodeMarker` | struct |  |  |  |  |  |

| `TransitionToPlaceArc` | struct | TransitionToPlaceArc { W, PhantomData<(T, P)> } |  |  |  |  |

| `TypedEventPredicate` | struct | TypedEventPredicate { _expr: String } |  |  |  |  |

| `TypedId` | struct | TypedId { pub id: String, _type: PhantomData<T> } |  |  |  |  |

| `TypedLoopNode` | struct | TypedLoopNode { pub children: Children } |  |  |  |  |

| `TypedObjectPredicate` | struct | TypedObjectPredicate { _expr: String } |  |  |  |  |

| `TypedPowlLoopNode` | struct | TypedPowlLoopNode { pub children: Children } |  |  |  |  |

| `TypedRelationPredicate` | struct | TypedRelationPredicate { _expr: String } |  |  |  |  |

| `UnknownVerifier` | struct |  |  |  |  |  |

| `WfNet` | struct | WfNet { _net: PetriNet, _final: Marking, _state: PhantomData<S> } |  |  |  |  |

| `WfNetConst` | struct | WfNetConst { _priv: () } |  |  |  |  |

| `Witnessed` | struct | Witnessed |  |  |  |  |

| `IsTrue` | trait |  |  |  |  |  |

| `IsValidArc` | trait |  |  |  |  |  |

| `VerifyCausalConsistency` | trait |  |  |  |  |  |

| `Witness` | trait |  |  |  |  |  |

| `crate::models::PackedKeyTable` | use | crate::models::PackedKeyTable |  |  |  |  |


### stubs/wasm4pm/src/lib.rs

| `AttributeValue` | enum | AttributeValue { String(std::string::String), Int(i64), Float(f64), Date(std::string::String), Boolean(bool), List(Vec<AttributeValue>), Container(HashMap<std::string::String, AttributeValue>), Null } |  |  |  |  |

| `new` | function | new(attributes: HashMap<std::string::String, AttributeValue>) -> Self |  |  |  |  |

| `DFG` | struct | DFG { pub nodes: Vec<DFGNode>, pub edges: Vec<DirectlyFollowsRelation>, pub start_activities: HashMap<std::string::String, usize>, pub end_activities: HashMap<std::string::String, usize> } |  |  |  |  |

| `DFGNode` | struct | DFGNode { pub id: std::string::String, pub label: std::string::String, pub frequency: usize } |  |  |  |  |

| `DirectlyFollowsRelation` | struct | DirectlyFollowsRelation { pub from: std::string::String, pub to: std::string::String, pub frequency: usize } |  |  |  |  |

| `Event` | struct | Event { pub attributes: HashMap<std::string::String, AttributeValue> } |  |  |  |  |

| `Trace` | struct | Trace { pub events: Vec<Event>, pub attributes: HashMap<std::string::String, AttributeValue> } |  |  |  |  |


### stubs/wasm4pm/src/lib.rs

| `AttributeValue` | enum | AttributeValue { String(std::string::String), Int(i64), Float(f64), Date(std::string::String), Boolean(bool), List(Vec<AttributeValue>), Container(HashMap<std::string::String, AttributeValue>), Null } |  |  |  |  |

| `new` | function | new(attributes: HashMap<std::string::String, AttributeValue>) -> Self |  |  |  |  |

| `DFG` | struct | DFG { pub nodes: Vec<DFGNode>, pub edges: Vec<DirectlyFollowsRelation>, pub start_activities: HashMap<std::string::String, usize>, pub end_activities: HashMap<std::string::String, usize> } |  |  |  |  |

| `DFGNode` | struct | DFGNode { pub id: std::string::String, pub label: std::string::String, pub frequency: usize } |  |  |  |  |

| `DirectlyFollowsRelation` | struct | DirectlyFollowsRelation { pub from: std::string::String, pub to: std::string::String, pub frequency: usize } |  |  |  |  |

| `Event` | struct | Event { pub attributes: HashMap<std::string::String, AttributeValue> } |  |  |  |  |

| `Trace` | struct | Trace { pub events: Vec<Event>, pub attributes: HashMap<std::string::String, AttributeValue> } |  |  |  |  |


### tests/advanced_crypto_chicago.rs

| `affidavit:chicago:bls` | str_key | CONTEXT = "affidavit:chicago:bls" |  |  |  |  |

| `affidavit:chicago:zk-range` | str_key | LABEL = "affidavit:chicago:zk-range" |  |  |  |  |


### tests/architecture_qualification.rs

| `git:fb09f991fab7995c1b7564f668065ef379303800` | str_key | SUBJECT = "git:fb09f991fab7995c1b7564f668065ef379303800" |  |  |  |  |

| `sha256:abb-0001` | str_key | ABB = "sha256:abb-0001" |  |  |  |  |

| `sha256:contract-0001` | str_key | CONTRACT = "sha256:contract-0001" |  |  |  |  |

| `sha256:producer-autofde-lab` | str_key | PRODUCER = "sha256:producer-autofde-lab" |  |  |  |  |

| `sha256:sbb-0001` | str_key | SBB = "sha256:sbb-0001" |  |  |  |  |


### tests/brce_ledger.rs

| `cc1577a8de1e4e92625386ebbecb0c6df9abd66a` | str_key | SUBJECT = "cc1577a8de1e4e92625386ebbecb0c6df9abd66a" |  |  |  |  |


### tests/catalog_tests.rs

| `filter_fixtures` | function | filter_fixtures( fixtures: Vec<FixtureMeta>, name: Option<&str>, events: Option<usize>, ) -> Vec<FixtureMeta> |  |  |  |  |

| `FixtureMeta` | struct | FixtureMeta { pub name: String, pub event_count: usize, pub description: String, pub path: Option<PathBuf> } |  |  |  |  |


### tests/completions_drift.rs

| `__affi_no_noun' -a ` | str_key | NOUN_GUARD = "__affi_no_noun' -a " |  |  |  |  |

| `__affi_no_verb' -a ` | str_key | VERB_GUARD = "__affi_no_verb' -a " |  |  |  |  |

| `__affi_using_noun ` | str_key | NOUN_SCOPE = "__affi_using_noun " |  |  |  |  |


### tests/crypto_trust_acvp.rs

| `fixtures/crypto_trust_acvp` | str_key | FIXTURE_DIR = "fixtures/crypto_trust_acvp" |  |  |  |  |


### tests/crypto_trust_e2e.rs

| `CTP-ENVELOPE-v1` | str_key | COURT_ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | COURT_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |


### tests/crypto_trust_envelope_cli.rs

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |


### tests/crypto_trust_evidence_cli.rs

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |


### tests/crypto_trust_jcs_differential.rs

| `fixtures/crypto_trust_jcs_corpus.json` | str_key | CORPUS_RELPATH = "fixtures/crypto_trust_jcs_corpus.json" |  |  |  |  |

| `fixtures/crypto_trust_jcs_expected.json` | str_key | EXPECTED_RELPATH = "fixtures/crypto_trust_jcs_expected.json" |  |  |  |  |


### tests/crypto_trust_kat_fixture.rs

| `AFFIDAVIT_KAT_EXPORT_FIXTURE` | str_key | EXPORT_ENV = "AFFIDAVIT_KAT_EXPORT_FIXTURE" |  |  |  |  |

| `affidavit::crypto_trust_kat::generate_corpus() -> export_json (JCS, RFC 8785); deterministic under KAT_DOMAIN_TAG=affidavit.crypto-trust-plane.v1; regenerate: cd <repo root> && AFFIDAVIT_KAT_EXPORT_FIXTURE=1 cargo test --features crypto-trust --test crypto_trust_kat_fixture export_fixture -- --exact --ignored` | str_key | GENERATED_FROM = "affidavit::crypto_trust_kat::generate_corpus() -> export_json (JCS, RFC 8785); deterministic under KAT_DOMAIN_TAG=affidavit.crypto-trust-plane.v1; regenerate: cd <repo root> && AFFIDAVIT_KAT_EXPORT_FIXTURE=1 cargo test --features crypto-trust --test crypto_trust_kat_fixture export_fixture -- --exact --ignored" |  |  |  |  |

| `corpus` | str_key | EXPECTED_HEADER_KEYS = "corpus" |  |  |  |  |

| `fixtures/crypto_trust_kat.json` | str_key | FIXTURE_RELPATH = "fixtures/crypto_trust_kat.json" |  |  |  |  |


### tests/crypto_trust_kat_vectors.rs

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `48563e42e5c6f6aa0d600fc8baf0d7455c37f7ac` | str_key | GENERATING_COMMIT = "48563e42e5c6f6aa0d600fc8baf0d7455c37f7ac" |  |  |  |  |

| `AFFIDAVIT_KATVECTORS_EXPORT_FIXTURE` | str_key | EXPORT_ENV = "AFFIDAVIT_KATVECTORS_EXPORT_FIXTURE" |  |  |  |  |

| `CTP-KATVECTORS-v1` | str_key | SCHEMA_VERSION = "CTP-KATVECTORS-v1" |  |  |  |  |

| `PASS: 16/16 seed pre-images` | str_key | CROSS_CHECK_SENTINEL = "PASS: 16/16 seed pre-images" |  |  |  |  |

| `affidavit-kat-subject` | str_key | CUSTODIAN_SUBJECT = "affidavit-kat-subject" |  |  |  |  |

| `affidavit.kat` | str_key | AUDIENCE = "affidavit.kat" |  |  |  |  |

| `affidavit.seal` | str_key | SEAL_AUDIENCE = "affidavit.seal" |  |  |  |  |

| `fixtures/crypto_trust_kat_vectors.json` | str_key | FIXTURE_RELPATH = "fixtures/crypto_trust_kat_vectors.json" |  |  |  |  |


### tests/crypto_trust_keys_cli.rs

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |


### tests/crypto_trust_mutation_court.rs

| `3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f` | str_key | PAYLOAD_HASH = "3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f" |  |  |  |  |

| `subject-a` | str_key | SUBJECT = "subject-a" |  |  |  |  |


### tests/crypto_trust_property.rs

| `a` | str_key | POOL = "a" |  |  |  |  |


### tests/crypto_trust_wire_fuzz.rs

| `../fixtures/crypto_trust_wire_corpus.json` | str_key | CORPUS = "../fixtures/crypto_trust_wire_corpus.json" |  |  |  |  |


### tests/maximalist_tracing.rs

| `verify_maximalist` | function | verify_maximalist(receipt: &Receipt, tracer: &T) -> Verdict |  |  |  |  |


### tests/property_based.rs

| `ArbitraryOperationEvent` | struct | ArbitraryOperationEvent { pub OperationEvent } |  |  |  |  |

| `ArbitraryReceipt` | struct | ArbitraryReceipt { pub Receipt } |  |  |  |  |


### tests/receipts_certified.rs

| `3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f` | str_key | PAYLOAD_HASH = "3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f" |  |  |  |  |

| `subject-a` | str_key | SUBJECT = "subject-a" |  |  |  |  |


### tests/release_identity.rs

| `CARGO_PKG_VERSION` | str_key | PKG_VERSION = "CARGO_PKG_VERSION" |  |  |  |  |


### tests/visualize_tests.rs

| `from_receipt_dfg` | function | from_receipt_dfg(receipt: &Receipt) -> Self |  |  |  |  |

| `to_dot` | function | to_dot(&self) -> String |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `GraphEdge` | struct | GraphEdge { pub source: String, pub target: String, pub label: String } |  |  |  |  |

| `GraphNode` | struct | GraphNode { pub id: String, pub label: String, pub node_type: String } |  |  |  |  |

| `ReceiptGraph` | struct | ReceiptGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> } |  |  |  |  |


### tools/confevo-rs/src/breeds/csp.rs

| `CspAc3` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/csp.rs

| `CspAc3` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/encode.rs

| `Engine` | enum | Engine { SatCdcl, CspAc3 } |  |  |  |  |

| `breed_id` | function | breed_id(self) -> &'static str |  |  |  |  |

| `encode_csp` | function | encode_csp(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> |  |  |  |  |

| `encode_sat` | function | encode_sat(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> |  |  |  |  |

| `forbid` | function | forbid(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `from_id` | function | from_id(s: &str) -> Option<Engine> |  |  |  |  |

| `generate_config` | function | generate_config( space: &FeatureSpace, query: &ConfigQuery, engine: Engine, ) -> Result<SemanticConfig, String> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `require` | function | require(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `ConfigQuery` | struct | ConfigQuery { pub require: Vec<String>, pub forbid: Vec<String> } |  |  |  |  |

| `SemanticConfig` | struct | SemanticConfig { pub engine_id: String, pub feasible: bool, pub genome: Option<Genome>, pub clash: Vec<String>, pub explanation: String, pub result: BreedResult } |  |  |  |  |


### tools/confevo-rs/src/breeds/encode.rs

| `Engine` | enum | Engine { SatCdcl, CspAc3 } |  |  |  |  |

| `breed_id` | function | breed_id(self) -> &'static str |  |  |  |  |

| `encode_csp` | function | encode_csp(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> |  |  |  |  |

| `encode_sat` | function | encode_sat(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> |  |  |  |  |

| `forbid` | function | forbid(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `from_id` | function | from_id(s: &str) -> Option<Engine> |  |  |  |  |

| `generate_config` | function | generate_config( space: &FeatureSpace, query: &ConfigQuery, engine: Engine, ) -> Result<SemanticConfig, String> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `require` | function | require(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `ConfigQuery` | struct | ConfigQuery { pub require: Vec<String>, pub forbid: Vec<String> } |  |  |  |  |

| `SemanticConfig` | struct | SemanticConfig { pub engine_id: String, pub feasible: bool, pub genome: Option<Genome>, pub clash: Vec<String>, pub explanation: String, pub result: BreedResult } |  |  |  |  |


### tools/confevo-rs/src/breeds/mod.rs

| `CSP_AC3` | const | CSP_AC3: CspAc3 |  |  |  |  |

| `SAT_CDCL` | const | SAT_CDCL: SatCdcl |  |  |  |  |

| `Verdict` | enum | Verdict { Sat, Unsat, Unknown } |  |  |  |  |

| `from_json` | function | from_json(text: &str) -> Result<Contract, String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_sat` | function | is_sat(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(key: impl Into<String>, value: impl Into<String>) -> Self |  |  |  |  |

| `push` | function | push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) |  |  |  |  |

| `run_named` | function | run_named(id: &str, contract: &Contract) -> Result<BreedResult, String> |  |  |  |  |

| `supported_breeds` | function | supported_breeds() -> &'static [&'static str] |  |  |  |  |

| `tag` | function | tag(self) -> &'static str |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `BreedResult` | struct | BreedResult { pub breed: String, pub status: String, pub facts: Vec<Fact>, pub selected: String, pub explanation: String, pub inference_trace: Vec<TraceStep>, pub run_id: String, pub output_hash: String, pub inference_step_count: usize, pub rules_evaluated: usize } |  |  |  |  |

| `Contract` | struct | Contract { pub intent: String, pub facts: Vec<Fact> } |  |  |  |  |

| `Fact` | struct | Fact { pub key: String, pub value: String } |  |  |  |  |

| `Trace` | struct | Trace { steps: Vec<TraceStep> } |  |  |  |  |

| `TraceStep` | struct | TraceStep { pub step: usize, pub kind: String, pub detail: String, pub depth: usize } |  |  |  |  |

| `Breed` | trait |  |  |  |  |  |

| `csp::CspAc3` | use | csp::CspAc3 |  |  |  |  |

| `sat::SatCdcl` | use | sat::SatCdcl |  |  |  |  |


### tools/confevo-rs/src/breeds/mod.rs

| `CSP_AC3` | const | CSP_AC3: CspAc3 |  |  |  |  |

| `SAT_CDCL` | const | SAT_CDCL: SatCdcl |  |  |  |  |

| `Verdict` | enum | Verdict { Sat, Unsat, Unknown } |  |  |  |  |

| `from_json` | function | from_json(text: &str) -> Result<Contract, String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_sat` | function | is_sat(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(key: impl Into<String>, value: impl Into<String>) -> Self |  |  |  |  |

| `push` | function | push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) |  |  |  |  |

| `run_named` | function | run_named(id: &str, contract: &Contract) -> Result<BreedResult, String> |  |  |  |  |

| `supported_breeds` | function | supported_breeds() -> &'static [&'static str] |  |  |  |  |

| `tag` | function | tag(self) -> &'static str |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `BreedResult` | struct | BreedResult { pub breed: String, pub status: String, pub facts: Vec<Fact>, pub selected: String, pub explanation: String, pub inference_trace: Vec<TraceStep>, pub run_id: String, pub output_hash: String, pub inference_step_count: usize, pub rules_evaluated: usize } |  |  |  |  |

| `Contract` | struct | Contract { pub intent: String, pub facts: Vec<Fact> } |  |  |  |  |

| `Fact` | struct | Fact { pub key: String, pub value: String } |  |  |  |  |

| `Trace` | struct | Trace { steps: Vec<TraceStep> } |  |  |  |  |

| `TraceStep` | struct | TraceStep { pub step: usize, pub kind: String, pub detail: String, pub depth: usize } |  |  |  |  |

| `Breed` | trait |  |  |  |  |  |

| `csp::CspAc3` | use | csp::CspAc3 |  |  |  |  |

| `sat::SatCdcl` | use | sat::SatCdcl |  |  |  |  |


### tools/confevo-rs/src/breeds/sat.rs

| `SatCdcl` | struct |  |  |  |  |  |


### tools/confevo-rs/src/breeds/sat.rs

| `SatCdcl` | struct |  |  |  |  |  |


### tools/confevo-rs/src/evolve.rs

| `run_ga` | function | run_ga( eval: &mut impl Evaluator, space: &FeatureSpace, cfg: &GaConfig, ) -> Result<GaResult, GaError> |  |  |  |  |

| `GaConfig` | struct | GaConfig { pub population: usize, pub generations: usize, pub seed: u64, pub mutation_rate: f64, pub crossover_rate: f64, pub elitism: usize, pub tournament_k: usize } |  |  |  |  |

| `GaError` | struct | GaError { pub String } |  |  |  |  |

| `GaResult` | struct | GaResult { pub best_genome: Genome, pub best_eval: EvalResult, pub history: Vec<GenerationRecord>, pub evaluations: usize } |  |  |  |  |

| `GenerationRecord` | struct | GenerationRecord { pub index: usize, pub best_score: f64, pub mean_score: f64, pub best_features: Vec<String> } |  |  |  |  |


### tools/confevo-rs/src/evolve.rs

| `run_ga` | function | run_ga( eval: &mut impl Evaluator, space: &FeatureSpace, cfg: &GaConfig, ) -> Result<GaResult, GaError> |  |  |  |  |

| `GaConfig` | struct | GaConfig { pub population: usize, pub generations: usize, pub seed: u64, pub mutation_rate: f64, pub crossover_rate: f64, pub elitism: usize, pub tournament_k: usize } |  |  |  |  |

| `GaError` | struct | GaError { pub String } |  |  |  |  |

| `GaResult` | struct | GaResult { pub best_genome: Genome, pub best_eval: EvalResult, pub history: Vec<GenerationRecord>, pub evaluations: usize } |  |  |  |  |

| `GenerationRecord` | struct | GenerationRecord { pub index: usize, pub best_score: f64, pub mean_score: f64, pub best_features: Vec<String> } |  |  |  |  |


### tools/confevo-rs/src/fitness.rs

| `cargo_available` | function | cargo_available() -> bool |  |  |  |  |

| `count_compiler_messages` | function | count_compiler_messages(stdout: &str) -> (u64, u64) |  |  |  |  |

| `generic` | function | generic() -> Self |  |  |  |  |

| `new` | function | new(base_errors: u64, poison: I) -> Self |  |  |  |  |

| `resolves_from_output` | function | resolves_from_output(combined: &str) -> bool |  |  |  |  |

| `score_from` | function | score_from( weights: &ScoreWeights, builds: bool, resolves: bool, error_count: u64, n_features: usize, elapsed_s: f64, ) -> f64 |  |  |  |  |

| `with_extra_args` | function | with_extra_args(mut self, args: I) -> Self |  |  |  |  |

| `with_weights` | function | with_weights(mut self, weights: ScoreWeights) -> Self |  |  |  |  |

| `failed to select a version` | str_key | RESOLVE_FAIL_MARKERS = "failed to select a version" |  |  |  |  |

| `CargoEvaluator` | struct | CargoEvaluator { manifest_dir: PathBuf, weights: ScoreWeights, extra_args: Vec<String> } |  |  |  |  |

| `EvalResult` | struct | EvalResult { pub features: Vec<String>, pub resolves: bool, pub builds: bool, pub error_count: u64, pub warn_count: u64, pub elapsed_s: f64, pub score: f64, pub from_cache: bool } |  |  |  |  |

| `ScoreWeights` | struct | ScoreWeights { pub build_bonus: f64, pub resolve_bonus: f64, pub per_feature: f64, pub error_penalty: f64, pub time_penalty: f64 } |  |  |  |  |

| `SyntheticEvaluator` | struct | SyntheticEvaluator { weights: ScoreWeights, base_errors: u64, poison: BTreeMap<String, u64> } |  |  |  |  |

| `Evaluator` | trait |  |  |  |  |  |


### tools/confevo-rs/src/fitness.rs

| `cargo_available` | function | cargo_available() -> bool |  |  |  |  |

| `count_compiler_messages` | function | count_compiler_messages(stdout: &str) -> (u64, u64) |  |  |  |  |

| `generic` | function | generic() -> Self |  |  |  |  |

| `new` | function | new(base_errors: u64, poison: I) -> Self |  |  |  |  |

| `resolves_from_output` | function | resolves_from_output(combined: &str) -> bool |  |  |  |  |

| `score_from` | function | score_from( weights: &ScoreWeights, builds: bool, resolves: bool, error_count: u64, n_features: usize, elapsed_s: f64, ) -> f64 |  |  |  |  |

| `with_extra_args` | function | with_extra_args(mut self, args: I) -> Self |  |  |  |  |

| `with_weights` | function | with_weights(mut self, weights: ScoreWeights) -> Self |  |  |  |  |

| `failed to select a version` | str_key | RESOLVE_FAIL_MARKERS = "failed to select a version" |  |  |  |  |

| `CargoEvaluator` | struct | CargoEvaluator { manifest_dir: PathBuf, weights: ScoreWeights, extra_args: Vec<String> } |  |  |  |  |

| `EvalResult` | struct | EvalResult { pub features: Vec<String>, pub resolves: bool, pub builds: bool, pub error_count: u64, pub warn_count: u64, pub elapsed_s: f64, pub score: f64, pub from_cache: bool } |  |  |  |  |

| `ScoreWeights` | struct | ScoreWeights { pub build_bonus: f64, pub resolve_bonus: f64, pub per_feature: f64, pub error_penalty: f64, pub time_penalty: f64 } |  |  |  |  |

| `SyntheticEvaluator` | struct | SyntheticEvaluator { weights: ScoreWeights, base_errors: u64, poison: BTreeMap<String, u64> } |  |  |  |  |

| `Evaluator` | trait |  |  |  |  |  |


### tools/confevo-rs/src/genome.rs

| `canonical` | function | canonical(&self, space: &FeatureSpace) -> Genome |  |  |  |  |

| `cargo_features_arg` | function | cargo_features_arg(&self) -> String |  |  |  |  |

| `contains` | function | contains(&self, feature: &str) -> bool |  |  |  |  |

| `empty` | function | empty() -> Self |  |  |  |  |

| `feature_set` | function | feature_set(&self) -> &BTreeSet<String> |  |  |  |  |

| `features` | function | features(&self) -> Vec<String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `key` | function | key(&self, space: &FeatureSpace) -> String |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(feats: I) -> Self |  |  |  |  |

| `random` | function | random(rng: &mut Rng, space: &FeatureSpace, p: f64) -> Self |  |  |  |  |

| `Genome` | struct | Genome { feats: BTreeSet<String> } |  |  |  |  |


### tools/confevo-rs/src/genome.rs

| `canonical` | function | canonical(&self, space: &FeatureSpace) -> Genome |  |  |  |  |

| `cargo_features_arg` | function | cargo_features_arg(&self) -> String |  |  |  |  |

| `contains` | function | contains(&self, feature: &str) -> bool |  |  |  |  |

| `empty` | function | empty() -> Self |  |  |  |  |

| `feature_set` | function | feature_set(&self) -> &BTreeSet<String> |  |  |  |  |

| `features` | function | features(&self) -> Vec<String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `key` | function | key(&self, space: &FeatureSpace) -> String |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(feats: I) -> Self |  |  |  |  |

| `random` | function | random(rng: &mut Rng, space: &FeatureSpace, p: f64) -> Self |  |  |  |  |

| `Genome` | struct | Genome { feats: BTreeSet<String> } |  |  |  |  |


### tools/confevo-rs/src/lib.rs

| `breeds::encode::{generate_config, ConfigQuery, Engine, SemanticConfig}` | use | breeds::encode::{generate_config, ConfigQuery, Engine, SemanticConfig} |  |  |  |  |

| `breeds::{run_named, Breed, BreedResult, Contract, Fact, Verdict}` | use | breeds::{run_named, Breed, BreedResult, Contract, Fact, Verdict} |  |  |  |  |

| `evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord}` | use | evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord} |  |  |  |  |

| `fitness::{ score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator, }` | use | fitness::{ score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator, } |  |  |  |  |

| `genome::Genome` | use | genome::Genome |  |  |  |  |

| `manifest::{feature_space_from_cargo_toml, ManifestError}` | use | manifest::{feature_space_from_cargo_toml, ManifestError} |  |  |  |  |

| `rng::Rng` | use | rng::Rng |  |  |  |  |

| `space::{FeatureSpace, SpaceError}` | use | space::{FeatureSpace, SpaceError} |  |  |  |  |


### tools/confevo-rs/src/lib.rs

| `breeds::encode::{generate_config, ConfigQuery, Engine, SemanticConfig}` | use | breeds::encode::{generate_config, ConfigQuery, Engine, SemanticConfig} |  |  |  |  |

| `breeds::{run_named, Breed, BreedResult, Contract, Fact, Verdict}` | use | breeds::{run_named, Breed, BreedResult, Contract, Fact, Verdict} |  |  |  |  |

| `evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord}` | use | evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord} |  |  |  |  |

| `fitness::{ score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator, }` | use | fitness::{ score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator, } |  |  |  |  |

| `genome::Genome` | use | genome::Genome |  |  |  |  |

| `manifest::{feature_space_from_cargo_toml, ManifestError}` | use | manifest::{feature_space_from_cargo_toml, ManifestError} |  |  |  |  |

| `rng::Rng` | use | rng::Rng |  |  |  |  |

| `space::{FeatureSpace, SpaceError}` | use | space::{FeatureSpace, SpaceError} |  |  |  |  |


### tools/confevo-rs/src/manifest.rs

| `ManifestError` | enum | ManifestError { Io(std::io::Error), NoFeaturesTable, Space(SpaceError) } |  |  |  |  |

| `feature_space_from_cargo_toml` | function | feature_space_from_cargo_toml( path: impl AsRef<Path>, include_default: bool, ) -> Result<FeatureSpace, ManifestError> |  |  |  |  |

| `feature_space_from_str` | function | feature_space_from_str( toml: &str, include_default: bool, ) -> Result<FeatureSpace, ManifestError> |  |  |  |  |


### tools/confevo-rs/src/manifest.rs

| `ManifestError` | enum | ManifestError { Io(std::io::Error), NoFeaturesTable, Space(SpaceError) } |  |  |  |  |

| `feature_space_from_cargo_toml` | function | feature_space_from_cargo_toml( path: impl AsRef<Path>, include_default: bool, ) -> Result<FeatureSpace, ManifestError> |  |  |  |  |

| `feature_space_from_str` | function | feature_space_from_str( toml: &str, include_default: bool, ) -> Result<FeatureSpace, ManifestError> |  |  |  |  |


### tools/confevo-rs/src/report.rs

| `Mode` | enum | Mode { DryRun, Real } |  |  |  |  |

| `to_json` | function | to_json( result: &GaResult, cfg: &GaConfig, mode: Mode, manifest: &str, universe: usize, ) -> String |  |  |  |  |

| `to_markdown` | function | to_markdown( result: &GaResult, cfg: &GaConfig, mode: Mode, manifest: &str, universe: usize, ) -> String |  |  |  |  |


### tools/confevo-rs/src/report.rs

| `Mode` | enum | Mode { DryRun, Real } |  |  |  |  |

| `to_json` | function | to_json( result: &GaResult, cfg: &GaConfig, mode: Mode, manifest: &str, universe: usize, ) -> String |  |  |  |  |

| `to_markdown` | function | to_markdown( result: &GaResult, cfg: &GaConfig, mode: Mode, manifest: &str, universe: usize, ) -> String |  |  |  |  |


### tools/confevo-rs/src/rng.rs

| `below` | function | below(&mut self, n: usize) -> usize |  |  |  |  |

| `gen_bool` | function | gen_bool(&mut self, p: f64) -> bool |  |  |  |  |

| `new` | function | new(seed: u64) -> Self |  |  |  |  |

| `next_f64` | function | next_f64(&mut self) -> f64 |  |  |  |  |

| `next_u64` | function | next_u64(&mut self) -> u64 |  |  |  |  |

| `Rng` | struct | Rng { state: u64 } |  |  |  |  |


### tools/confevo-rs/src/rng.rs

| `below` | function | below(&mut self, n: usize) -> usize |  |  |  |  |

| `gen_bool` | function | gen_bool(&mut self, p: f64) -> bool |  |  |  |  |

| `new` | function | new(seed: u64) -> Self |  |  |  |  |

| `next_f64` | function | next_f64(&mut self) -> f64 |  |  |  |  |

| `next_u64` | function | next_u64(&mut self) -> u64 |  |  |  |  |

| `Rng` | struct | Rng { state: u64 } |  |  |  |  |


### tools/confevo-rs/src/space.rs

| `SpaceError` | enum | SpaceError { DuplicateFeature(String), UnknownImplicationTarget(String, String), UnknownImplicationSource(String) } |  |  |  |  |

| `closure` | function | closure(&self, feats: &BTreeSet<String>) -> BTreeSet<String> |  |  |  |  |

| `contains` | function | contains(&self, name: &str) -> bool |  |  |  |  |

| `features` | function | features(&self) -> &[String] |  |  |  |  |

| `implications_of` | function | implications_of(&self, name: &str) -> impl Iterator<Item = &String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(features: F, implications: I) -> Result<Self, SpaceError> |  |  |  |  |

| `FeatureSpace` | struct | FeatureSpace { features: Vec<String>, implications: BTreeMap<String, BTreeSet<String>>, member: BTreeSet<String> } |  |  |  |  |


### tools/confevo-rs/src/space.rs

| `SpaceError` | enum | SpaceError { DuplicateFeature(String), UnknownImplicationTarget(String, String), UnknownImplicationSource(String) } |  |  |  |  |

| `closure` | function | closure(&self, feats: &BTreeSet<String>) -> BTreeSet<String> |  |  |  |  |

| `contains` | function | contains(&self, name: &str) -> bool |  |  |  |  |

| `features` | function | features(&self) -> &[String] |  |  |  |  |

| `implications_of` | function | implications_of(&self, name: &str) -> impl Iterator<Item = &String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(features: F, implications: I) -> Result<Self, SpaceError> |  |  |  |  |

| `FeatureSpace` | struct | FeatureSpace { features: Vec<String>, implications: BTreeMap<String, BTreeSet<String>>, member: BTreeSet<String> } |  |  |  |  |


### wip/1.2_diff_logic.rs

| `diff_json_receipts` | function | diff_json_receipts(old_json: &str, new_json: &str) -> anyhow::Result<DiffResult> |  |  |  |  |

| `diff_receipts` | function | diff_receipts(old: &Receipt, new: &Receipt) -> DiffResult |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `DiffResult` | struct | DiffResult { pub added: Vec<OperationEvent>, pub removed: Vec<OperationEvent>, pub modified: Vec<ModifiedEvent> } |  |  |  |  |

| `ModifiedEvent` | struct | ModifiedEvent { pub id: String, pub old: OperationEvent, pub new: OperationEvent } |  |  |  |  |


### wip/1.2_diff_observability.rs

| `DiffError` | enum | DiffError { Io( Json( FormatMismatch { left: String, right: String, }, ChainMismatch { left: Blake3Hash, right: Blake3Hash, }, Generic(String), } |  |  |  |  |

| `trace_diff` | function | trace_diff(left_path: &str, right_path: &str, f: F) -> DiffResult<T> |  |  |  |  |

| `DiffSummary` | struct | DiffSummary { pub left_event_count: usize, pub right_event_count: usize, pub total_differences: usize, pub added_count: usize, pub removed_count: usize, pub modified_count: usize } |  |  |  |  |

| `InstrumentedDiff` | trait |  |  |  |  |  |


### wip/1.3_visualize_logic.rs

| `build_graph` | function | build_graph(receipt: &Receipt) -> ReceiptGraph |  |  |  |  |

| `to_dot` | function | to_dot(graph: &ReceiptGraph) -> String |  |  |  |  |

| `to_json` | function | to_json(graph: &ReceiptGraph) -> anyhow::Result<String> |  |  |  |  |

| `GraphEdge` | struct | GraphEdge { pub from: String, pub to: String, pub weight: usize } |  |  |  |  |

| `GraphNode` | struct | GraphNode { pub id: String, pub label: String, pub event_count: usize } |  |  |  |  |

| `ReceiptGraph` | struct | ReceiptGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> } |  |  |  |  |


### wip/1.3_visualize_observability.rs

| `new` | function | new(receipt_path: &str, format: &str) -> Self |  |  |  |  |

| `trace_build` | function | trace_build(&self, event_count: usize, f: F) -> T |  |  |  |  |

| `trace_render` | function | trace_render(&self, node_count: usize, edge_count: usize, f: F) -> T |  |  |  |  |

| `VisualizeInstrumentation` | struct | VisualizeInstrumentation { pub receipt_path: String, pub format: String, pub start_time: Instant } |  |  |  |  |

| `VisualizeExt` | trait |  |  |  |  |  |


### wip/1.4_catalog_observability.rs

| `trace_catalog` | function | trace_catalog( filter_name: Option<&str>, filter_events: Option<usize>, f: F, ) -> T |  |  |  |  |

| `trace_catalog_scan` | function | trace_catalog_scan(source: &str, f: F) -> T |  |  |  |  |


### wip/1.5_completion_logic.rs

| `generate_completions` | function | generate_completions(shell_name: &str) -> Result<()> |  |  |  |  |

| `try_dispatch_completion` | function | try_dispatch_completion() -> Result<bool> |  |  |  |  |


### wip/1.5_completion_observability.rs

| `CompletionError` | enum | CompletionError { UnsupportedShell(String), Io( Generation(String), } |  |  |  |  |

| `Shell` | enum | Shell { Bash, Zsh, Fish, PowerShell, Elvish } |  |  |  |  |

| `completion` | function | completion(shell_name: String) -> anyhow::Result<()> |  |  |  |  |

| `trace_completion` | function | trace_completion(shell: Shell, f: F) -> T |  |  |  |  |


### wip/2.1_model_maximalist.rs

| `MiningError` | enum | MiningError { Him(String), Admission(String), OcelConversion(String) } |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) -> Result<PetriNet, MiningError> |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) -> Result<OCEL, MiningError> |  |  |  |  |


### wip/2.3_predict_maximalist.rs

| `PredictionError` | enum | PredictionError { InvalidTopK(usize), Wasm4pm(String) } |  |  |  |  |

| `predict_next` | function | predict_next( admitted: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, PredictionError> |  |  |  |  |

| `predict_next_with_model` | function | predict_next_with_model( model: &AdmittedReceipt, current_trace: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, PredictionError> |  |  |  |  |

| `ActivityPrediction` | struct | ActivityPrediction { pub activity: String, pub confidence: f64 } |  |  |  |  |

| `PredictionReport` | struct | PredictionReport { pub predictions: Vec<ActivityPrediction>, pub context_length: usize, pub model_type: String } |  |  |  |  |


### wip/2.4_2.5_lsp_maximalist.rs

| `from_receipt` | function | from_receipt(receipt: &Receipt, uri: &Url, text: &str) -> Self |  |  |  |  |

| `handle_definition` | function | handle_definition(pos: Position, index: &ReceiptIndex) -> Option<Vec<Location>> |  |  |  |  |

| `handle_hover` | function | handle_hover(pos: Position, index: &ReceiptIndex) -> Option<Hover> |  |  |  |  |

| `ObjectRefLocation` | struct | ObjectRefLocation { pub event_idx: usize, pub seq: u64 } |  |  |  |  |

| `ReceiptIndex` | struct | ReceiptIndex { pub receipt: Receipt, pub uri: Url, pub text: String, pub events: Vec<ReceiptSymbol>, pub object_refs: HashMap<String, Vec<ObjectRefLocation>> } |  |  |  |  |

| `ReceiptSymbol` | struct | ReceiptSymbol { pub event_id: String, pub seq: u64, pub event_type: String, pub range: Range, pub objects: Vec<ObjectRef>, pub payload_commitment: Blake3Hash } |  |  |  |  |


### wip/3.1_mutate_maximalist.rs

| `MutationKind` | enum | MutationKind { EventDrop, EventReorder, TypeChange, PayloadFlip } |  |  |  |  |

| `all_operators` | function | all_operators() -> Vec<Box<dyn MutationOperator>> |  |  |  |  |

| `AppliedMutation` | struct | AppliedMutation { pub kind: MutationKind, pub target_seq: u64, pub mutated_receipt: Receipt } |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |


### wip/3.2_3.3_generate_maximalist.rs

| `DEFAULT_SNIPPETS` | const | DEFAULT_SNIPPETS: &str |  |  |  |  |

| `TEST_FN_TEMPLATE` | const | TEST_FN_TEMPLATE: &str |  |  |  |  |

| `TEST_MODULE_TEMPLATE` | const | TEST_MODULE_TEMPLATE: &str |  |  |  |  |

| `filter_by_tag` | function | filter_by_tag(&self, tag: &str) -> Vec<&Snippet> |  |  |  |  |

| `find_by_name` | function | find_by_name(&self, pattern: &str) -> Vec<&Snippet> |  |  |  |  |

| `format_snippet` | function | format_snippet(&self, snippet: &Snippet) -> String |  |  |  |  |

| `from_json` | function | from_json(json: &str) -> Result<Self> |  |  |  |  |

| `generate_test_function` | function | generate_test_function(&self, pattern_name: &str, events: Vec<serde_json::Value>, expected_verdict: &str, expected_failure_stage: Option<&str> ) -> Result<String> |  |  |  |  |

| `generate_test_module` | function | generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) -> Result<String> |  |  |  |  |

| `main` | function | main() -> Result<()> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `CodegenEngine` | struct | CodegenEngine { tera: Tera } |  |  |  |  |

| `Snippet` | struct | Snippet { pub name: String, pub tags: Vec<String>, pub description: String, pub language: String, pub imports: Vec<String>, pub code: String } |  |  |  |  |

| `SnippetRegistry` | struct | SnippetRegistry { pub snippets: Vec<Snippet> } |  |  |  |  |


### wip/3.4_property_maximalist.rs

| `ArbitraryOperationEvent` | struct | ArbitraryOperationEvent { pub OperationEvent } |  |  |  |  |

| `ArbitraryReceipt` | struct | ArbitraryReceipt { pub Receipt } |  |  |  |  |


### wip/3.5_fixture_db_maximalist.rs

| `all` | function | all(&self) -> Vec<Fixture> |  |  |  |  |

| `delete_by_name` | function | delete_by_name(&mut self, name: &str) -> Result<bool> |  |  |  |  |

| `get_by_chain_hash` | function | get_by_chain_hash(&self, chain_hash: &str) -> Option<Fixture> |  |  |  |  |

| `get_by_name` | function | get_by_name(&self, name: &str) -> Option<Fixture> |  |  |  |  |

| `insert` | function | insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) -> Result<Fixture> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) -> Result<Self> |  |  |  |  |

| `reindex` | function | reindex(&mut self) -> Result<()> |  |  |  |  |

| `save` | function | save(&self) -> Result<()> |  |  |  |  |

| `search` | function | search(&self, query: &FixtureQuery) -> Vec<Fixture> |  |  |  |  |

| `Fixture` | struct | Fixture { pub id: String, pub name: String, pub tags: Vec<String>, pub event_count: usize, pub event_types: Vec<String>, pub chain_hash: String, pub inserted_at: String, pub receipt: Receipt } |  |  |  |  |

| `FixtureDatabase` | struct | FixtureDatabase { path: PathBuf, db: JsonDb, index_by_name: BTreeMap<String, usize>, index_by_event_count: BTreeMap<usize, Vec<usize>>, index_by_chain_hash: BTreeMap<String, usize> } |  |  |  |  |

| `FixtureQuery` | struct | FixtureQuery { pub name_contains: Option<String>, pub tag: Option<String>, pub min_events: Option<usize>, pub max_events: Option<usize>, pub event_type: Option<String>, pub limit: Option<usize> } |  |  |  |  |


### wip/4.2_4.5_metrics_maximalist.rs

| `SLO_AVAILABILITY_PCT` | const | SLO_AVAILABILITY_PCT: f64 |  |  |  |  |

| `SLO_ERROR_RATE_PCT` | const | SLO_ERROR_RATE_PCT: f64 |  |  |  |  |

| `SLO_LATENCY_P99_MS` | const | SLO_LATENCY_P99_MS: f64 |  |  |  |  |

| `SloViolation` | enum | SloViolation { LatencyP99 { observed_ms: f64, threshold_ms: f64 }, ErrorRate { observed_pct: f64, threshold_pct: f64 }, Availability { observed_pct: f64, threshold_pct: f64 } } |  |  |  |  |

| `check_slo` | function | check_slo(&self) -> anyhow::Result<(), SloViolation> |  |  |  |  |

| `compute_sli` | function | compute_sli(&self) -> anyhow::Result<ServiceLevelIndicators> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `new_noop` | function | new_noop() -> Self |  |  |  |  |

| `record_receipt_verified` | function | record_receipt_verified(&self, accepted: bool) |  |  |  |  |

| `record_stage_error` | function | record_stage_error(&self, _stage: &str) |  |  |  |  |

| `record_stage_latency` | function | record_stage_latency(&self, _stage: &str, duration: Duration) |  |  |  |  |

| `render` | function | render(&self) -> String |  |  |  |  |

| `with_window` | function | with_window(window_duration: Duration) -> Self |  |  |  |  |

| `MetricsCollector` | struct | MetricsCollector { inner: Mutex<CollectorState>, window_duration: Duration } |  |  |  |  |

| `PrometheusExporter` | struct | PrometheusExporter { collector: &'a MetricsCollector } |  |  |  |  |

| `ServiceLevelIndicators` | struct | ServiceLevelIndicators { pub latency_p99_ms: f64, pub error_rate_pct: f64, pub availability_pct: f64 } |  |  |  |  |


### wip/5.5_repl_maximalist.rs

| `run` | function | run() -> Result<()> |  |  |  |  |


### wip/B.1_throughput_observability.rs

| `compare_baseline` | function | compare_baseline(&mut self, bench_id: &str, current: f64, baseline: f64) -> anyhow::Result<()> |  |  |  |  |

| `load_baseline_value` | function | load_baseline_value(path: impl AsRef<Path>) -> anyhow::Result<f64> |  |  |  |  |

| `new` | function | new(assembler: &'a mut ChainAssembler, counter: &'a mut SeqCounter) -> Self |  |  |  |  |

| `observe_criterion_bench` | function | observe_criterion_bench(&mut self, criterion_root: &str, bench_id: &str) -> anyhow::Result<f64> |  |  |  |  |

| `record_throughput` | function | record_throughput(&mut self, bench_id: &str, ops_per_sec: f64) -> anyhow::Result<()> |  |  |  |  |

| `ThroughputObserver` | struct | ThroughputObserver { pub assembler: &'a mut ChainAssembler, pub counter: &'a mut SeqCounter } |  |  |  |  |


### wip/B.2_B.3_bench_maximalist.rs

| `CRITERION_CUSTOM_CSS` | const | CRITERION_CUSTOM_CSS: &str |  |  |  |  |

| `create` | str_key | SEQUENTIAL_ACTIVITIES = "create" |  |  |  |  |

| `release` | str_key | INTERLEAVED_ACTIVITIES = "release" |  |  |  |  |


### wip/arch_upgrade_maximalist.rs

| `to_noun_verb_error` | function | to_noun_verb_error(err: AffidavitError) -> NounVerbError |  |  |  |  |



<!-- AGENT-FORBIDDEN-END -->

## Signature/type/default/errors table

<!-- RIGID table: header order is fixed; rows come only from the query. -->

| Item | Type | Signature | Params | Defaults | Errors | Invariants |
|------|------|-----------|--------|----------|--------|------------|

| `MmrError` | enum | MmrError { IndexOutOfBounds { index: u64, total_leaves: u64, }, EmptyMmr, ProofLengthMismatch, PeakCountMismatch } |  |  |  |  |

| `append` | function | append(&mut self, leaf: Digest) -> u64 |  |  |  |  |

| `append_payload` | function | append_payload(&mut self, payload: &[u8]) -> u64 |  |  |  |  |

| `bag_peaks` | function | bag_peaks(peaks: &[Digest]) -> Digest |  |  |  |  |

| `hash_children` | function | hash_children(height: u32, left: &Digest, right: &Digest) -> Digest |  |  |  |  |

| `hash_leaf_payload` | function | hash_leaf_payload(payload: &[u8]) -> Digest |  |  |  |  |

| `mountain_heights` | function | mountain_heights(mut num_leaves: u64) -> Vec<u32> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `num_leaves` | function | num_leaves(&self) -> u64 |  |  |  |  |

| `peaks` | function | peaks(&self) -> Vec<Digest> |  |  |  |  |

| `prove` | function | prove(&self, leaf_index: u64) -> Result<MmrProof, MmrError> |  |  |  |  |

| `root` | function | root(&self) -> Digest |  |  |  |  |

| `verify` | function | verify(&self, leaf_digest: &Digest, expected_root: &Digest) -> bool |  |  |  |  |

| `mmr:v1:bag:` | str_key | DOMAIN_BAG = "mmr:v1:bag:" |  |  |  |  |

| `mmr:v1:leaf:` | str_key | DOMAIN_LEAF = "mmr:v1:leaf:" |  |  |  |  |

| `mmr:v1:node:` | str_key | DOMAIN_NODE = "mmr:v1:node:" |  |  |  |  |

| `MmrAccumulator` | struct | MmrAccumulator { leaves: Vec<Digest>, mountains: Vec<Vec<Vec<Digest>>>, _hasher: PhantomData<H> } |  |  |  |  |

| `MmrProof` | struct | MmrProof { pub leaf_index: u64, pub total_leaves: u64, pub siblings: Vec<Digest>, pub peaks: Vec<Digest> } |  |  |  |  |

| `MountainPeak` | struct | MountainPeak { pub height: u32, pub digest: Digest } |  |  |  |  |

| `MmrError` | enum | MmrError { IndexOutOfBounds { index: u64, total_leaves: u64, }, EmptyMmr, ProofLengthMismatch, PeakCountMismatch } |  |  |  |  |

| `append` | function | append(&mut self, leaf: Digest) -> u64 |  |  |  |  |

| `append_payload` | function | append_payload(&mut self, payload: &[u8]) -> u64 |  |  |  |  |

| `bag_peaks` | function | bag_peaks(peaks: &[Digest]) -> Digest |  |  |  |  |

| `hash_children` | function | hash_children(height: u32, left: &Digest, right: &Digest) -> Digest |  |  |  |  |

| `hash_leaf_payload` | function | hash_leaf_payload(payload: &[u8]) -> Digest |  |  |  |  |

| `mountain_heights` | function | mountain_heights(mut num_leaves: u64) -> Vec<u32> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `num_leaves` | function | num_leaves(&self) -> u64 |  |  |  |  |

| `peaks` | function | peaks(&self) -> Vec<Digest> |  |  |  |  |

| `prove` | function | prove(&self, leaf_index: u64) -> Result<MmrProof, MmrError> |  |  |  |  |

| `root` | function | root(&self) -> Digest |  |  |  |  |

| `verify` | function | verify(&self, leaf_digest: &Digest, expected_root: &Digest) -> bool |  |  |  |  |

| `mmr:v1:bag:` | str_key | DOMAIN_BAG = "mmr:v1:bag:" |  |  |  |  |

| `mmr:v1:leaf:` | str_key | DOMAIN_LEAF = "mmr:v1:leaf:" |  |  |  |  |

| `mmr:v1:node:` | str_key | DOMAIN_NODE = "mmr:v1:node:" |  |  |  |  |

| `MmrAccumulator` | struct | MmrAccumulator { leaves: Vec<Digest>, mountains: Vec<Vec<Vec<Digest>>>, _hasher: PhantomData<H> } |  |  |  |  |

| `MmrProof` | struct | MmrProof { pub leaf_index: u64, pub total_leaves: u64, pub siblings: Vec<Digest>, pub peaks: Vec<Digest> } |  |  |  |  |

| `MountainPeak` | struct | MountainPeak { pub height: u32, pub digest: Digest } |  |  |  |  |

| `mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, }` | use | mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, } |  |  |  |  |

| `mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, }` | use | mmr::{ bag_peaks, hash_children, hash_leaf_payload, mountain_heights, MmrAccumulator, MmrError, MmrProof, MountainPeak, } |  |  |  |  |

| `PROFILE` | const | PROFILE: &str |  |  |  |  |

| `absorb_into` | function | absorb_into(&self, state: &mut H::State) |  |  |  |  |

| `borrow` | function | borrow(&self) -> Event<'_> |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) -> Digest |  |  |  |  |

| `compute_chain_hash` | function | compute_chain_hash(events: &[Event<'_>]) -> Digest |  |  |  |  |

| `event` | function | event(mut self, event_type: &str, event_id: &str, commitment: Digest) -> Self |  |  |  |  |

| `events` | function | events(&self) -> &[OwnedEvent] |  |  |  |  |

| `finalize` | function | finalize(self) -> Receipt |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `profile` | function | profile(&self) -> &str |  |  |  |  |

| `verify` | function | verify(&self) -> Verdict |  |  |  |  |

| `affidavit-core/chain/v1` | str_key | DOMAIN = "affidavit-core/chain/v1" |  |  |  |  |

| `affidavit-core/v1` | str_key | PROFILE = "affidavit-core/v1" |  |  |  |  |

| `ChainBuilder` | struct | ChainBuilder { events: Vec<OwnedEvent>, _hasher: PhantomData<H> } |  |  |  |  |

| `Event` | struct | Event { pub seq: u64, pub event_id: &'a str, pub event_type: &'a str, pub commitment: Digest } |  |  |  |  |

| `OwnedEvent` | struct | OwnedEvent { pub seq: u64, pub event_id: String, pub event_type: String, pub commitment: Digest } |  |  |  |  |

| `Receipt` | struct | Receipt { events: Vec<OwnedEvent>, chain_hash: Digest, profile: &'static str, _seal: Seal } |  |  |  |  |

| `Seal` | struct | Seal |  |  |  |  |

| `owned::{ChainBuilder, OwnedEvent, Receipt, Seal}` | use | owned::{ChainBuilder, OwnedEvent, Receipt, Seal} |  |  |  |  |

| `PROFILE` | const | PROFILE: &str |  |  |  |  |

| `absorb_into` | function | absorb_into(&self, state: &mut H::State) |  |  |  |  |

| `borrow` | function | borrow(&self) -> Event<'_> |  |  |  |  |

| `chain_hash` | function | chain_hash(&self) -> Digest |  |  |  |  |

| `compute_chain_hash` | function | compute_chain_hash(events: &[Event<'_>]) -> Digest |  |  |  |  |

| `event` | function | event(mut self, event_type: &str, event_id: &str, commitment: Digest) -> Self |  |  |  |  |

| `events` | function | events(&self) -> &[OwnedEvent] |  |  |  |  |

| `finalize` | function | finalize(self) -> Receipt |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `profile` | function | profile(&self) -> &str |  |  |  |  |

| `verify` | function | verify(&self) -> Verdict |  |  |  |  |

| `affidavit-core/chain/v1` | str_key | DOMAIN = "affidavit-core/chain/v1" |  |  |  |  |

| `affidavit-core/v1` | str_key | PROFILE = "affidavit-core/v1" |  |  |  |  |

| `ChainBuilder` | struct | ChainBuilder { events: Vec<OwnedEvent>, _hasher: PhantomData<H> } |  |  |  |  |

| `Event` | struct | Event { pub seq: u64, pub event_id: &'a str, pub event_type: &'a str, pub commitment: Digest } |  |  |  |  |

| `OwnedEvent` | struct | OwnedEvent { pub seq: u64, pub event_id: String, pub event_type: String, pub commitment: Digest } |  |  |  |  |

| `Receipt` | struct | Receipt { events: Vec<OwnedEvent>, chain_hash: Digest, profile: &'static str, _seal: Seal } |  |  |  |  |

| `Seal` | struct | Seal |  |  |  |  |

| `owned::{ChainBuilder, OwnedEvent, Receipt, Seal}` | use | owned::{ChainBuilder, OwnedEvent, Receipt, Seal} |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_FIELDS` | const | ENVELOPE_FIELDS: [(&str, u32); 12] |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `Algorithm` | enum | Algorithm { Es256, HybridEs256MlDsa65, MlDsa65, SlhDsa128s } |  |  |  |  |

| `EnvelopeError` | enum | EnvelopeError { Malformed(&'static str), WrongVersion, NonCanonicalNumber(&'static str), BufferTooSmall { needed: usize, given: usize, }, NotYetValid(u64), Expired(u64) } |  |  |  |  |

| `Profile` | enum | Profile { Classical, Hybrid, Pqc } |  |  |  |  |

| `borrow` | function | borrow(&self) -> EnvelopeRef<'_> |  |  |  |  |

| `canonical_len` | function | canonical_len(&self) -> usize |  |  |  |  |

| `check_canonical` | function | check_canonical(&self) -> Result<(), EnvelopeError> |  |  |  |  |

| `from_json` | function | from_json(input: &'a [u8]) -> Result<Self, EnvelopeError> |  |  |  |  |

| `from_wire` | function | from_wire(s: &str) -> Result<Self, EnvelopeError> |  |  |  |  |

| `signing_input` | function | signing_input(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> |  |  |  |  |

| `signing_input_len` | function | signing_input_len(&self) -> usize |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) -> Result<(), EnvelopeError> |  |  |  |  |

| `wire_str` | function | wire_str(self) -> &'static str |  |  |  |  |

| `write_canonical` | function | write_canonical(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> |  |  |  |  |

| `write_signing_input` | function | write_signing_input(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V1_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V2_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `version` | str_key | ENVELOPE_FIELDS = "version" |  |  |  |  |

| `EnvelopeRef` | struct | EnvelopeRef { pub version: &'a str, pub algorithm: Algorithm, pub key_id: &'a str, pub profile: Profile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: &'a str } |  |  |  |  |

| `SignatureEnvelope` | struct | SignatureEnvelope { pub version: alloc::string::String, pub algorithm: Algorithm, pub key_id: alloc::string::String, pub profile: Profile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: alloc::string::String } |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_FIELDS` | const | ENVELOPE_FIELDS: [(&str, u32); 12] |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `Algorithm` | enum | Algorithm { Es256, HybridEs256MlDsa65, MlDsa65, SlhDsa128s } |  |  |  |  |

| `EnvelopeError` | enum | EnvelopeError { Malformed(&'static str), WrongVersion, NonCanonicalNumber(&'static str), BufferTooSmall { needed: usize, given: usize, }, NotYetValid(u64), Expired(u64) } |  |  |  |  |

| `Profile` | enum | Profile { Classical, Hybrid, Pqc } |  |  |  |  |

| `borrow` | function | borrow(&self) -> EnvelopeRef<'_> |  |  |  |  |

| `canonical_len` | function | canonical_len(&self) -> usize |  |  |  |  |

| `check_canonical` | function | check_canonical(&self) -> Result<(), EnvelopeError> |  |  |  |  |

| `from_json` | function | from_json(input: &'a [u8]) -> Result<Self, EnvelopeError> |  |  |  |  |

| `from_wire` | function | from_wire(s: &str) -> Result<Self, EnvelopeError> |  |  |  |  |

| `signing_input` | function | signing_input(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> |  |  |  |  |

| `signing_input_len` | function | signing_input_len(&self) -> usize |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) -> Result<alloc::vec::Vec<u8>, EnvelopeError> |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) -> Result<(), EnvelopeError> |  |  |  |  |

| `wire_str` | function | wire_str(self) -> &'static str |  |  |  |  |

| `write_canonical` | function | write_canonical(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> |  |  |  |  |

| `write_signing_input` | function | write_signing_input(&self, out: &mut [u8]) -> Result<usize, EnvelopeError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V1_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V2_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `version` | str_key | ENVELOPE_FIELDS = "version" |  |  |  |  |

| `EnvelopeRef` | struct | EnvelopeRef { pub version: &'a str, pub algorithm: Algorithm, pub key_id: &'a str, pub profile: Profile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: &'a str } |  |  |  |  |

| `SignatureEnvelope` | struct | SignatureEnvelope { pub version: alloc::string::String, pub algorithm: Algorithm, pub key_id: alloc::string::String, pub profile: Profile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: alloc::string::String } |  |  |  |  |

| `ZERO` | const | ZERO: Digest |  |  |  |  |

| `as_bytes` | function | as_bytes(&self) -> &[u8; 32] |  |  |  |  |

| `is_zero` | function | is_zero(&self) -> bool |  |  |  |  |

| `Digest` | struct | Digest { pub [u8; 32] } |  |  |  |  |

| `Fnv256` | struct |  |  |  |  |  |

| `ChainHasher` | trait |  |  |  |  |  |

| `ZERO` | const | ZERO: Digest |  |  |  |  |

| `as_bytes` | function | as_bytes(&self) -> &[u8; 32] |  |  |  |  |

| `is_zero` | function | is_zero(&self) -> bool |  |  |  |  |

| `Digest` | struct | Digest { pub [u8; 32] } |  |  |  |  |

| `Fnv256` | struct |  |  |  |  |  |

| `ChainHasher` | trait |  |  |  |  |  |

| `AUTHORITY_NONE` | const | AUTHORITY_NONE: &str |  |  |  |  |

| `AUTHZEN_STANDARD` | const | AUTHZEN_STANDARD: &str |  |  |  |  |

| `CONSEQUENCE_EVIDENCE_ONLY` | const | CONSEQUENCE_EVIDENCE_ONLY: &str |  |  |  |  |

| `SPIFFE_PREFIX` | const | SPIFFE_PREFIX: &str |  |  |  |  |

| `EvidenceError` | enum | EvidenceError { EmptyField(&'static str), InvalidSpiffeScheme, MissingTrustDomain, InvalidSpiffeAuthority, SpiffeQueryOrFragment, SpiffeIdMismatch, TrustDomainMismatch, WorkloadNotVerified, JwtNotAdmitted, InvalidPolicyDecisionPoint, PolicyDecisionPointMismatch, PrincipalMismatch, EffectMismatch } |  |  |  |  |

| `SvidType` | enum | SvidType { X509, Jwt } |  |  |  |  |

| `admit_authzen_evidence` | function | admit_authzen_evidence( request: AuthZenRequestRef<'_>, decision: AuthZenDecisionEvidenceRef<'_>, expected_policy_decision_point: &str, expected_principal: &str, expected_effect_digest: &str, ) -> Result<(), EvidenceError> |  |  |  |  |

| `admit_workload_identity` | function | admit_workload_identity( evidence: WorkloadIdentityEvidenceRef<'_>, expected_spiffe_id: &str, expected_trust_domain: &str, allow_jwt: bool, ) -> Result<(), EvidenceError> |  |  |  |  |

| `code` | function | code(self) -> &'static str |  |  |  |  |

| `parse` | function | parse(raw: &'a str) -> Result<Self, EvidenceError> |  |  |  |  |

| `EVIDENCE_ONLY` | str_key | CONSEQUENCE_EVIDENCE_ONLY = "EVIDENCE_ONLY" |  |  |  |  |

| `NONE` | str_key | AUTHORITY_NONE = "NONE" |  |  |  |  |

| `OpenID AuthZEN Authorization API 1.0` | str_key | AUTHZEN_STANDARD = "OpenID AuthZEN Authorization API 1.0" |  |  |  |  |

| `spiffe://` | str_key | SPIFFE_PREFIX = "spiffe://" |  |  |  |  |

| `AuthZenActionRef` | struct | AuthZenActionRef { pub name: &'a str } |  |  |  |  |

| `AuthZenDecisionEvidenceRef` | struct | AuthZenDecisionEvidenceRef { pub decision: bool, pub policy_decision_point: &'a str } |  |  |  |  |

| `AuthZenEntityRef` | struct | AuthZenEntityRef { pub entity_type: &'a str, pub id: &'a str } |  |  |  |  |

| `AuthZenRequestRef` | struct | AuthZenRequestRef { pub subject: AuthZenEntityRef<'a>, pub action: AuthZenActionRef<'a>, pub resource: AuthZenEntityRef<'a> } |  |  |  |  |

| `SpiffeIdRef` | struct | SpiffeIdRef { pub uri: &'a str, pub trust_domain: &'a str, pub path: &'a str } |  |  |  |  |

| `WorkloadIdentityEvidenceRef` | struct | WorkloadIdentityEvidenceRef { pub identity: SpiffeIdRef<'a>, pub svid_type: SvidType, pub bundle_digest: &'a str, pub verified: bool } |  |  |  |  |

| `AUTHORITY_NONE` | const | AUTHORITY_NONE: &str |  |  |  |  |

| `AUTHZEN_STANDARD` | const | AUTHZEN_STANDARD: &str |  |  |  |  |

| `CONSEQUENCE_EVIDENCE_ONLY` | const | CONSEQUENCE_EVIDENCE_ONLY: &str |  |  |  |  |

| `SPIFFE_PREFIX` | const | SPIFFE_PREFIX: &str |  |  |  |  |

| `EvidenceError` | enum | EvidenceError { EmptyField(&'static str), InvalidSpiffeScheme, MissingTrustDomain, InvalidSpiffeAuthority, SpiffeQueryOrFragment, SpiffeIdMismatch, TrustDomainMismatch, WorkloadNotVerified, JwtNotAdmitted, InvalidPolicyDecisionPoint, PolicyDecisionPointMismatch, PrincipalMismatch, EffectMismatch } |  |  |  |  |

| `SvidType` | enum | SvidType { X509, Jwt } |  |  |  |  |

| `admit_authzen_evidence` | function | admit_authzen_evidence( request: AuthZenRequestRef<'_>, decision: AuthZenDecisionEvidenceRef<'_>, expected_policy_decision_point: &str, expected_principal: &str, expected_effect_digest: &str, ) -> Result<(), EvidenceError> |  |  |  |  |

| `admit_workload_identity` | function | admit_workload_identity( evidence: WorkloadIdentityEvidenceRef<'_>, expected_spiffe_id: &str, expected_trust_domain: &str, allow_jwt: bool, ) -> Result<(), EvidenceError> |  |  |  |  |

| `code` | function | code(self) -> &'static str |  |  |  |  |

| `parse` | function | parse(raw: &'a str) -> Result<Self, EvidenceError> |  |  |  |  |

| `EVIDENCE_ONLY` | str_key | CONSEQUENCE_EVIDENCE_ONLY = "EVIDENCE_ONLY" |  |  |  |  |

| `NONE` | str_key | AUTHORITY_NONE = "NONE" |  |  |  |  |

| `OpenID AuthZEN Authorization API 1.0` | str_key | AUTHZEN_STANDARD = "OpenID AuthZEN Authorization API 1.0" |  |  |  |  |

| `spiffe://` | str_key | SPIFFE_PREFIX = "spiffe://" |  |  |  |  |

| `AuthZenActionRef` | struct | AuthZenActionRef { pub name: &'a str } |  |  |  |  |

| `AuthZenDecisionEvidenceRef` | struct | AuthZenDecisionEvidenceRef { pub decision: bool, pub policy_decision_point: &'a str } |  |  |  |  |

| `AuthZenEntityRef` | struct | AuthZenEntityRef { pub entity_type: &'a str, pub id: &'a str } |  |  |  |  |

| `AuthZenRequestRef` | struct | AuthZenRequestRef { pub subject: AuthZenEntityRef<'a>, pub action: AuthZenActionRef<'a>, pub resource: AuthZenEntityRef<'a> } |  |  |  |  |

| `SpiffeIdRef` | struct | SpiffeIdRef { pub uri: &'a str, pub trust_domain: &'a str, pub path: &'a str } |  |  |  |  |

| `WorkloadIdentityEvidenceRef` | struct | WorkloadIdentityEvidenceRef { pub identity: SpiffeIdRef<'a>, pub svid_type: SvidType, pub bundle_digest: &'a str, pub verified: bool } |  |  |  |  |

| `accumulator::{MmrAccumulator, MmrError, MmrProof}` | use | accumulator::{MmrAccumulator, MmrError, MmrProof} |  |  |  |  |

| `chain::{ChainBuilder, OwnedEvent, Receipt}` | use | chain::{ChainBuilder, OwnedEvent, Receipt} |  |  |  |  |

| `chain::{compute_chain_hash, Event, PROFILE}` | use | chain::{compute_chain_hash, Event, PROFILE} |  |  |  |  |

| `crypto_verify::SignatureEnvelope` | use | crypto_verify::SignatureEnvelope |  |  |  |  |

| `crypto_verify::{ Algorithm, EnvelopeError, EnvelopeRef, Profile, DOMAIN_TAG, ENVELOPE_FIELDS, ENVELOPE_VERSION, }` | use | crypto_verify::{ Algorithm, EnvelopeError, EnvelopeRef, Profile, DOMAIN_TAG, ENVELOPE_FIELDS, ENVELOPE_VERSION, } |  |  |  |  |

| `digest::{ChainHasher, Digest, Fnv256}` | use | digest::{ChainHasher, Digest, Fnv256} |  |  |  |  |

| `external_evidence::{ admit_authzen_evidence, admit_workload_identity, AuthZenActionRef, AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef, SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD, CONSEQUENCE_EVIDENCE_ONLY, SPIFFE_PREFIX, }` | use | external_evidence::{ admit_authzen_evidence, admit_workload_identity, AuthZenActionRef, AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef, SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD, CONSEQUENCE_EVIDENCE_ONLY, SPIFFE_PREFIX, } |  |  |  |  |

| `mining::{DirectlyFollowsGraph, Trace}` | use | mining::{DirectlyFollowsGraph, Trace} |  |  |  |  |

| `verifier::{verify, RejectReason, Verdict}` | use | verifier::{verify, RejectReason, Verdict} |  |  |  |  |

| `accumulator::{MmrAccumulator, MmrError, MmrProof}` | use | accumulator::{MmrAccumulator, MmrError, MmrProof} |  |  |  |  |

| `chain::{ChainBuilder, OwnedEvent, Receipt}` | use | chain::{ChainBuilder, OwnedEvent, Receipt} |  |  |  |  |

| `chain::{compute_chain_hash, Event, PROFILE}` | use | chain::{compute_chain_hash, Event, PROFILE} |  |  |  |  |

| `crypto_verify::SignatureEnvelope` | use | crypto_verify::SignatureEnvelope |  |  |  |  |

| `crypto_verify::{ Algorithm, EnvelopeError, EnvelopeRef, Profile, DOMAIN_TAG, ENVELOPE_FIELDS, ENVELOPE_VERSION, }` | use | crypto_verify::{ Algorithm, EnvelopeError, EnvelopeRef, Profile, DOMAIN_TAG, ENVELOPE_FIELDS, ENVELOPE_VERSION, } |  |  |  |  |

| `digest::{ChainHasher, Digest, Fnv256}` | use | digest::{ChainHasher, Digest, Fnv256} |  |  |  |  |

| `external_evidence::{ admit_authzen_evidence, admit_workload_identity, AuthZenActionRef, AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef, SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD, CONSEQUENCE_EVIDENCE_ONLY, SPIFFE_PREFIX, }` | use | external_evidence::{ admit_authzen_evidence, admit_workload_identity, AuthZenActionRef, AuthZenDecisionEvidenceRef, AuthZenEntityRef, AuthZenRequestRef, EvidenceError, SpiffeIdRef, SvidType, WorkloadIdentityEvidenceRef, AUTHORITY_NONE, AUTHZEN_STANDARD, CONSEQUENCE_EVIDENCE_ONLY, SPIFFE_PREFIX, } |  |  |  |  |

| `mining::{DirectlyFollowsGraph, Trace}` | use | mining::{DirectlyFollowsGraph, Trace} |  |  |  |  |

| `verifier::{verify, RejectReason, Verdict}` | use | verifier::{verify, RejectReason, Verdict} |  |  |  |  |

| `ConformanceVerdict` | enum | ConformanceVerdict { Conformant, NonConformant } |  |  |  |  |

| `fitness` | function | fitness(&self) -> f64 |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) -> bool |  |  |  |  |

| `replay` | function | replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) -> ConformanceResult |  |  |  |  |

| `verdict` | function | verdict(&self) -> ConformanceVerdict |  |  |  |  |

| `ConformanceResult` | struct | ConformanceResult { pub legal_moves: u64, pub total_moves: u64, pub start_ok: bool, pub end_ok: bool, pub first_violation: Option<usize> } |  |  |  |  |

| `ConformanceVerdict` | enum | ConformanceVerdict { Conformant, NonConformant } |  |  |  |  |

| `fitness` | function | fitness(&self) -> f64 |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) -> bool |  |  |  |  |

| `replay` | function | replay(model: &DirectlyFollowsGraph, trace: &Trace<'_>) -> ConformanceResult |  |  |  |  |

| `verdict` | function | verdict(&self) -> ConformanceVerdict |  |  |  |  |

| `ConformanceResult` | struct | ConformanceResult { pub legal_moves: u64, pub total_moves: u64, pub start_ok: bool, pub end_ok: bool, pub first_violation: Option<usize> } |  |  |  |  |

| `AlphaRelation` | enum | AlphaRelation { Causality, ReverseCausality, Parallel, Choice } |  |  |  |  |

| `activities` | function | activities(&self) -> &[String] |  |  |  |  |

| `from_dfg` | function | from_dfg(dfg: &DirectlyFollowsGraph) -> Self |  |  |  |  |

| `relation` | function | relation(&self, a: &str, b: &str) -> AlphaRelation |  |  |  |  |

| `Footprint` | struct | Footprint { activities: Vec<String>, follows: BTreeSet<(String, String)> } |  |  |  |  |

| `AlphaRelation` | enum | AlphaRelation { Causality, ReverseCausality, Parallel, Choice } |  |  |  |  |

| `activities` | function | activities(&self) -> &[String] |  |  |  |  |

| `from_dfg` | function | from_dfg(dfg: &DirectlyFollowsGraph) -> Self |  |  |  |  |

| `relation` | function | relation(&self, a: &str, b: &str) -> AlphaRelation |  |  |  |  |

| `Footprint` | struct | Footprint { activities: Vec<String>, follows: BTreeSet<(String, String)> } |  |  |  |  |

| `activity_list` | function | activity_list(&self) -> Vec<&str> |  |  |  |  |

| `add_trace` | function | add_trace(&mut self, trace: &Trace<'_>) |  |  |  |  |

| `directly_follows` | function | directly_follows(&self, a: &str, b: &str) -> u64 |  |  |  |  |

| `discover` | function | discover(traces: &[Trace<'_>]) -> Self |  |  |  |  |

| `follows` | function | follows(&self, a: &str, b: &str) -> bool |  |  |  |  |

| `from_activities` | function | from_activities(activities: &[&'a str]) -> Self |  |  |  |  |

| `from_events` | function | from_events(events: &[Event<'a>]) -> Self |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_end` | function | is_end(&self, a: &str) -> bool |  |  |  |  |

| `is_start` | function | is_start(&self, a: &str) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `DirectlyFollowsGraph` | struct | DirectlyFollowsGraph { pub edges: BTreeMap<(String, String), u64>, pub activities: BTreeSet<String>, pub start: BTreeMap<String, u64>, pub end: BTreeMap<String, u64>, pub trace_count: u64 } |  |  |  |  |

| `Trace` | struct | Trace { pub activities: Vec<&'a str> } |  |  |  |  |

| `conformance::{replay, ConformanceResult, ConformanceVerdict}` | use | conformance::{replay, ConformanceResult, ConformanceVerdict} |  |  |  |  |

| `footprint::{AlphaRelation, Footprint}` | use | footprint::{AlphaRelation, Footprint} |  |  |  |  |

| `stats::LogStatistics` | use | stats::LogStatistics |  |  |  |  |

| `activity_list` | function | activity_list(&self) -> Vec<&str> |  |  |  |  |

| `add_trace` | function | add_trace(&mut self, trace: &Trace<'_>) |  |  |  |  |

| `directly_follows` | function | directly_follows(&self, a: &str, b: &str) -> u64 |  |  |  |  |

| `discover` | function | discover(traces: &[Trace<'_>]) -> Self |  |  |  |  |

| `follows` | function | follows(&self, a: &str, b: &str) -> bool |  |  |  |  |

| `from_activities` | function | from_activities(activities: &[&'a str]) -> Self |  |  |  |  |

| `from_events` | function | from_events(events: &[Event<'a>]) -> Self |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_end` | function | is_end(&self, a: &str) -> bool |  |  |  |  |

| `is_start` | function | is_start(&self, a: &str) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `DirectlyFollowsGraph` | struct | DirectlyFollowsGraph { pub edges: BTreeMap<(String, String), u64>, pub activities: BTreeSet<String>, pub start: BTreeMap<String, u64>, pub end: BTreeMap<String, u64>, pub trace_count: u64 } |  |  |  |  |

| `Trace` | struct | Trace { pub activities: Vec<&'a str> } |  |  |  |  |

| `conformance::{replay, ConformanceResult, ConformanceVerdict}` | use | conformance::{replay, ConformanceResult, ConformanceVerdict} |  |  |  |  |

| `footprint::{AlphaRelation, Footprint}` | use | footprint::{AlphaRelation, Footprint} |  |  |  |  |

| `stats::LogStatistics` | use | stats::LogStatistics |  |  |  |  |

| `distinct_activities` | function | distinct_activities(&self) -> usize |  |  |  |  |

| `distinct_variants` | function | distinct_variants(&self) -> usize |  |  |  |  |

| `from_traces` | function | from_traces(traces: &[Trace<'_>]) -> Self |  |  |  |  |

| `most_frequent_variant` | function | most_frequent_variant(&self) -> Option<(&[String], u64)> |  |  |  |  |

| `LogStatistics` | struct | LogStatistics { pub event_count: u64, pub trace_count: u64, pub activity_frequency: BTreeMap<String, u64>, pub variants: BTreeMap<Vec<String>, u64> } |  |  |  |  |

| `distinct_activities` | function | distinct_activities(&self) -> usize |  |  |  |  |

| `distinct_variants` | function | distinct_variants(&self) -> usize |  |  |  |  |

| `from_traces` | function | from_traces(traces: &[Trace<'_>]) -> Self |  |  |  |  |

| `most_frequent_variant` | function | most_frequent_variant(&self) -> Option<(&[String], u64)> |  |  |  |  |

| `LogStatistics` | struct | LogStatistics { pub event_count: u64, pub trace_count: u64, pub activity_frequency: BTreeMap<String, u64>, pub variants: BTreeMap<Vec<String>, u64> } |  |  |  |  |

| `RejectReason` | enum | RejectReason { WrongProfile, ChainHashMismatch, SeqNotContiguous { index: usize, found: u64, }, DuplicateEventId, EmptyEventType { index: usize, }, ZeroCommitment { index: usize, } } |  |  |  |  |

| `Verdict` | enum | Verdict { Accept, Reject(RejectReason) } |  |  |  |  |

| `is_accept` | function | is_accept(&self) -> bool |  |  |  |  |

| `reason` | function | reason(&self) -> Option<RejectReason> |  |  |  |  |

| `verify` | function | verify( events: &[Event<'_>], chain_hash: &crate::digest::Digest, profile: &str, ) -> Verdict |  |  |  |  |

| `RejectReason` | enum | RejectReason { WrongProfile, ChainHashMismatch, SeqNotContiguous { index: usize, found: u64, }, DuplicateEventId, EmptyEventType { index: usize, }, ZeroCommitment { index: usize, } } |  |  |  |  |

| `Verdict` | enum | Verdict { Accept, Reject(RejectReason) } |  |  |  |  |

| `is_accept` | function | is_accept(&self) -> bool |  |  |  |  |

| `reason` | function | reason(&self) -> Option<RejectReason> |  |  |  |  |

| `verify` | function | verify( events: &[Event<'_>], chain_hash: &crate::digest::Digest, profile: &str, ) -> Verdict |  |  |  |  |

| `call` | function | call(request: &[u8]) -> Vec<u8> |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) -> Vec<u8> |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() -> Vec<u8> |  |  |  |  |

| `AbiError` | struct | AbiError { pub code: &'static str, pub message: String, pub details: Option<Value> } |  |  |  |  |

| `crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES}` | use | crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES} |  |  |  |  |

| `call` | function | call(request: &[u8]) -> Vec<u8> |  |  |  |  |

| `limit_response` | function | limit_response(name: &str, observed: usize, max: usize) -> Vec<u8> |  |  |  |  |

| `missing_buffer_response` | function | missing_buffer_response() -> Vec<u8> |  |  |  |  |

| `AbiError` | struct | AbiError { pub code: &'static str, pub message: String, pub details: Option<Value> } |  |  |  |  |

| `crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES}` | use | crate::abi_meta::{ABI_VERSION, MAX_REQUEST_BYTES} |  |  |  |  |

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `CRATE_NAME` | const | CRATE_NAME: &str |  |  |  |  |

| `ERROR_CODES` | const | ERROR_CODES: &[&str] |  |  |  |  |

| `EXPORT_PREFIX` | const | EXPORT_PREFIX: &str |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `OPS` | const | OPS: &[&str] |  |  |  |  |

| `af` | str_key | EXPORT_PREFIX = "af" |  |  |  |  |

| `affidavit-wasm` | str_key | CRATE_NAME = "affidavit-wasm" |  |  |  |  |

| `bad_json` | str_key | ERROR_CODES = "bad_json" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `ABI_VERSION` | const | ABI_VERSION: u32 |  |  |  |  |

| `CRATE_NAME` | const | CRATE_NAME: &str |  |  |  |  |

| `ERROR_CODES` | const | ERROR_CODES: &[&str] |  |  |  |  |

| `EXPORT_PREFIX` | const | EXPORT_PREFIX: &str |  |  |  |  |

| `MAX_JSON_DEPTH` | const | MAX_JSON_DEPTH: usize |  |  |  |  |

| `MAX_REQUEST_BYTES` | const | MAX_REQUEST_BYTES: usize |  |  |  |  |

| `OPS` | const | OPS: &[&str] |  |  |  |  |

| `af` | str_key | EXPORT_PREFIX = "af" |  |  |  |  |

| `affidavit-wasm` | str_key | CRATE_NAME = "affidavit-wasm" |  |  |  |  |

| `bad_json` | str_key | ERROR_CODES = "bad_json" |  |  |  |  |

| `capabilities` | str_key | OPS = "capabilities" |  |  |  |  |

| `op_jcs_canonicalize` | function | op_jcs_canonicalize(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_range_proof_verify` | function | op_range_proof_verify(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_smt_absence_verify` | function | op_smt_absence_verify(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_jcs_canonicalize` | function | op_jcs_canonicalize(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_range_proof_verify` | function | op_range_proof_verify(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `op_smt_absence_verify` | function | op_smt_absence_verify(req: &Value) -> Res<Map<String, Value>> |  |  |  |  |

| `SignatureInputError` | enum | SignatureInputError { Envelope(affidavit_core::crypto_verify::EnvelopeError), BadExpectedHex } |  |  |  |  |

| `code` | function | code(&self) -> &'static str |  |  |  |  |

| `derive_subject_digest` | function | derive_subject_digest(domain_tag: &str, subject: &[u8]) -> [u8; 32] |  |  |  |  |

| `verify_signature_input` | function | verify_signature_input( envelope_json: &[u8], expected_signing_input_hex: &str, ) -> Result<bool, SignatureInputError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f` | str_key | KAT_DIGEST_HEX = "3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `affidavit-paid-delivery/v1` | str_key | KAT_TAG = "affidavit-paid-delivery/v1" |  |  |  |  |

| `paid-delivery:evt-001` | str_key | KAT_SUBJECT = "paid-delivery:evt-001" |  |  |  |  |

| `SignatureInputError` | enum | SignatureInputError { Envelope(affidavit_core::crypto_verify::EnvelopeError), BadExpectedHex } |  |  |  |  |

| `code` | function | code(&self) -> &'static str |  |  |  |  |

| `derive_subject_digest` | function | derive_subject_digest(domain_tag: &str, subject: &[u8]) -> [u8; 32] |  |  |  |  |

| `verify_signature_input` | function | verify_signature_input( envelope_json: &[u8], expected_signing_input_hex: &str, ) -> Result<bool, SignatureInputError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f` | str_key | KAT_DIGEST_HEX = "3ca4fb8419bd2319a832f8c9b2a5e2d8f2e8105c17d5fc6115de0238a7ec5e3f" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `affidavit-paid-delivery/v1` | str_key | KAT_TAG = "affidavit-paid-delivery/v1" |  |  |  |  |

| `paid-delivery:evt-001` | str_key | KAT_SUBJECT = "paid-delivery:evt-001" |  |  |  |  |

| `af_abi_version` | function | af_abi_version() -> u32 |  |  |  |  |

| `af_alloc` | function | af_alloc(len: u32) -> *mut u8 |  |  |  |  |

| `af_call` | function | af_call(ptr: *mut u8, len: u32) -> u64 |  |  |  |  |

| `af_free` | function | af_free(ptr: *mut u8, len: u32) |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) -> *mut u8 |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) -> Vec<u8> |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `af_abi_version` | function | af_abi_version() -> u32 |  |  |  |  |

| `af_alloc` | function | af_alloc(len: u32) -> *mut u8 |  |  |  |  |

| `af_call` | function | af_call(ptr: *mut u8, len: u32) -> u64 |  |  |  |  |

| `af_free` | function | af_free(ptr: *mut u8, len: u32) |  |  |  |  |

| `alloc_buf` | function | alloc_buf(len: u32) -> *mut u8 |  |  |  |  |

| `call_buf` | function | call_buf(ptr: *mut u8, len: u32) -> Vec<u8> |  |  |  |  |

| `free_buf` | function | free_buf(ptr: *mut u8, len: u32) |  |  |  |  |

| `FORMAT_VERSION` | const | FORMAT_VERSION: &str |  |  |  |  |

| `GENESIS_SEED` | const | GENESIS_SEED: &str |  |  |  |  |

| `blake3_hex` | function | blake3_hex(bytes: &[u8]) -> String |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) -> Result<String, serde_json::Error> |  |  |  |  |

| `finish` | function | finish(outcomes: Vec<CheckOutcome>) -> Verdict |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) -> Result<String, serde_json::Error> |  |  |  |  |

| `seal` | function | seal(events: Vec<OperationEvent>) -> Result<Receipt, serde_json::Error> |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) -> Verdict |  |  |  |  |

| `affidavit-v` | str_key | GENESIS_SEED = "affidavit-v" |  |  |  |  |

| `core/v1` | str_key | FORMAT_VERSION = "core/v1" |  |  |  |  |

| `CheckOutcome` | struct | CheckOutcome { pub stage: String, pub passed: bool, pub detail: String } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub obj_type: String, pub qualifier: Option<String> } |  |  |  |  |

| `OperationEvent` | struct | OperationEvent { pub id: String, pub seq: u64, pub event_type: String, pub objects: Vec<ObjectRef>, pub payload_commitment: String } |  |  |  |  |

| `Receipt` | struct | Receipt { pub format_version: String, pub events: Vec<OperationEvent>, pub chain_hash: String } |  |  |  |  |

| `Verdict` | struct | Verdict { pub accepted: bool, pub profile: &'static str, pub outcomes: Vec<CheckOutcome>, pub reason: String } |  |  |  |  |

| `FORMAT_VERSION` | const | FORMAT_VERSION: &str |  |  |  |  |

| `GENESIS_SEED` | const | GENESIS_SEED: &str |  |  |  |  |

| `blake3_hex` | function | blake3_hex(bytes: &[u8]) -> String |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) -> Result<String, serde_json::Error> |  |  |  |  |

| `finish` | function | finish(outcomes: Vec<CheckOutcome>) -> Verdict |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) -> Result<String, serde_json::Error> |  |  |  |  |

| `seal` | function | seal(events: Vec<OperationEvent>) -> Result<Receipt, serde_json::Error> |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) -> Verdict |  |  |  |  |

| `affidavit-v` | str_key | GENESIS_SEED = "affidavit-v" |  |  |  |  |

| `core/v1` | str_key | FORMAT_VERSION = "core/v1" |  |  |  |  |

| `CheckOutcome` | struct | CheckOutcome { pub stage: String, pub passed: bool, pub detail: String } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub obj_type: String, pub qualifier: Option<String> } |  |  |  |  |

| `OperationEvent` | struct | OperationEvent { pub id: String, pub seq: u64, pub event_type: String, pub objects: Vec<ObjectRef>, pub payload_commitment: String } |  |  |  |  |

| `Receipt` | struct | Receipt { pub format_version: String, pub events: Vec<OperationEvent>, pub chain_hash: String } |  |  |  |  |

| `Verdict` | struct | Verdict { pub accepted: bool, pub profile: &'static str, pub outcomes: Vec<CheckOutcome>, pub reason: String } |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c` | str_key | ED_KEY = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c" |  |  |  |  |

| `72` | str_key | ED_INPUT = "72" |  |  |  |  |

| `92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00` | str_key | ED_SIG = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c` | str_key | ED_KEY = "3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c" |  |  |  |  |

| `72` | str_key | ED_INPUT = "72" |  |  |  |  |

| `92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00` | str_key | ED_SIG = "92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00" |  |  |  |  |

| `AFFIDAVIT_WASM` | env_key | std::env::var_os("AFFIDAVIT_WASM") |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `call` | function | call(&mut self, request: Value) -> Value |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, request: &[u8]) -> Vec<u8> |  |  |  |  |

| `memory_pages` | function | memory_pages(&self) -> u64 |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `wasm_path` | function | wasm_path() -> &'static PathBuf |  |  |  |  |

| `Host` | struct | Host { pub store: Store<WasiCtx>, pub memory: Memory, pub alloc: TypedFunc<u32, u32>, pub free: TypedFunc<(u32, u32), ()>, pub call: TypedFunc<(u32, u32), u64>, pub abi_version: TypedFunc<(), u32> } |  |  |  |  |

| `AFFIDAVIT_WASM` | env_key | std::env::var_os("AFFIDAVIT_WASM") |  |  |  |  |

| `CARGO` | env_key | std::env::var("CARGO") |  |  |  |  |

| `call` | function | call(&mut self, request: Value) -> Value |  |  |  |  |

| `call_raw` | function | call_raw(&mut self, request: &[u8]) -> Vec<u8> |  |  |  |  |

| `memory_pages` | function | memory_pages(&self) -> u64 |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `wasm_path` | function | wasm_path() -> &'static PathBuf |  |  |  |  |

| `Host` | struct | Host { pub store: Store<WasiCtx>, pub memory: Memory, pub alloc: TypedFunc<u32, u32>, pub free: TypedFunc<(u32, u32), ()>, pub call: TypedFunc<(u32, u32), u64>, pub abi_version: TypedFunc<(), u32> } |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V1_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V2_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V0_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V1_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f39333261343336613734336436376364222c226e6f6e6365223a5b34392c3134382c3234302c3232352c3135342c3234342c3133322c33332c32302c372c3138332c39382c3131382c3231312c34372c35305d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22505143222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b31382c36302c3234392c32382c3132382c3139332c3231312c33382c3132302c3139382c3232322c38302c3136342c38352c34332c35322c31372c37362c342c3137332c3232382c3137352c3234352c3232302c3130342c3131342c3235332c3230362c35352c3234382c3131342c36385d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | V2_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001bc7b22616c676f726974686d223a224859425249445f45533235365f4d4c5f4453413635222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f35316465646462336435333639386636222c226e6f6e6365223a5b3233312c3136322c36342c3131342c34372c3232322c33372c39392c34302c3131302c3132382c38392c3134332c3130322c3135382c34365d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22485942524944222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3130362c3137382c3131392c362c3133362c3131302c3137362c3139352c3135332c35302c3135362c302c33352c3137342c3135332c35382c3235342c3138352c3130362c36372c37312c33362c3136362c3139352c36382c3231332c31392c31332c3232302c39362c3137332c39315d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `../registry/op-examples.json` | str_key | EXAMPLES = "../registry/op-examples.json" |  |  |  |  |

| `../registry/op-examples.json` | str_key | EXAMPLES = "../registry/op-examples.json" |  |  |  |  |

| `AFFIDAVIT_WASM` | env_key | std::env::var_os("AFFIDAVIT_WASM") |  |  |  |  |

| `../registry/ARTIFACTS.sha256` | str_key | ARTIFACTS = "../registry/ARTIFACTS.sha256" |  |  |  |  |

| `../registry/capability-registry.json` | str_key | REGISTRY = "../registry/capability-registry.json" |  |  |  |  |

| `AFFIDAVIT_WASM` | env_key | std::env::var_os("AFFIDAVIT_WASM") |  |  |  |  |

| `../registry/ARTIFACTS.sha256` | str_key | ARTIFACTS = "../registry/ARTIFACTS.sha256" |  |  |  |  |

| `../registry/capability-registry.json` | str_key | REGISTRY = "../registry/capability-registry.json" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | KAT_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `fixtures/golden_receipt.json` | str_key | GOLDEN = "fixtures/golden_receipt.json" |  |  |  |  |

| `fixtures/tampered_receipt.json` | str_key | TAMPERED = "fixtures/tampered_receipt.json" |  |  |  |  |

| `6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d` | str_key | KAT_SIGNING_INPUT_HEX = "6166666964617669742e63727970746f2d74727573742d706c616e652e7631006166666964617669742e63727970746f2d74727573742d706c616e652e76310000000000000001ac7b22616c676f726974686d223a224553323536222c2261756469656e6365223a226166666964617669742e6b6174222c22657870697265735f6174223a343130323434343830302c2267656e65726174696f6e223a312c226b65795f6964223a2261666b315f61396333626434343631393565326532222c226e6f6e6365223a5b32352c3130372c3231372c34362c3134312c39342c39352c35342c3132302c37322c3230342c35392c34392c3135342c3234342c34325d2c226e6f745f6265666f7265223a313730303030303030302c22706f6c6963795f65706f6368223a312c2270726f66696c65223a22434c4153534943414c222c227265766f636174696f6e5f65706f6368223a302c227375626a6563745f646967657374223a5b3230362c36332c31312c32362c3132362c38302c3231382c3234352c3139302c3139312c3131302c3230332c3234302c35342c35322c39342c33362c3134312c32302c38382c352c3132372c36332c3132312c3235302c3132382c3130392c36362c3231302c3131302c36382c39375d2c2276657273696f6e223a224354502d454e56454c4f50452d7631227d" |  |  |  |  |

| `fixtures/golden_receipt.json` | str_key | GOLDEN = "fixtures/golden_receipt.json" |  |  |  |  |

| `fixtures/tampered_receipt.json` | str_key | TAMPERED = "fixtures/tampered_receipt.json" |  |  |  |  |

| `build` | script | next build |  |  |  |  |

| `dev` | script | next dev |  |  |  |  |

| `lint` | script | next lint |  |  |  |  |

| `start` | script | next start |  |  |  |  |

| `BLAKE3` | str_key | BENCH_DIGEST_ALGORITHM = "BLAKE3" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | BENCH_ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s` | str_key | BENCH_ALGORITHM_REGISTRY = "ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s" |  |  |  |  |

| `JCS-RFC8785` | str_key | BENCH_CANONICALIZATION = "JCS-RFC8785" |  |  |  |  |

| `affidavit crypto-trust-plane benchmark message` | str_key | BENCH_MESSAGE = "affidavit crypto-trust-plane benchmark message" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | BENCH_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `create` | str_key | SEQUENTIAL_ACTIVITIES = "create" |  |  |  |  |

| `release` | str_key | INTERLEAVED_ACTIVITIES = "release" |  |  |  |  |

| `crate::provenance::{ content_address, genesis_seed, fold_event, recompute_chain, is_valid_digest, RollingChain, RollingHash, }` | use | crate::provenance::{ content_address, genesis_seed, fold_event, recompute_chain, is_valid_digest, RollingChain, RollingHash, } |  |  |  |  |

| `crate::provenance::{ content_address, genesis_seed, fold_event, recompute_chain, is_valid_digest, RollingChain, RollingHash, }` | use | crate::provenance::{ content_address, genesis_seed, fold_event, recompute_chain, is_valid_digest, RollingChain, RollingHash, } |  |  |  |  |

| `ColorMode` | enum | ColorMode { Auto, Always, Never } |  |  |  |  |

| `OutputFormat` | enum | OutputFormat { Json, Yaml, Text } |  |  |  |  |

| `bold` | function | bold(text: &str) -> String |  |  |  |  |

| `color_enabled` | function | color_enabled(&self) -> bool |  |  |  |  |

| `dim` | function | dim(text: &str) -> String |  |  |  |  |

| `enabled` | function | enabled(&self, is_tty: bool) -> bool |  |  |  |  |

| `green` | function | green(text: &str) -> String |  |  |  |  |

| `init` | function | init(args: &GlobalArgs, service_name: &str) -> Result<()> |  |  |  |  |

| `print_output` | function | print_output(value: &T, format: &OutputFormat, text_fn: F) -> Result<()> |  |  |  |  |

| `red` | function | red(text: &str) -> String |  |  |  |  |

| `yellow` | function | yellow(text: &str) -> String |  |  |  |  |

| `GlobalArgs` | struct | GlobalArgs { pub format: OutputFormat, pub color: ColorMode, pub verbose: u8 } |  |  |  |  |

| `ColorMode` | enum | ColorMode { Auto, Always, Never } |  |  |  |  |

| `OutputFormat` | enum | OutputFormat { Json, Yaml, Text } |  |  |  |  |

| `bold` | function | bold(text: &str) -> String |  |  |  |  |

| `color_enabled` | function | color_enabled(&self) -> bool |  |  |  |  |

| `dim` | function | dim(text: &str) -> String |  |  |  |  |

| `enabled` | function | enabled(&self, is_tty: bool) -> bool |  |  |  |  |

| `green` | function | green(text: &str) -> String |  |  |  |  |

| `init` | function | init(args: &GlobalArgs, service_name: &str) -> Result<()> |  |  |  |  |

| `print_output` | function | print_output(value: &T, format: &OutputFormat, text_fn: F) -> Result<()> |  |  |  |  |

| `red` | function | red(text: &str) -> String |  |  |  |  |

| `yellow` | function | yellow(text: &str) -> String |  |  |  |  |

| `GlobalArgs` | struct | GlobalArgs { pub format: OutputFormat, pub color: ColorMode, pub verbose: u8 } |  |  |  |  |

| `Error` | enum | Error { Message(String), Io( Json( } |  |  |  |  |

| `msg` | function | msg(m: impl Into<String>) -> Self |  |  |  |  |

| `Error` | enum | Error { Message(String), Io( Json( } |  |  |  |  |

| `msg` | function | msg(m: impl Into<String>) -> Self |  |  |  |  |

| `chain::RollingChain` | use | chain::RollingChain |  |  |  |  |

| `error::{Error, Result}` | use | error::{Error, Result} |  |  |  |  |

| `chain::RollingChain` | use | chain::RollingChain |  |  |  |  |

| `error::{Error, Result}` | use | error::{Error, Result} |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) -> String |  |  |  |  |

| `current` | function | current(&self) -> &str |  |  |  |  |

| `finalize` | function | finalize(self) -> String |  |  |  |  |

| `fold_event` | function | fold_event(prev_hex: &str, payload: &[u8]) -> String |  |  |  |  |

| `genesis_seed` | function | genesis_seed(domain: &str) -> String |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_valid_digest` | function | is_valid_digest(s: &str) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(domain: &str) -> Self |  |  |  |  |

| `push` | function | push(&mut self, payload: &[u8]) |  |  |  |  |

| `recompute_chain` | function | recompute_chain( domain: &str, payloads: impl IntoIterator<Item = &'a [u8]>, ) -> String |  |  |  |  |

| `update` | function | update(&mut self, bytes: &[u8]) |  |  |  |  |

| `RollingChain` | struct | RollingChain { running: String, count: usize } |  |  |  |  |

| `RollingHash` | struct | RollingHash { hasher: blake3::Hasher } |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) -> String |  |  |  |  |

| `current` | function | current(&self) -> &str |  |  |  |  |

| `finalize` | function | finalize(self) -> String |  |  |  |  |

| `fold_event` | function | fold_event(prev_hex: &str, payload: &[u8]) -> String |  |  |  |  |

| `genesis_seed` | function | genesis_seed(domain: &str) -> String |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_valid_digest` | function | is_valid_digest(s: &str) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(domain: &str) -> Self |  |  |  |  |

| `push` | function | push(&mut self, payload: &[u8]) |  |  |  |  |

| `recompute_chain` | function | recompute_chain( domain: &str, payloads: impl IntoIterator<Item = &'a [u8]>, ) -> String |  |  |  |  |

| `update` | function | update(&mut self, bytes: &[u8]) |  |  |  |  |

| `RollingChain` | struct | RollingChain { running: String, count: usize } |  |  |  |  |

| `RollingHash` | struct | RollingHash { hasher: blake3::Hasher } |  |  |  |  |

| `init_otel` | function | init_otel(service_name: &str, endpoint: &str) -> Result<()> |  |  |  |  |

| `init_tracing` | function | init_tracing(service_name: &str) -> Result<()> |  |  |  |  |

| `new` | function | new(service_name: &str) -> Result<Self> |  |  |  |  |

| `TracingGuard` | struct | TracingGuard { _private: () } |  |  |  |  |

| `init_otel` | function | init_otel(service_name: &str, endpoint: &str) -> Result<()> |  |  |  |  |

| `init_tracing` | function | init_tracing(service_name: &str) -> Result<()> |  |  |  |  |

| `new` | function | new(service_name: &str) -> Result<Self> |  |  |  |  |

| `TracingGuard` | struct | TracingGuard { _private: () } |  |  |  |  |

| `UPDATE_GOLDEN` | env_key | std::env::var("UPDATE_GOLDEN") |  |  |  |  |

| `UPDATE_SNAPSHOTS` | env_key | std::env::var("UPDATE_SNAPSHOTS") |  |  |  |  |

| `assert_golden` | function | assert_golden(actual: &[u8], path: &Path) -> Result<()> |  |  |  |  |

| `assert_snapshot` | function | assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) |  |  |  |  |

| `build` | function | build(self) -> Result<TempReceipt> |  |  |  |  |

| `builder` | function | builder() -> TempReceiptBuilder |  |  |  |  |

| `chain_hash` | function | chain_hash(mut self, h: impl Into<String>) -> Self |  |  |  |  |

| `deterministic_uuid` | function | deterministic_uuid(seed: &str) -> String |  |  |  |  |

| `dir` | function | dir(&self) -> &Path |  |  |  |  |

| `event` | function | event(mut self, event: serde_json::Value) -> Self |  |  |  |  |

| `filename` | function | filename(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `format_version` | function | format_version(mut self, v: impl Into<String>) -> Self |  |  |  |  |

| `path` | function | path(&self) -> PathBuf |  |  |  |  |

| `profile` | function | profile(mut self, p: impl Into<String>) -> Self |  |  |  |  |

| `TempReceipt` | struct | TempReceipt { dir: tempfile::TempDir, filename: String } |  |  |  |  |

| `TempReceiptBuilder` | struct | TempReceiptBuilder { format_version: String, chain_hash: String, events: Vec<serde_json::Value>, profile: String, filename: String } |  |  |  |  |

| `tempfile` | use | tempfile |  |  |  |  |

| `UPDATE_GOLDEN` | env_key | std::env::var("UPDATE_GOLDEN") |  |  |  |  |

| `UPDATE_SNAPSHOTS` | env_key | std::env::var("UPDATE_SNAPSHOTS") |  |  |  |  |

| `assert_golden` | function | assert_golden(actual: &[u8], path: &Path) -> Result<()> |  |  |  |  |

| `assert_snapshot` | function | assert_snapshot(name: &str, actual: &str, snapshots_dir: &Path) |  |  |  |  |

| `build` | function | build(self) -> Result<TempReceipt> |  |  |  |  |

| `builder` | function | builder() -> TempReceiptBuilder |  |  |  |  |

| `chain_hash` | function | chain_hash(mut self, h: impl Into<String>) -> Self |  |  |  |  |

| `deterministic_uuid` | function | deterministic_uuid(seed: &str) -> String |  |  |  |  |

| `dir` | function | dir(&self) -> &Path |  |  |  |  |

| `event` | function | event(mut self, event: serde_json::Value) -> Self |  |  |  |  |

| `filename` | function | filename(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `format_version` | function | format_version(mut self, v: impl Into<String>) -> Self |  |  |  |  |

| `path` | function | path(&self) -> PathBuf |  |  |  |  |

| `profile` | function | profile(mut self, p: impl Into<String>) -> Self |  |  |  |  |

| `TempReceipt` | struct | TempReceipt { dir: tempfile::TempDir, filename: String } |  |  |  |  |

| `TempReceiptBuilder` | struct | TempReceiptBuilder { format_version: String, chain_hash: String, events: Vec<serde_json::Value>, profile: String, filename: String } |  |  |  |  |

| `tempfile` | use | tempfile |  |  |  |  |

| `GENESIS_SEED` | const | GENESIS_SEED: &[u8] |  |  |  |  |

| `append` | function | append(&mut self, event_bytes: &[u8]) -> Blake3Hash |  |  |  |  |

| `finalize` | function | finalize(self) -> String |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[impl AsRef<[u8]>]) -> String |  |  |  |  |

| `{{project-name}}-v` | str_key | GENESIS_SEED_STR = "{{project-name}}-v" |  |  |  |  |

| `ChainAssembler` | struct | ChainAssembler { running: Blake3Hash } |  |  |  |  |

| `GENESIS_SEED` | const | GENESIS_SEED: &[u8] |  |  |  |  |

| `append` | function | append(&mut self, event_bytes: &[u8]) -> Blake3Hash |  |  |  |  |

| `finalize` | function | finalize(self) -> String |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[impl AsRef<[u8]>]) -> String |  |  |  |  |

| `{{project-name}}-v` | str_key | GENESIS_SEED_STR = "{{project-name}}-v" |  |  |  |  |

| `ChainAssembler` | struct | ChainAssembler { running: Blake3Hash } |  |  |  |  |

| `ColorMode` | enum | ColorMode { Auto, On, Off } |  |  |  |  |

| `OutputFormat` | enum | OutputFormat { Json, Yaml, Text } |  |  |  |  |

| `NO_COLOR` | env_key | std::env::var("NO_COLOR") |  |  |  |  |

| `enabled` | function | enabled(self) -> bool |  |  |  |  |

| `print_output` | function | print_output(value: &T, format: OutputFormat) -> Result<()> |  |  |  |  |

| `Cli` | struct | Cli { long, global = true, value_enum, default_value_t = OutputFormat::Text, env = "OUTPUT_FORMAT" )] pub format: OutputFormat, long, global = true, value_enum, default_value_t = ColorMode::Auto, env = "COLOR_MODE" )] pub color: ColorMode, pub verbose: bool } |  |  |  |  |

| `ColorMode` | enum | ColorMode { Auto, On, Off } |  |  |  |  |

| `OutputFormat` | enum | OutputFormat { Json, Yaml, Text } |  |  |  |  |

| `NO_COLOR` | env_key | std::env::var("NO_COLOR") |  |  |  |  |

| `enabled` | function | enabled(self) -> bool |  |  |  |  |

| `print_output` | function | print_output(value: &T, format: OutputFormat) -> Result<()> |  |  |  |  |

| `Cli` | struct | Cli { long, global = true, value_enum, default_value_t = OutputFormat::Text, env = "OUTPUT_FORMAT" )] pub format: OutputFormat, long, global = true, value_enum, default_value_t = ColorMode::Auto, env = "COLOR_MODE" )] pub color: ColorMode, pub verbose: bool } |  |  |  |  |

| `AppError` | enum | AppError { Io( Serde( Other(String), } |  |  |  |  |

| `AppError` | enum | AppError { Io( Serde( Other(String), } |  |  |  |  |

| `error::AppError` | use | error::AppError |  |  |  |  |

| `types::{Blake3Hash, ObjectRef, canonical_bytes}` | use | types::{Blake3Hash, ObjectRef, canonical_bytes} |  |  |  |  |

| `error::AppError` | use | error::AppError |  |  |  |  |

| `types::{Blake3Hash, ObjectRef, canonical_bytes}` | use | types::{Blake3Hash, ObjectRef, canonical_bytes} |  |  |  |  |

| `ProfileId` | enum | ProfileId { CoreV1 } |  |  |  |  |

| `as_hex` | function | as_hex(&self) -> &str |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) -> Self |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) -> Self |  |  |  |  |

| `Blake3Hash` | struct | Blake3Hash { pub String } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub type_: String, pub qualifier: Option<String> } |  |  |  |  |

| `ProfileId` | enum | ProfileId { CoreV1 } |  |  |  |  |

| `as_hex` | function | as_hex(&self) -> &str |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `content_address` | function | content_address(bytes: &[u8]) -> Self |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) -> Self |  |  |  |  |

| `Blake3Hash` | struct | Blake3Hash { pub String } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub type_: String, pub qualifier: Option<String> } |  |  |  |  |

| `new` | function | new(receipt_path: impl AsRef<Path>, source_dir: impl AsRef<Path>) -> Self |  |  |  |  |

| `remediate` | function | remediate(&self) -> Result<String> |  |  |  |  |

| `AutoRemediator` | struct | AutoRemediator { pub receipt_path: PathBuf, pub source_dir: PathBuf } |  |  |  |  |

| `audit_workspace` | function | audit_workspace(&self) -> anyhow::Result<Vec<GovernanceReport>> |  |  |  |  |

| `handle_governance_audit` | function | handle_governance_audit() -> anyhow::Result<String> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `ArchitectureProposal` | struct | ArchitectureProposal { pub rule_reference: String, pub root_cause: String, pub proposed_change: String, pub confidence: f64 } |  |  |  |  |

| `GovernanceAgent` | struct | GovernanceAgent { receipts_dir: PathBuf, rules_path: PathBuf } |  |  |  |  |

| `GovernanceReport` | struct | GovernanceReport { pub receipt_id: String, pub verdict: Verdict, pub proposals: Vec<ArchitectureProposal> } |  |  |  |  |

| `verify_with_chaos` | function | verify_with_chaos(&self, receipt: &Receipt) -> Result<Verdict, String> |  |  |  |  |

| `ChaosVerifier` | struct | ChaosVerifier { pub latency_ms: u64, pub corruption_probability: f64, pub handle_drop_probability: f64 } |  |  |  |  |

| `new` | function | new(root: P) -> Self |  |  |  |  |

| `predict` | function | predict(&self) -> Option<Prediction> |  |  |  |  |

| `run_cli` | function | run_cli() |  |  |  |  |

| `shell_integration` | function | shell_integration() -> &'static str |  |  |  |  |

| `Prediction` | struct | Prediction { pub command: String, pub confidence: f64, pub reason: String, pub category: String } |  |  |  |  |

| `Telepathy` | struct | Telepathy { workspace_root: PathBuf, rules: Vec<Box<dyn TelepathyRule>> } |  |  |  |  |

| `DEFAULT_SHARD_SIZE` | const | DEFAULT_SHARD_SIZE: usize |  |  |  |  |

| `ShardingError` | enum | ShardingError { Dht(String), ChainMismatch { index: usize, expected: String, found: String }, SeqMismatch { index: usize, expected: u64, found: u64 }, BoundaryMismatch { index: usize, next: usize }, Failure(String) } |  |  |  |  |

| `new` | function | new(dht: Arc<dyn KademliaDHT>) -> Self |  |  |  |  |

| `shard_receipt` | function | shard_receipt(receipt: Receipt, shard_size: usize) -> Result<(ReceiptManifest, Vec<ReceiptShard>), crate::error::AffidavitError> |  |  |  |  |

| `verify_distributed` | function | verify_distributed(&self, receipt_id: &Blake3Hash) -> Result<Verdict, ShardingError> |  |  |  |  |

| `DistributedVerifier` | struct | DistributedVerifier { dht: Arc<dyn KademliaDHT> } |  |  |  |  |

| `ReceiptManifest` | struct | ReceiptManifest { pub sharding_version: String, pub receipt_id: Blake3Hash, pub total_events: u64, pub shard_count: usize, pub shard_size: usize, pub final_chain_hash: Blake3Hash } |  |  |  |  |

| `ReceiptShard` | struct | ReceiptShard { pub receipt_id: Blake3Hash, pub shard_index: usize, pub start_seq: u64, pub end_seq: u64, pub prev_chain_hash: Blake3Hash, pub shard_chain_hash: Blake3Hash, pub events: Vec<OperationEvent> } |  |  |  |  |

| `KademliaDHT` | trait |  |  |  |  |  |

| `State` | enum | State { Init = 0, Decode = 1, CheckFormat = 2, ChainIntegrity = 3, Continuity = 4, VerifyCommitments = 5, EvaluateProfile = 6, EmitVerdict = 7, Terminal = 8 } |  |  |  |  |

| `get` | function | get() -> State |  |  |  |  |

| `init` | function | init() |  |  |  |  |

| `terminate` | function | terminate() |  |  |  |  |

| `transition_to` | function | transition_to(expected_prev: State, next: State) |  |  |  |  |

| `CurrentState` | struct |  |  |  |  |  |

| `WGSL_SHADER` | const | WGSL_SHADER: &str |  |  |  |  |

| `is_accepted` | function | is_accepted(&self) -> bool |  |  |  |  |

| `new` | function | new() -> anyhow::Result<Self> |  |  |  |  |

| `prepare_batch` | function | prepare_batch( receipts: &[crate::types::Receipt], ) -> (Vec<GpuEvent>, Vec<GpuReceiptMetadata>) |  |  |  |  |

| `verify_batch` | function | verify_batch( &self, events: &[GpuEvent], metadata: &[GpuReceiptMetadata], ) -> anyhow::Result<Vec<GpuVerdict>> |  |  |  |  |

| `GpuEvent` | struct | GpuEvent { pub seq: u64, pub type_hash: [u32; 8], pub payload_commitment: [u32; 8], pub id_hash: [u32; 8] } |  |  |  |  |

| `GpuReceiptMetadata` | struct | GpuReceiptMetadata { pub event_start: u32, pub event_count: u32, pub expected_chain_hash: [u32; 8], pub format_version_hash: [u32; 8] } |  |  |  |  |

| `GpuVerdict` | struct | GpuVerdict { pub bitmask: u32 } |  |  |  |  |

| `GpuVerifier` | struct | GpuVerifier { device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>, pipeline: wgpu::ComputePipeline, bind_group_layout: wgpu::BindGroupLayout } |  |  |  |  |

| `get_hologram_svg` | function | get_hologram_svg(events: &[OperationEvent]) -> String |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `parse` | function | parse(&self, input: &str) -> OcpqQuery |  |  |  |  |

| `NlpQueryParser` | struct |  |  |  |  |  |

| `AGGREGATION_DURATION_NS` | const | AGGREGATION_DURATION_NS: &str |  |  |  |  |

| `ANOMALY_SCORE` | const | ANOMALY_SCORE: &str |  |  |  |  |

| `ATTRIBUTE_COUNT` | const | ATTRIBUTE_COUNT: &str |  |  |  |  |

| `BASIC_BLOCKS_EXECUTED` | const | BASIC_BLOCKS_EXECUTED: &str |  |  |  |  |

| `BLAKE3_BYTES_HASHED` | const | BLAKE3_BYTES_HASHED: &str |  |  |  |  |

| `BLAKE3_CHUNKS_PROCESSED` | const | BLAKE3_CHUNKS_PROCESSED: &str |  |  |  |  |

| `BLAKE3_HASH_DURATION_NS` | const | BLAKE3_HASH_DURATION_NS: &str |  |  |  |  |

| `BLOCK_SIZE` | const | BLOCK_SIZE: &str |  |  |  |  |

| `BRANCH_MISS_RATE` | const | BRANCH_MISS_RATE: &str |  |  |  |  |

| `BRANCH_PREDICTION_MISSES` | const | BRANCH_PREDICTION_MISSES: &str |  |  |  |  |

| `BRANCH_TOTAL` | const | BRANCH_TOTAL: &str |  |  |  |  |

| `BYTES_READ` | const | BYTES_READ: &str |  |  |  |  |

| `BYTES_WRITTEN` | const | BYTES_WRITTEN: &str |  |  |  |  |

| `CACHE_L1_HITS` | const | CACHE_L1_HITS: &str |  |  |  |  |

| `CACHE_L1_MISSES` | const | CACHE_L1_MISSES: &str |  |  |  |  |

| `CACHE_L2_HITS` | const | CACHE_L2_HITS: &str |  |  |  |  |

| `CACHE_L2_MISSES` | const | CACHE_L2_MISSES: &str |  |  |  |  |

| `CERTIFICATES_CHAIN_LENGTH` | const | CERTIFICATES_CHAIN_LENGTH: &str |  |  |  |  |

| `CERTIFICATES_EXPIRY_DAYS` | const | CERTIFICATES_EXPIRY_DAYS: &str |  |  |  |  |

| `CIPHER_MODE` | const | CIPHER_MODE: &str |  |  |  |  |

| `COMPRESSION_RATIO` | const | COMPRESSION_RATIO: &str |  |  |  |  |

| `CONSISTENCY_CHECK_DURATION_NS` | const | CONSISTENCY_CHECK_DURATION_NS: &str |  |  |  |  |

| `CONSTRAINTS_SATISFIED` | const | CONSTRAINTS_SATISFIED: &str |  |  |  |  |

| `CONSTRAINTS_TOTAL` | const | CONSTRAINTS_TOTAL: &str |  |  |  |  |

| `CONSTRAINTS_VIOLATED` | const | CONSTRAINTS_VIOLATED: &str |  |  |  |  |

| `CONTEXT_SWITCHES_INVOLUNTARY` | const | CONTEXT_SWITCHES_INVOLUNTARY: &str |  |  |  |  |

| `CONTEXT_SWITCHES_VOLUNTARY` | const | CONTEXT_SWITCHES_VOLUNTARY: &str |  |  |  |  |

| `CPU_ID` | const | CPU_ID: &str |  |  |  |  |

| `CRYPTO_BLAKE3` | const | CRYPTO_BLAKE3: &str |  |  |  |  |

| `CYCLES_TOTAL` | const | CYCLES_TOTAL: &str |  |  |  |  |

| `CYCLE_DETECTION_DURATION_NS` | const | CYCLE_DETECTION_DURATION_NS: &str |  |  |  |  |

| `DECRYPTION_DURATION_NS` | const | DECRYPTION_DURATION_NS: &str |  |  |  |  |

| `DISCOVERY_ALGORITHM` | const | DISCOVERY_ALGORITHM: &str |  |  |  |  |

| `DISK_LATENCY_NS` | const | DISK_LATENCY_NS: &str |  |  |  |  |

| `DNS_LOOKUP_DURATION_NS` | const | DNS_LOOKUP_DURATION_NS: &str |  |  |  |  |

| `EDGE_COUNT` | const | EDGE_COUNT: &str |  |  |  |  |

| `ENCRYPTION_DURATION_NS` | const | ENCRYPTION_DURATION_NS: &str |  |  |  |  |

| `ENTROPY_SOURCE` | const | ENTROPY_SOURCE: &str |  |  |  |  |

| `EVENT_COUNT` | const | EVENT_COUNT: &str |  |  |  |  |

| `EVENT_TYPE_COUNT` | const | EVENT_TYPE_COUNT: &str |  |  |  |  |

| `EVIDENCE_COUNT` | const | EVIDENCE_COUNT: &str |  |  |  |  |

| `EVIDENCE_SIZE_BYTES` | const | EVIDENCE_SIZE_BYTES: &str |  |  |  |  |

| `EXECUTION_INNER_LOOP` | const | EXECUTION_INNER_LOOP: &str |  |  |  |  |

| `EXPORTS_INVOKED` | const | EXPORTS_INVOKED: &str |  |  |  |  |

| `FILES_OPENED` | const | FILES_OPENED: &str |  |  |  |  |

| `FILTERING_DURATION_NS` | const | FILTERING_DURATION_NS: &str |  |  |  |  |

| `FORMAT_VERSION` | const | FORMAT_VERSION: &str |  |  |  |  |

| `FS_SYNC_COUNT` | const | FS_SYNC_COUNT: &str |  |  |  |  |

| `FUNCTIONS_INVOKED` | const | FUNCTIONS_INVOKED: &str |  |  |  |  |

| `HASH_THROUGHPUT` | const | HASH_THROUGHPUT: &str |  |  |  |  |

| `HYPOTHESIS_COUNT` | const | HYPOTHESIS_COUNT: &str |  |  |  |  |

| `ID_COLLISIONS` | const | ID_COLLISIONS: &str |  |  |  |  |

| `IMPORTS_FAILED` | const | IMPORTS_FAILED: &str |  |  |  |  |

| `IMPORTS_RESOLVED` | const | IMPORTS_RESOLVED: &str |  |  |  |  |

| `INIT_DURATION_NS` | const | INIT_DURATION_NS: &str |  |  |  |  |

| `INSTRUCTIONS_SKIPPED` | const | INSTRUCTIONS_SKIPPED: &str |  |  |  |  |

| `INSTRUCTIONS_TOTAL` | const | INSTRUCTIONS_TOTAL: &str |  |  |  |  |

| `INSTRUCTIONS_VERIFIED` | const | INSTRUCTIONS_VERIFIED: &str |  |  |  |  |

| `INSTRUCTION_RETIRED` | const | INSTRUCTION_RETIRED: &str |  |  |  |  |

| `IO_WAIT_DURATION_NS` | const | IO_WAIT_DURATION_NS: &str |  |  |  |  |

| `ITERATIONS_COUNT` | const | ITERATIONS_COUNT: &str |  |  |  |  |

| `JIT_CODE_SIZE_BYTES` | const | JIT_CODE_SIZE_BYTES: &str |  |  |  |  |

| `JIT_COMPILE_DURATION_NS` | const | JIT_COMPILE_DURATION_NS: &str |  |  |  |  |

| `JOIN_DURATION_NS` | const | JOIN_DURATION_NS: &str |  |  |  |  |

| `KEY_DERIVATION_DURATION_NS` | const | KEY_DERIVATION_DURATION_NS: &str |  |  |  |  |

| `KEY_ROTATION_COUNT` | const | KEY_ROTATION_COUNT: &str |  |  |  |  |

| `KEY_STRENGTH_BITS` | const | KEY_STRENGTH_BITS: &str |  |  |  |  |

| `LOG_SIZE_BYTES` | const | LOG_SIZE_BYTES: &str |  |  |  |  |

| `LOOP_ITERATIONS` | const | LOOP_ITERATIONS: &str |  |  |  |  |

| `MAC_VERIFY_DURATION_NS` | const | MAC_VERIFY_DURATION_NS: &str |  |  |  |  |

| `MAC_VERIFY_SUCCESS` | const | MAC_VERIFY_SUCCESS: &str |  |  |  |  |

| `MAPPING_DURATION_NS` | const | MAPPING_DURATION_NS: &str |  |  |  |  |

| `MEMORY_ALLOCATED` | const | MEMORY_ALLOCATED: &str |  |  |  |  |

| `MEMORY_FREED` | const | MEMORY_FREED: &str |  |  |  |  |

| `MEMORY_PEAK` | const | MEMORY_PEAK: &str |  |  |  |  |

| `MODULE_HASH` | const | MODULE_HASH: &str |  |  |  |  |

| `MODULE_SIZE_BYTES` | const | MODULE_SIZE_BYTES: &str |  |  |  |  |

| `NESTING_MAX_DEPTH` | const | NESTING_MAX_DEPTH: &str |  |  |  |  |

| `NETWORK_PACKETS_RECEIVED` | const | NETWORK_PACKETS_RECEIVED: &str |  |  |  |  |

| `NETWORK_PACKETS_SENT` | const | NETWORK_PACKETS_SENT: &str |  |  |  |  |

| `NODE_COUNT` | const | NODE_COUNT: &str |  |  |  |  |

| `NONCE_VALUE` | const | NONCE_VALUE: &str |  |  |  |  |

| `OBJECT_COUNT` | const | OBJECT_COUNT: &str |  |  |  |  |

| `OBJECT_TYPE_COUNT` | const | OBJECT_TYPE_COUNT: &str |  |  |  |  |

| `OCEL_EVENT_THROUGHPUT` | const | OCEL_EVENT_THROUGHPUT: &str |  |  |  |  |

| `OCEL_PARSING` | const | OCEL_PARSING: &str |  |  |  |  |

| `PADDING_SCHEME` | const | PADDING_SCHEME: &str |  |  |  |  |

| `PAGES_INITIAL` | const | PAGES_INITIAL: &str |  |  |  |  |

| `PAGES_MAXIMUM` | const | PAGES_MAXIMUM: &str |  |  |  |  |

| `PAGE_FAULTS_MAJOR` | const | PAGE_FAULTS_MAJOR: &str |  |  |  |  |

| `PAGE_FAULTS_MINOR` | const | PAGE_FAULTS_MINOR: &str |  |  |  |  |

| `PARSING_DURATION_NS` | const | PARSING_DURATION_NS: &str |  |  |  |  |

| `PATH_LENGTH` | const | PATH_LENGTH: &str |  |  |  |  |

| `PIPELINE_STALL_DURATION_NS` | const | PIPELINE_STALL_DURATION_NS: &str |  |  |  |  |

| `PROCESS_UPTIME_NS` | const | PROCESS_UPTIME_NS: &str |  |  |  |  |

| `PROJECTION_DURATION_NS` | const | PROJECTION_DURATION_NS: &str |  |  |  |  |

| `PROOF_SIZE_BYTES` | const | PROOF_SIZE_BYTES: &str |  |  |  |  |

| `RANDOM_BYTES_REQUESTED` | const | RANDOM_BYTES_REQUESTED: &str |  |  |  |  |

| `REACHABILITY_CHECK_DURATION_NS` | const | REACHABILITY_CHECK_DURATION_NS: &str |  |  |  |  |

| `RECURSION_DEPTH` | const | RECURSION_DEPTH: &str |  |  |  |  |

| `REFUTATION_COUNT` | const | REFUTATION_COUNT: &str |  |  |  |  |

| `REGISTERS_SPILLS` | const | REGISTERS_SPILLS: &str |  |  |  |  |

| `RELATIONSHIP_COUNT` | const | RELATIONSHIP_COUNT: &str |  |  |  |  |

| `REORDER_BUFFER_OCCUPANCY` | const | REORDER_BUFFER_OCCUPANCY: &str |  |  |  |  |

| `RULES_DURATION_NS` | const | RULES_DURATION_NS: &str |  |  |  |  |

| `RULES_EVALUATED` | const | RULES_EVALUATED: &str |  |  |  |  |

| `RUNTIME_ENGINE` | const | RUNTIME_ENGINE: &str |  |  |  |  |

| `SALT_LENGTH` | const | SALT_LENGTH: &str |  |  |  |  |

| `SATURATION_REACHED` | const | SATURATION_REACHED: &str |  |  |  |  |

| `SCHEMA_VALIDATION_DURATION_NS` | const | SCHEMA_VALIDATION_DURATION_NS: &str |  |  |  |  |

| `SEQUENCE_GAP_COUNT` | const | SEQUENCE_GAP_COUNT: &str |  |  |  |  |

| `SERIALIZATION_DURATION_NS` | const | SERIALIZATION_DURATION_NS: &str |  |  |  |  |

| `SIGNATURE_ALGORITHM` | const | SIGNATURE_ALGORITHM: &str |  |  |  |  |

| `SIGNATURE_VERIFY_DURATION_NS` | const | SIGNATURE_VERIFY_DURATION_NS: &str |  |  |  |  |

| `SOCKET_COUNT` | const | SOCKET_COUNT: &str |  |  |  |  |

| `SORTING_DURATION_NS` | const | SORTING_DURATION_NS: &str |  |  |  |  |

| `SPECULATIVE_EXECUTION_DURATION_NS` | const | SPECULATIVE_EXECUTION_DURATION_NS: &str |  |  |  |  |

| `SPECULATIVE_PATH_COUNT` | const | SPECULATIVE_PATH_COUNT: &str |  |  |  |  |

| `SPECULATIVE_RETIRED_COUNT` | const | SPECULATIVE_RETIRED_COUNT: &str |  |  |  |  |

| `SPECULATIVE_SQUASHED_COUNT` | const | SPECULATIVE_SQUASHED_COUNT: &str |  |  |  |  |

| `STACK_DEPTH_CURRENT` | const | STACK_DEPTH_CURRENT: &str |  |  |  |  |

| `STACK_DEPTH_MAX` | const | STACK_DEPTH_MAX: &str |  |  |  |  |

| `SUB_INSTRUCTION_MICRO_OPS` | const | SUB_INSTRUCTION_MICRO_OPS: &str |  |  |  |  |

| `SYSCALLS_COUNT` | const | SYSCALLS_COUNT: &str |  |  |  |  |

| `TABLE_ELEMENTS` | const | TABLE_ELEMENTS: &str |  |  |  |  |

| `TABLE_GROW_COUNT` | const | TABLE_GROW_COUNT: &str |  |  |  |  |

| `THREAD_ID` | const | THREAD_ID: &str |  |  |  |  |

| `TLB_MISSES` | const | TLB_MISSES: &str |  |  |  |  |

| `TRANSFORMATION_COUNT` | const | TRANSFORMATION_COUNT: &str |  |  |  |  |

| `TRAPS_COUNT` | const | TRAPS_COUNT: &str |  |  |  |  |

| `VERDICT_OUTCOME` | const | VERDICT_OUTCOME: &str |  |  |  |  |

| `VERIFICATION_REACHABILITY` | const | VERIFICATION_REACHABILITY: &str |  |  |  |  |

| `VERIFICATION_SUCCESS_RATE` | const | VERIFICATION_SUCCESS_RATE: &str |  |  |  |  |

| `VERIFY_WASM` | const | VERIFY_WASM: &str |  |  |  |  |

| `WASM_MEMORY_USAGE` | const | WASM_MEMORY_USAGE: &str |  |  |  |  |

| `WITNESS_ID` | const | WITNESS_ID: &str |  |  |  |  |

| `WITNESS_LEVEL` | const | WITNESS_LEVEL: &str |  |  |  |  |

| `all_attribute_keys` | function | all_attribute_keys() -> Vec<&'static str> |  |  |  |  |

| `record_crypto_blake3_maximalist` | function | record_crypto_blake3_maximalist( _bytes_hashed: u64, _duration_ns: u64, ) |  |  |  |  |

| `record_wasm_verify_maximalist` | function | record_wasm_verify_maximalist( _module_hash: &str, _instruction_count: u64, _memory_peak: u64, ) |  |  |  |  |

| `affidavit.crypto.blake3.bytes_hashed` | str_key | BLAKE3_BYTES_HASHED = "affidavit.crypto.blake3.bytes_hashed" |  |  |  |  |

| `affidavit.crypto.blake3.chunks_processed` | str_key | BLAKE3_CHUNKS_PROCESSED = "affidavit.crypto.blake3.chunks_processed" |  |  |  |  |

| `affidavit.crypto.blake3.hash_duration_ns` | str_key | BLAKE3_HASH_DURATION_NS = "affidavit.crypto.blake3.hash_duration_ns" |  |  |  |  |

| `affidavit.crypto.block.size` | str_key | BLOCK_SIZE = "affidavit.crypto.block.size" |  |  |  |  |

| `affidavit.crypto.certificates.chain_length` | str_key | CERTIFICATES_CHAIN_LENGTH = "affidavit.crypto.certificates.chain_length" |  |  |  |  |

| `affidavit.crypto.certificates.expiry_days` | str_key | CERTIFICATES_EXPIRY_DAYS = "affidavit.crypto.certificates.expiry_days" |  |  |  |  |

| `affidavit.crypto.cipher.mode` | str_key | CIPHER_MODE = "affidavit.crypto.cipher.mode" |  |  |  |  |

| `affidavit.crypto.decryption_duration_ns` | str_key | DECRYPTION_DURATION_NS = "affidavit.crypto.decryption_duration_ns" |  |  |  |  |

| `affidavit.crypto.encryption.duration_ns` | str_key | ENCRYPTION_DURATION_NS = "affidavit.crypto.encryption.duration_ns" |  |  |  |  |

| `affidavit.crypto.entropy.source` | str_key | ENTROPY_SOURCE = "affidavit.crypto.entropy.source" |  |  |  |  |

| `affidavit.crypto.iterations.count` | str_key | ITERATIONS_COUNT = "affidavit.crypto.iterations.count" |  |  |  |  |

| `affidavit.crypto.key.derivation_duration_ns` | str_key | KEY_DERIVATION_DURATION_NS = "affidavit.crypto.key.derivation_duration_ns" |  |  |  |  |

| `affidavit.crypto.key.rotation_count` | str_key | KEY_ROTATION_COUNT = "affidavit.crypto.key.rotation_count" |  |  |  |  |

| `affidavit.crypto.key.strength_bits` | str_key | KEY_STRENGTH_BITS = "affidavit.crypto.key.strength_bits" |  |  |  |  |

| `affidavit.crypto.mac.verify_duration_ns` | str_key | MAC_VERIFY_DURATION_NS = "affidavit.crypto.mac.verify_duration_ns" |  |  |  |  |

| `affidavit.crypto.mac.verify_success` | str_key | MAC_VERIFY_SUCCESS = "affidavit.crypto.mac.verify_success" |  |  |  |  |

| `affidavit.crypto.nonce.value` | str_key | NONCE_VALUE = "affidavit.crypto.nonce.value" |  |  |  |  |

| `affidavit.crypto.padding.scheme` | str_key | PADDING_SCHEME = "affidavit.crypto.padding.scheme" |  |  |  |  |

| `affidavit.crypto.random.bytes_requested` | str_key | RANDOM_BYTES_REQUESTED = "affidavit.crypto.random.bytes_requested" |  |  |  |  |

| `affidavit.crypto.salt.length` | str_key | SALT_LENGTH = "affidavit.crypto.salt.length" |  |  |  |  |

| `affidavit.crypto.signature.algorithm` | str_key | SIGNATURE_ALGORITHM = "affidavit.crypto.signature.algorithm" |  |  |  |  |

| `affidavit.crypto.signature.verify_duration_ns` | str_key | SIGNATURE_VERIFY_DURATION_NS = "affidavit.crypto.signature.verify_duration_ns" |  |  |  |  |

| `affidavit.execution.basic_blocks.executed` | str_key | BASIC_BLOCKS_EXECUTED = "affidavit.execution.basic_blocks.executed" |  |  |  |  |

| `affidavit.execution.branch.prediction_misses` | str_key | BRANCH_PREDICTION_MISSES = "affidavit.execution.branch.prediction_misses" |  |  |  |  |

| `affidavit.execution.branch.total` | str_key | BRANCH_TOTAL = "affidavit.execution.branch.total" |  |  |  |  |

| `affidavit.execution.cache.l1.hits` | str_key | CACHE_L1_HITS = "affidavit.execution.cache.l1.hits" |  |  |  |  |

| `affidavit.execution.cache.l1.misses` | str_key | CACHE_L1_MISSES = "affidavit.execution.cache.l1.misses" |  |  |  |  |

| `affidavit.execution.cache.l2.hits` | str_key | CACHE_L2_HITS = "affidavit.execution.cache.l2.hits" |  |  |  |  |

| `affidavit.execution.cache.l2.misses` | str_key | CACHE_L2_MISSES = "affidavit.execution.cache.l2.misses" |  |  |  |  |

| `affidavit.execution.context_switches.involuntary` | str_key | CONTEXT_SWITCHES_INVOLUNTARY = "affidavit.execution.context_switches.involuntary" |  |  |  |  |

| `affidavit.execution.context_switches.voluntary` | str_key | CONTEXT_SWITCHES_VOLUNTARY = "affidavit.execution.context_switches.voluntary" |  |  |  |  |

| `affidavit.execution.cpu.id` | str_key | CPU_ID = "affidavit.execution.cpu.id" |  |  |  |  |

| `affidavit.execution.cycles.total` | str_key | CYCLES_TOTAL = "affidavit.execution.cycles.total" |  |  |  |  |

| `affidavit.execution.functions.invoked` | str_key | FUNCTIONS_INVOKED = "affidavit.execution.functions.invoked" |  |  |  |  |

| `affidavit.execution.instructions.retired` | str_key | INSTRUCTION_RETIRED = "affidavit.execution.instructions.retired" |  |  |  |  |

| `affidavit.execution.loop.iterations` | str_key | LOOP_ITERATIONS = "affidavit.execution.loop.iterations" |  |  |  |  |

| `affidavit.execution.page_faults.major` | str_key | PAGE_FAULTS_MAJOR = "affidavit.execution.page_faults.major" |  |  |  |  |

| `affidavit.execution.page_faults.minor` | str_key | PAGE_FAULTS_MINOR = "affidavit.execution.page_faults.minor" |  |  |  |  |

| `affidavit.execution.process.uptime_ns` | str_key | PROCESS_UPTIME_NS = "affidavit.execution.process.uptime_ns" |  |  |  |  |

| `affidavit.execution.recursion.depth` | str_key | RECURSION_DEPTH = "affidavit.execution.recursion.depth" |  |  |  |  |

| `affidavit.execution.registers.spills` | str_key | REGISTERS_SPILLS = "affidavit.execution.registers.spills" |  |  |  |  |

| `affidavit.execution.syscalls.count` | str_key | SYSCALLS_COUNT = "affidavit.execution.syscalls.count" |  |  |  |  |

| `affidavit.execution.thread.id` | str_key | THREAD_ID = "affidavit.execution.thread.id" |  |  |  |  |

| `affidavit.execution.tlb.misses` | str_key | TLB_MISSES = "affidavit.execution.tlb.misses" |  |  |  |  |

| `affidavit.io.bytes_read` | str_key | BYTES_READ = "affidavit.io.bytes_read" |  |  |  |  |

| `affidavit.io.bytes_written` | str_key | BYTES_WRITTEN = "affidavit.io.bytes_written" |  |  |  |  |

| `affidavit.io.disk.latency_ns` | str_key | DISK_LATENCY_NS = "affidavit.io.disk.latency_ns" |  |  |  |  |

| `affidavit.io.dns_lookup_duration_ns` | str_key | DNS_LOOKUP_DURATION_NS = "affidavit.io.dns_lookup_duration_ns" |  |  |  |  |

| `affidavit.io.files_opened` | str_key | FILES_OPENED = "affidavit.io.files_opened" |  |  |  |  |

| `affidavit.io.fs.sync_count` | str_key | FS_SYNC_COUNT = "affidavit.io.fs.sync_count" |  |  |  |  |

| `affidavit.io.network.packets_received` | str_key | NETWORK_PACKETS_RECEIVED = "affidavit.io.network.packets_received" |  |  |  |  |

| `affidavit.io.network.packets_sent` | str_key | NETWORK_PACKETS_SENT = "affidavit.io.network.packets_sent" |  |  |  |  |

| `affidavit.io.socket_count` | str_key | SOCKET_COUNT = "affidavit.io.socket_count" |  |  |  |  |

| `affidavit.io.wait_duration_ns` | str_key | IO_WAIT_DURATION_NS = "affidavit.io.wait_duration_ns" |  |  |  |  |

| `affidavit.metric.branch_miss_rate` | str_key | BRANCH_MISS_RATE = "affidavit.metric.branch_miss_rate" |  |  |  |  |

| `affidavit.metric.hash_throughput` | str_key | HASH_THROUGHPUT = "affidavit.metric.hash_throughput" |  |  |  |  |

| `affidavit.metric.ocel_event_throughput` | str_key | OCEL_EVENT_THROUGHPUT = "affidavit.metric.ocel_event_throughput" |  |  |  |  |

| `affidavit.metric.verification_success_rate` | str_key | VERIFICATION_SUCCESS_RATE = "affidavit.metric.verification_success_rate" |  |  |  |  |

| `affidavit.metric.wasm_memory_usage` | str_key | WASM_MEMORY_USAGE = "affidavit.metric.wasm_memory_usage" |  |  |  |  |

| `affidavit.ocel.aggregation.duration_ns` | str_key | AGGREGATION_DURATION_NS = "affidavit.ocel.aggregation.duration_ns" |  |  |  |  |

| `affidavit.ocel.anomaly_score` | str_key | ANOMALY_SCORE = "affidavit.ocel.anomaly_score" |  |  |  |  |

| `affidavit.ocel.attribute.count` | str_key | ATTRIBUTE_COUNT = "affidavit.ocel.attribute.count" |  |  |  |  |

| `affidavit.ocel.compression.ratio` | str_key | COMPRESSION_RATIO = "affidavit.ocel.compression.ratio" |  |  |  |  |

| `affidavit.ocel.discovery.algorithm` | str_key | DISCOVERY_ALGORITHM = "affidavit.ocel.discovery.algorithm" |  |  |  |  |

| `affidavit.ocel.event.count` | str_key | EVENT_COUNT = "affidavit.ocel.event.count" |  |  |  |  |

| `affidavit.ocel.event.type_count` | str_key | EVENT_TYPE_COUNT = "affidavit.ocel.event.type_count" |  |  |  |  |

| `affidavit.ocel.filtering.duration_ns` | str_key | FILTERING_DURATION_NS = "affidavit.ocel.filtering.duration_ns" |  |  |  |  |

| `affidavit.ocel.id.collisions` | str_key | ID_COLLISIONS = "affidavit.ocel.id.collisions" |  |  |  |  |

| `affidavit.ocel.join.duration_ns` | str_key | JOIN_DURATION_NS = "affidavit.ocel.join.duration_ns" |  |  |  |  |

| `affidavit.ocel.log.size_bytes` | str_key | LOG_SIZE_BYTES = "affidavit.ocel.log.size_bytes" |  |  |  |  |

| `affidavit.ocel.mapping.duration_ns` | str_key | MAPPING_DURATION_NS = "affidavit.ocel.mapping.duration_ns" |  |  |  |  |

| `affidavit.ocel.nesting.max_depth` | str_key | NESTING_MAX_DEPTH = "affidavit.ocel.nesting.max_depth" |  |  |  |  |

| `affidavit.ocel.object.count` | str_key | OBJECT_COUNT = "affidavit.ocel.object.count" |  |  |  |  |

| `affidavit.ocel.object.type_count` | str_key | OBJECT_TYPE_COUNT = "affidavit.ocel.object.type_count" |  |  |  |  |

| `affidavit.ocel.parsing.duration_ns` | str_key | PARSING_DURATION_NS = "affidavit.ocel.parsing.duration_ns" |  |  |  |  |

| `affidavit.ocel.projection.duration_ns` | str_key | PROJECTION_DURATION_NS = "affidavit.ocel.projection.duration_ns" |  |  |  |  |

| `affidavit.ocel.relationship.count` | str_key | RELATIONSHIP_COUNT = "affidavit.ocel.relationship.count" |  |  |  |  |

| `affidavit.ocel.sequence.gap_count` | str_key | SEQUENCE_GAP_COUNT = "affidavit.ocel.sequence.gap_count" |  |  |  |  |

| `affidavit.ocel.serialization.duration_ns` | str_key | SERIALIZATION_DURATION_NS = "affidavit.ocel.serialization.duration_ns" |  |  |  |  |

| `affidavit.ocel.sorting.duration_ns` | str_key | SORTING_DURATION_NS = "affidavit.ocel.sorting.duration_ns" |  |  |  |  |

| `affidavit.ocel.transformation.count` | str_key | TRANSFORMATION_COUNT = "affidavit.ocel.transformation.count" |  |  |  |  |

| `affidavit.span.crypto_blake3` | str_key | CRYPTO_BLAKE3 = "affidavit.span.crypto_blake3" |  |  |  |  |

| `affidavit.span.execution_inner_loop` | str_key | EXECUTION_INNER_LOOP = "affidavit.span.execution_inner_loop" |  |  |  |  |

| `affidavit.span.ocel_parsing` | str_key | OCEL_PARSING = "affidavit.span.ocel_parsing" |  |  |  |  |

| `affidavit.span.verification_reachability` | str_key | VERIFICATION_REACHABILITY = "affidavit.span.verification_reachability" |  |  |  |  |

| `affidavit.span.verify_wasm` | str_key | VERIFY_WASM = "affidavit.span.verify_wasm" |  |  |  |  |

| `affidavit.speculative.duration_ns` | str_key | SPECULATIVE_EXECUTION_DURATION_NS = "affidavit.speculative.duration_ns" |  |  |  |  |

| `affidavit.speculative.micro_ops` | str_key | SUB_INSTRUCTION_MICRO_OPS = "affidavit.speculative.micro_ops" |  |  |  |  |

| `affidavit.speculative.path_count` | str_key | SPECULATIVE_PATH_COUNT = "affidavit.speculative.path_count" |  |  |  |  |

| `affidavit.speculative.pipeline_stall_ns` | str_key | PIPELINE_STALL_DURATION_NS = "affidavit.speculative.pipeline_stall_ns" |  |  |  |  |

| `affidavit.speculative.reorder_buffer_occupancy` | str_key | REORDER_BUFFER_OCCUPANCY = "affidavit.speculative.reorder_buffer_occupancy" |  |  |  |  |

| `affidavit.speculative.retired_count` | str_key | SPECULATIVE_RETIRED_COUNT = "affidavit.speculative.retired_count" |  |  |  |  |

| `affidavit.speculative.squashed_count` | str_key | SPECULATIVE_SQUASHED_COUNT = "affidavit.speculative.squashed_count" |  |  |  |  |

| `affidavit.verification.consistency.check_duration_ns` | str_key | CONSISTENCY_CHECK_DURATION_NS = "affidavit.verification.consistency.check_duration_ns" |  |  |  |  |

| `affidavit.verification.constraints.satisfied` | str_key | CONSTRAINTS_SATISFIED = "affidavit.verification.constraints.satisfied" |  |  |  |  |

| `affidavit.verification.constraints.total` | str_key | CONSTRAINTS_TOTAL = "affidavit.verification.constraints.total" |  |  |  |  |

| `affidavit.verification.constraints.violated` | str_key | CONSTRAINTS_VIOLATED = "affidavit.verification.constraints.violated" |  |  |  |  |

| `affidavit.verification.cycle.detection_duration_ns` | str_key | CYCLE_DETECTION_DURATION_NS = "affidavit.verification.cycle.detection_duration_ns" |  |  |  |  |

| `affidavit.verification.edge.count` | str_key | EDGE_COUNT = "affidavit.verification.edge.count" |  |  |  |  |

| `affidavit.verification.evidence.count` | str_key | EVIDENCE_COUNT = "affidavit.verification.evidence.count" |  |  |  |  |

| `affidavit.verification.evidence.size_bytes` | str_key | EVIDENCE_SIZE_BYTES = "affidavit.verification.evidence.size_bytes" |  |  |  |  |

| `affidavit.verification.format.version` | str_key | FORMAT_VERSION = "affidavit.verification.format.version" |  |  |  |  |

| `affidavit.verification.hypothesis.count` | str_key | HYPOTHESIS_COUNT = "affidavit.verification.hypothesis.count" |  |  |  |  |

| `affidavit.verification.node.count` | str_key | NODE_COUNT = "affidavit.verification.node.count" |  |  |  |  |

| `affidavit.verification.path.length` | str_key | PATH_LENGTH = "affidavit.verification.path.length" |  |  |  |  |

| `affidavit.verification.proof.size_bytes` | str_key | PROOF_SIZE_BYTES = "affidavit.verification.proof.size_bytes" |  |  |  |  |

| `affidavit.verification.reachability.check_duration_ns` | str_key | REACHABILITY_CHECK_DURATION_NS = "affidavit.verification.reachability.check_duration_ns" |  |  |  |  |

| `affidavit.verification.refutation.count` | str_key | REFUTATION_COUNT = "affidavit.verification.refutation.count" |  |  |  |  |

| `affidavit.verification.rules.duration_ns` | str_key | RULES_DURATION_NS = "affidavit.verification.rules.duration_ns" |  |  |  |  |

| `affidavit.verification.rules.evaluated` | str_key | RULES_EVALUATED = "affidavit.verification.rules.evaluated" |  |  |  |  |

| `affidavit.verification.saturation.reached` | str_key | SATURATION_REACHED = "affidavit.verification.saturation.reached" |  |  |  |  |

| `affidavit.verification.schema.validation_duration_ns` | str_key | SCHEMA_VALIDATION_DURATION_NS = "affidavit.verification.schema.validation_duration_ns" |  |  |  |  |

| `affidavit.verification.verdict.outcome` | str_key | VERDICT_OUTCOME = "affidavit.verification.verdict.outcome" |  |  |  |  |

| `affidavit.verification.witness.id` | str_key | WITNESS_ID = "affidavit.verification.witness.id" |  |  |  |  |

| `affidavit.verification.witness.level` | str_key | WITNESS_LEVEL = "affidavit.verification.witness.level" |  |  |  |  |

| `affidavit.wasm.exports.invoked` | str_key | EXPORTS_INVOKED = "affidavit.wasm.exports.invoked" |  |  |  |  |

| `affidavit.wasm.imports.failed` | str_key | IMPORTS_FAILED = "affidavit.wasm.imports.failed" |  |  |  |  |

| `affidavit.wasm.imports.resolved` | str_key | IMPORTS_RESOLVED = "affidavit.wasm.imports.resolved" |  |  |  |  |

| `affidavit.wasm.init.duration_ns` | str_key | INIT_DURATION_NS = "affidavit.wasm.init.duration_ns" |  |  |  |  |

| `affidavit.wasm.instructions.skipped` | str_key | INSTRUCTIONS_SKIPPED = "affidavit.wasm.instructions.skipped" |  |  |  |  |

| `affidavit.wasm.instructions.total` | str_key | INSTRUCTIONS_TOTAL = "affidavit.wasm.instructions.total" |  |  |  |  |

| `affidavit.wasm.instructions.verified` | str_key | INSTRUCTIONS_VERIFIED = "affidavit.wasm.instructions.verified" |  |  |  |  |

| `affidavit.wasm.jit.code_size_bytes` | str_key | JIT_CODE_SIZE_BYTES = "affidavit.wasm.jit.code_size_bytes" |  |  |  |  |

| `affidavit.wasm.jit.compile_duration_ns` | str_key | JIT_COMPILE_DURATION_NS = "affidavit.wasm.jit.compile_duration_ns" |  |  |  |  |

| `affidavit.wasm.memory.allocated` | str_key | MEMORY_ALLOCATED = "affidavit.wasm.memory.allocated" |  |  |  |  |

| `affidavit.wasm.memory.freed` | str_key | MEMORY_FREED = "affidavit.wasm.memory.freed" |  |  |  |  |

| `affidavit.wasm.memory.peak` | str_key | MEMORY_PEAK = "affidavit.wasm.memory.peak" |  |  |  |  |

| `affidavit.wasm.module.hash` | str_key | MODULE_HASH = "affidavit.wasm.module.hash" |  |  |  |  |

| `affidavit.wasm.module.size_bytes` | str_key | MODULE_SIZE_BYTES = "affidavit.wasm.module.size_bytes" |  |  |  |  |

| `affidavit.wasm.pages.initial` | str_key | PAGES_INITIAL = "affidavit.wasm.pages.initial" |  |  |  |  |

| `affidavit.wasm.pages.maximum` | str_key | PAGES_MAXIMUM = "affidavit.wasm.pages.maximum" |  |  |  |  |

| `affidavit.wasm.runtime.engine` | str_key | RUNTIME_ENGINE = "affidavit.wasm.runtime.engine" |  |  |  |  |

| `affidavit.wasm.stack.depth.current` | str_key | STACK_DEPTH_CURRENT = "affidavit.wasm.stack.depth.current" |  |  |  |  |

| `affidavit.wasm.stack.depth.max` | str_key | STACK_DEPTH_MAX = "affidavit.wasm.stack.depth.max" |  |  |  |  |

| `affidavit.wasm.table.elements` | str_key | TABLE_ELEMENTS = "affidavit.wasm.table.elements" |  |  |  |  |

| `affidavit.wasm.table.grow_count` | str_key | TABLE_GROW_COUNT = "affidavit.wasm.table.grow_count" |  |  |  |  |

| `affidavit.wasm.traps.count` | str_key | TRAPS_COUNT = "affidavit.wasm.traps.count" |  |  |  |  |

| `compile` | function | compile(&self) -> Vec<u8> |  |  |  |  |

| `new` | function | new(receipt: Receipt) -> Self |  |  |  |  |

| `ReceiptWasmCompiler` | struct | ReceiptWasmCompiler { receipt: Receipt } |  |  |  |  |

| `affi_trace_sink.log` | str_key | TRACE_SINK = "affi_trace_sink.log" |  |  |  |  |

| `tests/autogen_tdd_witness.rs` | str_key | TARGET_TEST = "tests/autogen_tdd_witness.rs" |  |  |  |  |

| `new` | function | new(receipt: &'a Receipt, initial_state: S, transition: F) -> Self |  |  |  |  |

| `run_repl` | function | run_repl(&mut self) -> io::Result<()> |  |  |  |  |

| `TimeTravelDebugger` | struct | TimeTravelDebugger { receipt: &'a Receipt, states: Vec<S>, cursor: usize } |  |  |  |  |

| `AffidavitRefusal` | enum | AffidavitRefusal { OcelLawViolation(OcelRefusal), StructuralLawViolation { stage: String, reason: String, } } |  |  |  |  |

| `admit` | function | admit(receipt: Receipt) -> Result<AdmittedReceipt, AffidavitRefusal> |  |  |  |  |

| `ARCHITECTURE_QUERY_SCHEMA` | const | ARCHITECTURE_QUERY_SCHEMA: &str |  |  |  |  |

| `ARCHITECTURE_RECEIPT_SCHEMA` | const | ARCHITECTURE_RECEIPT_SCHEMA: &str |  |  |  |  |

| `ArchitectureRefusal` | enum | ArchitectureRefusal { MissingExactSubject, MissingEvidence, UnknownPromotion, MutableOrChangedContract, MutableOrChangedSbb, CrossSubjectReuse, DoAuthorityForbidden, ReplayMismatch, MalformedDigest { field: &'static str, }, SchemaMismatch, NotAReplacement, ChainBroken, StandingChainMismatch, NotSupersedable, AlreadySuperseded, OrphanChainLink, ForgedEvidence, Malformed } |  |  |  |  |

| `ArchitectureStanding` | enum | ArchitectureStanding { Unknown, Candidate, Qualified, Refused, Superseded } |  |  |  |  |

| `EvidenceSource` | enum | EvidenceSource { AutofdeLab, Xaas, Runtime } |  |  |  |  |

| `admit` | function | admit( &mut self, receipt: ArchitectureQualificationReceipt, ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `admit_supersession` | function | admit_supersession( &mut self, supersession: Supersession, ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `binding_digest` | function | binding_digest(&self) -> String |  |  |  |  |

| `certify` | function | certify( abb_digest: impl Into<String>, contract_digest: impl Into<String>, sbb_digest: impl Into<String>, exact_subject_digest: impl Into<String>, qualification_evidence_digests: Vec<String>, producer_digest: impl Into<String>, artifact_digests: Vec<String>, standing: ArchitectureStanding, ) -> Result<Self, ArchitectureRefusal> |  |  |  |  |

| `certify_from_evidence` | function | certify_from_evidence( abb_digest: impl Into<String>, contract_digest: impl Into<String>, sbb_digest: impl Into<String>, exact_subject_digest: impl Into<String>, evidence: &[QualificationEvidence], producer_digest: impl Into<String>, artifact_digests: Vec<String>, standing: ArchitectureStanding, ) -> Result<Self, ArchitectureRefusal> |  |  |  |  |

| `current_qualified` | function | current_qualified(&self, abb_digest: &str) -> Vec<&ArchitectureQualificationReceipt> |  |  |  |  |

| `from_json_verified` | function | from_json_verified(json: &str) -> Result<Self, ArchitectureRefusal> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `observe` | function | observe( source: EvidenceSource, producer_digest: impl Into<String>, bytes: &[u8], ) -> Result<Self, ArchitectureRefusal> |  |  |  |  |

| `query_json` | function | query_json(&self, abb_digest: &str) -> String |  |  |  |  |

| `standing_of` | function | standing_of(&self, receipt_digest: &str) -> Option<ArchitectureStanding> |  |  |  |  |

| `supersede` | function | supersede( &self, new_sbb_digest: impl Into<String>, new_subject_digest: impl Into<String>, evidence: Vec<String>, ) -> Result<Supersession, ArchitectureRefusal> |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `verify` | function | verify( &self, prior: &ArchitectureQualificationReceipt, ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_bytes` | function | verify_bytes(&self, bytes: &[u8]) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_chain` | function | verify_chain(&self, prior: &Self) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_evidence` | function | verify_evidence( &self, observed: &[(QualificationEvidence, &[u8])], ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_integrity` | function | verify_integrity(&self) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `verify_replay` | function | verify_replay( &self, expected_abb_digest: &str, expected_contract_digest: &str, expected_sbb_digest: &str, expected_subject_digest: &str, ) -> Result<(), ArchitectureRefusal> |  |  |  |  |

| `affidavit.architecture-qualification.v2` | str_key | ARCHITECTURE_RECEIPT_SCHEMA = "affidavit.architecture-qualification.v2" |  |  |  |  |

| `affidavit.architecture-standing-query.v1` | str_key | ARCHITECTURE_QUERY_SCHEMA = "affidavit.architecture-standing-query.v1" |  |  |  |  |

| `ArchitectureQualificationReceipt` | struct | ArchitectureQualificationReceipt { pub schema: String, pub abb_digest: String, pub contract_digest: String, pub sbb_digest: String, pub exact_subject_digest: String, pub qualification_evidence_digests: Vec<String>, pub producer_digest: String, pub artifact_digests: Vec<String>, pub standing: ArchitectureStanding, pub confers_do_authority: bool, pub prior_receipt_digest: Option<String>, pub superseded_by_receipt_digest: Option<String>, pub receipt_digest: String } |  |  |  |  |

| `ArchitectureStandingLedger` | struct | ArchitectureStandingLedger { receipts: BTreeMap<String, ArchitectureQualificationReceipt>, retired_by: BTreeMap<String, String> } |  |  |  |  |

| `QualificationEvidence` | struct | QualificationEvidence { pub source: EvidenceSource, pub producer_digest: String, pub content_digest: String } |  |  |  |  |

| `Supersession` | struct | Supersession { pub retired: ArchitectureQualificationReceipt, pub successor: ArchitectureQualificationReceipt } |  |  |  |  |

| `REVOCATION_TOMBSTONE` | const | REVOCATION_TOMBSTONE: StateValue |  |  |  |  |

| `DoWitness` | enum | DoWitness { Revoked { id: StateKey, proof: InclusionProof, }, Admitted(AbsenceWitness) } |  |  |  |  |

| `FastPathRefusal` | enum | FastPathRefusal { Replay, ClockSkew( Revoked, Fence( } |  |  |  |  |

| `FenceError` | enum | FenceError { Tree(String), AlreadyRevoked(String), NotRevoked(String), WitnessStaleOrInvalid, Revoked } |  |  |  |  |

| `confirm` | function | confirm(&self, fence: &AuthorityFence) -> Result<(), FenceError> |  |  |  |  |

| `gate_do` | function | gate_do(witness: &DoWitness, live_root: Option<StateRoot>) -> Result<(), FenceError> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `prepare_do_permit` | function | prepare_do_permit(&mut self, id: &StateKey) -> Result<DoWitness, FenceError> |  |  |  |  |

| `prove_revoked` | function | prove_revoked(&mut self, id: &StateKey) -> Result<DoWitness, FenceError> |  |  |  |  |

| `revoke` | function | revoke(&mut self, id: &StateKey) -> Result<StateRoot, FenceError> |  |  |  |  |

| `root` | function | root(&self) -> Option<StateRoot> |  |  |  |  |

| `routes_to_sibling_slot` | function | routes_to_sibling_slot(queried: &StateKey, neighbor: &StateKey, depth: usize) -> bool |  |  |  |  |

| `unrevoke` | function | unrevoke(&mut self, id: &StateKey) -> Result<Option<StateRoot>, FenceError> |  |  |  |  |

| `verify_fast_path` | function | verify_fast_path( fence: &mut AuthorityFence, replay: &mut ReplayFilter, clock: &mut HlcClock, claim: &ConsequenceClaim, ) -> Result<AdmittedProof, FastPathRefusal> |  |  |  |  |

| `AdmittedProof` | struct | AdmittedProof { pub permit: DoWitness, pub admitted_at: HlcTimestamp, pub gate_root: Option<StateRoot> } |  |  |  |  |

| `AuthorityFence` | struct | AuthorityFence { tree: StateTree } |  |  |  |  |

| `ConsequenceClaim` | struct | ConsequenceClaim { pub consequence_id: [u8; 32], pub authority_id: [u8; 32], pub received: HlcTimestamp } |  |  |  |  |

| `bench_throughput` | function | bench_throughput(iterations: u32) -> Result<()> |  |  |  |  |

| `bench_variance_on_receipt` | function | bench_variance_on_receipt(path: &str, iterations: u32) -> Result<()> |  |  |  |  |

| `bench_variance_suite` | function | bench_variance_suite(_iterations: u32) -> Result<()> |  |  |  |  |

| `run_profile_workload` | function | run_profile_workload(seconds: u64, _receipt_path: Option<&str>) -> Result<()> |  |  |  |  |

| `run` | function | run() -> Result<()> |  |  |  |  |

| `EnvelopeError` | enum | EnvelopeError { Encode(String), Decode(String) } |  |  |  |  |

| `seal` | function | seal(value: &T) -> Result<Vec<u8>, EnvelopeError> |  |  |  |  |

| `sealed_commitment` | function | sealed_commitment( value: &T, ) -> Result<crate::types::Blake3Hash, EnvelopeError> |  |  |  |  |

| `unseal` | function | unseal(bytes: &[u8]) -> Result<T, EnvelopeError> |  |  |  |  |

| `BlsError` | enum | BlsError { CommitteeMismatch { sigs: usize, keys: usize, }, VerificationFailed } |  |  |  |  |

| `aggregate_committee` | function | aggregate_committee( signatures: &[Signature<TinyBLS381>], public_keys: &[PublicKey<TinyBLS381>], ) -> Result<(Signature<TinyBLS381>, PublicKey<TinyBLS381>), BlsError> |  |  |  |  |

| `generate` | function | generate(rng: &mut R) -> Self |  |  |  |  |

| `public` | function | public(&self) -> PublicKey<TinyBLS381> |  |  |  |  |

| `sign` | function | sign(&self, context: &[u8], message: &[u8]) -> Signature<TinyBLS381> |  |  |  |  |

| `verify_committee` | function | verify_committee( context: &[u8], message: &[u8], aggregate_signature: &Signature<TinyBLS381>, aggregate_key: &PublicKey<TinyBLS381>, ) -> bool |  |  |  |  |

| `affidavit:bls:v1:committee` | str_key | CONTEXT = "affidavit:bls:v1:committee" |  |  |  |  |

| `CommitteeKey` | struct | CommitteeKey { pub KeypairVT<TinyBLS381> } |  |  |  |  |

| `ALL` | const | ALL: [Rule; 8] |  |  |  |  |

| `CANONICALIZATION` | const | CANONICALIZATION: &str |  |  |  |  |

| `GENESIS` | const | GENESIS: &str |  |  |  |  |

| `PROFILE_ACTUATION` | const | PROFILE_ACTUATION: &str |  |  |  |  |

| `PROFILE_RECONCILIATION` | const | PROFILE_RECONCILIATION: &str |  |  |  |  |

| `PROFILE_REPLAY` | const | PROFILE_REPLAY: &str |  |  |  |  |

| `Admission` | enum | Admission { Admitted, Refused(String), Blocked(String), Unsupported(String) } |  |  |  |  |

| `BrceError` | enum | BrceError { Io( Malformed { line: usize, reason: String, }, Refused(String), Blocked(String), } |  |  |  |  |

| `Entry` | enum | Entry { Parsed { request: Request, request_digest: String, }, Routed { request_digest: String, route: RouteDecision, route_digest: String, }, Admitted { request_digest: String, admission: Admission, admission_digest: String, }, Constructed { action: ConstructedAction, construct_digest: String, }, Prepared { consequence_id: String, attempt_id: String, construct_digest: String, grant: AuthorityGrant, at: u64, }, Done { consequence_id: String, attempt_id: String, construct_digest: String, result: ActuationResult, executor_identity: String, at: u64, }, Receipted { receipt: BrceReceipt, }, Reconciled { consequence_id: String, prepared_record_digest: String, verdict: ReconciliationVerdict, evidence_digest: String, }, Refusal { stage: String, reason: String, } } |  |  |  |  |

| `Observation` | enum | Observation { Effect(String), NoEffect, Unknown } |  |  |  |  |

| `ReconciliationVerdict` | enum | ReconciliationVerdict { EffectConfirmed, NoEffectConfirmed, ExecutionUnknown, BlockedReconciliation } |  |  |  |  |

| `Rule` | enum | Rule { ChainIntegrity, ZeroUnreceiptedActuation, DoRequiresAuthority, ConstructRequiresAdmission, ReceiptDigestValid, AtMostOnceConsequence, CrashWindowReconciled, UnknownNotPromoted } |  |  |  |  |

| `actuate` | function | actuate( &mut self, admitted: &Admitted, action: &ConstructedAction, grant: &AuthorityGrant, attempt_id: &str, actuator: &mut (impl Actuator + Observer), now: u64, ) -> Result<BrceReceipt, BrceError> |  |  |  |  |

| `admit` | function | admit( &mut self, request: Request, route: RouteDecision, policy: impl Fn(&Request, &RouteDecision) -> Admission, ) -> Result<Admitted, BrceError> |  |  |  |  |

| `admitted` | function | admitted(&self) -> bool |  |  |  |  |

| `append` | function | append(&mut self, entry: Entry) -> Result<&LedgerRecord, BrceError> |  |  |  |  |

| `compute_digest` | function | compute_digest(&self) -> String |  |  |  |  |

| `construct` | function | construct( &mut self, admitted: &Admitted, consequence_id: &str, idempotent: bool, ) -> Result<ConstructedAction, BrceError> |  |  |  |  |

| `construct_digest` | function | construct_digest(&self) -> String |  |  |  |  |

| `court` | function | court(ledger: &BrceLedger, world: &dyn Observer) -> CourtVerdict |  |  |  |  |

| `digest` | function | digest(value: &T) -> String |  |  |  |  |

| `entries` | function | entries(&self) -> Vec<Entry> |  |  |  |  |

| `execute` | function | execute( &mut self, action: &ConstructedAction, attempt_id: &str, actuator: &mut dyn Actuator, now: u64, ) -> Result<ActuationResult, BrceError> |  |  |  |  |

| `from_entries` | function | from_entries(entries: impl IntoIterator<Item = Entry>) -> Self |  |  |  |  |

| `from_records` | function | from_records(records: Vec<LedgerRecord>) -> Self |  |  |  |  |

| `grant_digest` | function | grant_digest(&self) -> String |  |  |  |  |

| `head` | function | head(&self) -> String |  |  |  |  |

| `mutant_suite` | function | mutant_suite(base: &BrceLedger, world: &dyn Observer) -> Vec<MutantOutcome> |  |  |  |  |

| `new` | function | new(root: impl Into<PathBuf>) -> std::io::Result<Self> |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) -> Result<Self, BrceError> |  |  |  |  |

| `pending_consequences` | function | pending_consequences(ledger: &BrceLedger) -> BTreeSet<String> |  |  |  |  |

| `prepare` | function | prepare( &mut self, action: &ConstructedAction, grant: &AuthorityGrant, attempt_id: &str, now: u64, ) -> Result<(), BrceError> |  |  |  |  |

| `receipt` | function | receipt( &mut self, admitted: &Admitted, action: &ConstructedAction, grant: &AuthorityGrant, attempt_id: &str, observer: &dyn Observer, ) -> Result<BrceReceipt, BrceError> |  |  |  |  |

| `reconcile` | function | reconcile( &mut self, observer: &dyn Observer, ) -> Result<Vec<(String, ReconciliationVerdict)>, BrceError> |  |  |  |  |

| `record_digest` | function | record_digest(seq: u64, prev: &str, entry: &Entry) -> String |  |  |  |  |

| `records` | function | records(&self) -> &[LedgerRecord] |  |  |  |  |

| `refused_rules` | function | refused_rules(&self) -> BTreeSet<Rule> |  |  |  |  |

| `replay_digest` | function | replay_digest(ledger: &BrceLedger) -> String |  |  |  |  |

| `root` | function | root(&self) -> &Path |  |  |  |  |

| `seal` | function | seal(mut self) -> Self |  |  |  |  |

| `snapshot` | function | snapshot(world: &dyn Observer) -> Self |  |  |  |  |

| `0000000000000000000000000000000000000000000000000000000000000000` | str_key | GENESIS = "0000000000000000000000000000000000000000000000000000000000000000" |  |  |  |  |

| `affidavit/brce-actuation/v1` | str_key | PROFILE_ACTUATION = "affidavit/brce-actuation/v1" |  |  |  |  |

| `affidavit/brce-reconciliation/v1` | str_key | PROFILE_RECONCILIATION = "affidavit/brce-reconciliation/v1" |  |  |  |  |

| `affidavit/brce-replay/v1` | str_key | PROFILE_REPLAY = "affidavit/brce-replay/v1" |  |  |  |  |

| `serde_json-struct-order+blake3` | str_key | CANONICALIZATION = "serde_json-struct-order+blake3" |  |  |  |  |

| `ActuationEvidence` | struct | ActuationEvidence { pub executor_identity: String, pub started_at: u64, pub completed_at: u64, pub result_class: String, pub result_digest: String } |  |  |  |  |

| `ActuationResult` | struct | ActuationResult { pub result_class: String, pub result_digest: String, pub changed: bool } |  |  |  |  |

| `Admitted` | struct | Admitted { pub request: Request, pub request_digest: String, pub route_digest: String, pub admission_digest: String } |  |  |  |  |

| `AuthorityGrant` | struct | AuthorityGrant { pub grant_id: String, pub issuer: String, pub subject: String, pub operation: String, pub target: String, pub construct_digest: String, pub expires_at: u64, pub maximum_uses: u32 } |  |  |  |  |

| `BrceLedger` | struct | BrceLedger { records: Vec<LedgerRecord>, path: Option<PathBuf> } |  |  |  |  |

| `BrcePipeline` | struct | BrcePipeline { pub ledger: BrceLedger, run_id: String, tool_version: String } |  |  |  |  |

| `BrceReceipt` | struct | BrceReceipt { pub profile: String, pub run_id: String, pub subject: String, pub request_digest: String, pub route_digest: String, pub admission_digest: String, pub construct_digest: String, pub authority_grant_digest: String, pub consequence_id: String, pub attempt_id: String, pub actuation: ActuationEvidence, pub effect: EffectEvidence, pub verification: VerificationEvidence, pub replay: ReplayIdentity, pub canonicalization: String, pub previous_receipt: String, pub receipt_digest: String } |  |  |  |  |

| `ConstructedAction` | struct | ConstructedAction { pub request_digest: String, pub subject: String, pub operation: String, pub target: String, pub parameters: BTreeMap<String, String>, pub consequence_id: String, pub idempotent: bool } |  |  |  |  |

| `CourtRefusal` | struct | CourtRefusal { pub rule: Rule, pub seq: Option<u64>, pub detail: String } |  |  |  |  |

| `CourtVerdict` | struct | CourtVerdict { pub standing: String, pub rules: Vec<Rule>, pub refusals: Vec<CourtRefusal>, pub records: usize, pub receipts: usize, pub consequences_observed: usize, pub ledger_head: String, pub replay_digest: String } |  |  |  |  |

| `EffectEvidence` | struct | EffectEvidence { pub executed: bool, pub changed: bool, pub effect_digest: String } |  |  |  |  |

| `FileActuator` | struct | FileActuator { root: PathBuf } |  |  |  |  |

| `LedgerRecord` | struct | LedgerRecord { pub seq: u64, pub prev: String, pub entry: Entry, pub digest: String } |  |  |  |  |

| `MutantOutcome` | struct | MutantOutcome { pub rule: Rule, pub mutation: String, pub standing: String, pub refused_rules: Vec<Rule>, pub killed: bool } |  |  |  |  |

| `ReplayIdentity` | struct | ReplayIdentity { pub command_or_entrypoint: String, pub tool_identity: String, pub tool_version: String } |  |  |  |  |

| `Request` | struct | Request { pub request_id: String, pub subject: String, pub operation: String, pub target: String, pub parameters: BTreeMap<String, String>, pub requester: String } |  |  |  |  |

| `RouteDecision` | struct | RouteDecision { pub capability: String, pub executor_class: String } |  |  |  |  |

| `StaticWorld` | struct | StaticWorld { pub effects: BTreeMap<String, String> } |  |  |  |  |

| `VerificationEvidence` | struct | VerificationEvidence { pub verifier_identity: String, pub verdict: String, pub evidence_digest: String } |  |  |  |  |

| `Actuator` | trait |  |  |  |  |  |

| `Observer` | trait |  |  |  |  |  |

| `JcsError` | enum | JcsError { Serialization(String) } |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &serde_json::Value) -> Result<Vec<u8>, JcsError> |  |  |  |  |

| `canonical_commitment` | function | canonical_commitment( value: &serde_json::Value, ) -> Result<crate::types::Blake3Hash, JcsError> |  |  |  |  |

| `to_jcs` | function | to_jcs(value: &T) -> Result<Vec<u8>, JcsError> |  |  |  |  |

| `CanonicalTimeError` | enum | CanonicalTimeError { Malformed(String), Overflow } |  |  |  |  |

| `canonical_string` | function | canonical_string(ts: Timestamp) -> String |  |  |  |  |

| `canonicalize` | function | canonicalize(s: &str) -> Result<String, CanonicalTimeError> |  |  |  |  |

| `checked_add_ms` | function | checked_add_ms(ts: Timestamp, millis: u64) -> Result<Timestamp, CanonicalTimeError> |  |  |  |  |

| `parse` | function | parse(s: &str) -> Result<Timestamp, CanonicalTimeError> |  |  |  |  |

| `iso8601_timestamp::Timestamp` | use | iso8601_timestamp::Timestamp |  |  |  |  |

| `format_catalog` | function | format_catalog(fixtures: &[Fixture]) -> String |  |  |  |  |

| `list_fixtures` | function | list_fixtures( db: &FixtureDatabase, name_filter: Option<String>, events_filter: Option<usize>, ) -> Vec<Fixture> |  |  |  |  |

| `CausalError` | enum | CausalError { CycleDetected(String), UnknownNode(String) } |  |  |  |  |

| `add_dependency` | function | add_dependency(&mut self, from: K, to: K) |  |  |  |  |

| `contains` | function | contains(&self, node: K) -> bool |  |  |  |  |

| `edge_count` | function | edge_count(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `topological_order` | function | topological_order(&self) -> Result<Vec<K>, CausalError> |  |  |  |  |

| `verify_acyclic` | function | verify_acyclic(&self) -> Result<(), CausalError> |  |  |  |  |

| `CausalGraph` | struct | CausalGraph { graph: DiGraphMap<K, ()> } |  |  |  |  |

| `FORMAT_VERSION` | const | FORMAT_VERSION: &str |  |  |  |  |

| `GENESIS_SEED` | const | GENESIS_SEED: &[u8] |  |  |  |  |

| `WORKING_PATH` | const | WORKING_PATH: &str |  |  |  |  |

| `ChainError` | enum | ChainError { Encode( Decode( Io { path: String, source: std::io::Error, }, } |  |  |  |  |

| `append` | function | append(&mut self, event: OperationEvent) -> Result<(), ChainError> |  |  |  |  |

| `content_address` | function | content_address(receipt: &Receipt) -> Result<Blake3Hash, ChainError> |  |  |  |  |

| `deserialize_receipt` | function | deserialize_receipt(bytes: &[u8]) -> Result<Receipt, ChainError> |  |  |  |  |

| `events` | function | events(&self) -> &[OperationEvent] |  |  |  |  |

| `finalize` | function | finalize(self) -> Receipt |  |  |  |  |

| `from_events` | function | from_events(events: Vec<OperationEvent>) -> Result<Self, ChainError> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `load_working` | function | load_working() -> Result<Vec<OperationEvent>, ChainError> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `recompute_chain` | function | recompute_chain(events: &[OperationEvent]) -> Result<Blake3Hash, ChainError> |  |  |  |  |

| `save_receipt` | function | save_receipt(receipt: &Receipt, path: &Path) -> Result<(), ChainError> |  |  |  |  |

| `save_working` | function | save_working(events: &[OperationEvent]) -> Result<(), ChainError> |  |  |  |  |

| `serialize_receipt` | function | serialize_receipt(receipt: &Receipt) -> Result<Vec<u8>, ChainError> |  |  |  |  |

| `.affi/working.json` | str_key | WORKING_PATH = ".affi/working.json" |  |  |  |  |

| `affidavit-v` | str_key | GENESIS_SEED_STR = "affidavit-v" |  |  |  |  |

| `core/v1` | str_key | FORMAT_VERSION = "core/v1" |  |  |  |  |

| `ChainAssembler` | struct | ChainAssembler { events: Vec<OperationEvent>, running: Blake3Hash } |  |  |  |  |

| `assemble` | function | assemble(out: Option<&str>) -> Result<crate::types::AssembleOutput> |  |  |  |  |

| `emit` | function | emit( event_type: &str, objects: &[String], payload: &str, ) -> Result<crate::types::EmitOutput> |  |  |  |  |

| `show` | function | show(receipt: &str) -> Result<Receipt> |  |  |  |  |

| `verify` | function | verify(receipt: &str) -> Result<(i32, crate::types::Verdict)> |  |  |  |  |

| `ATTESTATION_FORMAT` | const | ATTESTATION_FORMAT: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `AttestationError` | enum | AttestationError { BadFormat(String), BadSignature, AttributeMismatch(String), Serialization(String) } |  |  |  |  |

| `AttestationKind` | enum | AttestationKind { Self_, Hardware, Import } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `attest_key` | function | attest_key( record: &KeyRecord, kind: AttestationKind, device: Option<(&str, bool, bool)>, signer: &Es256SigningKey, signer_kid: &str, at: u64, ) -> Result<AttestationRecord, AttestationError> |  |  |  |  |

| `verify_attestation` | function | verify_attestation( att: &AttestationRecord, signer_pk: &[u8], ) -> Result<bool, AttestationError> |  |  |  |  |

| `CTP-ATTEST-v1` | str_key | ATTESTATION_FORMAT = "CTP-ATTEST-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `AttestationRecord` | struct | AttestationRecord { pub format: String, pub key_id: String, pub kind: String, pub device_model: Option<String>, pub secure_hardware: bool, pub non_exportable: bool, pub attested_at: u64, pub signature: Vec<u8> } |  |  |  |  |

| `CANONICALIZATION` | const | CANONICALIZATION: &str |  |  |  |  |

| `DIGEST_ALGORITHM` | const | DIGEST_ALGORITHM: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `CanonicalError` | enum | CanonicalError { Json(String), NonCanonicalNumber(String) } |  |  |  |  |

| `digest` | function | digest(domain: &str, parts: &[&[u8]]) -> [u8; 32] |  |  |  |  |

| `digest_hex` | function | digest_hex(domain: &str, parts: &[&[u8]]) -> String |  |  |  |  |

| `domain_separated` | function | domain_separated(domain: &str, parts: &[&[u8]]) -> Vec<u8> |  |  |  |  |

| `jcs` | function | jcs(value: &Value) -> Result<String, CanonicalError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `BLAKE3` | str_key | DIGEST_ALGORITHM = "BLAKE3" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `JCS-RFC8785` | str_key | CANONICALIZATION = "JCS-RFC8785" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `CRL_FILE` | const | CRL_FILE: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `CrlFileError` | enum | CrlFileError { WrongFormat(String), Io(String), Serialization(String), Verification(String), Staleness(String) } |  |  |  |  |

| `load_and_apply` | function | load_and_apply( path: Option<&Path>, revocations: &mut RevocationList, issuer_pk: &[u8], current_epoch: u64, max_staleness: u64, now: u64, ) -> Result<usize, CrlFileError> |  |  |  |  |

| `new` | function | new(path: impl Into<PathBuf>) -> CrlFile |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `publish_verified` | function | publish_verified( revocations: &RevocationList, issuer: &Es256SigningKey, issuer_kid: &KeyId, epoch: u64, at: u64, out: Option<&Path>, ) -> Result<PathBuf, CrlFileError> |  |  |  |  |

| `read` | function | read(path: Option<&Path>) -> Result<SignedRevocationList, CrlFileError> |  |  |  |  |

| `write` | function | write( list: &SignedRevocationList, path: Option<&Path>, ) -> Result<PathBuf, CrlFileError> |  |  |  |  |

| `.affi/crl.json` | str_key | CRL_FILE = ".affi/crl.json" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `compromised` | str_key | REASON = "compromised" |  |  |  |  |

| `k-a` | str_key | KID_A = "k-a" |  |  |  |  |

| `k-b` | str_key | KID_B = "k-b" |  |  |  |  |

| `CrlFile` | struct | CrlFile { path: PathBuf } |  |  |  |  |

| `DOCTOR_CHECK_DOMAIN` | const | DOCTOR_CHECK_DOMAIN: &str |  |  |  |  |

| `ENVELOPE_LAW_CHECK_ID` | const | ENVELOPE_LAW_CHECK_ID: &str |  |  |  |  |

| `ES256_SELFTEST_CHECK_ID` | const | ES256_SELFTEST_CHECK_ID: &str |  |  |  |  |

| `PQC_SELFTEST_CHECK_ID` | const | PQC_SELFTEST_CHECK_ID: &str |  |  |  |  |

| `STORE_INTEGRITY_CHECK_ID` | const | STORE_INTEGRITY_CHECK_ID: &str |  |  |  |  |

| `envelope_law_finding` | function | envelope_law_finding() -> Finding |  |  |  |  |

| `es256_selftest_finding` | function | es256_selftest_finding() -> Finding |  |  |  |  |

| `pqc_selftest_finding` | function | pqc_selftest_finding() -> Finding |  |  |  |  |

| `run_crypto_checks` | function | run_crypto_checks() -> Vec<Finding> |  |  |  |  |

| `store_integrity_finding` | function | store_integrity_finding(path: &Path) -> Finding |  |  |  |  |

| `affidavit.crypto-trust-plane.v1/doctor` | str_key | DOCTOR_CHECK_DOMAIN = "affidavit.crypto-trust-plane.v1/doctor" |  |  |  |  |

| `crypto-envelope-law` | str_key | ENVELOPE_LAW_CHECK_ID = "crypto-envelope-law" |  |  |  |  |

| `crypto-es256-selftest` | str_key | ES256_SELFTEST_CHECK_ID = "crypto-es256-selftest" |  |  |  |  |

| `crypto-pqc-selftest` | str_key | PQC_SELFTEST_CHECK_ID = "crypto-pqc-selftest" |  |  |  |  |

| `crypto-store-integrity` | str_key | STORE_INTEGRITY_CHECK_ID = "crypto-store-integrity" |  |  |  |  |

| `ALGORITHM_NAME` | const | ALGORITHM_NAME: &str |  |  |  |  |

| `ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE` | const | ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE: &str |  |  |  |  |

| `KEY_MATERIAL_EXPORTABLE` | const | KEY_MATERIAL_EXPORTABLE: bool |  |  |  |  |

| `PROVIDER_KIND` | const | PROVIDER_KIND: &str |  |  |  |  |

| `TARGET_OS` | const | TARGET_OS: &str |  |  |  |  |

| `EnclaveError` | enum | EnclaveError { SecurityFramework(String), KeyNotFound(String), UnsupportedPlatform } |  |  |  |  |

| `delete_enclave_key` | function | delete_enclave_key(label: &str) -> Result<(), EnclaveError> |  |  |  |  |

| `enclave_sign` | function | enclave_sign(label: &str, msg: &[u8]) -> Result<Vec<u8>, EnclaveError> |  |  |  |  |

| `enclave_verify` | function | enclave_verify( public_key_sec1: &[u8], msg: &[u8], sig_der: &[u8], ) -> Result<bool, EnclaveError> |  |  |  |  |

| `generate_enclave_key` | function | generate_enclave_key(label: &str) -> Result<EnclaveKeyRef, EnclaveError> |  |  |  |  |

| `ES256` | str_key | ALGORITHM_NAME = "ES256" |  |  |  |  |

| `SECURE_ENCLAVE` | str_key | PROVIDER_KIND = "SECURE_ENCLAVE" |  |  |  |  |

| `Unblock for errSecMissingEntitlement (-34018): Secure Enclave keygen succeeds ` | str_key | ENCLAVE_KEYCHAIN_UNBLOCK_RECIPE = "Unblock for errSecMissingEntitlement (-34018): Secure Enclave keygen succeeds " |  |  |  |  |

| `macos` | str_key | TARGET_OS = "macos" |  |  |  |  |

| `EnclaveKeyRef` | struct | EnclaveKeyRef { pub key_id: KeyId, pub label: String, pub public_key_sec1: Vec<u8> } |  |  |  |  |

| `imp::{delete_enclave_key, enclave_sign, enclave_verify, generate_enclave_key}` | use | imp::{delete_enclave_key, enclave_sign, enclave_verify, generate_enclave_key} |  |  |  |  |

| `ENVELOPE_FIELDS` | const | ENVELOPE_FIELDS: [(&str, u32); 12] |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `EnvelopeError` | enum | EnvelopeError { Malformed(String), Expired(u64), NotYetValid(u64), ReplayRejected(String), WrongVersion(String) } |  |  |  |  |

| `envelope_document` | function | envelope_document(&self) -> serde_json::Value |  |  |  |  |

| `from_bytes` | function | from_bytes(b: &[u8]) -> Result<Self, EnvelopeError> |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64, window_seconds: u64) -> usize |  |  |  |  |

| `record` | function | record( &mut self, kid: &str, nonce: [u8; 16], at: u64, window_seconds: u64, ) -> Result<(), EnvelopeError> |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> |  |  |  |  |

| `signing_input` | function | signing_input(&self) -> Vec<u8> |  |  |  |  |

| `signing_input_checked` | function | signing_input_checked(&self) -> Result<Vec<u8>, EnvelopeError> |  |  |  |  |

| `to_bytes` | function | to_bytes(&self) -> Result<Vec<u8>, EnvelopeError> |  |  |  |  |

| `window_live` | function | window_live(&self, now: u64) -> Result<(), EnvelopeError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `version` | str_key | ENVELOPE_FIELDS = "version" |  |  |  |  |

| `NonceJournal` | struct | NonceJournal { seen: BTreeMap<String, (String, u64)> } |  |  |  |  |

| `SignatureEnvelope` | struct | SignatureEnvelope { pub version: String, pub algorithm: AlgorithmId, pub key_id: KeyId, pub profile: CryptoProfile, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: [u8; 16], pub not_before: u64, pub expires_at: u64, pub subject_digest: [u8; 32], pub audience: String } |  |  |  |  |

| `ALGORITHM_NAME` | const | ALGORITHM_NAME: &str |  |  |  |  |

| `DETERMINISTIC` | const | DETERMINISTIC: bool |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `PUBLIC_KEY_LENGTH` | const | PUBLIC_KEY_LENGTH: usize |  |  |  |  |

| `SPEC_REF` | const | SPEC_REF: &str |  |  |  |  |

| `Es256Error` | enum | Es256Error { P256(String), MalformedPublicKey, MalformedSignature } |  |  |  |  |

| `from_seed` | function | from_seed(seed: &[u8; 32]) -> Result<Self, Es256Error> |  |  |  |  |

| `generate` | function | generate() -> Result<Self, Es256Error> |  |  |  |  |

| `key_id_fingerprint` | function | key_id_fingerprint(&self) -> crate::crypto_trust_keys::KeyFingerprint |  |  |  |  |

| `public_key_sec1` | function | public_key_sec1(&self) -> Vec<u8> |  |  |  |  |

| `sign` | function | sign(&self, msg: &[u8]) -> Vec<u8> |  |  |  |  |

| `verify_es256` | function | verify_es256( public_key_sec1: &[u8], msg: &[u8], sig_der: &[u8], ) -> Result<bool, Es256Error> |  |  |  |  |

| `A6E3C57DD01ABE90086538398355DD4C3B17AA873382B0F24D6129493D8AAD60` | str_key | RFC6979_K = "A6E3C57DD01ABE90086538398355DD4C3B17AA873382B0F24D6129493D8AAD60" |  |  |  |  |

| `C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6721` | str_key | RFC6979_X = "C9AFA9D845BA75166B5C215767B1D6934E50C3DB36E89B127B8A622B120F6721" |  |  |  |  |

| `EFD48B2AACB6A8FD1140DD9CD45E81D69D2C877B56AAF991C34D0EA84EAF3716` | str_key | RFC6979_R = "EFD48B2AACB6A8FD1140DD9CD45E81D69D2C877B56AAF991C34D0EA84EAF3716" |  |  |  |  |

| `ES256` | str_key | ALGORITHM_NAME = "ES256" |  |  |  |  |

| `F7CB1C942D657C41D436C7A1B6E29F65F3E900DBB9AFF4064DC4AB2F843ACDA8` | str_key | RFC6979_S = "F7CB1C942D657C41D436C7A1B6E29F65F3E900DBB9AFF4064DC4AB2F843ACDA8" |  |  |  |  |

| `FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551` | str_key | P256_ORDER_HEX = "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551" |  |  |  |  |

| `RFC 6979 / SEC 2` | str_key | SPEC_REF = "RFC 6979 / SEC 2" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `sample` | str_key | RFC6979_MESSAGE = "sample" |  |  |  |  |

| `Es256SigningKey` | struct | Es256SigningKey { secret: SigningKey } |  |  |  |  |

| `JOURNAL_DOMAIN` | const | JOURNAL_DOMAIN: &str |  |  |  |  |

| `JOURNAL_FORMAT` | const | JOURNAL_FORMAT: &str |  |  |  |  |

| `JOURNAL_GENESIS` | const | JOURNAL_GENESIS: &str |  |  |  |  |

| `JournalError` | enum | JournalError { SeqGap { expected: u64, got: u64, }, ChainBroken { at: u64, }, Serialization(String) } |  |  |  |  |

| `append` | function | append(&mut self, draft: JournalEntryDraft) -> Result<JournalEntry, JournalError> |  |  |  |  |

| `by_epoch` | function | by_epoch(&self, epoch: u64) -> Vec<&JournalEntry> |  |  |  |  |

| `by_key` | function | by_key(&self, kid: &str) -> Vec<&JournalEntry> |  |  |  |  |

| `entries` | function | entries(&self) -> &[JournalEntry] |  |  |  |  |

| `from_jsonl` | function | from_jsonl(s: &str) -> Result<Self, JournalError> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `record_receipt` | function | record_receipt( journal: &mut StandingJournal, receipt: &crate::crypto_trust_verify::CryptoStandingReceipt, policy_epoch: u64, revocation_epoch: u64, ) -> Result<JournalEntry, JournalError> |  |  |  |  |

| `to_jsonl` | function | to_jsonl(&self) -> String |  |  |  |  |

| `verify_chain` | function | verify_chain(&self) -> Result<(), JournalError> |  |  |  |  |

| `0000000000000000000000000000000000000000000000000000000000000000` | str_key | JOURNAL_GENESIS = "0000000000000000000000000000000000000000000000000000000000000000" |  |  |  |  |

| `1111111111111111111111111111111111111111111111111111111111111111` | str_key | COMMITMENT = "1111111111111111111111111111111111111111111111111111111111111111" |  |  |  |  |

| `2222222222222222222222222222222222222222222222222222222222222222` | str_key | RECEIPT_HASH = "2222222222222222222222222222222222222222222222222222222222222222" |  |  |  |  |

| `CTP-JOURNAL-v1` | str_key | JOURNAL_FORMAT = "CTP-JOURNAL-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | JOURNAL_DOMAIN = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `afk1_aaaaaaaaaaaaaaaa` | str_key | KID_A = "afk1_aaaaaaaaaaaaaaaa" |  |  |  |  |

| `afk1_bbbbbbbbbbbbbbbb` | str_key | KID_B = "afk1_bbbbbbbbbbbbbbbb" |  |  |  |  |

| `JournalEntry` | struct | JournalEntry { pub seq: u64, pub prev: String, pub key_id: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub standing: String, pub envelope_commitment: String, pub receipt_hash: String, pub entry_hash: String } |  |  |  |  |

| `JournalEntryDraft` | struct | JournalEntryDraft { pub key_id: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub standing: String, pub envelope_commitment: String, pub receipt_hash: String } |  |  |  |  |

| `StandingJournal` | struct | StandingJournal { entries: Vec<JournalEntry> } |  |  |  |  |

| `PERSIST_DOMAIN_TAG` | const | PERSIST_DOMAIN_TAG: &str |  |  |  |  |

| `PERSIST_FILE` | const | PERSIST_FILE: &str |  |  |  |  |

| `PersistError` | enum | PersistError { Replay { kid: String, }, Io(String), Corruption { line: usize, reason: String, }, Serialization(String), Lock(String) } |  |  |  |  |

| `entries` | function | entries(&self) -> &[NonceRecord] |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `load` | function | load(path: impl Into<PathBuf>) -> Result<Self, PersistError> |  |  |  |  |

| `open_or_create` | function | open_or_create(path: impl Into<PathBuf>) -> Result<Self, PersistError> |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64, window_seconds: u64) -> Result<usize, PersistError> |  |  |  |  |

| `record` | function | record( &mut self, kid: &str, nonce: [u8; 16], at: u64, window_seconds: u64, ) -> Result<(), PersistError> |  |  |  |  |

| `refresh` | function | refresh(&mut self) -> Result<(), PersistError> |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> |  |  |  |  |

| `verify_integrity` | function | verify_integrity(&self) -> Result<(), PersistError> |  |  |  |  |

| `.affi/nonce-journal.jsonl` | str_key | PERSIST_FILE = ".affi/nonce-journal.jsonl" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | PERSIST_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `afk1_aaaaaaaaaaaaaaaa` | str_key | KID_A = "afk1_aaaaaaaaaaaaaaaa" |  |  |  |  |

| `afk1_bbbbbbbbbbbbbbbb` | str_key | KID_B = "afk1_bbbbbbbbbbbbbbbb" |  |  |  |  |

| `NonceLedgerFile` | struct | NonceLedgerFile { path: PathBuf, entries: Vec<NonceRecord> } |  |  |  |  |

| `NonceRecord` | struct | NonceRecord { pub kid: String, pub nonce_hex: String, pub seen_at: u64 } |  |  |  |  |

| `JwksError` | enum | JwksError { MalformedPublicKey(String, String), UnsupportedAlgorithm(String), FeatureRequired, DuplicateKid(String) } |  |  |  |  |

| `export_jwk` | function | export_jwk(record: &KeyRecord) -> Result<serde_json::Value, JwksError> |  |  |  |  |

| `export_jwks` | function | export_jwks(records: &[KeyRecord]) -> Result<serde_json::Value, JwksError> |  |  |  |  |

| `ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_` | str_key | ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_" |  |  |  |  |

| `KAT_ALGORITHM_REGISTRY` | const | KAT_ALGORITHM_REGISTRY: &str |  |  |  |  |

| `KAT_DOMAIN_TAG` | const | KAT_DOMAIN_TAG: &str |  |  |  |  |

| `KAT_ENVELOPE_VERSION` | const | KAT_ENVELOPE_VERSION: &str |  |  |  |  |

| `KAT_SCHEMA_VERSION` | const | KAT_SCHEMA_VERSION: &str |  |  |  |  |

| `KatError` | enum | KatError { AlgorithmUnknown(String), VerificationFailed { vector_id: String, }, MalformedVector { vector_id: String, }, Serialization(String) } |  |  |  |  |

| `build_vector` | function | build_vector(index: usize, algorithm: &str) -> Result<KatVector, KatError> |  |  |  |  |

| `export_json` | function | export_json(corpus: &[KatVector]) -> String |  |  |  |  |

| `generate_corpus` | function | generate_corpus() -> Vec<KatVector> |  |  |  |  |

| `import_json` | function | import_json(s: &str) -> Result<Vec<KatVector>, KatError> |  |  |  |  |

| `verify_corpus` | function | verify_corpus(vectors: &[KatVector]) -> Result<KatReport, KatError> |  |  |  |  |

| `verify_vector` | function | verify_vector(vector: &KatVector) -> Result<bool, KatError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | KAT_ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `CTP-KAT-v1` | str_key | KAT_SCHEMA_VERSION = "CTP-KAT-v1" |  |  |  |  |

| `ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s` | str_key | KAT_ALGORITHM_REGISTRY = "ED25519|Ed25519@@ES256|Es256@@ES256+ML-DSA-65|HybridEs256MlDsa65@@ES256K|Es256k@@ML-DSA-65|MlDsa65@@SLH-DSA-SHA2-128s|SlhDsa128s" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | KAT_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `KatReport` | struct | KatReport { pub total: usize, pub passed: usize } |  |  |  |  |

| `KatVector` | struct | KatVector { pub vector_id: String, pub algorithm: String, pub seed_hex: String, pub public_key_hex: String, pub message_hex: String, pub signature_hex: String } |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `AlgorithmId` | enum | AlgorithmId { Es256, HybridEs256MlDsa65, MlDsa65, SlhDsa128s, Ed25519, Es256k } |  |  |  |  |

| `CryptoProfile` | enum | CryptoProfile { Classical, Hybrid, Pqc } |  |  |  |  |

| `KeyOrigin` | enum | KeyOrigin { HardwareAttested { device: String }, Imported { source: String }, Generated } |  |  |  |  |

| `KeyProviderKind` | enum | KeyProviderKind { Hsm, SecureEnclave, Software } |  |  |  |  |

| `PublicKeyMaterial` | enum | PublicKeyMaterial { Es256Sec1(Vec<u8>), Hybrid { es256: Vec<u8>, mldsa65: Vec<u8> }, MlDsa65(Vec<u8>), SlhDsa128s(Vec<u8>), Ed25519(Vec<u8>), Es256kSec1(Vec<u8>) } |  |  |  |  |

| `RegistryError` | enum | RegistryError { Duplicate(KeyId), DuplicateFingerprint(String), Unknown(KeyId) } |  |  |  |  |

| `algorithm` | function | algorithm(&self) -> AlgorithmId |  |  |  |  |

| `all` | function | all() -> &'static [AlgorithmId] |  |  |  |  |

| `as_hex` | function | as_hex(&self) -> String |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(&self) -> Vec<u8> |  |  |  |  |

| `encoded_len` | function | encoded_len(&self) -> usize |  |  |  |  |

| `fingerprint_public_key` | function | fingerprint_public_key( algorithm: AlgorithmId, public_key: &PublicKeyMaterial, ) -> KeyFingerprint |  |  |  |  |

| `from_fingerprint` | function | from_fingerprint(fingerprint: &KeyFingerprint) -> KeyId |  |  |  |  |

| `key_material_exportable` | function | key_material_exportable(self) -> bool |  |  |  |  |

| `kind` | function | kind(self) -> &'static str |  |  |  |  |

| `new` | function | new() -> InMemoryKeyRegistry |  |  |  |  |

| `profile` | function | profile(self) -> CryptoProfile |  |  |  |  |

| `public_key_len` | function | public_key_len(self) -> Option<usize> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `CustodianIdentity` | struct | CustodianIdentity { pub subject: String, pub device: Option<String>, pub org: Option<String> } |  |  |  |  |

| `InMemoryKeyRegistry` | struct | InMemoryKeyRegistry { records: BTreeMap<KeyId, KeyRecord>, fingerprints: BTreeSet<KeyFingerprint> } |  |  |  |  |

| `KeyFingerprint` | struct | KeyFingerprint { pub [u8; 32] } |  |  |  |  |

| `KeyId` | struct | KeyId { pub String } |  |  |  |  |

| `KeyRecord` | struct | KeyRecord { pub id: KeyId, pub algorithm: AlgorithmId, pub fingerprint: KeyFingerprint, pub custodian: CustodianIdentity, pub origin: KeyOrigin, pub public_key: PublicKeyMaterial, pub created_epoch: u64 } |  |  |  |  |

| `KeyRegistry` | trait |  |  |  |  |  |

| `MAX_REVOCATION_STALENESS_SECONDS` | const | MAX_REVOCATION_STALENESS_SECONDS: u64 |  |  |  |  |

| `NONCE_WINDOW_SECONDS` | const | NONCE_WINDOW_SECONDS: u64 |  |  |  |  |

| `REPLAY_KEY` | const | REPLAY_KEY: &str |  |  |  |  |

| `LifecycleRefusal` | enum | LifecycleRefusal { Revoked(String, u64, String), NoActiveEpoch(String), EpochExpired { kid: String, index: u64, max: u64 }, EpochsExhausted(String), AlreadyRetired(u64), NonMonotonic(u64, u64) } |  |  |  |  |

| `active_at` | function | active_at(&self, now: u64) -> bool |  |  |  |  |

| `active_epoch` | function | active_epoch(&self, kid: &str) -> Option<&KeyEpoch> |  |  |  |  |

| `admit_opening` | function | admit_opening(&self, kid: &str, history: &[KeyEpoch]) -> Result<(), LifecycleRefusal> |  |  |  |  |

| `current_epoch` | function | current_epoch(&self) -> u64 |  |  |  |  |

| `epochs` | function | epochs(&self, kid: &str) -> &[KeyEpoch] |  |  |  |  |

| `is_revoked` | function | is_revoked(&self, kid: &str) -> bool |  |  |  |  |

| `new` | function | new(policy: RotationPolicy) -> Self |  |  |  |  |

| `open_epoch` | function | open_epoch(&mut self, kid: &str, at: u64) -> Result<u64, LifecycleRefusal> |  |  |  |  |

| `policy` | function | policy(&self) -> &RotationPolicy |  |  |  |  |

| `require_active_epoch` | function | require_active_epoch(&self, kid: &str, now: u64) -> Result<&KeyEpoch, LifecycleRefusal> |  |  |  |  |

| `retire_epoch` | function | retire_epoch(&mut self, kid: &str, index: u64, at: u64) |  |  |  |  |

| `revoke` | function | revoke(&mut self, kid: &str, at: u64, reason: String) |  |  |  |  |

| `revoked_at` | function | revoked_at(&self, kid: &str) -> Option<u64> |  |  |  |  |

| `signature_epoch_live` | function | signature_epoch_live( &self, kid: &str, sig_revocation_epoch: u64, now: u64, ) -> Result<(), LifecycleRefusal> |  |  |  |  |

| `validate_epoch` | function | validate_epoch( &self, kid: &str, epoch: &KeyEpoch, now: u64, ) -> Result<(), LifecycleRefusal> |  |  |  |  |

| `validate_signature_key` | function | validate_signature_key(&self, kid: &str, now: u64) -> Result<(), LifecycleRefusal> |  |  |  |  |

| `compromised` | str_key | REASON = "compromised" |  |  |  |  |

| `k-1` | str_key | KID = "k-1" |  |  |  |  |

| `kid,nonce` | str_key | REPLAY_KEY = "kid,nonce" |  |  |  |  |

| `KeyEpoch` | struct | KeyEpoch { pub index: u64, pub activated_at: u64, pub retired_at: Option<u64> } |  |  |  |  |

| `LifecycleLedger` | struct | LifecycleLedger { epochs: BTreeMap<String, Vec<KeyEpoch>>, policy: RotationPolicy } |  |  |  |  |

| `RevocationList` | struct | RevocationList { records: BTreeMap<String, RevocationRecord> } |  |  |  |  |

| `RevocationRecord` | struct | RevocationRecord { pub revoked_at: u64, pub reason: String } |  |  |  |  |

| `RotationPolicy` | struct | RotationPolicy { pub max_epochs_in_flight: usize, pub max_age_seconds: u64 } |  |  |  |  |

| `HEAD_SIGNING_DOMAIN` | const | HEAD_SIGNING_DOMAIN: &str |  |  |  |  |

| `OPS_FILE` | const | OPS_FILE: &str |  |  |  |  |

| `OPS_FORMAT` | const | OPS_FORMAT: &str |  |  |  |  |

| `LogOpsError` | enum | LogOpsError { CommitmentMismatch, Journal(String), Transparency(String), Signing(String), Serialization(String) } |  |  |  |  |

| `audit` | function | audit(&self) -> Result<AuditReport, LogOpsError> |  |  |  |  |

| `from_jsonl` | function | from_jsonl(s: &str) -> Result<Self, LogOpsError> |  |  |  |  |

| `heads` | function | heads(&self) -> &[SignedTreeHead] |  |  |  |  |

| `journal` | function | journal(&self) -> &StandingJournal |  |  |  |  |

| `leaf` | function | leaf(&self, index: u64) -> Option<&[u8; 32]> |  |  |  |  |

| `log` | function | log(&self) -> &TransparencyLog |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `prove_inclusion` | function | prove_inclusion(&self, leaf: u64) -> Result<(InclusionProof, [u8; 32]), LogOpsError> |  |  |  |  |

| `publish_head` | function | publish_head( &self, signer: &Es256SigningKey, kid: &str, now: u64, ) -> Result<SignedTreeHead, LogOpsError> |  |  |  |  |

| `record` | function | record( &mut self, receipt: &CryptoStandingReceipt, commitment: [u8; 32], signer: &Es256SigningKey, signer_kid: &str, now: u64, ) -> Result<(JournalSeq, LeafIndex), LogOpsError> |  |  |  |  |

| `to_jsonl` | function | to_jsonl(&self) -> String |  |  |  |  |

| `verify_head` | function | verify_head(head: &SignedTreeHead, public_key_sec1: &[u8]) -> Result<bool, LogOpsError> |  |  |  |  |

| `.affi/transparency.jsonl` | str_key | OPS_FILE = ".affi/transparency.jsonl" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `CTP-LOGOPS-v1` | str_key | OPS_FORMAT = "CTP-LOGOPS-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | HEAD_SIGNING_DOMAIN = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `afk1_transparency-log-ops` | str_key | SIGNER_KID = "afk1_transparency-log-ops" |  |  |  |  |

| `AuditReport` | struct | AuditReport { pub journal_entries: u64, pub log_leaves: u64, pub consistent: bool } |  |  |  |  |

| `SignedTreeHead` | struct | SignedTreeHead { pub tree_size: u64, pub head_hex: String, pub timestamp: u64, pub kid: String, pub signature: Vec<u8> } |  |  |  |  |

| `TrustLogOps` | struct | TrustLogOps { journal: StandingJournal, log: TransparencyLog, heads: Vec<SignedTreeHead> } |  |  |  |  |

| `DEFAULT_WINDOW_SECONDS` | const | DEFAULT_WINDOW_SECONDS: u64 |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `JOURNAL_FORMAT` | const | JOURNAL_FORMAT: &str |  |  |  |  |

| `REPLAY_KEY` | const | REPLAY_KEY: &str |  |  |  |  |

| `STORE_FILE` | const | STORE_FILE: &str |  |  |  |  |

| `NonceStoreError` | enum | NonceStoreError { Io( WrongFormat { expected: String, found: String, }, Corrupt { line: usize, reason: String, }, ReplayRejected(String), Lock(String), } |  |  |  |  |

| `create` | function | create(path: P) -> Result<Self, NonceStoreError> |  |  |  |  |

| `default_journal_path` | function | default_journal_path() -> PathBuf |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `open` | function | open(path: P, window_seconds: u64) -> Result<Self, NonceStoreError> |  |  |  |  |

| `open_default` | function | open_default() -> Result<Self, NonceStoreError> |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `prune` | function | prune(&mut self, now: u64) -> Result<usize, NonceStoreError> |  |  |  |  |

| `record` | function | record(&mut self, kid: &str, nonce: &[u8; 16], at: u64) -> Result<(), NonceStoreError> |  |  |  |  |

| `seen` | function | seen(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> |  |  |  |  |

| `.affi/keys.json` | str_key | STORE_FILE = ".affi/keys.json" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `CTP-NONCE-JOURNAL-v1` | str_key | JOURNAL_FORMAT = "CTP-NONCE-JOURNAL-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `kid,nonce` | str_key | REPLAY_KEY = "kid,nonce" |  |  |  |  |

| `DiskNonceJournal` | struct | DiskNonceJournal { path: PathBuf, entries: BTreeMap<String, (String, u64)>, window_seconds: u64 } |  |  |  |  |

| `HYBRID_SPEC_REF` | const | HYBRID_SPEC_REF: &str |  |  |  |  |

| `ML_DSA_65_PUBLIC_KEY_LEN` | const | ML_DSA_65_PUBLIC_KEY_LEN: usize |  |  |  |  |

| `ML_DSA_65_SEED_LEN` | const | ML_DSA_65_SEED_LEN: usize |  |  |  |  |

| `ML_DSA_65_SIGNATURE_LEN` | const | ML_DSA_65_SIGNATURE_LEN: usize |  |  |  |  |

| `ML_DSA_65_SPEC_REF` | const | ML_DSA_65_SPEC_REF: &str |  |  |  |  |

| `PQC_DOMAIN_TAG` | const | PQC_DOMAIN_TAG: &str |  |  |  |  |

| `SLH_DSA_128S_SEED_LEN` | const | SLH_DSA_128S_SEED_LEN: usize |  |  |  |  |

| `SLH_DSA_128S_SPEC_REF` | const | SLH_DSA_128S_SPEC_REF: &str |  |  |  |  |

| `PqcError` | enum | PqcError { MlDsa(String), SlhDsa(String), Es256Half( MalformedPublicKey, MalformedSignature, } |  |  |  |  |

| `hybrid_sign` | function | hybrid_sign(secret: &HybridSecret, msg: &[u8]) -> Result<HybridSignature, PqcError> |  |  |  |  |

| `hybrid_verify` | function | hybrid_verify( es256_pk: &[u8], mldsa65_pk: &[u8], msg: &[u8], sig: &HybridSignature, ) -> Result<bool, PqcError> |  |  |  |  |

| `ml_dsa65_from_seed` | function | ml_dsa65_from_seed(seed: &[u8; ML_DSA_65_SEED_LEN]) -> MlDsa65KeyPair |  |  |  |  |

| `ml_dsa65_generate` | function | ml_dsa65_generate() -> Result<MlDsa65KeyPair, PqcError> |  |  |  |  |

| `ml_dsa65_sign` | function | ml_dsa65_sign( seed: &[u8; ML_DSA_65_SEED_LEN], msg: &[u8], rnd: &[u8; 32], ) -> Result<Vec<u8>, PqcError> |  |  |  |  |

| `ml_dsa65_verify` | function | ml_dsa65_verify(public: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, PqcError> |  |  |  |  |

| `slh_dsa128s_from_seed` | function | slh_dsa128s_from_seed(seeds: &[u8; SLH_DSA_128S_SEED_LEN]) -> SlhDsa128sKeyPair |  |  |  |  |

| `slh_dsa128s_generate` | function | slh_dsa128s_generate() -> Result<SlhDsa128sKeyPair, PqcError> |  |  |  |  |

| `slh_dsa128s_sign` | function | slh_dsa128s_sign( seeds: &[u8; SLH_DSA_128S_SEED_LEN], msg: &[u8], ) -> Result<Vec<u8>, PqcError> |  |  |  |  |

| `slh_dsa128s_verify` | function | slh_dsa128s_verify(public: &[u8], msg: &[u8], sig: &[u8]) -> Result<bool, PqcError> |  |  |  |  |

| `FIPS 204` | str_key | ML_DSA_65_SPEC_REF = "FIPS 204" |  |  |  |  |

| `FIPS 205` | str_key | SLH_DSA_128S_SPEC_REF = "FIPS 205" |  |  |  |  |

| `affidavit pqc lane: ML-DSA-65 / SLH-DSA-SHA2-128s / hybrid` | str_key | MSG = "affidavit pqc lane: ML-DSA-65 / SLH-DSA-SHA2-128s / hybrid" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | PQC_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `draft-ietf-lamps-pq-composite-sig` | str_key | HYBRID_SPEC_REF = "draft-ietf-lamps-pq-composite-sig" |  |  |  |  |

| `HybridSecret` | struct | HybridSecret { pub es256: Es256SigningKey, pub mldsa65_seed: [u8; ML_DSA_65_SEED_LEN] } |  |  |  |  |

| `HybridSignature` | struct | HybridSignature { pub es256_der: Vec<u8>, pub mldsa65: Vec<u8> } |  |  |  |  |

| `MlDsa65KeyPair` | struct | MlDsa65KeyPair { pub seed: [u8; ML_DSA_65_SEED_LEN], pub public: Vec<u8> } |  |  |  |  |

| `SlhDsa128sKeyPair` | struct | SlhDsa128sKeyPair { pub seeds: [u8; SLH_DSA_128S_SEED_LEN], pub public: Vec<u8> } |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `ProviderRefusal` | enum | ProviderRefusal { KeyMismatch { envelope: String, provider: String, }, AlgorithmMismatch { envelope: &'static str, provider: &'static str, }, MalformedEnvelope(String), EmptySignature, Provider(String) } |  |  |  |  |

| `from_seed` | function | from_seed(id: KeyId, seed: &[u8; 32]) -> Result<Self, ProviderRefusal> |  |  |  |  |

| `generate` | function | generate(id: KeyId) -> Result<Self, ProviderRefusal> |  |  |  |  |

| `key_record` | function | key_record(&self, custodian: CustodianIdentity, created_epoch: u64) -> KeyRecord |  |  |  |  |

| `sign_with_provider` | function | sign_with_provider( provider: &P, envelope: SignatureEnvelope, ) -> Result<DetachedSignature, ProviderRefusal> |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `DetachedSignature` | struct | DetachedSignature { pub envelope: SignatureEnvelope, pub signature: Vec<u8> } |  |  |  |  |

| `SoftwareEs256Provider` | struct | SoftwareEs256Provider { id: KeyId, key: Es256SigningKey } |  |  |  |  |

| `SigningProvider` | trait |  |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `QUORUM_DEFAULT_K` | const | QUORUM_DEFAULT_K: usize |  |  |  |  |

| `QUORUM_MAX_N` | const | QUORUM_MAX_N: usize |  |  |  |  |

| `QuorumError` | enum | QuorumError { KExceedsN { k: usize, n: usize }, InsufficientShares { got: usize, needed: usize }, DuplicateSigner(String), InvalidShare(String), AlgorithmRefused(crate::crypto_trust_keys::AlgorithmId), Registry(String) } |  |  |  |  |

| `allowed_algorithms` | function | allowed_algorithms(&self) -> &BTreeSet<AlgorithmId> |  |  |  |  |

| `new` | function | new(registry: &'a R) -> QuorumEngine<'a, R> |  |  |  |  |

| `verify_quorum` | function | verify_quorum( &self, signing_input: &[u8], shares: &[SignatureShare], k: usize, ) -> Result<QuorumVerdict, QuorumError> |  |  |  |  |

| `with_allowed_algorithms` | function | with_allowed_algorithms( mut self, algs: impl IntoIterator<Item = AlgorithmId>, ) -> QuorumEngine<'a, R> |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `QuorumEngine` | struct | QuorumEngine { registry: &'a R, allowed_algorithms: BTreeSet<AlgorithmId> } |  |  |  |  |

| `QuorumVerdict` | struct | QuorumVerdict { pub satisfied: bool, pub valid_shares: usize, pub distinct_signers: usize } |  |  |  |  |

| `SignatureShare` | struct | SignatureShare { pub key_id: crate::crypto_trust_keys::KeyId, pub algorithm: crate::crypto_trust_keys::AlgorithmId, pub signature: Vec<u8> } |  |  |  |  |

| `CRL_FORMAT` | const | CRL_FORMAT: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `THIS_EPOCH` | const | THIS_EPOCH: u64 |  |  |  |  |

| `RevocationPubError` | enum | RevocationPubError { BadFormat(String), BadSignature, "stale publication: epoch {published} vs current {current}, max staleness {max_staleness}" )] StaleEpoch { published: u64, current: u64, max_staleness: u64, }, Serialization(String), Lifecycle(String) } |  |  |  |  |

| `apply_to` | function | apply_to( revocations: &mut RevocationList, crl: &SignedRevocationList, issuer_pk_sec1: &[u8], current_epoch: u64, max_staleness: u64, now: u64, ) -> Result<usize, RevocationPubError> |  |  |  |  |

| `publish` | function | publish( revocations: &RevocationList, issuer: &Es256SigningKey, issuer_kid: &KeyId, epoch: u64, at: u64, ) -> Result<SignedRevocationList, RevocationPubError> |  |  |  |  |

| `verify_publication` | function | verify_publication( crl: &SignedRevocationList, issuer_pk_sec1: &[u8], ) -> Result<bool, RevocationPubError> |  |  |  |  |

| `CTP-CRL-v1` | str_key | CRL_FORMAT = "CTP-CRL-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `compromised` | str_key | REASON = "compromised" |  |  |  |  |

| `k-a` | str_key | KID_A = "k-a" |  |  |  |  |

| `k-b` | str_key | KID_B = "k-b" |  |  |  |  |

| `RevocationRecordMirror` | struct | RevocationRecordMirror { pub kid: String, pub revoked_at: u64, pub reason: String } |  |  |  |  |

| `SignedRevocationList` | struct | SignedRevocationList { pub format: String, pub issuer_kid: String, pub epoch: u64, pub published_at: u64, pub revoked: Vec<RevocationRecordMirror>, pub signature: Vec<u8> } |  |  |  |  |

| `MIGRATIONS` | const | MIGRATIONS: &[(&str, &str, &str)] |  |  |  |  |

| `ROTATION_DIGEST_LABEL` | const | ROTATION_DIGEST_LABEL: &[u8] |  |  |  |  |

| `ROTATION_DOMAIN_TAG` | const | ROTATION_DOMAIN_TAG: &str |  |  |  |  |

| `RotationError` | enum | RotationError { MigrationRefused(CryptoProfile, CryptoProfile), SuccessorSignatureInvalid, RecordTampered, Provider(String), Serialization(String) } |  |  |  |  |

| `admission_allowed` | function | admission_allowed(from: CryptoProfile, to: CryptoProfile) -> bool |  |  |  |  |

| `assert_not_downgrade` | function | assert_not_downgrade(from: CryptoProfile, to: CryptoProfile) -> Result<(), RotationError> |  |  |  |  |

| `assert_not_downgrade_under` | function | assert_not_downgrade_under( from: CryptoProfile, to: CryptoProfile, policy: &MigrationPolicy, ) -> Result<(), RotationError> |  |  |  |  |

| `rotate_es256_to_hybrid` | function | rotate_es256_to_hybrid( old: &Es256SigningKey, hybrid_secret: &HybridSecret, at: u64, ) -> Result<RotationRecord, RotationError> |  |  |  |  |

| `verify_rotation` | function | verify_rotation( record: &RotationRecord, hybrid_pk_es256: &[u8], hybrid_pk_mldsa65: &[u8], ) -> Result<bool, RotationError> |  |  |  |  |

| `CLASSICAL` | str_key | MIGRATIONS = "CLASSICAL" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | ROTATION_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `ctp.rotation.record.v1` | str_key | ROTATION_DIGEST_LABEL = "ctp.rotation.record.v1" |  |  |  |  |

| `MigrationPolicy` | struct | MigrationPolicy { pub strict_upgrades_only: bool } |  |  |  |  |

| `RotationCeremony` | struct |  |  |  |  |  |

| `RotationRecord` | struct | RotationRecord { pub old_key_id: String, pub new_key_id: String, pub from_profile: String, pub to_profile: String, pub rotated_at: u64, pub successor_signature: Vec<u8> } |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ROTATION_STORE_FILE` | const | ROTATION_STORE_FILE: &str |  |  |  |  |

| `STORE_FORMAT` | const | STORE_FORMAT: &str |  |  |  |  |

| `RotationStoreError` | enum | RotationStoreError { Io(String), Serialization { line: u64, message: String }, WrongFormat { line: u64, found: String }, Tampered { line: u64, kid: String }, Verification { line: u64, message: String } } |  |  |  |  |

| `append` | function | append( &self, record: &RotationRecord, successor_public_es256: &[u8], successor_public_mldsa65: &[u8], ) -> Result<u64, RotationStoreError> |  |  |  |  |

| `latest_for_predecessor` | function | latest_for_predecessor( &self, kid: &str, ) -> Result<Option<VerifiedRotation>, RotationStoreError> |  |  |  |  |

| `latest_for_successor` | function | latest_for_successor( &self, kid: &str, ) -> Result<Option<VerifiedRotation>, RotationStoreError> |  |  |  |  |

| `load` | function | load(&self) -> Result<Vec<VerifiedRotation>, RotationStoreError> |  |  |  |  |

| `new` | function | new(path: impl Into<PathBuf>) -> RotationStore |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `.affi/rotations.jsonl` | str_key | ROTATION_STORE_FILE = ".affi/rotations.jsonl" |  |  |  |  |

| `CTP-ROTSTORE-v1` | str_key | STORE_FORMAT = "CTP-ROTSTORE-v1" |  |  |  |  |

| `CTP_ROTSTORE_XPROC_STORE` | str_key | XPROC_ENV = "CTP_ROTSTORE_XPROC_STORE" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `RotationStore` | struct | RotationStore { path: PathBuf } |  |  |  |  |

| `RotationStoreEnvelope` | struct | RotationStoreEnvelope { pub format: String, pub record: RotationRecord, pub successor_public_es256: Vec<u8>, pub successor_public_mldsa65: Vec<u8> } |  |  |  |  |

| `VerifiedRotation` | struct | VerifiedRotation { pub line: u64, pub record: RotationRecord, pub successor_public_es256: Vec<u8>, pub successor_public_mldsa65: Vec<u8> } |  |  |  |  |

| `ENVELOPE_FIELD_NAMES` | const | ENVELOPE_FIELD_NAMES: [&str; 12] |  |  |  |  |

| `SA2A_APPROVAL_TAG` | const | SA2A_APPROVAL_TAG: &str |  |  |  |  |

| `Sa2aWireError` | enum | Sa2aWireError { WrongVersion(String), UnknownAlgorithm(String), MalformedHex(String), Malformed(String) } |  |  |  |  |

| `approval_to_envelope` | function | approval_to_envelope(a: &Sa2aApproval) -> Result<SignatureEnvelope, Sa2aWireError> |  |  |  |  |

| `envelope_sa2a_signing_input` | function | envelope_sa2a_signing_input( env: &SignatureEnvelope, principal: &str, ) -> Result<Vec<u8>, Sa2aWireError> |  |  |  |  |

| `envelope_to_approval` | function | envelope_to_approval( env: &SignatureEnvelope, principal: &str, ) -> Result<Sa2aApproval, Sa2aWireError> |  |  |  |  |

| `sa2a_signing_input` | function | sa2a_signing_input(a: &Sa2aApproval) -> Vec<u8> |  |  |  |  |

| `sa2a_signing_input_checked` | function | sa2a_signing_input_checked(a: &Sa2aApproval) -> Result<Vec<u8>, Sa2aWireError> |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `SA2A-C2-APPROVAL-v1` | str_key | SA2A_APPROVAL_TAG = "SA2A-C2-APPROVAL-v1" |  |  |  |  |

| `subject-a` | str_key | PRINCIPAL = "subject-a" |  |  |  |  |

| `version` | str_key | ENVELOPE_FIELD_NAMES = "version" |  |  |  |  |

| `Sa2aApproval` | struct | Sa2aApproval { pub v: String, pub alg: String, pub kid: String, pub effect_digest: String, pub principal: String, pub policy_epoch: u64, pub revocation_epoch: u64, pub generation: u32, pub nonce: String, pub not_before: u64, pub expires: u64, pub audience: String } |  |  |  |  |

| `DIGEST_ALGORITHM` | const | DIGEST_ALGORITHM: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `SEALED_RECEIPT_FORMAT` | const | SEALED_RECEIPT_FORMAT: &str |  |  |  |  |

| `SealError` | enum | SealError { SubjectMismatch, Chain(String), Verification(String), Serialization(String) } |  |  |  |  |

| `seal_receipt` | function | seal_receipt( receipt: &Receipt, envelope: SignatureEnvelope, signature: Vec<u8>, ) -> Result<SealedReceipt, SealError> |  |  |  |  |

| `subject_digest_of` | function | subject_digest_of(receipt: &Receipt) -> Result<[u8; 32], SealError> |  |  |  |  |

| `verify_sealed` | function | verify_sealed( sealed: &SealedReceipt, engine: &VerificationEngine, ) -> Result<CryptographicVerdict, SealError> |  |  |  |  |

| `BLAKE3` | str_key | DIGEST_ALGORITHM = "BLAKE3" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `PQ-SEAL-v1` | str_key | SEALED_RECEIPT_FORMAT = "PQ-SEAL-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `SealedReceipt` | struct | SealedReceipt { pub base: Receipt, pub envelope: SignatureEnvelope, pub signature: Vec<u8> } |  |  |  |  |

| `DIGEST_ALGORITHM` | const | DIGEST_ALGORITHM: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `STORE_FILE` | const | STORE_FILE: &str |  |  |  |  |

| `STORE_FORMAT` | const | STORE_FORMAT: &str |  |  |  |  |

| `StoreError` | enum | StoreError { ChecksumMismatch { claimed: String, recomputed: String, }, WrongFormat(String), Io(String), Canonical(String), Registry( Serialization( } |  |  |  |  |

| `checksum_for` | function | checksum_for(records: &[KeyRecord]) -> Result<String, StoreError> |  |  |  |  |

| `load` | function | load(&self) -> Result<KeyStoreFile, StoreError> |  |  |  |  |

| `open` | function | open(path: P) -> Result<FileKeyStore, StoreError> |  |  |  |  |

| `path` | function | path(&self) -> &Path |  |  |  |  |

| `records` | function | records(&self) -> Result<Vec<KeyRecord>, StoreError> |  |  |  |  |

| `register_checked` | function | register_checked(&mut self, record: KeyRecord) -> Result<(), StoreError> |  |  |  |  |

| `.affi/keys.json` | str_key | STORE_FILE = ".affi/keys.json" |  |  |  |  |

| `BLAKE3` | str_key | DIGEST_ALGORITHM = "BLAKE3" |  |  |  |  |

| `CTP-STORE-v1` | str_key | STORE_FORMAT = "CTP-STORE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `FileKeyStore` | struct | FileKeyStore { path: PathBuf, records: BTreeMap<KeyId, KeyRecord> } |  |  |  |  |

| `KeyStoreFile` | struct | KeyStoreFile { pub format: String, pub records: Vec<KeyRecord>, pub checksum: String } |  |  |  |  |

| `CONSISTENCY_DOMAIN` | const | CONSISTENCY_DOMAIN: &str |  |  |  |  |

| `INCLUSION_DOMAIN` | const | INCLUSION_DOMAIN: &str |  |  |  |  |

| `LOG_FORMAT` | const | LOG_FORMAT: &str |  |  |  |  |

| `MERKLE_DOMAIN` | const | MERKLE_DOMAIN: &str |  |  |  |  |

| `TransparencyError` | enum | TransparencyError { LeafOutOfRange(u64), SizeOutOfRange(u64, u64) } |  |  |  |  |

| `append` | function | append(&mut self, commitment: [u8; 32]) -> u64 |  |  |  |  |

| `consistency_proof` | function | consistency_proof(&self, first: u64) -> Result<ConsistencyProof, TransparencyError> |  |  |  |  |

| `head` | function | head(&self) -> [u8; 32] |  |  |  |  |

| `inclusion_proof` | function | inclusion_proof(&self, idx: u64) -> Result<InclusionProof, TransparencyError> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `leaf` | function | leaf(&self, index: u64) -> Option<&[u8; 32]> |  |  |  |  |

| `len` | function | len(&self) -> u64 |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `verify_consistency` | function | verify_consistency( proof: &ConsistencyProof, first_head: &[u8; 32], second_head: &[u8; 32], ) -> bool |  |  |  |  |

| `verify_inclusion` | function | verify_inclusion(leaf: &[u8; 32], proof: &InclusionProof, head: &[u8; 32]) -> bool |  |  |  |  |

| `CTP-TRANS-v1` | str_key | LOG_FORMAT = "CTP-TRANS-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1/consistency` | str_key | CONSISTENCY_DOMAIN = "affidavit.crypto-trust-plane.v1/consistency" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1/inclusion` | str_key | INCLUSION_DOMAIN = "affidavit.crypto-trust-plane.v1/inclusion" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1/merkle` | str_key | MERKLE_DOMAIN = "affidavit.crypto-trust-plane.v1/merkle" |  |  |  |  |

| `ConsistencyProof` | struct | ConsistencyProof { pub first: u64, pub second: u64, pub path: Vec<[u8; 32]> } |  |  |  |  |

| `InclusionProof` | struct | InclusionProof { pub leaf_index: u64, pub tree_size: u64, pub path: Vec<[u8; 32]> } |  |  |  |  |

| `TransparencyLog` | struct | TransparencyLog { leaves: Vec<[u8; 32]> } |  |  |  |  |

| `CRYPTO_STANDING_PROFILE` | const | CRYPTO_STANDING_PROFILE: &str |  |  |  |  |

| `DEFAULT_PROFILE` | const | DEFAULT_PROFILE: &str |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `ENVELOPE_VERSION` | const | ENVELOPE_VERSION: &str |  |  |  |  |

| `MAX_REVOCATION_STALENESS_SECONDS` | const | MAX_REVOCATION_STALENESS_SECONDS: u64 |  |  |  |  |

| `MIN_PROFILE` | const | MIN_PROFILE: &str |  |  |  |  |

| `CryptographicStanding` | enum | CryptographicStanding { Valid, Invalid, Expired, Revoked, ReplayRejected, UnknownKey, ProfileRefused, Malformed } |  |  |  |  |

| `StandingReceiptError` | enum | StandingReceiptError { ReceiptHashMismatch { claimed: String, recomputed: String }, BadSignature { signer_kid: String }, Serialization(String) } |  |  |  |  |

| `VerifyRefusal` | enum | VerifyRefusal { MalformedEnvelope(String), UnknownKey(String), KeyRevoked(String), StaleRevocationEpoch(String), Expired(u64), NotYetValid(u64), ReplayRejected(String), ProfileRefused(crate::crypto_trust_keys::AlgorithmId), AudienceRefused(String), ProfileFloor(CryptoProfile), Provider(String), InvalidSignature(String), SubjectMismatch(String) } |  |  |  |  |

| `all` | function | all() -> &'static [CryptographicStanding] |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `certify_signed` | function | certify_signed( &self, env: &SignatureEnvelope, signature: &[u8], subject: &str, signing: &Es256SigningKey, ) -> Result<CryptoStandingReceipt, VerifyRefusal> |  |  |  |  |

| `from_graph_defaults` | function | from_graph_defaults() -> Self |  |  |  |  |

| `new` | function | new( registry: InMemoryKeyRegistry, revocations: RevocationList, nonces: NonceJournal, policy: TrustPolicy, ) -> Self |  |  |  |  |

| `nonce_seen_at` | function | nonce_seen_at(&self, kid: &str, nonce: &[u8; 16]) -> Option<u64> |  |  |  |  |

| `policy` | function | policy(&self) -> &TrustPolicy |  |  |  |  |

| `prune_nonces` | function | prune_nonces(&mut self) -> usize |  |  |  |  |

| `register_key` | function | register_key(&mut self, record: KeyRecord) -> Result<(), RegistryError> |  |  |  |  |

| `registry` | function | registry(&self) -> &InMemoryKeyRegistry |  |  |  |  |

| `revocations` | function | revocations(&self) -> &RevocationList |  |  |  |  |

| `revoke_key` | function | revoke_key(&mut self, kid: &str, at: u64, reason: String) |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), StandingReceiptError> |  |  |  |  |

| `verify_envelope` | function | verify_envelope( &self, env: &SignatureEnvelope, signature: &[u8], ) -> Result<CryptographicVerdict, VerifyRefusal> |  |  |  |  |

| `with_now` | function | with_now(mut self, now: u64) -> Self |  |  |  |  |

| `CLASSICAL` | str_key | MIN_PROFILE = "CLASSICAL" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `HYBRID` | str_key | DEFAULT_PROFILE = "HYBRID" |  |  |  |  |

| `VALID` | str_key | EXPECTED = "VALID" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `affidavit/crypto-standing/v1` | str_key | CRYPTO_STANDING_PROFILE = "affidavit/crypto-standing/v1" |  |  |  |  |

| `CryptoStandingReceipt` | struct | CryptoStandingReceipt { pub profile: String, pub subject: String, pub envelope_commitment: String, pub key_id: String, pub algorithm: String, pub standing: CryptographicStanding, pub observed_at: u64, pub receipt_hash: String, pub signer_kid: String, pub signer_public_key: Vec<u8>, pub signature: Vec<u8>, _seal: () } |  |  |  |  |

| `CryptographicVerdict` | struct | CryptographicVerdict { pub standing: CryptographicStanding, pub key_id: Option<KeyId>, pub subject_digest: [u8; 32] } |  |  |  |  |

| `TrustPolicy` | struct | TrustPolicy { pub allowed_audiences: std::collections::BTreeSet<String>, pub min_profile: CryptoProfile, pub allowed: std::collections::BTreeSet<AlgorithmId>, pub max_revocation_staleness_seconds: u64, pub now: u64 } |  |  |  |  |

| `VerificationEngine` | struct | VerificationEngine { registry: InMemoryKeyRegistry, revocations: RevocationList, nonces: RefCell<NonceJournal>, policy: TrustPolicy } |  |  |  |  |

| `DOMAIN_TAG` | const | DOMAIN_TAG: &str |  |  |  |  |

| `MIN_WITNESSES` | const | MIN_WITNESSES: usize |  |  |  |  |

| `WitnessError` | enum | WitnessError { InsufficientWitnesses { got: usize, needed: usize }, DuplicateWitness(String), UnknownWitness(String), MalformedCosignature(String), Serialization(String) } |  |  |  |  |

| `collect` | function | collect( &self, witnesses: &[(&str, &Es256SigningKey)], ) -> Result<Vec<WitnessSignature>, WitnessError> |  |  |  |  |

| `head` | function | head(&self) -> &SignedTreeHead |  |  |  |  |

| `new` | function | new(head: SignedTreeHead) -> Self |  |  |  |  |

| `preimage` | function | preimage(&self) -> Result<[u8; 32], WitnessError> |  |  |  |  |

| `verify_cosigned` | function | verify_cosigned( head: &SignedTreeHead, cosigs: &[WitnessSignature], min: usize, pks: &[(&str, &[u8])], ) -> Result<CosignVerdict, WitnessError> |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `afk1_transparency-log-ops` | str_key | SIGNER_KID = "afk1_transparency-log-ops" |  |  |  |  |

| `CosignVerdict` | struct | CosignVerdict { pub cosigned: bool, pub valid_witnesses: usize, pub distinct_witnesses: usize } |  |  |  |  |

| `WitnessCosign` | struct | WitnessCosign { head: SignedTreeHead } |  |  |  |  |

| `WitnessSignature` | struct | WitnessSignature { pub kid: String, pub signature: Vec<u8> } |  |  |  |  |

| `DFCM_AUTHORITY_CEILING` | const | DFCM_AUTHORITY_CEILING: &str |  |  |  |  |

| `DFCM_CLAIM_CEILING` | const | DFCM_CLAIM_CEILING: &str |  |  |  |  |

| `DFCM_PROFILE` | const | DFCM_PROFILE: &str |  |  |  |  |

| `ClosureGap` | enum | ClosureGap { MissingSubject { repository: String, expected_candidate: String, }, ExactSubjectMoved { repository: String, expected_candidate: String, observed_candidates: Vec<String>, }, MissingEvidence { subject: ExactSubject, evidence: EvidenceKey, }, StandingMismatch { subject: ExactSubject, required: String, observed: String, }, MissingStandingReceipt { subject: ExactSubject, } } |  |  |  |  |

| `DfcmRefusal` | enum | DfcmRefusal { EmptyField(&'static str), NoObligations, ZeroFrontierBudget, DuplicateObligation(String), NoProofPaths(String), DuplicatePath(String, String), EmptyPath(String, String), DuplicateSubjectRequirement(ExactSubject), DuplicateEvidenceRequirement(ExactSubject, EvidenceKey), DuplicateObservation(ExactSubject), DuplicateEvidenceWitness(ExactSubject, EvidenceKey), MalformedBlake3(&'static str), StandingReceiptInvalid(ExactSubject, String), StandingReceiptSubjectMismatch(ExactSubject, ExactSubject), StandingReceiptStandingMismatch(ExactSubject, String, String), FrontierBudgetExceeded { budget: usize, required: usize }, NonCanonicalOrder(&'static str), WrongProfile, WrongClaimCeiling, WrongAuthorityCeiling, ObligationEvaluationMismatch, FrontierMismatch, ClosureMismatch, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `EvidenceKind` | enum | EvidenceKind { ImplementedSource, ObservedExecution, ExactHeadCourt, HostedCi, RuntimeStanding, Merge, Publication } |  |  |  |  |

| `certify_dfcm` | function | certify_dfcm( mut release_profile: DfcmProfile, mut observations: Vec<SubjectObservation>, ) -> Result<DfcmReceipt, DfcmRefusal> |  |  |  |  |

| `v26_9_18_profile` | function | v26_9_18_profile(s: V26_9_18Subjects) -> DfcmProfile |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), DfcmRefusal> |  |  |  |  |

| `EVIDENCE_ONLY_NO_SELECT_CONSTRUCT_OR_DO_AUTHORITY` | str_key | DFCM_AUTHORITY_CEILING = "EVIDENCE_ONLY_NO_SELECT_CONSTRUCT_OR_DO_AUTHORITY" |  |  |  |  |

| `EXACT_SUBJECT_RELEASE_CLOSURE_ONLY_NO_RUNTIME_MERGE_PUBLICATION_OR_DEPLOYMENT_INFERENCE` | str_key | DFCM_CLAIM_CEILING = "EXACT_SUBJECT_RELEASE_CLOSURE_ONLY_NO_RUNTIME_MERGE_PUBLICATION_OR_DEPLOYMENT_INFERENCE" |  |  |  |  |

| `affidavit/dfcm/v1` | str_key | DFCM_PROFILE = "affidavit/dfcm/v1" |  |  |  |  |

| `DfcmProfile` | struct | DfcmProfile { pub release: String, pub certifier: ExactSubject, pub frontier_budget: usize, pub obligations: Vec<Obligation> } |  |  |  |  |

| `DfcmReceipt` | struct | DfcmReceipt { pub profile: String, pub claim_ceiling: String, pub authority_ceiling: String, pub release_profile: DfcmProfile, pub observations: Vec<SubjectObservation>, pub obligations: Vec<ObligationEvaluation>, pub minimal_frontiers: Vec<MinimalFrontier>, pub closed: bool, pub receipt_hash: Blake3Hash } |  |  |  |  |

| `EvidenceKey` | struct | EvidenceKey { pub kind: EvidenceKind, pub court: String } |  |  |  |  |

| `EvidenceWitness` | struct | EvidenceWitness { pub key: EvidenceKey, pub commitment: Blake3Hash } |  |  |  |  |

| `ExactSubject` | struct | ExactSubject { pub repository: String, pub candidate: String } |  |  |  |  |

| `MinimalFrontier` | struct | MinimalFrontier { pub gaps: Vec<ClosureGap> } |  |  |  |  |

| `Obligation` | struct | Obligation { pub id: String, pub description: String, pub paths: Vec<ProofPath> } |  |  |  |  |

| `ObligationEvaluation` | struct | ObligationEvaluation { pub obligation: String, pub satisfied: bool, pub paths: Vec<PathEvaluation> } |  |  |  |  |

| `PathEvaluation` | struct | PathEvaluation { pub name: String, pub satisfied: bool, pub gaps: Vec<ClosureGap> } |  |  |  |  |

| `ProofPath` | struct | ProofPath { pub name: String, pub subjects: Vec<SubjectRequirement> } |  |  |  |  |

| `SubjectObservation` | struct | SubjectObservation { pub subject: ExactSubject, pub standing: Standing, pub evidence: Vec<EvidenceWitness>, pub standing_receipt: Option<StandingReceipt> } |  |  |  |  |

| `SubjectRequirement` | struct | SubjectRequirement { pub subject: ExactSubject, pub required_evidence: Vec<EvidenceKey>, pub required_standing: Option<Standing>, pub require_standing_receipt: bool } |  |  |  |  |

| `V26_9_18Subjects` | struct | V26_9_18Subjects { pub affidavit: ExactSubject, pub ggen: ExactSubject, pub bcinr: ExactSubject, pub ash_r2rml: ExactSubject, pub ggen_igniter: ExactSubject, pub wasm4pm: ExactSubject, pub unrdf: ExactSubject } |  |  |  |  |

| `INTERNAL` | const | INTERNAL: i32 |  |  |  |  |

| `IO_ERROR` | const | IO_ERROR: i32 |  |  |  |  |

| `OK` | const | OK: i32 |  |  |  |  |

| `REJECT` | const | REJECT: i32 |  |  |  |  |

| `SLA_BREACH` | const | SLA_BREACH: i32 |  |  |  |  |

| `USAGE_ERROR` | const | USAGE_ERROR: i32 |  |  |  |  |

| `ErrorCode` | enum | ErrorCode { ChainHashMismatch = 1001, GenesisHashMismatch = 1002, SeqGap = 1003, DuplicateEventId = 1004, InvalidCommitment = 1005, TamperedReceipt = 1006, UnknownFormatVersion = 1100, MalformedReceipt = 1101, MissingEventType = 1102, UnknownProfile = 1200, ProfileViolation = 1201, ReceiptNotFound = 1300, ReceiptUnreadable = 1301, WorkingDirMissing = 1302, InvalidObjectId = 1400, InvalidEventType = 1401 } |  |  |  |  |

| `code` | function | code(self) -> u16 |  |  |  |  |

| `exit_code` | function | exit_code(self) -> i32 |  |  |  |  |

| `from_error` | function | from_error(code: ErrorCode, err: &dyn std::error::Error) -> Self |  |  |  |  |

| `hint` | function | hint(self) -> Option<&'static str> |  |  |  |  |

| `message` | function | message(self) -> &'static str |  |  |  |  |

| `new` | function | new(code: ErrorCode, message: impl Into<String>) -> Self |  |  |  |  |

| `with_hint` | function | with_hint(mut self, hint: impl Into<String>) -> Self |  |  |  |  |

| `with_span` | function | with_span(mut self, file: impl Into<String>, line: Option<u32>) -> Self |  |  |  |  |

| `Diag` | struct | Diag { pub code: u16, pub message: String, pub hint: Option<String>, pub span: Option<Span> } |  |  |  |  |

| `Span` | struct | Span { pub file: String, pub line: Option<u32> } |  |  |  |  |

| `diff_json_receipts` | function | diff_json_receipts(old_json: &str, new_json: &str) -> anyhow::Result<DiffResult> |  |  |  |  |

| `diff_receipts` | function | diff_receipts(old: &Receipt, new: &Receipt) -> DiffResult |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `DiffEntry` | struct | DiffEntry { pub seq: u64, pub event_type: String, pub commitment_prefix: String } |  |  |  |  |

| `DiffResult` | struct | DiffResult { pub added: Vec<DiffEntry>, pub removed: Vec<DiffEntry>, pub modified: Vec<ModifiedEntry> } |  |  |  |  |

| `ModifiedEntry` | struct | ModifiedEntry { pub seq: u64, pub old: DiffEntry, pub new: DiffEntry } |  |  |  |  |

| `ACTIVITY_KEY` | const | ACTIVITY_KEY: &str |  |  |  |  |

| `conformance_metrics` | function | conformance_metrics(receipt: &Receipt) -> (f64, f64) |  |  |  |  |

| `discover_dfg_summary` | function | discover_dfg_summary(receipt: &Receipt) -> (usize, usize, usize, usize) |  |  |  |  |

| `discover_from_admitted` | function | discover_from_admitted(admitted: &crate::types::AdmittedReceipt) -> String |  |  |  |  |

| `discover_process_tree` | function | discover_process_tree(receipt: &Receipt) -> String |  |  |  |  |

| `project_to_event_log` | function | project_to_event_log(receipt: &Receipt) -> EventLog |  |  |  |  |

| `quality_metrics` | function | quality_metrics(receipt: &Receipt) -> (f64, f64, f64) |  |  |  |  |

| `quality_metrics_from_admitted` | function | quality_metrics_from_admitted(admitted: &crate::types::AdmittedReceipt) -> (f64, f64, f64) |  |  |  |  |

| `concept:name` | str_key | ACTIVITY_KEY = "concept:name" |  |  |  |  |

| `DOCTOR_CHECKS` | const | DOCTOR_CHECKS: [&'static dyn DoctorCheck] |  |  |  |  |

| `FindingStatus` | enum | FindingStatus { Ok, Warn, Fail } |  |  |  |  |

| `auto_fixable` | function | auto_fixable(mut self) -> Self |  |  |  |  |

| `fail` | function | fail( id: &'static str, message: impl Into<String>, remediation: impl Into<String>, ) -> Self |  |  |  |  |

| `label` | function | label(&self) -> &'static str |  |  |  |  |

| `ok` | function | ok(id: &'static str, message: impl Into<String>) -> Self |  |  |  |  |

| `run_all` | function | run_all() -> Vec<Finding> |  |  |  |  |

| `warn` | function | warn( id: &'static str, message: impl Into<String>, remediation: impl Into<String>, ) -> Self |  |  |  |  |

| `Finding` | struct | Finding { pub id: &'static str, pub status: FindingStatus, pub message: String, pub remediation: Option<String>, pub auto_fixable: bool } |  |  |  |  |

| `DoctorCheck` | trait |  |  |  |  |  |

| `ECOSYSTEM_AUTHORITY_CEILING` | const | ECOSYSTEM_AUTHORITY_CEILING: &str |  |  |  |  |

| `ECOSYSTEM_CLAIM_CEILING` | const | ECOSYSTEM_CLAIM_CEILING: &str |  |  |  |  |

| `ECOSYSTEM_PROFILE` | const | ECOSYSTEM_PROFILE: &str |  |  |  |  |

| `EcosystemRefusal` | enum | EcosystemRefusal { NoRequirements, ZeroAliveRequirement(EcosystemRole), DuplicateRequirement(EcosystemRole), DuplicateMember { role: EcosystemRole, subject: String, candidate: String, }, MemberInvalid { subject: String, reason: String, }, NonCanonicalOrder(&'static str), EmptyField(&'static str), MalformedBlake3(&'static str), WrongProfile, WrongClaimCeiling, WrongAuthorityCeiling, CoverageMismatch, StandingMismatch, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `EcosystemRole` | enum | EcosystemRole { SemanticAuthority, Manufacture, PlanningControl, WorldExecution, FormalProof, RuntimeExecution, ProcessEvidence, Configuration, SupplyChain, Verification, Replay } |  |  |  |  |

| `certify_ecosystem` | function | certify_ecosystem( admitted: &AdmittedReceipt, mut observation: EcosystemObservation, ) -> Result<EcosystemReceipt, EcosystemRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), EcosystemRefusal> |  |  |  |  |

| `EVIDENCE_ONLY_NO_AMBIENT_ACTUATION_AUTHORITY` | str_key | ECOSYSTEM_AUTHORITY_CEILING = "EVIDENCE_ONLY_NO_AMBIENT_ACTUATION_AUTHORITY" |  |  |  |  |

| `VERIFIED_STANDING_COMPOSITION_ONLY_NO_TRANSITIVE_TRUTH_OR_ACTUATION_CLAIM` | str_key | ECOSYSTEM_CLAIM_CEILING = "VERIFIED_STANDING_COMPOSITION_ONLY_NO_TRANSITIVE_TRUTH_OR_ACTUATION_CLAIM" |  |  |  |  |

| `affidavit/ecosystem/v1` | str_key | ECOSYSTEM_PROFILE = "affidavit/ecosystem/v1" |  |  |  |  |

| `EcosystemMember` | struct | EcosystemMember { pub role: EcosystemRole, pub standing_receipt: StandingReceipt } |  |  |  |  |

| `EcosystemObservation` | struct | EcosystemObservation { pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub requirements: Vec<RoleRequirement>, pub members: Vec<EcosystemMember>, pub previous_receipt: Option<Blake3Hash> } |  |  |  |  |

| `EcosystemReceipt` | struct | EcosystemReceipt { pub profile: String, pub claim_ceiling: String, pub authority_ceiling: String, pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub admitted_receipt_hash: Blake3Hash, pub requirements: Vec<RoleRequirement>, pub members: Vec<EcosystemMember>, pub coverage: Vec<RoleCoverage>, pub standing: Standing, pub previous_receipt: Option<Blake3Hash>, pub receipt_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `RoleCoverage` | struct | RoleCoverage { pub role: EcosystemRole, pub required_alive: usize, pub total: usize, pub alive: usize, pub partial_alive: usize, pub unknown: usize, pub blocked: usize, pub build_broken: usize, pub unsupported: usize, pub satisfied: bool } |  |  |  |  |

| `RoleRequirement` | struct | RoleRequirement { pub role: EcosystemRole, pub minimum_alive: usize } |  |  |  |  |

| `Ed25519WitnessError` | enum | Ed25519WitnessError { MalformedPublicKey, MalformedSignature, VerificationFailed } |  |  |  |  |

| `from_rng` | function | from_rng(rng: &mut R) -> Self |  |  |  |  |

| `generate` | function | generate() -> Self |  |  |  |  |

| `public` | function | public(&self) -> [u8; 32] |  |  |  |  |

| `sign` | function | sign(&self, message: &[u8]) -> [u8; 64] |  |  |  |  |

| `verify_witness` | function | verify_witness( public_key: &[u8; 32], message: &[u8], signature: &[u8; 64], ) -> Result<(), Ed25519WitnessError> |  |  |  |  |

| `WitnessKeyPair` | struct | WitnessKeyPair { signing: SigningKey } |  |  |  |  |

| `ERRC_CLAIM_CEILING` | const | ERRC_CLAIM_CEILING: &str |  |  |  |  |

| `ERRC_PROFILE` | const | ERRC_PROFILE: &str |  |  |  |  |

| `ERRC_SOURCE_ARTIFACT` | const | ERRC_SOURCE_ARTIFACT: &str |  |  |  |  |

| `ERRC_SOURCE_COMMIT` | const | ERRC_SOURCE_COMMIT: &str |  |  |  |  |

| `ERRC_SOURCE_REPOSITORY` | const | ERRC_SOURCE_REPOSITORY: &str |  |  |  |  |

| `ErrcQuadrant` | enum | ErrcQuadrant { Eliminate, Reduce, Raise, Create } |  |  |  |  |

| `ErrcRefusal` | enum | ErrcRefusal { EmptyField(&'static str), MalformedBlake3(&'static str), NoClaims, NoPreservationFence, DuplicateClaimId(String), DuplicateInvariantId(String), DuplicateFactorCoordinate { target: String, metric: String, unit: String, }, DirectionViolation { claim_id: String, quadrant: ErrcQuadrant, baseline: u64, candidate: u64, }, NonCanonicalOrder(&'static str), WrongProfile, WrongSource, WrongClaimCeiling, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `certify_errc` | function | certify_errc( admitted: &AdmittedReceipt, mut observation: ErrcObservation, ) -> Result<ErrcReceipt, ErrcRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), ErrcRefusal> |  |  |  |  |

| `60d38265b8d1d94c43f04ca6bdb8537184e510a8` | str_key | ERRC_SOURCE_COMMIT = "60d38265b8d1d94c43f04ca6bdb8537184e510a8" |  |  |  |  |

| `DECLARED_DIRECTIONAL_TRANSFORMATION_ONLY_NO_CAUSAL_OR_OPTIMALITY_CLAIM` | str_key | ERRC_CLAIM_CEILING = "DECLARED_DIRECTIONAL_TRANSFORMATION_ONLY_NO_CAUSAL_OR_OPTIMALITY_CLAIM" |  |  |  |  |

| `affidavit/errc/v1` | str_key | ERRC_PROFILE = "affidavit/errc/v1" |  |  |  |  |

| `scripts/ci_errc.py` | str_key | ERRC_SOURCE_ARTIFACT = "scripts/ci_errc.py" |  |  |  |  |

| `seanchatmangpt/ggen-legacy` | str_key | ERRC_SOURCE_REPOSITORY = "seanchatmangpt/ggen-legacy" |  |  |  |  |

| `ErrcClaim` | struct | ErrcClaim { pub id: String, pub target: String, pub quadrant: ErrcQuadrant, pub measure: ErrcMeasure, pub evidence_commitment: Blake3Hash } |  |  |  |  |

| `ErrcMeasure` | struct | ErrcMeasure { pub metric: String, pub unit: String, pub baseline: u64, pub candidate: u64 } |  |  |  |  |

| `ErrcObservation` | struct | ErrcObservation { pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub claims: Vec<ErrcClaim>, pub preserved_invariants: Vec<PreservedInvariant>, pub replay: ReplayEvidence, pub previous_receipt: Option<Blake3Hash> } |  |  |  |  |

| `ErrcReceipt` | struct | ErrcReceipt { pub profile: String, pub source: ErrcSource, pub claim_ceiling: String, pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub admitted_receipt_hash: Blake3Hash, pub claims: Vec<ErrcClaim>, pub preserved_invariants: Vec<PreservedInvariant>, pub quadrant_counts: QuadrantCounts, pub replay: ReplayEvidence, pub previous_receipt: Option<Blake3Hash>, pub receipt_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `ErrcSource` | struct | ErrcSource { pub repository: String, pub commit: String, pub artifact: String } |  |  |  |  |

| `PreservedInvariant` | struct | PreservedInvariant { pub id: String, pub statement: String, pub evidence_commitment: Blake3Hash } |  |  |  |  |

| `QuadrantCounts` | struct | QuadrantCounts { pub eliminate: u64, pub reduce: u64, pub raise: u64, pub create: u64 } |  |  |  |  |

| `ERRC_CLAIM_ASSURANCE_CEILING` | const | ERRC_CLAIM_ASSURANCE_CEILING: &str |  |  |  |  |

| `ERRC_CLAIM_ASSURANCE_PROFILE` | const | ERRC_CLAIM_ASSURANCE_PROFILE: &str |  |  |  |  |

| `ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT` | const | ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT: &str |  |  |  |  |

| `ErrcClaimAssuranceRefusal` | enum | ErrcClaimAssuranceRefusal { ParentInvalid(ErrcRefusal), EmptyWitnessField { claim_id: String, field: &'static str, }, MalformedEvidenceCommitment(String), NoExclusions(String), EmptyExclusion(String), DuplicateExclusion { claim_id: String, exclusion: String, }, NoClaims, MissingWitness(String), UnexpectedWitness(String), DuplicateWitness(String), NonCanonicalOrder(&'static str), WrongProfile, WrongSource, WrongClaimCeiling, ParentHashMismatch, ClaimSetMismatch, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `certify_errc_claim_assurance` | function | certify_errc_claim_assurance( parent: &ErrcReceipt, mut witnesses: Vec<ErrcClaimWitness>, ) -> Result<ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), ErrcClaimAssuranceRefusal> |  |  |  |  |

| `verify_against` | function | verify_against(&self, parent: &ErrcReceipt) -> Result<(), ErrcClaimAssuranceRefusal> |  |  |  |  |

| `CLAIM_WITNESS_COMPLETENESS_AND_BINDING_ONLY_NO_TRUTH_CAUSALITY_PRODUCTION_COMPLIANCE_OPTIMALITY_OR_ACTUATION_CLAIM` | str_key | ERRC_CLAIM_ASSURANCE_CEILING = "CLAIM_WITNESS_COMPLETENESS_AND_BINDING_ONLY_NO_TRUTH_CAUSALITY_PRODUCTION_COMPLIANCE_OPTIMALITY_OR_ACTUATION_CLAIM" |  |  |  |  |

| `affidavit/errc-claim-assurance/v1` | str_key | ERRC_CLAIM_ASSURANCE_PROFILE = "affidavit/errc-claim-assurance/v1" |  |  |  |  |

| `governance/claims-register.md` | str_key | ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT = "governance/claims-register.md" |  |  |  |  |

| `ErrcClaimAssuranceReceipt` | struct | ErrcClaimAssuranceReceipt { pub profile: String, pub source: ErrcSource, pub claim_ceiling: String, pub errc_receipt_hash: Blake3Hash, pub claim_ids: Vec<String>, pub witnesses: Vec<ErrcClaimWitness>, pub receipt_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `ErrcClaimWitness` | struct | ErrcClaimWitness { pub claim_id: String, pub verifier: String, pub evidence_locator: String, pub observed_result: String, pub evidence_commitment: Blake3Hash, pub exclusions: Vec<String> } |  |  |  |  |

| `AffidavitError` | enum | AffidavitError { Io( Json( Parse(String), Validation(String), AdmissionRefused(String), VerificationFailed(String), Execution(String), WorkingReceipt(String), ContentAddressing(String), Discovery(String), Lsp(String), Ocel( Chain( Pqc( Mining( Sharding( Prediction( Slo( } |  |  |  |  |

| `ChainError` | enum | ChainError { Encode( Decode( Io { path: String, source: std::io::Error, }, } |  |  |  |  |

| `MiningError` | enum | MiningError { Him(String), Admission(String), OcelConversion(String) } |  |  |  |  |

| `OcelError` | enum | OcelError { EmptyEventType, EmptyEventId, EmptyObjectId(usize), EmptyObjectType(usize), MalformedObjectRef(String) } |  |  |  |  |

| `PqcError` | enum | PqcError { Signing(String), Verification(String), Encapsulation(String), Chain( } |  |  |  |  |

| `PredictionError` | enum | PredictionError { Wasm4pm(String), Admission(String), InvalidTopK(usize) } |  |  |  |  |

| `ShardingError` | enum | ShardingError { Dht(String), ChainMismatch { index: usize, expected: String, found: String, }, SeqMismatch { index: usize, expected: u64, found: u64, }, BoundaryMismatch { index: usize, next: usize, }, Failure(String) } |  |  |  |  |

| `SloViolation` | enum | SloViolation { LatencyP99 { observed_ms: f64, threshold_ms: f64, }, ErrorRate { observed_pct: f64, threshold_pct: f64, }, "availability SLO breach: observed {observed_pct:.3}% < threshold {threshold_pct:.3}%" )] Availability { observed_pct: f64, threshold_pct: f64, } } |  |  |  |  |

| `build` | function | build(self, counter: &mut SeqCounter) -> Result<OperationEvent, crate::ocel::OcelError> |  |  |  |  |

| `new` | function | new(event_type: impl Into<String>) -> Self |  |  |  |  |

| `object` | function | object(mut self, id: impl Into<String>, object_type: impl Into<String>) -> Self |  |  |  |  |

| `payload` | function | payload(mut self, payload: impl Into<Vec<u8>>) -> Self |  |  |  |  |

| `payload_str` | function | payload_str(mut self, payload: impl Into<String>) -> Self |  |  |  |  |

| `qualified_object` | function | qualified_object( mut self, id: impl Into<String>, object_type: impl Into<String>, qualifier: impl Into<String>, ) -> Self |  |  |  |  |

| `EventBuilder` | struct | EventBuilder { event_type: String, objects: Vec<ObjectRef>, payload: Vec<u8> } |  |  |  |  |

| `ManifestRefusal` | enum | ManifestRefusal { ManifestDigestMismatch, SubjectIdentityChanged, AuthorityBindingChanged, PolicyBindingChanged, OntologyBindingChanged, ToolSurfaceChanged, IntentBindingChanged, InvalidManifest(&'static str) } |  |  |  |  |

| `binding` | function | binding(&self) -> Result<ExecutionBinding, ManifestRefusal> |  |  |  |  |

| `digest` | function | digest(&self) -> Result<String, ManifestRefusal> |  |  |  |  |

| `requalification_reason` | function | requalification_reason( before: &ExecutionManifest, after: &ExecutionManifest, ) -> Result<Option<ManifestRefusal>, ManifestRefusal> |  |  |  |  |

| `validate` | function | validate(&self) -> Result<(), ManifestRefusal> |  |  |  |  |

| `verify_binding` | function | verify_binding( manifest: &ExecutionManifest, binding: &ExecutionBinding, ) -> Result<(), ManifestRefusal> |  |  |  |  |

| `ExecutionBinding` | struct | ExecutionBinding { pub execution_manifest_digest: String, pub exact_subject: String, pub subject_digest: String, pub intent_digest: String, pub authority_grant_digest: Option<String> } |  |  |  |  |

| `ExecutionManifest` | struct | ExecutionManifest { pub exact_subject: String, pub subject_digest: String, pub repository: Option<String>, pub base_sha: Option<String>, pub ontology_digest: Option<String>, pub policy_digest: Option<String>, pub capability_manifest_digest: Option<String>, pub tool_manifest_digest: Option<String>, pub planner_identity: Option<String>, pub planner_version: Option<String>, pub generator_identity: Option<String>, pub generator_version: Option<String>, pub runtime_identity: Option<String>, pub runtime_version: Option<String>, pub dependency_lock_digest: Option<String>, pub authority_grant_digest: Option<String>, pub intent_digest: String, pub environment_constraints: BTreeMap<String, String> } |  |  |  |  |

| `STANDING_CAPABILITY` | const | STANDING_CAPABILITY: &str |  |  |  |  |

| `STANDING_CAPABILITY_DIGEST` | const | STANDING_CAPABILITY_DIGEST: &str |  |  |  |  |

| `accepted` | function | accepted(&self) -> bool |  |  |  |  |

| `cli_standing_authority` | function | cli_standing_authority(scope: &str) -> AuthorityEnvelope<RustTypestateLaw> |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify(receipt: &str, observation: &str) -> CourtOutcome |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(receipt: &str) -> CourtOutcome |  |  |  |  |

| `errc_assure` | function | errc_assure(parent: &str, witnesses: &str) -> CourtOutcome |  |  |  |  |

| `errc_certify` | function | errc_certify(receipt: &str, observation: &str) -> CourtOutcome |  |  |  |  |

| `errc_verify` | function | errc_verify(receipt: &str) -> CourtOutcome |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance(receipt: &str, parent: Option<&str>) -> CourtOutcome |  |  |  |  |

| `reason` | function | reason(&self) -> &str |  |  |  |  |

| `standing_certify` | function | standing_certify(receipt: &str, observation: &str, scope: &str) -> CourtOutcome |  |  |  |  |

| `standing_verify` | function | standing_verify(receipt: &str) -> CourtOutcome |  |  |  |  |

| `affidavit.certify-standing` | str_key | STANDING_CAPABILITY = "affidavit.certify-standing" |  |  |  |  |

| `blake3:affidavit-standing-v2` | str_key | STANDING_CAPABILITY_DIGEST = "blake3:affidavit-standing-v2" |  |  |  |  |

| `CourtOutcome` | struct | CourtOutcome { pub code: i32, pub report: serde_json::Value } |  |  |  |  |

| `all` | function | all(&self) -> Vec<Fixture> |  |  |  |  |

| `delete_by_name` | function | delete_by_name(&mut self, name: &str) -> Result<bool> |  |  |  |  |

| `get_by_chain_hash` | function | get_by_chain_hash(&self, chain_hash: &str) -> Option<Fixture> |  |  |  |  |

| `get_by_name` | function | get_by_name(&self, name: &str) -> Option<Fixture> |  |  |  |  |

| `insert` | function | insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) -> Result<Fixture> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) -> Result<Self> |  |  |  |  |

| `reindex` | function | reindex(&mut self) -> Result<()> |  |  |  |  |

| `save` | function | save(&self) -> Result<()> |  |  |  |  |

| `search` | function | search(&self, query: &FixtureQuery) -> Vec<Fixture> |  |  |  |  |

| `Fixture` | struct | Fixture { pub id: String, pub name: String, pub tags: Vec<String>, pub event_count: usize, pub event_types: Vec<String>, pub chain_hash: String, pub inserted_at: String, pub receipt: Receipt } |  |  |  |  |

| `FixtureDatabase` | struct | FixtureDatabase { path: PathBuf, db: JsonDb, index_by_name: BTreeMap<String, usize>, index_by_event_count: BTreeMap<usize, Vec<usize>>, index_by_chain_hash: BTreeMap<String, usize> } |  |  |  |  |

| `FixtureQuery` | struct | FixtureQuery { pub name_contains: Option<String>, pub tag: Option<String>, pub min_events: Option<usize>, pub max_events: Option<usize>, pub event_type: Option<String>, pub limit: Option<usize> } |  |  |  |  |

| `CrownRefusal` | enum | CrownRefusal { InvalidCompositionDigest, InvalidReceiptDigest(String), InvalidPredecessorField { field: &'static str, value: String }, DuplicatePredecessorWorkOrder(String), PredecessorReceiptMismatch, PredecessorNotAlive(String), WrongGateSet(Vec<u8>), DuplicateGate(u8), GateSubjectMismatch { gate: u8 }, VacuousPass { gate: u8 }, MissingFalsifier { gate: u8 }, Gate12NotPositive } |  |  |  |  |

| `CrownStanding` | enum | CrownStanding { Alive, PartialAlive, Refused } |  |  |  |  |

| `GateStatus` | enum | GateStatus { Pass, Open, Refused, Blocked } |  |  |  |  |

| `certify_gall_crown` | function | certify_gall_crown(manifest: &CrownManifest) -> Result<CrownReceipt, CrownRefusal> |  |  |  |  |

| `CrownManifest` | struct | CrownManifest { pub schema: String, pub composition_digest: String, pub predecessor_receipts: Vec<String>, pub predecessor_witnesses: Vec<PredecessorWitness>, pub gates: Vec<GateWitness> } |  |  |  |  |

| `CrownReceipt` | struct | CrownReceipt { pub schema: String, pub composition_digest: String, pub predecessor_receipts: Vec<String>, pub predecessor_witnesses: Vec<PredecessorWitness>, pub gates: Vec<GateWitness>, pub standing: CrownStanding, pub evidence_ceiling: String, pub receipt_digest: String } |  |  |  |  |

| `GateWitness` | struct | GateWitness { pub gate: u8, pub name: String, pub status: GateStatus, pub subject_digest: String, pub evidence_digest: String, pub positive_observed: bool, pub falsifier_required: bool, pub falsifier_attempted: bool } |  |  |  |  |

| `PredecessorWitness` | struct | PredecessorWitness { pub receipt_iri: String, pub receipt_digest: String, pub work_order_iri: String, pub graph_digest: String, pub repository_identity: String, pub head_sha: String, pub standing: String } |  |  |  |  |

| `DEFAULT_SNIPPETS` | const | DEFAULT_SNIPPETS: &str |  |  |  |  |

| `TEST_FN_TEMPLATE` | const | TEST_FN_TEMPLATE: &str |  |  |  |  |

| `TEST_MODULE_TEMPLATE` | const | TEST_MODULE_TEMPLATE: &str |  |  |  |  |

| `filter_by_tag` | function | filter_by_tag(&self, tag: &str) -> Vec<&Snippet> |  |  |  |  |

| `find_by_name` | function | find_by_name(&self, pattern: &str) -> Vec<&Snippet> |  |  |  |  |

| `format_snippet` | function | format_snippet(&self, snippet: &Snippet) -> String |  |  |  |  |

| `from_json` | function | from_json(json: &str) -> Result<Self> |  |  |  |  |

| `generate_test_function` | function | generate_test_function(&self, pattern_name: &str, events: Vec<serde_json::Value>, expected_verdict: &str, expected_failure_stage: Option<&str> ) -> Result<String> |  |  |  |  |

| `generate_test_module` | function | generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) -> Result<String> |  |  |  |  |

| `main` | function | main() -> Result<()> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `CodegenEngine` | struct | CodegenEngine { tera: Tera } |  |  |  |  |

| `Snippet` | struct | Snippet { pub name: String, pub tags: Vec<String>, pub description: String, pub language: String, pub imports: Vec<String>, pub code: String } |  |  |  |  |

| `SnippetRegistry` | struct | SnippetRegistry { pub snippets: Vec<Snippet> } |  |  |  |  |

| `CheckStatus` | enum | CheckStatus { Ok, Warn, Fail } |  |  |  |  |

| `anomaly_detect` | function | anomaly_detect( receipts_path: String, sensitivity: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `assemble` | function | assemble(out: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `assemble_and_notarize` | function | assemble_and_notarize( notary_provider: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `assemble_with_signature` | function | assemble_with_signature( signing_method: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `attest` | function | attest( receipt: String, attestation_type: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `audit` | function | audit() -> Result<()> |  |  |  |  |

| `bus_factor` | function | bus_factor(receipts_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `catalog` | function | catalog(filter_name: Option<String>, filter_events: Option<usize>) -> Result<()> |  |  |  |  |

| `causality_chain` | function | causality_chain( start_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `conformance` | function | conformance(receipt: String) -> Result<()> |  |  |  |  |

| `coverage_analysis` | function | coverage_analysis( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `dependency_matrix` | function | dependency_matrix( receipts_path: String, output_matrix: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `diagnose` | function | diagnose(receipt: String) -> Result<()> |  |  |  |  |

| `diff` | function | diff(receipt_a: String, receipt_b: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `doctor` | function | doctor(receipts: Option<String>, fix: bool) -> Result<()> |  |  |  |  |

| `dora_metrics` | function | dora_metrics( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify( receipt: String, observation: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit` | function | emit(r -> Result<()> |  |  |  |  |

| `emit_batch` | function | emit_batch(batch_file: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_cicd` | function | emit_from_cicd(provider: String, job_status: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_cloud` | function | emit_from_cloud( provider: String, resource_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_github` | function | emit_from_github(repo: String, event_type: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_gitlab` | function | emit_from_gitlab(repo: String, event_type: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_monitoring` | function | emit_from_monitoring( provider: String, alert_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_quality` | function | emit_from_quality(working_dir: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `emit_from_security` | function | emit_from_security( provider: String, vuln_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_ocel_quality_measurement` | function | emit_ocel_quality_measurement( working_dir: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_ocel_quality_violation` | function | emit_ocel_quality_violation( working_dir: Option<String>, baseline_commits: Option<u32>, format: Option<String>, rules: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_violation_causal_chain` | function | emit_violation_causal_chain( receipt_path: String, metric_filter: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `envelope_export` | function | envelope_export(sealed_file: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `envelope_list` | function | envelope_list(store: Option<String>) -> Result<()> |  |  |  |  |

| `envelope_sign` | function | envelope_sign(receipt: String, key_file: String, out: Option<String>) -> Result<()> |  |  |  |  |

| `envelope_verify` | function | envelope_verify( sealed_file: String, store: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_assure` | function | errc_assure( parent: String, witnesses: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_certify` | function | errc_certify( receipt: String, observation: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_verify` | function | errc_verify(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance( receipt: String, parent: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `evidence_crl_apply` | function | evidence_crl_apply(file: String, store: Option<String>) -> Result<()> |  |  |  |  |

| `evidence_crl_publish` | function | evidence_crl_publish( kid: String, epoch: u64, store: Option<String>, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `evidence_heads` | function | evidence_heads(journal_file: Option<String>) -> Result<()> |  |  |  |  |

| `evidence_journal` | function | evidence_journal(subject: String, out: Option<String>) -> Result<()> |  |  |  |  |

| `explain_incident` | function | explain_incident( incident_desc: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `find_blast_radius` | function | find_blast_radius( change_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `fix_receipt` | function | fix_receipt( receipt: String, action: Option<String>, dry_run: bool, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `gdpr_proof` | function | gdpr_proof( receipts_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `graph` | function | graph(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `guide_search` | function | guide_search(keyword: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `hipaa` | function | hipaa(receipts_path: String, out: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `inspect` | function | inspect(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `install_git_hook` | function | install_git_hook(threshold: Option<String>) -> Result<()> |  |  |  |  |

| `keys_generate` | function | keys_generate(algorithm: String, custodian: String, out: Option<String>) -> Result<()> |  |  |  |  |

| `keys_import` | function | keys_import( algorithm: String, public_key_hex: String, custodian: String, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `keys_list` | function | keys_list(store: Option<String>) -> Result<()> |  |  |  |  |

| `keys_revoke` | function | keys_revoke(kid: String, reason: String, store: Option<String>) -> Result<()> |  |  |  |  |

| `keys_rotate` | function | keys_rotate(kid: String, store: Option<String>, out: Option<String>) -> Result<()> |  |  |  |  |

| `license_compliance` | function | license_compliance( receipts_path: String, license_policy: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `model` | function | model(receipt: String) -> Result<()> |  |  |  |  |

| `monitor` | function | monitor( watch: Option<String>, _metrics: Option<String>, _rules: Option<String>, baseline_commits: Option<u32>, interval: Option<u64>, output: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `notarize` | function | notarize(receipt: String, out: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `orphaned_code` | function | orphaned_code( receipts_path: String, days: Option<u32>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `pci_dss` | function | pci_dss(receipts_path: String, out: Option<String>, format: Option<String>) -> Result<()> |  |  |  |  |

| `policy_enforce` | function | policy_enforce( receipts_path: String, policy_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `portfolio_health` | function | portfolio_health( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `predict` | function | predict( receipts_path: String, prediction_type: String, _model: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `profile` | function | profile(receipt: Option<String>, duration: Option<u64>) -> Result<()> |  |  |  |  |

| `query` | function | query(q: String, receipts_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `receipt_throughput` | function | receipt_throughput(iterations: Option<u32>) -> Result<()> |  |  |  |  |

| `replay` | function | replay(receipt: String) -> Result<()> |  |  |  |  |

| `root_cause` | function | root_cause( effect_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_attest` | function | sbom_attest( sbom_path: String, receipt: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_blast_radius` | function | sbom_blast_radius( sbom_path: String, component: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_compliance` | function | sbom_compliance( sbom_path: String, framework: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_emit` | function | sbom_emit(sbom_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `sbom_ntia` | function | sbom_ntia(sbom_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `sbom_scan` | function | sbom_scan( sbom_path: String, advisories_path: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `search` | function | search(pattern: String, receipts_path: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `security_debt` | function | security_debt( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `send_violation_webhook` | function | send_violation_webhook( violation: &crate::quality::QualityViolation, webhook_url: &str, ) -> anyhow::Result<()> |  |  |  |  |

| `show` | function | show(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `sign` | function | sign( receipt: String, key_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `soc2_audit` | function | soc2_audit( receipts_path: String, soc2_type: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `standing_certify` | function | standing_certify( receipt: String, observation: String, scope: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `standing_verify` | function | standing_verify(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `stats` | function | stats(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `team_velocity` | function | team_velocity( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `tech_debt` | function | tech_debt( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `test` | function | test() -> Result<()> |  |  |  |  |

| `timeline` | function | timeline( receipts_path: String, start_time: Option<String>, end_time: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `trend_analysis` | function | trend_analysis( receipts_path: String, metric: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `variance` | function | variance(receipt: Option<String>, iterations: Option<u32>) -> Result<()> |  |  |  |  |

| `verify` | function | verify( receipt: String, format: Option<String>, _profile: Option<String>, _strict: Option<bool>, ) -> Result<()> |  |  |  |  |

| `verify_compliance` | function | verify_compliance(receipt: String, framework: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `verify_family` | function | verify_family(receipts_dir: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `verify_sla` | function | verify_sla(receipt: String, sla_file: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `visualize` | function | visualize(format: String, receipt: String) -> Result<()> |  |  |  |  |

| `why` | function | why(receipt: String, format: Option<String>) -> Result<()> |  |  |  |  |

| `.affi/crl.json` | str_key | EVIDENCE_CRL_FILE = ".affi/crl.json" |  |  |  |  |

| `.affi/standing-journal.jsonl` | str_key | EVIDENCE_JOURNAL_FILE = ".affi/standing-journal.jsonl" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `AFFI_NOTARY_KEY` | str_key | ENV_NOTARY_KEY = "AFFI_NOTARY_KEY" |  |  |  |  |

| `AFFI_SIGNING_KEY_PATH` | str_key | ENV_SIGNING_KEY_PATH = "AFFI_SIGNING_KEY_PATH" |  |  |  |  |

| `CTP-REVOCATIONS-v1` | str_key | REVOCATIONS_SIDECAR_FORMAT = "CTP-REVOCATIONS-v1" |  |  |  |  |

| `affidavit-crypto-trust` | str_key | SIGNING_METHOD_TRUST_PLANE = "affidavit-crypto-trust" |  |  |  |  |

| `affidavit-notary-local` | str_key | NOTARY_AUDIENCE = "affidavit-notary-local" |  |  |  |  |

| `affidavit.cli` | str_key | ENVELOPE_SIGN_AUDIENCE = "affidavit.cli" |  |  |  |  |

| `linear-3` | str_key | BUILTIN_FIXTURES = "linear-3" |  |  |  |  |

| `local-key-file-unattributed` | str_key | KEY_FILE_CUSTODIAN = "local-key-file-unattributed" |  |  |  |  |

| `DoctorFinding` | struct | DoctorFinding { pub check: String, pub status: CheckStatus, pub message: String, pub remediation: Option<String>, pub auto_fixable: bool } |  |  |  |  |

| `RevocationSidecarEntry` | struct | RevocationSidecarEntry { pub kid: String, pub revoked_at: u64, pub reason: String } |  |  |  |  |

| `HlcError` | enum | HlcError { ClockSkewExceeded { received_ms: u64, skew_ms: u64, } } |  |  |  |  |

| `concurrent` | function | concurrent(a: HlcTimestamp, b: HlcTimestamp) -> bool |  |  |  |  |

| `happened_before` | function | happened_before(a: HlcTimestamp, b: HlcTimestamp) -> bool |  |  |  |  |

| `peek` | function | peek(&self) -> HlcTimestamp |  |  |  |  |

| `receive` | function | receive(&mut self, received: HlcTimestamp) -> Result<HlcTimestamp, HlcError> |  |  |  |  |

| `send` | function | send(&mut self) -> HlcTimestamp |  |  |  |  |

| `with_skew_bound` | function | with_skew_bound(max_skew_ms: u64) -> Self |  |  |  |  |

| `HlcClock` | struct | HlcClock { last: HlcTimestamp, max_skew_ms: u64 } |  |  |  |  |

| `HlcTimestamp` | struct | HlcTimestamp { pub physical_ms: u64, pub logical: u32 } |  |  |  |  |

| `run` | function | run() -> clap_noun_verb::Result<()> |  |  |  |  |

| `architecture::{ ArchitectureQualificationReceipt, ArchitectureRefusal, ArchitectureStanding, ArchitectureStandingLedger, EvidenceSource, QualificationEvidence, Supersession, ARCHITECTURE_QUERY_SCHEMA, ARCHITECTURE_RECEIPT_SCHEMA, }` | use | architecture::{ ArchitectureQualificationReceipt, ArchitectureRefusal, ArchitectureStanding, ArchitectureStandingLedger, EvidenceSource, QualificationEvidence, Supersession, ARCHITECTURE_QUERY_SCHEMA, ARCHITECTURE_RECEIPT_SCHEMA, } |  |  |  |  |

| `ecosystem::{ certify_ecosystem, EcosystemMember, EcosystemObservation, EcosystemReceipt, EcosystemRefusal, EcosystemRole, RoleCoverage, RoleRequirement, ECOSYSTEM_AUTHORITY_CEILING, ECOSYSTEM_CLAIM_CEILING, ECOSYSTEM_PROFILE, }` | use | ecosystem::{ certify_ecosystem, EcosystemMember, EcosystemObservation, EcosystemReceipt, EcosystemRefusal, EcosystemRole, RoleCoverage, RoleRequirement, ECOSYSTEM_AUTHORITY_CEILING, ECOSYSTEM_CLAIM_CEILING, ECOSYSTEM_PROFILE, } |  |  |  |  |

| `errc::{ certify_errc, ErrcClaim, ErrcMeasure, ErrcObservation, ErrcQuadrant, ErrcReceipt, ErrcRefusal, ErrcSource, PreservedInvariant, QuadrantCounts, ERRC_CLAIM_CEILING, ERRC_PROFILE, ERRC_SOURCE_ARTIFACT, ERRC_SOURCE_COMMIT, ERRC_SOURCE_REPOSITORY, }` | use | errc::{ certify_errc, ErrcClaim, ErrcMeasure, ErrcObservation, ErrcQuadrant, ErrcReceipt, ErrcRefusal, ErrcSource, PreservedInvariant, QuadrantCounts, ERRC_CLAIM_CEILING, ERRC_PROFILE, ERRC_SOURCE_ARTIFACT, ERRC_SOURCE_COMMIT, ERRC_SOURCE_REPOSITORY, } |  |  |  |  |

| `errc_claim_assurance::{ certify_errc_claim_assurance, ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal, ErrcClaimWitness, ERRC_CLAIM_ASSURANCE_CEILING, ERRC_CLAIM_ASSURANCE_PROFILE, ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT, }` | use | errc_claim_assurance::{ certify_errc_claim_assurance, ErrcClaimAssuranceReceipt, ErrcClaimAssuranceRefusal, ErrcClaimWitness, ERRC_CLAIM_ASSURANCE_CEILING, ERRC_CLAIM_ASSURANCE_PROFILE, ERRC_CLAIM_ASSURANCE_SOURCE_ARTIFACT, } |  |  |  |  |

| `error::AffidavitError` | use | error::AffidavitError |  |  |  |  |

| `federation::{CourtOutcome, STANDING_CAPABILITY, STANDING_CAPABILITY_DIGEST}` | use | federation::{CourtOutcome, STANDING_CAPABILITY, STANDING_CAPABILITY_DIGEST} |  |  |  |  |

| `standing::{ certify_standing, AuthorityBinding, ExecutionEvidence, ReplayEvidence, Standing, StandingObservation, StandingReceipt, StandingRefusal, SubjectIdentity, VerificationEvidence, STANDING_PROFILE, }` | use | standing::{ certify_standing, AuthorityBinding, ExecutionEvidence, ReplayEvidence, Standing, StandingObservation, StandingReceipt, StandingRefusal, SubjectIdentity, VerificationEvidence, STANDING_PROFILE, } |  |  |  |  |

| `types::{ canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt, Verdict, }` | use | types::{ canonical_bytes, Blake3Hash, CheckOutcome, ObjectRef, OperationEvent, ProfileId, Receipt, Verdict, } |  |  |  |  |

| `DIAGNOSTIC_SOURCE` | const | DIAGNOSTIC_SOURCE: &str |  |  |  |  |

| `verdict_to_diagnostics` | function | verdict_to_diagnostics(verdict: &crate::types::Verdict) -> Vec<Diagnostic> |  |  |  |  |

| `affidavit` | str_key | DIAGNOSTIC_SOURCE = "affidavit" |  |  |  |  |

| `goto_definition_for_event_type` | function | goto_definition_for_event_type(event_type: &str) -> Option<Location> |  |  |  |  |

| `hover_for_event_id` | function | hover_for_event_id(event_id: &str, receipt: &Receipt) -> Option<Hover> |  |  |  |  |

| `diagnostics::*` | use | diagnostics::* |  |  |  |  |

| `goto_definition::*` | use | goto_definition::* |  |  |  |  |

| `hover::*` | use | hover::* |  |  |  |  |

| `SLO_AVAILABILITY_PCT` | const | SLO_AVAILABILITY_PCT: f64 |  |  |  |  |

| `SLO_ERROR_RATE_PCT` | const | SLO_ERROR_RATE_PCT: f64 |  |  |  |  |

| `SLO_LATENCY_P99_MS` | const | SLO_LATENCY_P99_MS: f64 |  |  |  |  |

| `SloViolation` | enum | SloViolation { LatencyP99 { observed_ms: f64, threshold_ms: f64 }, ErrorRate { observed_pct: f64, threshold_pct: f64 }, Availability { observed_pct: f64, threshold_pct: f64 } } |  |  |  |  |

| `check_slo` | function | check_slo(&self) -> anyhow::Result<(), SloViolation> |  |  |  |  |

| `compute_sli` | function | compute_sli(&self) -> anyhow::Result<ServiceLevelIndicators> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `new_noop` | function | new_noop() -> Self |  |  |  |  |

| `record_receipt_verified` | function | record_receipt_verified(&self, accepted: bool) |  |  |  |  |

| `record_stage_error` | function | record_stage_error(&self, _stage: &str) |  |  |  |  |

| `record_stage_latency` | function | record_stage_latency(&self, _stage: &str, duration: Duration) |  |  |  |  |

| `render` | function | render(&self) -> String |  |  |  |  |

| `with_window` | function | with_window(window_duration: Duration) -> Self |  |  |  |  |

| `MetricsCollector` | struct | MetricsCollector { inner: Mutex<CollectorState>, window_duration: Duration } |  |  |  |  |

| `PrometheusExporter` | struct | PrometheusExporter { collector: &'a MetricsCollector } |  |  |  |  |

| `ServiceLevelIndicators` | struct | ServiceLevelIndicators { pub latency_p99_ms: f64, pub error_rate_pct: f64, pub availability_pct: f64 } |  |  |  |  |

| `MiningError` | enum | MiningError { Him(String), Admission(String), OcelConversion(String), Alignment(String) } |  |  |  |  |

| `alignment_fitness_score` | function | alignment_fitness_score( admitted: &AdmittedReceipt, model: &PetriNet, ) -> Result<AlignmentReport, MiningError> |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) -> Result<PetriNet, MiningError> |  |  |  |  |

| `interpret_score` | function | interpret_score(fitness: f64) -> &'static str |  |  |  |  |

| `predict_next` | function | predict_next( admitted: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, MiningError> |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) -> Result<OCEL, MiningError> |  |  |  |  |

| `ActivityPrediction` | struct | ActivityPrediction { pub activity: String, pub confidence: f64 } |  |  |  |  |

| `AlignmentReport` | struct | AlignmentReport { pub fitness: f64, pub activity_coverage: f64, pub simplicity: f64, pub interpretation: String, pub unfit_events: u32, pub model_moves: u32 } |  |  |  |  |

| `PredictionReport` | struct | PredictionReport { pub predictions: Vec<ActivityPrediction>, pub context_length: usize, pub model_type: String } |  |  |  |  |

| `MiningError` | enum | MiningError { Him(String), Admission(String), OcelConversion(String) } |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) -> Result<PetriNet, MiningError> |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) -> Result<OCEL, MiningError> |  |  |  |  |

| `MutationKind` | enum | MutationKind { EventDrop, EventReorder, TypeChange, PayloadFlip } |  |  |  |  |

| `all_operators` | function | all_operators() -> Vec<Box<dyn MutationOperator>> |  |  |  |  |

| `AppliedMutation` | struct | AppliedMutation { pub kind: MutationKind, pub target_seq: u64, pub mutated_receipt: Receipt } |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |

| `MutationKind` | enum | MutationKind { EventDrop, EventReorder, TypeChange, PayloadFlip } |  |  |  |  |

| `all_operators` | function | all_operators() -> Vec<Box<dyn MutationOperator>> |  |  |  |  |

| `AppliedMutation` | struct | AppliedMutation { pub kind: MutationKind, pub target_seq: u64, pub mutated_receipt: Receipt } |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |

| `build_event` | function | build_event( event_type: impl Into<String>, objects: Vec<ObjectRef>, payload: &[u8], counter: &mut SeqCounter, ) -> std::result::Result<OperationEvent, OcelError> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `next_seq` | function | next_seq(&mut self) -> u64 |  |  |  |  |

| `object_ref` | function | object_ref(id: impl Into<String>, obj_type: impl Into<String>) -> ObjectRef |  |  |  |  |

| `parse_object_ref` | function | parse_object_ref(spec: &str) -> std::result::Result<ObjectRef, OcelError> |  |  |  |  |

| `peek` | function | peek(&self) -> u64 |  |  |  |  |

| `qualified_object_ref` | function | qualified_object_ref( id: impl Into<String>, obj_type: impl Into<String>, qualifier: impl Into<String>, ) -> ObjectRef |  |  |  |  |

| `starting_at` | function | starting_at(value: u64) -> Self |  |  |  |  |

| `validate_event` | function | validate_event(event: &OperationEvent) -> std::result::Result<(), OcelError> |  |  |  |  |

| `SeqCounter` | struct | SeqCounter { value: u64 } |  |  |  |  |

| `crate::error::OcelError` | use | crate::error::OcelError |  |  |  |  |

| `Format` | enum | Format { Human, Json, Yaml } |  |  |  |  |

| `data` | function | data(&mut self, value: &T) -> io::Result<()> |  |  |  |  |

| `diag` | function | diag(&mut self, diag: &crate::diag::Diag) -> io::Result<()> |  |  |  |  |

| `from_str` | function | from_str(s: &str) -> Self |  |  |  |  |

| `info` | function | info(&mut self, msg: &str) -> io::Result<()> |  |  |  |  |

| `json` | function | json(&mut self, value: &serde_json::Value) -> io::Result<()> |  |  |  |  |

| `line` | function | line(args: std::fmt::Arguments<'_>) |  |  |  |  |

| `new` | function | new(format: Format) -> Self |  |  |  |  |

| `no_color` | function | no_color(mut self) -> Self |  |  |  |  |

| `print_` | function | print_(args: std::fmt::Arguments<'_>) |  |  |  |  |

| `quiet` | function | quiet(mut self) -> Self |  |  |  |  |

| `verbose` | function | verbose(mut self) -> Self |  |  |  |  |

| `warn` | function | warn(&mut self, msg: &str) -> io::Result<()> |  |  |  |  |

| `with_format` | function | with_format(format: Format) -> Self |  |  |  |  |

| `with_sinks` | function | with_sinks( format: Format, stdout: Box<dyn Write + Send>, stderr: Box<dyn Write + Send>, ) -> Self |  |  |  |  |

| `Out` | struct | Out { format: Format, color: bool, quiet: bool, verbose: bool, stdout: Box<dyn Write + Send>, stderr: Box<dyn Write + Send> } |  |  |  |  |

| `GateVerdict` | enum | GateVerdict { Admitted, Refused(String) } |  |  |  |  |

| `PolicyError` | enum | PolicyError { Parse(String), Request(String), NoDecision } |  |  |  |  |

| `evaluate` | function | evaluate( &self, principal: &str, action: &str, resource: &str, ) -> Result<GateVerdict, PolicyError> |  |  |  |  |

| `from_policies` | function | from_policies(policies: &str) -> Result<Self, PolicyError> |  |  |  |  |

| `PolicyGate` | struct | PolicyGate { authorizer: Authorizer, policies: PolicySet, entities: Entities } |  |  |  |  |

| `verify_foreign_receipt` | function | verify_foreign_receipt( consequence_digest: &str, authority: ForeignAuthority<'_>, observation: ForeignObservation<'_>, receipt: ForeignReceipt<'_>, ) -> Result<(), &'static str> |  |  |  |  |

| `ForeignAuthority` | struct | ForeignAuthority { pub decision_id: &'a str, pub consequence_digest: &'a str } |  |  |  |  |

| `ForeignObservation` | struct | ForeignObservation { pub consequence_digest: &'a str, pub observation_digest: &'a str } |  |  |  |  |

| `ForeignReceipt` | struct | ForeignReceipt { pub authority_decision_id: &'a str, pub consequence_digest: &'a str, pub observation_digest: &'a str, pub replay_key: &'a str } |  |  |  |  |

| `PredictionError` | enum | PredictionError { InvalidTopK(usize), Wasm4pm(String) } |  |  |  |  |

| `predict_next` | function | predict_next( admitted: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, PredictionError> |  |  |  |  |

| `predict_next_with_model` | function | predict_next_with_model( model: &AdmittedReceipt, current_trace: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, PredictionError> |  |  |  |  |

| `ActivityPrediction` | struct | ActivityPrediction { pub activity: String, pub confidence: f64 } |  |  |  |  |

| `PredictionReport` | struct | PredictionReport { pub predictions: Vec<ActivityPrediction>, pub context_length: usize, pub model_type: String } |  |  |  |  |

| `Notification` | enum | Notification { FileChanged(PathBuf), IntervalElapsed, Error(String) } |  |  |  |  |

| `QualityViolation` | enum | QualityViolation { Rule1Sigma { metric: String, value: f64, threshold: f64, z_score: f64, severity: String, }, Rule9InRow { metric: String, consecutive: usize }, RuleTrend { metric: String, direction: String, count: usize, }, RuleAlternating { metric: String, oscillations: usize }, Rule2of3Beyond2Sigma { metric: String, count: usize, threshold: f64, }, Rule4of5Beyond1Sigma { metric: String, count: usize, threshold: f64, }, Rule15InRowWithin1Sigma { metric: String, count: usize, threshold: f64, severity: String, } } |  |  |  |  |

| `add_measurement` | function | add_measurement(&mut self, metric_name: &str, value: f64) |  |  |  |  |

| `description` | function | description(&self) -> String |  |  |  |  |

| `measure_code_quality` | function | measure_code_quality(src_path: &str) -> anyhow::Result<CodeQualityMetrics> |  |  |  |  |

| `metric` | function | metric(&self) -> &str |  |  |  |  |

| `new` | function | new(baseline_mean: f64, baseline_stddev: f64, window_size: usize) -> Self |  |  |  |  |

| `run_watch_loop` | function | run_watch_loop(&mut self) -> Result<()> |  |  |  |  |

| `run_watch_loop_async` | function | run_watch_loop_async( path: &str, interval_secs: u64, ) -> Result<tokio::task::JoinHandle<Result<()>>> |  |  |  |  |

| `severity` | function | severity(&self) -> &str |  |  |  |  |

| `CodeQualityMetrics` | struct | CodeQualityMetrics { pub stub_ratio: f64, pub type_coverage: f64, pub churn: usize, pub comment_ratio: f64, pub cyclomatic_complexity: f64, pub maintainability_index: f64, pub cognitive_complexity: f64, pub clippy_warnings: usize, pub rustfmt_violations: usize, pub cargo_deny_issues: usize, pub cargo_audit_vulnerabilities: usize, pub test_coverage: f64, pub doc_coverage: f64, pub timestamp: u64 } |  |  |  |  |

| `FileWatcher` | struct | FileWatcher { pub path: PathBuf, rx: Receiver<Notification>, last_measure_time: std::time::Instant, debounce_delay_ms: u64 } |  |  |  |  |

| `WesternElectricAnalyzer` | struct | WesternElectricAnalyzer { pub baseline_mean: f64, pub baseline_stddev: f64, pub rolling_window: VecDeque<f64>, pub window_size: usize, pub control_limits: (f64, f64), pub violations: Vec<QualityViolation> } |  |  |  |  |

| `amplify_severity_for_correlated_violations` | function | amplify_severity_for_correlated_violations( violations: &[QualityViolation], correlations: &[MetricCorrelation], ) -> HashMap<String, f64> |  |  |  |  |

| `analyze_correlations` | function | analyze_correlations( history: &[CodeQualityMetrics], violations: &[QualityViolation], ) -> CorrelationAnalysis |  |  |  |  |

| `compute_metric_correlations` | function | compute_metric_correlations(history: &[CodeQualityMetrics]) -> Vec<MetricCorrelation> |  |  |  |  |

| `detect_simultaneous_violations` | function | detect_simultaneous_violations( violations: &[QualityViolation], ) -> Vec<SimultaneousViolation> |  |  |  |  |

| `direction` | function | direction(&self) -> &str |  |  |  |  |

| `infer_root_cause` | function | infer_root_cause( metrics: &[CodeQualityMetrics], violation: &QualityViolation, ) -> RootCauseHypothesis |  |  |  |  |

| `is_actionable` | function | is_actionable(&self) -> bool |  |  |  |  |

| `is_compound` | function | is_compound(&self) -> bool |  |  |  |  |

| `is_significant` | function | is_significant(&self) -> bool |  |  |  |  |

| `max_severity` | function | max_severity(&self) -> &str |  |  |  |  |

| `new` | function | new( metric_names: Vec<String>, severities: Vec<String>, timestamps: Vec<u64>, time_window_secs: u64, ) -> Self |  |  |  |  |

| `strength` | function | strength(&self) -> &str |  |  |  |  |

| `CorrelationAnalysis` | struct | CorrelationAnalysis { pub metric_correlations: Vec<MetricCorrelation>, pub simultaneous_violations: Vec<SimultaneousViolation>, pub root_causes: Vec<RootCauseHypothesis>, pub amplified_severities: HashMap<String, f64>, pub timestamp: u64 } |  |  |  |  |

| `MetricCorrelation` | struct | MetricCorrelation { pub metric_a: String, pub metric_b: String, pub pearson_coefficient: f64, pub sample_count: usize, pub standard_error: f64 } |  |  |  |  |

| `RootCauseHypothesis` | struct | RootCauseHypothesis { pub causal_metric: String, pub affected_metric: String, pub correlation: f64, pub confidence: f64, pub lag_seconds: u64, pub evidence: String } |  |  |  |  |

| `SimultaneousViolation` | struct | SimultaneousViolation { pub timestamps: Vec<u64>, pub metric_names: Vec<String>, pub severities: Vec<String>, pub time_window_secs: u64, pub violation_count: usize } |  |  |  |  |

| `RuleVariant` | enum | RuleVariant { Rule1SigmaAt1(String), Rule1SigmaAt2(String), Rule1SigmaAt3(String), Rule1SigmaAtCustom(String, f64), RuleConsecutiveWindow6(String), RuleConsecutiveWindow9(String), RuleConsecutiveWindow15(String), RuleConsecutiveWindow20(String), RuleConsecutiveWindow30(String), RuleTrendWindow6(String), RuleTrendWindow9(String), RuleTrendWindow15(String), RuleTrendWindow20(String), RuleAlternating(String), Rule2of3Beyond2Sigma(String), Rule2of3Beyond1Sigma(String), Rule3of3Beyond2Sigma(String), Rule4of5Beyond1Sigma(String), Rule5of5Beyond1Sigma(String), Rule3of5Beyond1Sigma(String), Rule15InRowWithin1Sigma(String), Rule20InRowWithin1Sigma(String), Rule10InRowWithin1Sigma(String) } |  |  |  |  |

| `add_custom_threshold` | function | add_custom_threshold(mut self, rule_name: String, threshold: f64) -> Self |  |  |  |  |

| `add_measurement` | function | add_measurement(&mut self, metric_name: &str, value: f64) |  |  |  |  |

| `compute` | function | compute(violations: &[QualityViolation]) -> Self |  |  |  |  |

| `compute_aggregate_severity` | function | compute_aggregate_severity(violations: &[QualityViolation]) -> AggregatedSeverity |  |  |  |  |

| `description` | function | description(&self) -> String |  |  |  |  |

| `detect_all_rule_variants` | function | detect_all_rule_variants( metrics: &[f64], config: &WesternElectricConfig, ) -> Vec<RuleVariant> |  |  |  |  |

| `detect_rule_storms` | function | detect_rule_storms(violations: &[QualityViolation]) -> Vec<RuleStorm> |  |  |  |  |

| `finalize` | function | finalize(&mut self) -> AggregatedSeverity |  |  |  |  |

| `metric` | function | metric(&self) -> &str |  |  |  |  |

| `new` | function | new(baseline_mean: f64, baseline_stddev: f64) -> Self |  |  |  |  |

| `severity` | function | severity(&self) -> &'static str |  |  |  |  |

| `with_enabled_rules` | function | with_enabled_rules( mut self, rule1: bool, rule2: bool, rule3: bool, rule4: bool, rule5: bool, rule6: bool, rule7: bool, ) -> Self |  |  |  |  |

| `with_sigmas` | function | with_sigmas(mut self, primary: f64, secondary: f64, tertiary: f64) -> Self |  |  |  |  |

| `AggregatedSeverity` | struct | AggregatedSeverity { pub total_violations: usize, pub critical_count: usize, pub high_count: usize, pub medium_count: usize, pub low_count: usize, pub rule_storm_count: usize, pub worst_severity: String, pub affected_metrics: Vec<String>, pub summary: String } |  |  |  |  |

| `EnhancedWesternElectricAnalyzer` | struct | EnhancedWesternElectricAnalyzer { pub config: WesternElectricConfig, pub rolling_windows: HashMap<usize, VecDeque<f64>>, pub detected_rules: Vec<RuleVariant>, pub rule_storms: Vec<RuleStorm>, pub severity_report: Option<AggregatedSeverity> } |  |  |  |  |

| `RuleStorm` | struct | RuleStorm { pub metric: String, pub rules: Vec<RuleVariant>, pub aggregate_severity: String, pub rule_count: usize, pub is_severe: bool, pub summary: String } |  |  |  |  |

| `WesternElectricConfig` | struct | WesternElectricConfig { pub baseline_mean: f64, pub baseline_stddev: f64, pub primary_sigma: f64, pub secondary_sigma: f64, pub tertiary_sigma: f64, pub custom_thresholds: HashMap<String, f64>, pub window_sizes: Vec<usize>, pub rule1_enabled: bool, pub rule2_enabled: bool, pub rule3_enabled: bool, pub rule4_enabled: bool, pub rule5_enabled: bool, pub rule6_enabled: bool, pub rule7_enabled: bool, pub min_consecutive_violations: usize, pub rule_storm_threshold: usize } |  |  |  |  |

| `ObjectViolation` | enum | ObjectViolation { FileViolation { file_path: String, violation_type: String, value: f64, threshold: f64, severity: String, }, ModuleViolation { module_name: String, violation_type: String, mean_value: f64, stddev_value: f64, severity: String, }, PackageViolation { package_name: String, violation_type: String, current_score: f64, threshold: f64, severity: String, }, APIBreakingChange { from_package: String, to_package: String, description: String, severity: String, }, DependencyViolation { package_name: String, dependency_name: String, reason: String, severity: String, } } |  |  |  |  |

| `aggregate_module_metrics` | function | aggregate_module_metrics(files: &[FileQualityMetrics]) -> ModuleQualityMetrics |  |  |  |  |

| `compute_package_health` | function | compute_package_health(modules: &[ModuleQualityMetrics]) -> PackageHealthScore |  |  |  |  |

| `description` | function | description(&self) -> String |  |  |  |  |

| `detect_object_level_violations` | function | detect_object_level_violations( object_metric: &FileQualityMetrics, baseline: f64, stddev: f64, ) -> Vec<ObjectViolation> |  |  |  |  |

| `maintainability_index` | function | maintainability_index(&self) -> f64 |  |  |  |  |

| `measure_file_quality` | function | measure_file_quality(path: &str) -> anyhow::Result<FileQualityMetrics> |  |  |  |  |

| `new` | function | new(path: String) -> Self |  |  |  |  |

| `severity` | function | severity(&self) -> &str |  |  |  |  |

| `update_health_score` | function | update_health_score(&mut self) |  |  |  |  |

| `FileQualityMetrics` | struct | FileQualityMetrics { pub path: String, pub stub_ratio: f64, pub cyclomatic_complexity: f64, pub max_cyclomatic_complexity: f64, pub test_coverage: f64, pub doc_coverage: f64, pub loc: usize, pub comment_lines: usize, pub public_items: usize, pub documented_public_items: usize, pub cognitive_complexity: f64, pub type_coverage: f64 } |  |  |  |  |

| `ModuleQualityMetrics` | struct | ModuleQualityMetrics { pub module_name: String, pub file_count: usize, pub mean_stub_ratio: f64, pub stddev_stub_ratio: f64, pub mean_cyclomatic_complexity: f64, pub stddev_cyclomatic_complexity: f64, pub max_cyclomatic_complexity: f64, pub mean_test_coverage: f64, pub stddev_test_coverage: f64, pub mean_doc_coverage: f64, pub stddev_doc_coverage: f64, pub total_loc: usize, pub total_public_items: usize, pub total_documented_items: usize, pub mean_type_coverage: f64 } |  |  |  |  |

| `PackageHealthScore` | struct | PackageHealthScore { pub package_name: String, pub health_score: f64, pub test_coverage: f64, pub doc_coverage: f64, pub stub_ratio: f64, pub mean_complexity: f64, pub violation_count: usize, pub api_breaking_changes: usize, pub dependency_violations: usize } |  |  |  |  |

| `add_event` | function | add_event(&mut self, event: OcelQualityEvent) |  |  |  |  |

| `build_causal_chain` | function | build_causal_chain( violation_event: &OcelQualityEvent, event_log: &[OcelQualityEvent], ) -> Result<ViolationCausalChain, OcelError> |  |  |  |  |

| `correlate_violations_across_objects` | function | correlate_violations_across_objects( log: &OcelQualityLog, ) -> Result<Vec<ObjectCorrelation>, OcelError> |  |  |  |  |

| `events_by_type` | function | events_by_type(&self, quality_event_type: &str) -> Vec<&OcelQualityEvent> |  |  |  |  |

| `from_metrics` | function | from_metrics( object_id: impl Into<String>, object_type: impl Into<String>, metrics: &CodeQualityMetrics, ) -> Self |  |  |  |  |

| `measure_to_ocel_event` | function | measure_to_ocel_event( event_id: &str, seq: u64, metrics: &CodeQualityMetrics, objects: &[ObjectRef], ) -> Result<OcelQualityEvent, OcelError> |  |  |  |  |

| `measurements` | function | measurements(&self) -> Vec<&OcelQualityEvent> |  |  |  |  |

| `new` | function | new(log_id: impl Into<String>, timestamp: u64) -> Self |  |  |  |  |

| `violation_to_ocel_event` | function | violation_to_ocel_event( event_id: &str, seq: u64, violation: &QualityViolation, triggered_by_event_id: &str, objects: &[ObjectRef], ) -> Result<OcelQualityEvent, OcelError> |  |  |  |  |

| `violations` | function | violations(&self) -> Vec<&OcelQualityEvent> |  |  |  |  |

| `ObjectCorrelation` | struct | ObjectCorrelation { pub metric: String, pub severity: String, pub object_ids: Vec<String>, pub object_count: usize, pub event_ids: Vec<String>, pub first_detected_seq: u64, pub last_detected_seq: u64 } |  |  |  |  |

| `ObjectQualityRecord` | struct | ObjectQualityRecord { pub object_id: String, pub object_type: String, pub stub_ratio: f64, pub type_coverage: f64, pub cyclomatic_complexity: f64, pub cognitive_complexity: f64, pub test_coverage: f64, pub doc_coverage: f64, pub clippy_warnings: usize, pub churn: usize, pub measured_at: u64 } |  |  |  |  |

| `OcelQualityEvent` | struct | OcelQualityEvent { pub event: OperationEvent, pub quality_event_type: String, pub triggered_by_event_id: Option<String>, pub quality_payload: serde_json::Value, pub severity: Option<String> } |  |  |  |  |

| `OcelQualityLog` | struct | OcelQualityLog { pub log_id: String, pub events: Vec<OcelQualityEvent>, pub object_records: BTreeMap<String, ObjectQualityRecord>, pub causal_chains: BTreeMap<String, ViolationCausalChain>, pub metric_correlations: BTreeMap<String, Vec<String>>, pub created_at: u64, pub updated_at: u64 } |  |  |  |  |

| `ViolationCausalChain` | struct | ViolationCausalChain { pub root_measurement_event_id: String, pub violation_event_id: String, pub remediation_event_id: Option<String>, pub measured_value: f64, pub threshold: f64, pub violation: QualityViolation, pub remediation_action: Option<String>, pub remediation_status: String, pub event_sequence: Vec<String> } |  |  |  |  |

| `DimensionError` | enum | DimensionError { RowCount { expected: usize, got: usize, }, ColCount { row: usize, expected: usize, got: usize, } } |  |  |  |  |

| `MatrixError` | enum | MatrixError { Cell { row: usize, col: usize, source: QuantizationRefusal, }, Dimensions( } |  |  |  |  |

| `QuantizationRefusal` | enum | QuantizationRefusal { NotFinite { score: f64, }, OutOfRange { score: f64, } } |  |  |  |  |

| `bits` | function | bits(&self) -> &[Vec<Q16F16Bits>] |  |  |  |  |

| `from_fractions` | function | from_fractions( n_nodes: usize, lenses: usize, rows: &[Vec<(u64, u64)>], ) -> Result<Self, DimensionError> |  |  |  |  |

| `from_scores` | function | from_scores( n_nodes: usize, lenses: usize, rows: &[Vec<f64>], ) -> Result<Self, MatrixError> |  |  |  |  |

| `lenses` | function | lenses(&self) -> usize |  |  |  |  |

| `nodes` | function | nodes(&self) -> usize |  |  |  |  |

| `quantize` | function | quantize(score: f64) -> Result<Q16F16Bits, QuantizationRefusal> |  |  |  |  |

| `quantize_fraction` | function | quantize_fraction(num: u64, den: u64) -> Q16F16Bits |  |  |  |  |

| `zeroed` | function | zeroed(n_nodes: usize, lenses: usize) -> Self |  |  |  |  |

| `PayoffMatrix` | struct | PayoffMatrix { matrix: Vec<Vec<Q16F16Bits>>, lenses: usize } |  |  |  |  |

| `CERTIFIED_RECEIPT_AUDIENCE` | const | CERTIFIED_RECEIPT_AUDIENCE: &str |  |  |  |  |

| `PAID_DELIVERY_DOMAIN` | const | PAID_DELIVERY_DOMAIN: &str |  |  |  |  |

| `VALIDITY_WINDOW_SECONDS` | const | VALIDITY_WINDOW_SECONDS: u64 |  |  |  |  |

| `build_canonical_subject` | function | build_canonical_subject(payload_hash_hex: &str, subject: &str) -> String |  |  |  |  |

| `certify_paid_delivery_payload` | function | certify_paid_delivery_payload( payload_hash_hex: &str, subject: &str, signing: &Es256SigningKey, ) -> Result<CertifiedReceiptEnvelope, VerifyRefusal> |  |  |  |  |

| `verify_certified_paid_delivery` | function | verify_certified_paid_delivery( certified: &CertifiedReceiptEnvelope, payload_hash_hex: &str, subject: &str, ) -> Result<(), VerifyRefusal> |  |  |  |  |

| `affidavit-paid-delivery/v1` | str_key | PAID_DELIVERY_DOMAIN = "affidavit-paid-delivery/v1" |  |  |  |  |

| `affidavit.cli` | str_key | CERTIFIED_RECEIPT_AUDIENCE = "affidavit.cli" |  |  |  |  |

| `CertifiedReceiptEnvelope` | struct | CertifiedReceiptEnvelope { pub envelope: SignatureEnvelope, pub receipt: CryptoStandingReceipt, pub signature_hex: String, pub verifying_key_sec1_hex: String } |  |  |  |  |

| `REGISTRY` | const | REGISTRY: &[VerbEntry] |  |  |  |  |

| `VerbGroup` | enum | VerbGroup { Core, Diagnostics, Analysis, Ingestion, Compliance, Attestation, Sbom, Insights, Engineering, Tooling, Federation } |  |  |  |  |

| `by_group` | function | by_group(group: VerbGroup) -> Vec<&'static VerbEntry> |  |  |  |  |

| `description` | function | description(self) -> &'static str |  |  |  |  |

| `did_you_mean` | function | did_you_mean(input: &str) -> Vec<&'static VerbEntry> |  |  |  |  |

| `label` | function | label(self) -> &'static str |  |  |  |  |

| `lookup` | function | lookup(verb: &str, noun: &str) -> Option<&'static VerbEntry> |  |  |  |  |

| `new` | function | new( verb: &'static str, noun: &'static str, group: VerbGroup, summary: &'static str, keywords: &'static [&'static str], ) -> Self |  |  |  |  |

| `search` | function | search(query: &str) -> Vec<&'static VerbEntry> |  |  |  |  |

| `verb_count` | function | verb_count() -> usize |  |  |  |  |

| `with_example` | function | with_example(mut self, example: &'static str) -> Self |  |  |  |  |

| `emit` | str_key | REGISTRY = "emit" |  |  |  |  |

| `VerbEntry` | struct | VerbEntry { pub verb: &'static str, pub noun: &'static str, pub group: VerbGroup, pub summary: &'static str, pub keywords: &'static [&'static str], pub example: Option<&'static str> } |  |  |  |  |

| `ReplayFilterError` | enum | ReplayFilterError { Full(usize) } |  |  |  |  |

| `capacity` | function | capacity(&self) -> usize |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(capacity: usize) -> Self |  |  |  |  |

| `probably_seen` | function | probably_seen(&self, id: &[u8; 32]) -> bool |  |  |  |  |

| `retract` | function | retract(&mut self, id: &[u8; 32]) -> bool |  |  |  |  |

| `witness` | function | witness(&mut self, id: &[u8; 32]) -> Result<bool, ReplayFilterError> |  |  |  |  |

| `ReplayFilter` | struct | ReplayFilter { filter: CuckooFilter<std::collections::hash_map::DefaultHasher>, capacity: usize } |  |  |  |  |

| `ComponentType` | enum | ComponentType { Application, Library, Framework, Container, OperatingSystem, Device, Firmware, File, Platform, DeviceDriver, MachineLearningModel, Data } |  |  |  |  |

| `SbomError` | enum | SbomError { Parse(String), UnrecognizedFormat(String), MissingField(String) } |  |  |  |  |

| `SbomFormat` | enum | SbomFormat { Spdx23, Spdx30, CycloneDx15, CycloneDx16, SwidTag } |  |  |  |  |

| `canonicalize` | function | canonicalize(&mut self) |  |  |  |  |

| `component` | function | component(&self, bom_ref: &str) -> Option<&Component> |  |  |  |  |

| `content_address` | function | content_address(&self) -> Blake3Hash |  |  |  |  |

| `detect_format` | function | detect_format(doc: &serde_json::Value) -> Result<SbomFormat, SbomError> |  |  |  |  |

| `expr` | function | expr(expression: impl Into<String>) -> Self |  |  |  |  |

| `family` | function | family(&self) -> &'static str |  |  |  |  |

| `has_unique_identifier` | function | has_unique_identifier(&self) -> bool |  |  |  |  |

| `id` | function | id(spdx_id: impl Into<String>) -> Self |  |  |  |  |

| `is_conformant` | function | is_conformant(&self) -> bool |  |  |  |  |

| `label` | function | label(&self) -> String |  |  |  |  |

| `library` | function | library( bom_ref: impl Into<String>, name: impl Into<String>, version: impl Into<String>, ) -> Self |  |  |  |  |

| `license_labels` | function | license_labels(&self) -> Vec<String> |  |  |  |  |

| `missing` | function | missing(&self) -> Vec<&'static str> |  |  |  |  |

| `new` | function | new(algorithm: impl Into<String>, value: impl Into<String>) -> Self |  |  |  |  |

| `ntia_minimum_elements` | function | ntia_minimum_elements(&self) -> NtiaMinimumElements |  |  |  |  |

| `parse` | function | parse(s: &str) -> ComponentType |  |  |  |  |

| `parse_cyclonedx` | function | parse_cyclonedx(doc: &serde_json::Value) -> Result<Sbom, SbomError> |  |  |  |  |

| `parse_sbom_json` | function | parse_sbom_json(json: &str) -> Result<Sbom, SbomError> |  |  |  |  |

| `parse_spdx` | function | parse_spdx(doc: &serde_json::Value) -> Result<Sbom, SbomError> |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `transitive_dependencies` | function | transitive_dependencies(&self, root: &str) -> Vec<String> |  |  |  |  |

| `Component` | struct | Component { pub bom_ref: String, pub name: String, pub version: String, pub component_type: ComponentType, pub purl: Option<String>, pub cpe: Option<String>, pub supplier: Option<Supplier>, pub author: Option<String>, pub licenses: Vec<License>, pub hashes: Vec<Hash>, pub description: Option<String>, pub scope: Option<String> } |  |  |  |  |

| `Dependency` | struct | Dependency { pub dependent: String, pub depends_on: Vec<String> } |  |  |  |  |

| `Hash` | struct | Hash { pub algorithm: String, pub value: String } |  |  |  |  |

| `License` | struct | License { pub spdx_id: Option<String>, pub expression: Option<String>, pub name: Option<String>, pub url: Option<String> } |  |  |  |  |

| `NtiaMinimumElements` | struct | NtiaMinimumElements { pub supplier_name: bool, pub component_name: bool, pub version: bool, pub unique_identifiers: bool, pub dependency_relationship: bool, pub author: bool, pub timestamp: bool } |  |  |  |  |

| `Sbom` | struct | Sbom { pub format: SbomFormat, pub spec_version: String, pub serial_number: Option<String>, pub version: u32, pub metadata: SbomMetadata, pub components: Vec<Component>, pub dependencies: Vec<Dependency> } |  |  |  |  |

| `SbomMetadata` | struct | SbomMetadata { pub author: Option<String>, pub supplier: Option<Supplier>, pub tools: Vec<Tool>, pub primary_component: Option<String>, pub timestamp: u64 } |  |  |  |  |

| `Supplier` | struct | Supplier { pub name: String, pub url: Option<String>, pub contact: Option<String> } |  |  |  |  |

| `Tool` | struct | Tool { pub vendor: Option<String>, pub name: String, pub version: Option<String> } |  |  |  |  |

| `add_cleanup_action` | function | add_cleanup_action(mut self, action: F) -> Self |  |  |  |  |

| `bundle` | function | bundle(&self) -> Option<&SbomForensicsBundle> |  |  |  |  |

| `cleanup` | function | cleanup(mut self) |  |  |  |  |

| `create_bundle` | function | create_bundle( &self, sbom_id: &str, sbom_format: &str, description: Option<String>, ) -> SbomForensicsBundle |  |  |  |  |

| `load_bundle` | function | load_bundle(&self, path: PathBuf) -> Result<SbomForensicsBundle> |  |  |  |  |

| `new` | function | new() -> Result<Self> |  |  |  |  |

| `redact_sensitive` | function | redact_sensitive(&self, value: &str) -> String |  |  |  |  |

| `save_bundle` | function | save_bundle(&self, bundle: &SbomForensicsBundle, path: PathBuf) -> Result<()> |  |  |  |  |

| `take_bundle` | function | take_bundle(mut self) -> Option<SbomForensicsBundle> |  |  |  |  |

| `with_path` | function | with_path(mut self, path: PathBuf) -> Self |  |  |  |  |

| `with_redaction` | function | with_redaction(mut self, redact: bool) -> Self |  |  |  |  |

| `AttestationData` | struct | AttestationData { pub slsa_version: String, pub builder: Option<String>, pub timestamp: u64, pub environment_hash: String } |  |  |  |  |

| `BundleMetadata` | struct | BundleMetadata { pub version: String, pub created_at: u64, pub sbom_id: String, pub bundle_id: String, pub sbom_format: String, pub description: Option<String> } |  |  |  |  |

| `ComplianceRecord` | struct | ComplianceRecord { pub framework: String, pub passed: bool, pub score: f64, pub failures: Vec<String> } |  |  |  |  |

| `ComponentNode` | struct | ComponentNode { pub purl: String, pub name: String, pub version: String, pub supplier: Option<String>, pub licenses: Vec<String> } |  |  |  |  |

| `RiskPropagationRecord` | struct | RiskPropagationRecord { pub component: String, pub root_cve: String, pub propagated_to: Vec<String>, pub blast_radius: usize } |  |  |  |  |

| `SbomArtifactCollector` | struct | SbomArtifactCollector { work_dir: PathBuf, redact_sensitive: bool } |  |  |  |  |

| `SbomArtifactGuard` | struct | SbomArtifactGuard { bundle: Option<SbomForensicsBundle>, path: Option<PathBuf>, cleanup_actions: Vec<Box<dyn FnOnce() + Send + Sync>> } |  |  |  |  |

| `SbomForensicsBundle` | struct | SbomForensicsBundle { pub metadata: BundleMetadata, pub graph: Option<SupplyChainGraph>, pub vulnerabilities: Vec<VulnerabilityRecord>, pub compliance: Vec<ComplianceRecord>, pub risk_propagation: Vec<RiskPropagationRecord>, pub attestation: Option<AttestationData>, pub redactions: HashMap<String, String> } |  |  |  |  |

| `SupplyChainGraph` | struct | SupplyChainGraph { pub component_count: usize, pub edge_count: usize, pub adjacency: HashMap<String, Vec<String>>, pub components: HashMap<String, ComponentNode> } |  |  |  |  |

| `VulnerabilityRecord` | struct | VulnerabilityRecord { pub cve_id: String, pub affected_component: String, pub severity: String, pub cvss_score: Option<f64>, pub vex_status: Option<String> } |  |  |  |  |

| `ComplianceError` | enum | ComplianceError { EmptySbom } |  |  |  |  |

| `Framework` | enum | Framework { Ntia, ExecutiveOrder14028, Slsa, InToto, Cisa, Cscrm, Iso27001, Soc2, Vex } |  |  |  |  |

| `SlsaLevel` | enum | SlsaLevel { L0, L1, L2, L3, L4 } |  |  |  |  |

| `assess_all` | function | assess_all(sbom: &Sbom) -> Result<Vec<ComplianceResult>, ComplianceError> |  |  |  |  |

| `assess_slsa` | function | assess_slsa(sbom: &Sbom) -> Result<(SlsaLevel, ComplianceResult), ComplianceError> |  |  |  |  |

| `check_cisa` | function | check_cisa(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_cscrm` | function | check_cscrm(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_eo_14028` | function | check_eo_14028(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_in_toto` | function | check_in_toto(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_iso_27001` | function | check_iso_27001(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_ntia` | function | check_ntia(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_slsa` | function | check_slsa(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `check_soc2` | function | check_soc2(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `display_name` | function | display_name(&self) -> &'static str |  |  |  |  |

| `rank` | function | rank(&self) -> u8 |  |  |  |  |

| `requirement_count` | function | requirement_count(&self) -> usize |  |  |  |  |

| `score` | function | score(&self) -> f64 |  |  |  |  |

| `supported_frameworks` | function | supported_frameworks() -> &'static [&'static str] |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `vex_readiness` | function | vex_readiness(sbom: &Sbom) -> Result<ComplianceResult, ComplianceError> |  |  |  |  |

| `ComplianceResult` | struct | ComplianceResult { pub framework: String, pub level: Option<String>, pub passed: bool, pub satisfied: Vec<String>, pub failed: Vec<String>, pub notes: Vec<String> } |  |  |  |  |

| `EVENT_ATTEST` | const | EVENT_ATTEST: &str |  |  |  |  |

| `EVENT_COMPONENT_CATALOGUED` | const | EVENT_COMPONENT_CATALOGUED: &str |  |  |  |  |

| `EVENT_DEPENDENCY_RESOLVED` | const | EVENT_DEPENDENCY_RESOLVED: &str |  |  |  |  |

| `EVENT_GENERATE` | const | EVENT_GENERATE: &str |  |  |  |  |

| `EVENT_IMPORT` | const | EVENT_IMPORT: &str |  |  |  |  |

| `EVENT_LICENSE_DETECTED` | const | EVENT_LICENSE_DETECTED: &str |  |  |  |  |

| `EVENT_SUPPLIER_ATTESTED` | const | EVENT_SUPPLIER_ATTESTED: &str |  |  |  |  |

| `OBJECT_DOCUMENT` | const | OBJECT_DOCUMENT: &str |  |  |  |  |

| `SbomOcelError` | enum | SbomOcelError { EmptyDocument, UnknownComponent(String), Ocel( } |  |  |  |  |

| `build_sbom_causal_chain` | function | build_sbom_causal_chain( events: &[SbomOcelEvent], sbom: &Sbom, root_bom_ref: &str, ) -> Result<SbomCausalChain, SbomOcelError> |  |  |  |  |

| `component_object_type` | function | component_object_type(component: &Component) -> String |  |  |  |  |

| `correlate_components_by_license` | function | correlate_components_by_license( events: &[SbomOcelEvent], sbom: &Sbom, ) -> Vec<ObjectCorrelation> |  |  |  |  |

| `license_object_type` | function | license_object_type(label: &str) -> String |  |  |  |  |

| `sbom_to_ocel_events` | function | sbom_to_ocel_events( sbom: &Sbom, counter: &mut SeqCounter, ) -> Result<Vec<SbomOcelEvent>, SbomOcelError> |  |  |  |  |

| `supplier_object_type` | function | supplier_object_type(name: &str) -> String |  |  |  |  |

| `supported_event_types` | function | supported_event_types() -> &'static [&'static str] |  |  |  |  |

| `vulnerability_object_type` | function | vulnerability_object_type(id: &str) -> String |  |  |  |  |

| `sbom-document` | str_key | OBJECT_DOCUMENT = "sbom-document" |  |  |  |  |

| `sbom:attest` | str_key | EVENT_ATTEST = "sbom:attest" |  |  |  |  |

| `sbom:component-catalogued` | str_key | EVENT_COMPONENT_CATALOGUED = "sbom:component-catalogued" |  |  |  |  |

| `sbom:dependency-resolved` | str_key | EVENT_DEPENDENCY_RESOLVED = "sbom:dependency-resolved" |  |  |  |  |

| `sbom:generate` | str_key | EVENT_GENERATE = "sbom:generate" |  |  |  |  |

| `sbom:import` | str_key | EVENT_IMPORT = "sbom:import" |  |  |  |  |

| `sbom:license-detected` | str_key | EVENT_LICENSE_DETECTED = "sbom:license-detected" |  |  |  |  |

| `sbom:supplier-attested` | str_key | EVENT_SUPPLIER_ATTESTED = "sbom:supplier-attested" |  |  |  |  |

| `ObjectCorrelation` | struct | ObjectCorrelation { pub object_type: String, pub object_ids: Vec<String>, pub event_ids: Vec<String> } |  |  |  |  |

| `SbomCausalChain` | struct | SbomCausalChain { pub document_event_id: String, pub component_event_id: String, pub dependency_event_ids: Vec<String>, pub root_component: String, pub transitive_component_count: usize } |  |  |  |  |

| `SbomOcelEvent` | struct | SbomOcelEvent { pub event: OperationEvent, pub sbom_event_type: String, pub payload: serde_json::Value } |  |  |  |  |

| `UNKNOWN_SUPPLIER` | const | UNKNOWN_SUPPLIER: &str |  |  |  |  |

| `SupplyChainError` | enum | SupplyChainError { UnknownComponent(String), EmptyGraph } |  |  |  |  |

| `attest_provenance` | function | attest_provenance(sbom: &Sbom, receipt_ref: Option<&str>) -> ProvenanceAttestation |  |  |  |  |

| `blast_radius` | function | blast_radius( graph: &DependencyGraph, bom_ref: &str, ) -> Result<BlastRadius, SupplyChainError> |  |  |  |  |

| `build_report` | function | build_report(sbom: &Sbom, spof_threshold: usize) -> SupplyChainReport |  |  |  |  |

| `contains` | function | contains(&self, bom_ref: &str) -> bool |  |  |  |  |

| `depth` | function | depth(&self, root: &str) -> usize |  |  |  |  |

| `direct_dependencies` | function | direct_dependencies(&self, bom_ref: &str) -> Vec<String> |  |  |  |  |

| `direct_dependents` | function | direct_dependents(&self, bom_ref: &str) -> Vec<String> |  |  |  |  |

| `edge_count` | function | edge_count(&self) -> usize |  |  |  |  |

| `from_sbom` | function | from_sbom(sbom: &Sbom) -> Self |  |  |  |  |

| `is_cyclic` | function | is_cyclic(&self) -> bool |  |  |  |  |

| `node_count` | function | node_count(&self) -> usize |  |  |  |  |

| `nodes` | function | nodes(&self) -> Vec<String> |  |  |  |  |

| `single_points_of_failure` | function | single_points_of_failure( graph: &DependencyGraph, _sbom: &Sbom, threshold: usize, ) -> Vec<String> |  |  |  |  |

| `supplier_concentration` | function | supplier_concentration(sbom: &Sbom) -> Vec<SupplierConcentration> |  |  |  |  |

| `transitive_dependencies` | function | transitive_dependencies(&self, bom_ref: &str) -> Vec<String> |  |  |  |  |

| `transitive_dependents` | function | transitive_dependents(&self, bom_ref: &str) -> Vec<String> |  |  |  |  |

| `UNKNOWN` | str_key | UNKNOWN_SUPPLIER = "UNKNOWN" |  |  |  |  |

| `BlastRadius` | struct | BlastRadius { pub component: String, pub directly_impacted: usize, pub transitively_impacted: usize, pub impacted: Vec<String> } |  |  |  |  |

| `DependencyGraph` | struct | DependencyGraph { nodes: BTreeSet<String>, forward: BTreeMap<String, BTreeSet<String>>, reverse: BTreeMap<String, BTreeSet<String>> } |  |  |  |  |

| `ProvenanceAttestation` | struct | ProvenanceAttestation { pub sbom_address: String, pub primary_component: Option<String>, pub builder: Option<String>, pub attested_supplier: Option<String>, pub dependency_edges: usize, pub generated_from_receipt: Option<String> } |  |  |  |  |

| `SupplierConcentration` | struct | SupplierConcentration { pub supplier: String, pub component_count: usize, pub share: f64, pub components: Vec<String> } |  |  |  |  |

| `SupplyChainReport` | struct | SupplyChainReport { pub component_count: usize, pub edge_count: usize, pub max_depth: usize, pub is_cyclic: bool, pub supplier_count: usize, pub top_supplier_share: f64, pub spof_count: usize } |  |  |  |  |

| `Severity` | enum | Severity { None, Low, Medium, High, Critical } |  |  |  |  |

| `VexStatus` | enum | VexStatus { NotAffected, Affected, Fixed, UnderInvestigation } |  |  |  |  |

| `VulnerabilityError` | enum | VulnerabilityError { InvalidCvssScore(f64), UnknownComponent(String) } |  |  |  |  |

| `apply_vex` | function | apply_vex(matches: &[VulnerabilityMatch], vex: &[VexStatement]) -> Vec<VulnerabilityMatch> |  |  |  |  |

| `build_report` | function | build_report( sbom: &Sbom, vulns: &[Vulnerability], vex: &[VexStatement], ) -> VulnerabilityReport |  |  |  |  |

| `cvss_band` | function | cvss_band(score: f64) -> &'static str |  |  |  |  |

| `from_cvss` | function | from_cvss(score: f64) -> Severity |  |  |  |  |

| `from_score` | function | from_score(base_score: f64) -> Self |  |  |  |  |

| `match_vulnerabilities` | function | match_vulnerabilities(sbom: &Sbom, vulns: &[Vulnerability]) -> Vec<VulnerabilityMatch> |  |  |  |  |

| `new` | function | new(id: impl Into<String>, base_score: f64) -> Self |  |  |  |  |

| `propagate_risk` | function | propagate_risk( sbom: &Sbom, matches: &[VulnerabilityMatch], ) -> Vec<(String, String, Severity)> |  |  |  |  |

| `severity` | function | severity(&self) -> Severity |  |  |  |  |

| `suppresses` | function | suppresses(&self) -> bool |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `validate` | function | validate(&self) -> Result<(), VulnerabilityError> |  |  |  |  |

| `with_vector` | function | with_vector(base_score: f64, vector: impl Into<String>) -> Self |  |  |  |  |

| `CvssVector` | struct | CvssVector { pub base_score: f64, pub vector: Option<String> } |  |  |  |  |

| `VexStatement` | struct | VexStatement { pub vuln_id: String, pub component_bom_ref: String, pub status: VexStatus, pub justification: Option<String> } |  |  |  |  |

| `Vulnerability` | struct | Vulnerability { pub id: String, pub description: Option<String>, pub cvss: CvssVector, pub affected_purls: Vec<String>, pub affected_cpes: Vec<String>, pub fixed_versions: Vec<String> } |  |  |  |  |

| `VulnerabilityMatch` | struct | VulnerabilityMatch { pub vuln_id: String, pub component_bom_ref: String, pub severity: Severity, pub matched_by: String } |  |  |  |  |

| `VulnerabilityReport` | struct | VulnerabilityReport { pub total_components: usize, pub total_matches: usize, pub by_severity: BTreeMap<String, usize>, pub max_severity: Severity, pub exploitable_after_vex: usize } |  |  |  |  |

| `Secp256k1WitnessError` | enum | Secp256k1WitnessError { MalformedSchnorrKey, SchnorrVerificationFailed, MalformedEcdsaKey, EcdsaVerificationFailed } |  |  |  |  |

| `WitnessSigningError` | enum | WitnessSigningError { InvalidSeed } |  |  |  |  |

| `from_seed` | function | from_seed(seed: &[u8; 32]) -> Result<Self, WitnessSigningError> |  |  |  |  |

| `public_key_sec1` | function | public_key_sec1(&self) -> [u8; 33] |  |  |  |  |

| `sign` | function | sign(&self, msg: &[u8]) -> [u8; 64] |  |  |  |  |

| `verify_bip340` | function | verify_bip340( x_only_public_key: &[u8; 32], message: &[u8], signature: &[u8; 64], ) -> Result<(), Secp256k1WitnessError> |  |  |  |  |

| `verify_bip340_raw` | function | verify_bip340_raw( x_only_public_key: &[u8; 32], message: &[u8; 32], signature: &[u8; 64], ) -> Result<(), Secp256k1WitnessError> |  |  |  |  |

| `verify_ecdsa` | function | verify_ecdsa( compressed_sec1_public_key: &[u8; 33], message: &[u8], signature: &[u8], ) -> Result<(), Secp256k1WitnessError> |  |  |  |  |

| `WitnessSigningKey` | struct | WitnessSigningKey { signing: EcdsaSigningKey } |  |  |  |  |

| `SeqBitmapError` | enum | SeqBitmapError { Gap(u32), Duplicate(u32) } |  |  |  |  |

| `count` | function | count(&self) -> u64 |  |  |  |  |

| `has` | function | has(&self, seq: u32) -> bool |  |  |  |  |

| `max` | function | max(&self) -> Option<u32> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `record` | function | record(&mut self, seq: u32) -> Result<(), SeqBitmapError> |  |  |  |  |

| `verify_contiguous` | function | verify_contiguous(&self) -> Result<(), SeqBitmapError> |  |  |  |  |

| `SeqContiguityCertifier` | struct | SeqContiguityCertifier { bitmap: RoaringBitmap } |  |  |  |  |

| `CAMPAIGN_DOMAIN` | const | CAMPAIGN_DOMAIN: &str |  |  |  |  |

| `EVENT_COMMIT` | const | EVENT_COMMIT: &str |  |  |  |  |

| `EVENT_COURT` | const | EVENT_COURT: &str |  |  |  |  |

| `EVENT_RESIDUE` | const | EVENT_RESIDUE: &str |  |  |  |  |

| `AuthorityCeiling` | enum | AuthorityCeiling { Observe, Select, Construct, Do } |  |  |  |  |

| `BrokenTerm` | enum | BrokenTerm { MuOnO, AdmissionVacuous, MuUnlawful, RMissingIdentity, RMissingAuthority, RMissingConsequence, RMissingReplay, RMissingStanding, RNotFedBack } |  |  |  |  |

| `SjRefusal` | enum | SjRefusal { EmptyKey { key: &'static str, }, BadCommitSha { key: &'static str, value: String, }, BadSha { key: &'static str, value: String, }, BadStanding(String), MissingBrokenTerm(String), EmptyReplay, BadCommand { index: usize, reason: &'static str, }, Canonical(String), Chain(String), ChainTamper(String), DigestMismatch { expected: String, claimed: String, }, ChainHeadMismatch, EventClaimMismatch, Decode(String) } |  |  |  |  |

| `StandingValue` | enum | StandingValue { Unknown, PartialAlive, Alive, Blocked( BuildBroken, Unsupported( Refused(String), } |  |  |  |  |

| `as_str` | function | as_str(self) -> &'static str |  |  |  |  |

| `canonical_json` | function | canonical_json(&self) -> Result<String, SjRefusal> |  |  |  |  |

| `chain_events` | function | chain_events(&self) -> Vec<OperationEvent> |  |  |  |  |

| `finalize` | function | finalize(self) -> Result<SjRecord, SjRefusal> |  |  |  |  |

| `from_json` | function | from_json(bytes: &str) -> Result<Self, SjRefusal> |  |  |  |  |

| `new` | function | new(draft: SjCampaignDraft) -> Result<Self, SjRefusal> |  |  |  |  |

| `requires_broken_term` | function | requires_broken_term(&self) -> bool |  |  |  |  |

| `subject_digest` | function | subject_digest(&self) -> Result<[u8; 32], SjRefusal> |  |  |  |  |

| `subject_digest_hex` | function | subject_digest_hex(&self) -> Result<String, SjRefusal> |  |  |  |  |

| `to_json` | function | to_json(&self) -> Result<String, SjRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), SjRefusal> |  |  |  |  |

| `affidavit-campaign/v1` | str_key | CAMPAIGN_DOMAIN = "affidavit-campaign/v1" |  |  |  |  |

| `campaign-commit` | str_key | EVENT_COMMIT = "campaign-commit" |  |  |  |  |

| `campaign-court-witness` | str_key | EVENT_COURT = "campaign-court-witness" |  |  |  |  |

| `campaign-residue` | str_key | EVENT_RESIDUE = "campaign-residue" |  |  |  |  |

| `Authority` | struct | Authority { pub ceiling: AuthorityCeiling, pub grant: String, pub actor: String } |  |  |  |  |

| `CommitRecord` | struct | CommitRecord { pub sha: String, pub summary: String, pub court_results: Vec<String> } |  |  |  |  |

| `Consequence` | struct | Consequence { pub commits: Vec<String>, pub files_changed: Vec<String>, pub remote_effects: Vec<String> } |  |  |  |  |

| `Identity` | struct | Identity { pub subject: String, pub repo: String, pub subject_sha: String, pub base_sha: String, pub subject_digest: SubjectDigest } |  |  |  |  |

| `OriginAuthority` | struct | OriginAuthority { pub ceiling: Option<AuthorityCeiling>, pub grant: String, pub actor: String } |  |  |  |  |

| `Provider` | struct | Provider { pub name: String, pub transport: Option<String> } |  |  |  |  |

| `Replay` | struct | Replay { pub commands: Vec<ReplayCommand>, pub durable_location: Option<String> } |  |  |  |  |

| `ReplayBinding` | struct | ReplayBinding { pub event_ids: Vec<String>, pub chain_head_hash: String, pub predecessor_work_order_ids: Vec<String> } |  |  |  |  |

| `ReplayCommand` | struct | ReplayCommand { pub cmd: String, pub exit: i32, pub cwd: String, pub summary: Option<String>, pub output_sha256: Option<String> } |  |  |  |  |

| `SjCampaign` | struct | SjCampaign { draft: SjCampaignDraft } |  |  |  |  |

| `SjCampaignDraft` | struct | SjCampaignDraft { pub work_order_id: String, pub origin_ceiling: Option<AuthorityCeiling>, pub origin_grant: String, pub origin_actor: String, pub provider_name: String, pub provider_execution_id: String, pub subject: String, pub repo: String, pub subject_sha: String, pub base_sha: String, pub commits: Vec<CommitRecord>, pub residue_declaration: String, pub files_changed: Vec<String>, pub remote_effects: Vec<String>, pub replay_commands: Vec<ReplayCommand>, pub durable_location: Option<String>, pub standing: StandingValue, pub derived_from: String, pub broken_term: Option<BrokenTerm>, pub predecessor_work_order_ids: Vec<String>, pub authority: Authority } |  |  |  |  |

| `SjRecord` | struct | SjRecord { pub base: Receipt, pub document: SjRecordDocument } |  |  |  |  |

| `SjRecordDocument` | struct | SjRecordDocument { pub work_order_id: String, pub origin_authority: OriginAuthority, pub provider: Provider, pub provider_execution_id: String, pub identity: Identity, pub authority: Authority, pub consequence: Consequence, pub replay: Replay, pub replay_binding: ReplayBinding, pub standing: Standing } |  |  |  |  |

| `Standing` | struct | Standing { pub value: StandingValue, pub derived_from: String, pub broken_term: Option<BrokenTerm> } |  |  |  |  |

| `SubjectDigest` | struct | SubjectDigest { pub algorithm: String, pub value: String } |  |  |  |  |

| `SmtError` | enum | SmtError { Tree(String), NoProof } |  |  |  |  |

| `common_prefix_len` | function | common_prefix_len( a: &BitSlice<u8, bitvec::prelude::Msb0>, b: &BitSlice<u8, bitvec::prelude::Msb0>, ) -> usize |  |  |  |  |

| `get` | function | get(&mut self, key: &StateKey) -> Result<Option<StateValue>, SmtError> |  |  |  |  |

| `insert` | function | insert(&mut self, key: &StateKey, value: &StateValue) -> Result<StateRoot, SmtError> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `prove_absence` | function | prove_absence(&mut self, key: &StateKey) -> Result<AbsenceWitness, SmtError> |  |  |  |  |

| `prove_inclusion` | function | prove_inclusion(&mut self, key: &StateKey) -> Result<InclusionProof, SmtError> |  |  |  |  |

| `remove` | function | remove(&mut self, key: &StateKey) -> Result<Option<StateRoot>, SmtError> |  |  |  |  |

| `root` | function | root(&self) -> Option<StateRoot> |  |  |  |  |

| `verify_absence` | function | verify_absence( root: &StateRoot, neighbor_value: &StateValue, witness: &AbsenceWitness, ) -> bool |  |  |  |  |

| `verify_inclusion` | function | verify_inclusion( root: &StateRoot, expected_value: &StateValue, proof: &InclusionProof, ) -> bool |  |  |  |  |

| `AbsenceWitness` | struct | AbsenceWitness { pub queried_key: StateKey, pub neighbor_key: StateKey, pub neighbor_proof: InclusionProof, pub claimed_prefix_bits: usize } |  |  |  |  |

| `InclusionProof` | struct | InclusionProof { pub key: StateKey, pub steps: Vec<(bool, Vec<u8>)> } |  |  |  |  |

| `StateTree` | struct | StateTree { tree: Monotree<monotree::database::MemoryDB, MonotreeBlake3>, root: Option<monotree::Hash>, keys: std::collections::BTreeMap<StateKey, StateValue> } |  |  |  |  |

| `monotree::HASH_LEN` | use | monotree::HASH_LEN |  |  |  |  |

| `STANDING_PROFILE` | const | STANDING_PROFILE: &str |  |  |  |  |

| `Standing` | enum | Standing { Unknown, PartialAlive, Alive, Blocked, BuildBroken, Unsupported } |  |  |  |  |

| `StandingRefusal` | enum | StandingRefusal { AuthorityEnvelopeRejected(Vec<AuthorityRefusal>), UnsupportedAuthorityConstraint, EmptyField(&'static str), MalformedBlake3(&'static str), AliveMissingExecution, AliveExecutionFailed(i32), AliveMissingVerification, AliveVerificationFailed(i32), AliveMissingReplay, WrongProfile, ReceiptHashMismatch, Serialization(String) } |  |  |  |  |

| `certify_standing` | function | certify_standing( admitted: &AdmittedReceipt, authority: &AuthorityEnvelope<W>, observation: StandingObservation, ) -> Result<StandingReceipt, StandingRefusal> |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), StandingRefusal> |  |  |  |  |

| `affidavit/standing/v2` | str_key | STANDING_PROFILE = "affidavit/standing/v2" |  |  |  |  |

| `AuthorityBinding` | struct | AuthorityBinding { pub witness_key: String, pub capability: String, pub capability_digest: String, pub scope: String, pub constraints: Vec<String>, pub data_minimization_note: String, pub fairness_attestation_ref: String } |  |  |  |  |

| `ExecutionEvidence` | struct | ExecutionEvidence { pub command: String, pub exit_code: i32, pub result_commitment: Blake3Hash } |  |  |  |  |

| `ReplayEvidence` | struct | ReplayEvidence { pub command: String, pub environment_commitment: Blake3Hash } |  |  |  |  |

| `StandingObservation` | struct | StandingObservation { pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub standing: Standing, pub execution: Option<ExecutionEvidence>, pub verification: Option<VerificationEvidence>, pub replay: Option<ReplayEvidence>, pub previous_receipt: Option<Blake3Hash> } |  |  |  |  |

| `StandingReceipt` | struct | StandingReceipt { pub profile: String, pub subject: SubjectIdentity, pub observation_commitment: Blake3Hash, pub admitted_receipt_hash: Blake3Hash, pub authority: AuthorityBinding, pub standing: Standing, pub execution: Option<ExecutionEvidence>, pub verification: Option<VerificationEvidence>, pub replay: Option<ReplayEvidence>, pub previous_receipt: Option<Blake3Hash>, pub receipt_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `SubjectIdentity` | struct | SubjectIdentity { pub subject: String, pub base: String, pub tree: String, pub candidate: String } |  |  |  |  |

| `VerificationEvidence` | struct | VerificationEvidence { pub command: String, pub exit_code: i32, pub report_commitment: Blake3Hash } |  |  |  |  |

| `QuorumError` | enum | QuorumError { Protocol(String), VerificationFailed, MalformedKeyMaterial, BelowThreshold { expected: usize, got: usize, } } |  |  |  |  |

| `aggregate_signature` | function | aggregate_signature( package: &SigningPackage, shares: &[(ParticipantId, frost::round2::SignatureShare)], public_key_package: &PublicKeyPackage, threshold: u16, ) -> Result<[u8; 64], QuorumError> |  |  |  |  |

| `generate_quorum` | function | generate_quorum( total: u16, threshold: u16, rng: &mut R, ) -> Result<DealtQuorum, QuorumError> |  |  |  |  |

| `round1_commit` | function | round1_commit( secret_share: &SecretShare, rng: &mut R, ) -> Result<(SigningNonces, frost::round1::SigningCommitments), QuorumError> |  |  |  |  |

| `round2_sign` | function | round2_sign( secret_share: &SecretShare, nonces: &SigningNonces, package: &SigningPackage, ) -> Result<frost::round2::SignatureShare, QuorumError> |  |  |  |  |

| `signing_package` | function | signing_package( commitments: &[(ParticipantId, frost::round1::SigningCommitments)], message: &[u8], threshold: u16, ) -> Result<SigningPackage, QuorumError> |  |  |  |  |

| `verify_quorum` | function | verify_quorum( group_public_key: &GroupPublicKey, message: &[u8], signature_bytes: &[u8; 64], ) -> Result<(), QuorumError> |  |  |  |  |

| `DealtQuorum` | struct | DealtQuorum { pub shares: BTreeMap<ParticipantId, SecretShare>, pub public_key_package: PublicKeyPackage, pub group_key: GroupPublicKey, pub threshold: u16 } |  |  |  |  |

| `GroupPublicKey` | struct | GroupPublicKey { pub [u8; 32] } |  |  |  |  |

| `ParticipantId` | struct | ParticipantId { pub u16 } |  |  |  |  |

| `AFFI_TRACE_SINK` | env_key | std::env::var("AFFI_TRACE_SINK") |  |  |  |  |

| `captured_spans` | function | captured_spans() -> Vec<SpanRecord> |  |  |  |  |

| `clear_spans` | function | clear_spans() |  |  |  |  |

| `trace_assemble` | function | trace_assemble(event_count: usize, f: F) -> T |  |  |  |  |

| `trace_emit` | function | trace_emit(event_type: &str, _object_count: usize, f: F) -> T |  |  |  |  |

| `trace_show` | function | trace_show(receipt_path: &str, f: F) -> T |  |  |  |  |

| `trace_verify` | function | trace_verify(receipt_path: &str, f: F) -> T |  |  |  |  |

| `SpanRecord` | struct | SpanRecord { pub operation: String, pub target: String } |  |  |  |  |

| `ProfileId` | enum | ProfileId { CoreV1 } |  |  |  |  |

| `as_hex` | function | as_hex(&self) -> &str |  |  |  |  |

| `as_str` | function | as_str(&self) -> &'static str |  |  |  |  |

| `canonical_bytes` | function | canonical_bytes(value: &T) -> Result<Vec<u8>, serde_json::Error> |  |  |  |  |

| `from_bytes` | function | from_bytes(bytes: &[u8]) -> Self |  |  |  |  |

| `from_hex` | function | from_hex(hex: impl Into<String>) -> Self |  |  |  |  |

| `AffidavitReceiptChain` | struct |  |  |  |  |  |

| `AssembleOutput` | struct | AssembleOutput { pub receipt_path: String, pub content_address: String, pub event_count: usize } |  |  |  |  |

| `Blake3Hash` | struct | Blake3Hash { pub String } |  |  |  |  |

| `CheckOutcome` | struct | CheckOutcome { pub stage: String, pub passed: bool, pub detail: String } |  |  |  |  |

| `EmitOutput` | struct | EmitOutput { pub event_id: String, pub seq: u64, pub event_type: String, pub commitment: String } |  |  |  |  |

| `EventSummary` | struct | EventSummary { pub seq: u64, pub id: String, pub event_type: String, pub object_count: usize, pub commitment: String } |  |  |  |  |

| `InspectionReport` | struct | InspectionReport { pub event_count: usize, pub format_version: String, pub chain_hash: String, pub chain_integrity_valid: bool, pub event_types: std::collections::BTreeMap<String, usize>, pub object_types: std::collections::BTreeMap<String, usize>, pub events: Vec<EventSummary> } |  |  |  |  |

| `ObjectRef` | struct | ObjectRef { pub id: String, pub obj_type: String, pub qualifier: Option<String> } |  |  |  |  |

| `OperationEvent` | struct | OperationEvent { pub id: String, pub seq: u64, pub event_type: String, pub objects: Vec<ObjectRef>, pub payload_commitment: Blake3Hash } |  |  |  |  |

| `QualityMeasurement` | struct | QualityMeasurement { pub timestamp: u64, pub stubs: QualityMetricValue, pub types: QualityMetricValue, pub churn: QualityMetricValue, pub comments: QualityMetricValue, pub complexity: QualityMetricValue, pub clippy_warnings: QualityMetricValue, pub rustfmt_violations: QualityMetricValue, pub cargo_deny_issues: QualityMetricValue, pub cargo_audit_vulnerabilities: QualityMetricValue, pub test_coverage: QualityMetricValue, pub doc_coverage: QualityMetricValue } |  |  |  |  |

| `QualityMetricValue` | struct | QualityMetricValue { pub value: f64, pub description: String } |  |  |  |  |

| `QualityViolationEvent` | struct | QualityViolationEvent { pub rule: String, pub metric: String, pub value: f64, pub threshold: f64, pub z_score: f64, pub severity: String, pub description: String } |  |  |  |  |

| `Receipt` | struct | Receipt { pub format_version: String, pub events: Vec<OperationEvent>, pub chain_hash: Blake3Hash, _seal: () } |  |  |  |  |

| `StatsOutput` | struct | StatsOutput { pub event_count: usize, pub chain_depth: usize, pub chain_hash: String, pub event_type_histogram: std::collections::BTreeMap<String, usize>, pub object_type_histogram: std::collections::BTreeMap<String, usize> } |  |  |  |  |

| `Verdict` | struct | Verdict { pub accepted: bool, pub profile: ProfileId, pub outcomes: Vec<CheckOutcome>, pub reason: String } |  |  |  |  |

| `anomaly_detect` | function | anomaly_detect( receipts_path: String, sensitivity: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `assemble` | function | assemble( out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `assemble_and_notarize` | function | assemble_and_notarize( notary_provider: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `assemble_with_signature` | function | assemble_with_signature( signing_method: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `assure` | function | assure( parent: String, witnesses: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `attest` | function | attest( receipt: String, attestation_type: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `audit` | function | audit( ) -> Result<()> |  |  |  |  |

| `bus_factor` | function | bus_factor( receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `catalog` | function | catalog( filter_name: Option<String>, filter_events: Option<usize>, ) -> Result<()> |  |  |  |  |

| `causality_chain` | function | causality_chain( start_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `conformance` | function | conformance( receipt: String, ) -> Result<()> |  |  |  |  |

| `coverage_analysis` | function | coverage_analysis( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `dependency_matrix` | function | dependency_matrix( receipts_path: String, output_matrix: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `diagnose` | function | diagnose( receipt: String, ) -> Result<()> |  |  |  |  |

| `diff` | function | diff( receipt_a: String, receipt_b: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `doctor` | function | doctor( receipts: Option<String>, fix: bool, ) -> Result<()> |  |  |  |  |

| `dora_metrics` | function | dora_metrics( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `ecosystem_certify` | function | ecosystem_certify( receipt: String, observation: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `ecosystem_verify` | function | ecosystem_verify( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit` | function | emit( event_type: String, object: String, payload: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_batch` | function | emit_batch( batch_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_cicd` | function | emit_from_cicd( provider: String, job_status: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_cloud` | function | emit_from_cloud( provider: String, resource_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_github` | function | emit_from_github( repo: String, event_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_gitlab` | function | emit_from_gitlab( repo: String, event_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_monitoring` | function | emit_from_monitoring( provider: String, alert_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_sbom` | function | emit_from_sbom( sbom_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `emit_from_security` | function | emit_from_security( provider: String, vuln_type: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `envelope_export` | function | envelope_export( sealed_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `envelope_list` | function | envelope_list( store: Option<String>, ) -> Result<()> |  |  |  |  |

| `envelope_sign` | function | envelope_sign( receipt: String, key_file: String, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `envelope_verify` | function | envelope_verify( sealed_file: String, store: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_assure` | function | errc_assure( parent: String, witnesses: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_certify` | function | errc_certify( receipt: String, observation: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_verify` | function | errc_verify( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `errc_verify_assurance` | function | errc_verify_assurance( receipt: String, parent: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `evidence_crl_apply` | function | evidence_crl_apply( file: String, store: Option<String>, ) -> Result<()> |  |  |  |  |

| `evidence_crl_publish` | function | evidence_crl_publish( kid: String, epoch: u64, store: Option<String>, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `evidence_heads` | function | evidence_heads( journal_file: Option<String>, ) -> Result<()> |  |  |  |  |

| `evidence_journal` | function | evidence_journal( subject: String, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `explain_incident` | function | explain_incident( incident_desc: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `find_blast_radius` | function | find_blast_radius( change_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `fix` | function | fix( receipt: String, action: Option<String>, dry_run: Option<bool>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `fix_receipt` | function | fix_receipt( receipt: String, action: Option<String>, dry_run: bool, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `gdpr_proof` | function | gdpr_proof( receipts_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `graph` | function | graph( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `guide_search` | function | guide_search( keyword: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `hipaa` | function | hipaa( receipts_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `inspect` | function | inspect( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `install_git_hook` | function | install_git_hook( threshold: Option<String>, ) -> Result<()> |  |  |  |  |

| `keys_generate` | function | keys_generate( algorithm: String, custodian: String, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `keys_import` | function | keys_import( algorithm: String, public_key_hex: String, custodian: String, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `keys_list` | function | keys_list( store: Option<String>, ) -> Result<()> |  |  |  |  |

| `keys_revoke` | function | keys_revoke( kid: String, reason: String, store: Option<String>, ) -> Result<()> |  |  |  |  |

| `keys_rotate` | function | keys_rotate( kid: String, store: Option<String>, out: Option<String>, ) -> Result<()> |  |  |  |  |

| `license_compliance` | function | license_compliance( receipts_path: String, license_policy: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `model` | function | model( receipt: String, ) -> Result<()> |  |  |  |  |

| `monitor` | function | monitor( watch: Option<String>, metrics: Option<String>, rules: Option<String>, baseline_commits: Option<u32>, interval: Option<u64>, output: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `notarize` | function | notarize( receipt: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `orphaned_code` | function | orphaned_code( receipts_path: String, days: Option<u32>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `pci_dss` | function | pci_dss( receipts_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `policy_enforce` | function | policy_enforce( receipts_path: String, policy_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `portfolio_health` | function | portfolio_health( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `predict` | function | predict( receipts_path: String, prediction_type: String, model: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `profile` | function | profile( receipt: Option<String>, duration: Option<u64>, ) -> Result<()> |  |  |  |  |

| `query` | function | query( q: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `receipt_throughput` | function | receipt_throughput( iterations: Option<u32>, ) -> Result<()> |  |  |  |  |

| `replay` | function | replay( receipt: String, ) -> Result<()> |  |  |  |  |

| `root_cause` | function | root_cause( effect_event: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_attest` | function | sbom_attest( sbom_path: String, receipt: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_blast_radius` | function | sbom_blast_radius( sbom_path: String, component: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_compliance` | function | sbom_compliance( sbom_path: String, framework: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_emit` | function | sbom_emit( sbom_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_ntia` | function | sbom_ntia( sbom_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sbom_scan` | function | sbom_scan( sbom_path: String, advisories_path: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `search` | function | search( pattern: String, receipts_path: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `security_debt` | function | security_debt( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `show` | function | show( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `sign` | function | sign( receipt: String, key_path: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `soc2_audit` | function | soc2_audit( receipts_path: String, soc2_type: Option<String>, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `standing_certify` | function | standing_certify( receipt: String, observation: String, scope: String, out: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `standing_verify` | function | standing_verify( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `stats` | function | stats( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `team_velocity` | function | team_velocity( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `tech_debt` | function | tech_debt( receipts_path: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `test` | function | test( ) -> Result<()> |  |  |  |  |

| `timeline` | function | timeline( receipts_path: String, start_time: Option<String>, end_time: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `trend_analysis` | function | trend_analysis( receipts_path: String, metric: String, time_range: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `variance` | function | variance( receipt: Option<String>, iterations: Option<u32>, ) -> Result<()> |  |  |  |  |

| `verify` | function | verify( receipt: String, format: Option<String>, profile: Option<String>, strict: Option<bool>, ) -> Result<()> |  |  |  |  |

| `verify_assurance` | function | verify_assurance( receipt: String, parent: Option<String>, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `verify_compliance` | function | verify_compliance( receipt: String, framework: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `verify_family` | function | verify_family( receipts_dir: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `verify_sla` | function | verify_sla( receipt: String, sla_file: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `visualize` | function | visualize( format: String, receipt: String, ) -> Result<()> |  |  |  |  |

| `why` | function | why( receipt: String, format: Option<String>, ) -> Result<()> |  |  |  |  |

| `verify` | function | verify(receipt: &Receipt) -> Verdict |  |  |  |  |

| `build_graph` | function | build_graph(receipt: &Receipt) -> ReceiptGraph |  |  |  |  |

| `to_dot` | function | to_dot(graph: &ReceiptGraph) -> String |  |  |  |  |

| `to_json` | function | to_json(graph: &ReceiptGraph) -> anyhow::Result<String> |  |  |  |  |

| `GraphEdge` | struct | GraphEdge { pub from: String, pub to: String, pub weight: usize } |  |  |  |  |

| `GraphNode` | struct | GraphNode { pub id: String, pub label: String, pub event_count: usize } |  |  |  |  |

| `ReceiptGraph` | struct | ReceiptGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> } |  |  |  |  |

| `WasmCourtError` | enum | WasmCourtError { Compile(String), Instantiation(String), Export(String), Trap { message: String, }, OutOfFuel { budget: u64, } } |  |  |  |  |

| `new` | function | new() -> Result<Self, WasmCourtError> |  |  |  |  |

| `run_bounded` | function | run_bounded( &self, wasm_bytes: &[u8], export: &str, fuel_budget: u64, ) -> Result<u64, WasmCourtError> |  |  |  |  |

| `WasmCourt` | struct | WasmCourt { engine: Engine } |  |  |  |  |

| `ArchiveError` | enum | ArchiveError { Invalid(String), RangeCheck } |  |  |  |  |

| `freeze` | function | freeze(archive: &ReceiptArchive) -> Result<Vec<u8>, ArchiveError> |  |  |  |  |

| `thaw` | function | thaw(bytes: &[u8]) -> Result<&Archived<ReceiptArchive>, ArchiveError> |  |  |  |  |

| `unfreeze` | function | unfreeze(bytes: &[u8]) -> Result<ReceiptArchive, ArchiveError> |  |  |  |  |

| `validate_archived_range` | function | validate_archived_range( bytes: &[u8], min_seq: u64, max_seq: u64, ) -> Result<(), ArchiveError> |  |  |  |  |

| `ReceiptArchive` | struct | ReceiptArchive { pub chain_id_hex: String, pub entries: Vec<SeqEntry> } |  |  |  |  |

| `SeqEntry` | struct | SeqEntry { pub seq: u64, pub commitment_hex: String } |  |  |  |  |

| `RangeProofError` | enum | RangeProofError { Generation(String), MalformedEncoding, VerificationFailed } |  |  |  |  |

| `prove_range` | function | prove_range( value: u64, bits: usize, label: &'static [u8], rng: &mut R, ) -> Result<RangeWitness, RangeProofError> |  |  |  |  |

| `verify_range` | function | verify_range( commitment: &[u8; 32], proof_bytes: &[u8], bits: usize, label: &'static [u8], ) -> Result<(), RangeProofError> |  |  |  |  |

| `affidavit:zk-range:v1:effect-budget` | str_key | LABEL = "affidavit:zk-range:v1:effect-budget" |  |  |  |  |

| `RangeWitness` | struct | RangeWitness { pub commitment: [u8; 32], pub blinding: [u8; 32], pub proof: Vec<u8>, pub bits: usize } |  |  |  |  |

| `generate_digest` | function | generate_digest(data: &[u8]) -> Digest |  |  |  |  |

| `verify_digest` | function | verify_digest(data: &[u8], expected: &Digest) -> bool |  |  |  |  |

| `Digest` | struct | Digest { [u8; 32] } |  |  |  |  |

| `generate_digest` | function | generate_digest(data: &[u8]) -> Digest |  |  |  |  |

| `verify_digest` | function | verify_digest(data: &[u8], expected: &Digest) -> bool |  |  |  |  |

| `Digest` | struct | Digest { [u8; 32] } |  |  |  |  |

| `MAX_POWL_DEPTH` | const | MAX_POWL_DEPTH: usize |  |  |  |  |

| `ArcDirection` | enum | ArcDirection { PlaceToTransition, TransitionToPlace } |  |  |  |  |

| `ArcDirectionConst` | enum | ArcDirectionConst { PlaceToTransition, TransitionToPlace } |  |  |  |  |

| `BpmnEvent` | enum | BpmnEvent { Start, Intermediate, End, Boundary } |  |  |  |  |

| `BpmnGateway` | enum | BpmnGateway { Exclusive, Parallel, Inclusive, EventBased, Complex } |  |  |  |  |

| `BpmnNodeKind` | enum | BpmnNodeKind { Event(BpmnEvent), Task(BpmnTask), Gateway(BpmnGateway) } |  |  |  |  |

| `BpmnRefusal` | enum | BpmnRefusal { DanglingEdge, MissingStart, MissingStartEvent, MissingEndEvent, EmptyProcess, DuplicateNodeId, LaneNodeNotDeclared } |  |  |  |  |

| `CausalConsistency` | enum | CausalConsistency { Unknown, Consistent, HasCycles, HasContradictions } |  |  |  |  |

| `CausalNetRefusal` | enum | CausalNetRefusal { MissingActivity, InvalidDependencyScore, DisconnectedGraph } |  |  |  |  |

| `CompatDiagnostic` | enum | CompatDiagnostic { MissingWitness, MissingRoundTripFixture, RawEvidenceExportedAsAdmitted, LossyProjectionWithoutPolicy, HiddenFlattening, MissingRefusalPath, MissingReceiptShape, UnreachablePrimitive, MigrationRecommended } |  |  |  |  |

| `ComplianceKind` | enum | ComplianceKind { Monitoring, Audit, Certification } |  |  |  |  |

| `ConformanceRefusal` | enum | ConformanceRefusal { EmptyModel, TooManyDeviations, InvalidProfile, MissingLog, MissingModel, MissingDeviationPath, FitnessUnavailable, PrecisionUnavailable, F1Unavailable, GeneralizationUnavailable, SimplicityUnavailable } |  |  |  |  |

| `ConformanceVerdict` | enum | ConformanceVerdict { PerfectAlignment, FitnessDeficit, DeadlockEncountered } |  |  |  |  |

| `CorrelationSchema` | enum | CorrelationSchema { ByCase, ByObject, ByTimestamp, ByAttribute } |  |  |  |  |

| `CubeDimensionKind` | enum | CubeDimensionKind { Activity, Resource, Time, DataAttribute, ObjectType, CaseAttribute } |  |  |  |  |

| `DeclareRefusal` | enum | DeclareRefusal { EmptyActivity, MissingActivation, BinaryRequiresTarget, UnsupportedTemplate, MissingTarget, InvalidTemplateArity, EmptyObjectScope, SynchronizationViolation } |  |  |  |  |

| `DeclareScope` | enum | DeclareScope { SingleObjectScope(String), MultiObjectScope(Vec<String>), SynchronizedObjectScope(Vec<String>), CrossObjectScope(String, String), GlobalScope } |  |  |  |  |

| `DeclareTemplate` | enum | DeclareTemplate { Existence, Absence, Init, Existence2, Existence3, Absence2, Absence3, RespondedExistence, CoExistence, Response, Precedence, Succession, AlternateResponse, AlternatePrecedence, AlternateSuccession, ChainResponse, ChainPrecedence, ChainSuccession, NotSuccession, NotChainSuccession, NotCoExistence, ExclusiveChoice } |  |  |  |  |

| `DfgRefusal` | enum | DfgRefusal { EmptyGraph, DanglingEdge, IsolatedNode } |  |  |  |  |

| `DiagnosticKind` | enum | DiagnosticKind { Warning, Error, Info } |  |  |  |  |

| `DiagnosticSeverity` | enum | DiagnosticSeverity { Error, Warning, Info } |  |  |  |  |

| `EventLogRefusal` | enum | EventLogRefusal { EmptyLog, EmptyTrace, MissingActivity, NonMonotonicTrace } |  |  |  |  |

| `EventPredicateKind` | enum | EventPredicateKind { ActivityEquals, AttributeEquals, TimestampInRange } |  |  |  |  |

| `EvidenceMode` | enum | EvidenceMode { Raw, Parsed, Admitted, Refused, Projected, Exportable, Witnessed, Receipted } |  |  |  |  |

| `FilterShape` | enum | FilterShape { Activity, Timeframe, Variant, Attribute, ObjectType } |  |  |  |  |

| `FormatKind` | enum | FormatKind { OcelJson, OcelXml, OcelSqlite, XesXml, BpmnXml, PetriPnml, PowlJson } |  |  |  |  |

| `InstanceCreationKind` | enum | InstanceCreationKind { Static, Dynamic } |  |  |  |  |

| `InteropRefusal` | enum | InteropRefusal { UnsupportedShape, MissingGrounding, SchemaConflict, UngroundedArtifact, FlatClaimOverObjectCentric, DimensionShapeMismatch } |  |  |  |  |

| `KernelRefusal` | enum | KernelRefusal { ZeroSize, TooDense } |  |  |  |  |

| `LifecycleRefusal` | enum | LifecycleRefusal { EmptyLifecycle, InvalidTransition, OrphanEvent } |  |  |  |  |

| `LossFunction` | enum | LossFunction { MeanSquared, CrossEntropy, Hinge } |  |  |  |  |

| `LossPolicy` | enum | LossPolicy { RefuseLoss, AllowLossWithReport, AllowLossSilent, AllowNamedProjection } |  |  |  |  |

| `LossRefusal` | enum | LossRefusal { InvalidParameters, NumericalInstability } |  |  |  |  |

| `OCELAttributeValue` | enum | OCELAttributeValue { Integer(i64), Float(f64), Boolean(bool), String(std::string::String), Time(std::string::String), Null } |  |  |  |  |

| `ObjectCentricity` | enum | ObjectCentricity { CaseCentric, ObjectCentric, Mixed } |  |  |  |  |

| `ObjectLifecyclePhase` | enum | ObjectLifecyclePhase { Created, Active, Modified, Archived, Deleted } |  |  |  |  |

| `ObjectPredicateKind` | enum | ObjectPredicateKind { AttributeEquals, TypeEquals } |  |  |  |  |

| `ObjectTypeCardinality` | enum | ObjectTypeCardinality { One, ZeroOrOne, OneOrMany, ZeroOrMany } |  |  |  |  |

| `OcDeclareRefusal` | enum | OcDeclareRefusal { EmptyObjectTypeList, SynchronizationRequiresMultipleTypes, ScopeMismatch } |  |  |  |  |

| `OcelAttributeValue` | enum | OcelAttributeValue { Integer(i64), Float(f64), Boolean(bool), String(std::string::String), TimestampNs(u64), List(Vec<OcelAttributeValue>), Map(Vec<(std::string::String, OcelAttributeValue)>), Null } |  |  |  |  |

| `OcelRefusal` | enum | OcelRefusal { EmptyEventObjectLinks, DanglingEventObjectLink } |  |  |  |  |

| `OcpqRefusal` | enum | OcpqRefusal { EmptyFilter, UnsupportedScope } |  |  |  |  |

| `OcpqScopeKind` | enum | OcpqScopeKind { Open, Closed, SingleType } |  |  |  |  |

| `PerspectiveRefusal` | enum | PerspectiveRefusal { MissingPerspective, ConflictingPerspectives } |  |  |  |  |

| `PetriNetRefusal` | enum | PetriNetRefusal { IsolatedPlace, InvalidStructure, EmptyNet } |  |  |  |  |

| `PetriRefusal` | enum | PetriRefusal { IsolatedPlace, IsolatedTransition, MissingInitialMarking, InvalidWeight, MissingFinalMarking, UnsafeNet, InvalidInstanceBounds, ObjectTypeNotPreserved, DeadTransition } |  |  |  |  |

| `Pm4pyShape` | enum | Pm4pyShape { EventLog, ObjectCentricLog, PetriNet, ProcessTree, Bpmn, DirectlyFollowsGraph, Declare } |  |  |  |  |

| `Powl8Op` | enum | Powl8Op { NoOp = 0, Sequence = 1, Choice = 2, Parallel = 3, PartialOrder = 4, Loop = 5, Silent = 6, Or = 7, ChoiceGraph = 8 } |  |  |  |  |

| `Powl8OpError` | enum | Powl8OpError { InvalidDiscriminant } |  |  |  |  |

| `PowlNodeKind` | enum | PowlNodeKind { Atom(String), Silent, PartialOrder(Vec<PowlNodeId>), Choice(Vec<PowlNodeId>), Loop { body: PowlNodeId, redo: Option<PowlNodeId> }, ChoiceGraph { nodes: Vec<PowlNodeId>, edges: Vec<(usize, usize)> } } |  |  |  |  |

| `PowlProjectionState` | enum | PowlProjectionState { Unknown, ProcessTreeProjectable, ExceedsProcessTree, RefusedProjection } |  |  |  |  |

| `PowlRefusal` | enum | PowlRefusal { CyclicPartialOrder, InvalidLoop, InvalidChoiceArity { declared: usize, required_min: usize }, ChoiceGraphDisconnected } |  |  |  |  |

| `PredicateKind` | enum | PredicateKind { Event(String), Object(String), Relation(String), Temporal(String), Cardinality { min: usize, max: usize }, Nested, ChildSetBound { branch_label: String, min: usize, max: usize }, E2ORelation { event_var: String, object_var: String, qualifier: Option<String> }, O2ORelation { source_var: String, target_var: String, qualifier: Option<String> }, TimeBetweenEvents { from_var: String, to_var: String } } |  |  |  |  |

| `PredictionHorizon` | enum | PredictionHorizon { FullCase, Events(usize), TimeUnits(u64) } |  |  |  |  |

| `PredictionRefusal` | enum | PredictionRefusal { InsufficientData, InvalidModel, ConvergenceFailure, MissingPrefix, MissingTarget, EmptyPrefix, TargetUnsupported, NonPrefixTrace, ConstraintNotNamed } |  |  |  |  |

| `PredictionTarget` | enum | PredictionTarget { NextActivity, OutcomeLabel, RemainingTime, DriftSignal, Risk, ComplianceConstraint } |  |  |  |  |

| `ProcessPerspective` | enum | ProcessPerspective { ControlFlow, Data, Resource, Time } |  |  |  |  |

| `ProcessShapeKind` | enum | ProcessShapeKind { Event, Trace, EventLog, EventStream, XesLog, OcelLog, DirectlyFollowsGraph, ObjectCentricDfg, PetriNet, WorkflowNet, ObjectCentricPetriNet, ProcessTree, Powl, DeclareModel, ObjectCentricDeclareModel, LogSkeleton, OcpqQuery, Alignment, TokenReplay, ConformanceVerdict, PredictionProblem, Receipt } |  |  |  |  |

| `ProcessTreeNode` | enum | ProcessTreeNode { Activity(String), Operator { operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId> } } |  |  |  |  |

| `ProcessTreeOperator` | enum | ProcessTreeOperator { Sequence, Xor, Parallel, Or, Loop, Silent } |  |  |  |  |

| `ProcessTreeRefusal` | enum | ProcessTreeRefusal { EmptyTree, OrphanNode, InvalidLoopArity, UnsupportedOperator, MissingRoot, DanglingNodeReference, TauLeafWithChildren, BelowMinimumArity, InvalidArity, CycleDetected } |  |  |  |  |

| `QualityDimension` | enum | QualityDimension { Fitness, Precision, F1, Generalization, Simplicity } |  |  |  |  |

| `QualityMetricKind` | enum | QualityMetricKind { Fitness, Precision, F1, Generalization, Simplicity } |  |  |  |  |

| `ReceiptRefusal` | enum | ReceiptRefusal { EmptyChain, HashMismatch, InvalidDigest, MissingSubject, MissingWitness, MissingDigest, MissingReplayHint, BrokenChainLink(usize) } |  |  |  |  |

| `ReceiptVerdict` | enum | ReceiptVerdict { Admitted, Refused(ReceiptRefusal) } |  |  |  |  |

| `RelationLaw` | enum | RelationLaw { EventToObject, ObjectToObject, ObjectToEvent } |  |  |  |  |

| `RelationPredicateKind` | enum | RelationPredicateKind { E2O, O2O, TimeBetweenEvents } |  |  |  |  |

| `ReplayHintKind` | enum | ReplayHintKind { FromStart, FromSeq(u64), Latest } |  |  |  |  |

| `SoundnessState` | enum | SoundnessState { Unknown, Claimed, Witnessed } |  |  |  |  |

| `StandaloneChoiceGraphNode` | enum | StandaloneChoiceGraphNode { Start, End, Activity(String), SubModel(usize) } |  |  |  |  |

| `SummaryShape` | enum | SummaryShape { Counts, TraceVariants, ActivityDistribution, TimingProfile, ObjectTypeDistribution } |  |  |  |  |

| `TemporalOrder` | enum | TemporalOrder { Before, After, Concurrent, Unknown } |  |  |  |  |

| `TemporalRefusal` | enum | TemporalRefusal { ZeroBound, ConflictingConstraints } |  |  |  |  |

| `TemporalRelation` | enum | TemporalRelation { Before, After, During, Concurrent } |  |  |  |  |

| `WitnessFamily` | enum | WitnessFamily { Standard, ApiGrammar, Implementation, Paper } |  |  |  |  |

| `WorkflowPattern` | enum | WorkflowPattern { Sequence, ParallelSplit, Synchronization, ExclusiveChoice, SimpleMerge, MultiChoice, StructuredSynchronizingMerge, MultiMerge, StructuredDiscriminator, ArbitraryCycles, ImplicitTermination, MultipleInstancesWithoutSync, MultipleInstancesWithDesignTimeKnowledge, DeferredChoice, InterleavedParallelRouting, CancelActivity, CancelCase } |  |  |  |  |

| `activity` | function | activity(&self) -> &str |  |  |  |  |

| `add_arc` | function | add_arc(&mut self, a: Arc) |  |  |  |  |

| `add_place` | function | add_place(&mut self, p: Place) |  |  |  |  |

| `add_transition` | function | add_transition(&mut self, t: Transition) |  |  |  |  |

| `admit_flat` | function | admit_flat(&self) -> Result<(), InteropRefusal> |  |  |  |  |

| `admit_shape` | function | admit_shape(&self) -> Result<(), ProcessTreeRefusal> |  |  |  |  |

| `admits` | function | admits(&self, count: usize) -> bool |  |  |  |  |

| `arcs` | function | arcs(&self) -> &[Arc] |  |  |  |  |

| `arity` | function | arity(&self) -> u8 |  |  |  |  |

| `as_f64` | function | as_f64(&self) -> f64 |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `assert_epsilon_close` | function | assert_epsilon_close(actual: f64, expected: f64) |  |  |  |  |

| `at_ns` | function | at_ns(mut self, ts: u64) -> Self |  |  |  |  |

| `attribute` | function | attribute(&self) -> &str |  |  |  |  |

| `attributes` | function | attributes(&self) -> &[OcelAttribute] |  |  |  |  |

| `binary` | function | binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) -> Self |  |  |  |  |

| `boolean` | function | boolean(key: &str, value: bool) -> Self |  |  |  |  |

| `by` | function | by(mut self, resource: &str) -> Self |  |  |  |  |

| `case_id` | function | case_id(&self) -> &str |  |  |  |  |

| `category` | function | category(&self) -> &str |  |  |  |  |

| `check_filter_shape` | function | check_filter_shape(shape: Pm4pyShape, filter: FilterShape) -> Result<(), InteropRefusal> |  |  |  |  |

| `claim_sound` | function | claim_sound(self) -> WfNet<SoundnessClaimed> |  |  |  |  |

| `conformance_rate` | function | conformance_rate(&self) -> f64 |  |  |  |  |

| `count` | function | count(&self) -> usize |  |  |  |  |

| `count_objects_of_type` | function | count_objects_of_type(&self, t: &str) -> usize |  |  |  |  |

| `declare_separable` | function | declare_separable(net: WfNetConst<S>) -> Self |  |  |  |  |

| `default` | function | default() -> Self |  |  |  |  |

| `den` | function | den(&self) -> u64 |  |  |  |  |

| `direction` | function | direction(&self) -> ArcDirection |  |  |  |  |

| `duration_ns` | function | duration_ns(&self) -> Option<u64> |  |  |  |  |

| `e2o` | function | e2o(&self, event_id: &str) -> Vec<(&str, &str)> |  |  |  |  |

| `edges` | function | edges(&self) -> &[BpmnEdge] |  |  |  |  |

| `empty` | function | empty() -> Self |  |  |  |  |

| `eval` | function | eval(&self, event_id: &str) -> Option<HashMap<std::string::String, OCELAttributeValue>> |  |  |  |  |

| `event` | function | event(id: &str, e: BpmnEvent) -> Self |  |  |  |  |

| `event_count` | function | event_count(&self) -> usize |  |  |  |  |

| `event_id` | function | event_id(&self) -> &str |  |  |  |  |

| `event_object_links` | function | event_object_links(&self) -> &[EventObjectLink] |  |  |  |  |

| `event_set` | function | event_set(&self) -> &Vec<OCELEvent> |  |  |  |  |

| `events` | function | events(&self) -> &[OcelEvent] |  |  |  |  |

| `expression` | function | expression(&self) -> &str |  |  |  |  |

| `extend_with` | function | extend_with(&mut self, env: ReceiptEnvelope) -> Result<(), ReceiptRefusal> |  |  |  |  |

| `final_marking` | function | final_marking(&self) -> Option<&Marking> |  |  |  |  |

| `float` | function | float(key: &str, value: f64) -> Self |  |  |  |  |

| `fnv1a_64` | function | fnv1a_64(bytes: &[u8]) -> u64 |  |  |  |  |

| `frequency` | function | frequency(&self) -> DfgWeight |  |  |  |  |

| `from_events` | function | from_events(events: impl IntoIterator<Item = Event>) -> Self |  |  |  |  |

| `from_owned` | function | from_owned(s: String) -> Self |  |  |  |  |

| `from_static` | function | from_static(s: &'static str) -> Self |  |  |  |  |

| `from_traces` | function | from_traces(traces: impl IntoIterator<Item = Trace>) -> Self |  |  |  |  |

| `gateway` | function | gateway(id: &str, g: BpmnGateway) -> Self |  |  |  |  |

| `get` | function | get(&self) -> f64 |  |  |  |  |

| `get_by_hash` | function | get_by_hash(&self, hash: u64) -> Option<&V> |  |  |  |  |

| `get_by_key` | function | get_by_key(&self, key: &str) -> Option<&V> |  |  |  |  |

| `id` | function | id(&self) -> &str |  |  |  |  |

| `initial_marking` | function | initial_marking(&self) -> &Marking |  |  |  |  |

| `inner` | function | inner(&self) -> &T |  |  |  |  |

| `insert` | function | insert(&mut self, hash: u64, key: String, value: V) |  |  |  |  |

| `integer` | function | integer(key: &str, value: i64) -> Self |  |  |  |  |

| `internal` | function | internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) -> Self |  |  |  |  |

| `into_admitted` | function | into_admitted(self) -> Evidence<T, state::Admitted, C> |  |  |  |  |

| `into_evidence` | function | into_evidence(self) -> Evidence<T, state::Admitted, W> |  |  |  |  |

| `into_exportable` | function | into_exportable(self) -> Evidence<T, state::Exportable, C> |  |  |  |  |

| `into_inner` | function | into_inner(self) -> T |  |  |  |  |

| `into_lost` | function | into_lost(self) -> Dropped |  |  |  |  |

| `into_parsed` | function | into_parsed(self) -> Evidence<T, state::Parsed, C> |  |  |  |  |

| `into_projected` | function | into_projected(self) -> Evidence<T, state::Projected, C> |  |  |  |  |

| `into_reason` | function | into_reason(self) -> R |  |  |  |  |

| `into_receipted` | function | into_receipted(self) -> Evidence<T, state::Receipted, C> |  |  |  |  |

| `is_chain` | function | is_chain(&self) -> bool |  |  |  |  |

| `is_consistent` | function | is_consistent(&self) -> bool |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_grounded` | function | is_grounded(&self) -> bool |  |  |  |  |

| `is_lossless` | function | is_lossless(&self) -> bool |  |  |  |  |

| `is_negative` | function | is_negative(&self) -> bool |  |  |  |  |

| `is_object_centric` | function | is_object_centric(&self) -> bool |  |  |  |  |

| `is_perfect` | function | is_perfect(&self) -> bool |  |  |  |  |

| `is_silent` | function | is_silent(&self) -> bool |  |  |  |  |

| `is_synchronized` | function | is_synchronized(&self) -> bool |  |  |  |  |

| `is_well_shaped` | function | is_well_shaped(&self) -> bool |  |  |  |  |

| `iter` | function | iter(&self) -> impl Iterator<Item = &ReceiptEnvelope> |  |  |  |  |

| `kind` | function | kind(&self) -> QualityMetricKind |  |  |  |  |

| `label` | function | label(&self) -> &str |  |  |  |  |

| `lanes` | function | lanes(&self) -> &[BpmnLane] |  |  |  |  |

| `leaf` | function | leaf(id: ProcessTreeNodeId, label: &str) -> Self |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `length` | function | length(&self) -> usize |  |  |  |  |

| `lifecycle` | function | lifecycle(&self) -> Option<&str> |  |  |  |  |

| `mark` | function | mark(&mut self, place_id: &str, tokens: usize) |  |  |  |  |

| `max` | function | max(&self) -> usize |  |  |  |  |

| `members` | function | members(&self) -> &[String] |  |  |  |  |

| `min` | function | min(&self) -> usize |  |  |  |  |

| `name` | function | name(&self) -> &str |  |  |  |  |

| `net` | function | net(&self) -> &PetriNet |  |  |  |  |

| `new` | function | new(value: T) -> Self |  |  |  |  |

| `node_count` | function | node_count(&self) -> usize |  |  |  |  |

| `node_ids` | function | node_ids(&self) -> &[String] |  |  |  |  |

| `nodes` | function | nodes(&self) -> &[BpmnNode] |  |  |  |  |

| `num` | function | num(&self) -> u64 |  |  |  |  |

| `o2o` | function | o2o(&self, object_id: &str) -> Vec<(&str, &str)> |  |  |  |  |

| `object_changes` | function | object_changes(&self) -> &[ObjectChange] |  |  |  |  |

| `object_id` | function | object_id(&self) -> &str |  |  |  |  |

| `object_object_links` | function | object_object_links(&self) -> &[ObjectObjectLink] |  |  |  |  |

| `object_set` | function | object_set(&self) -> &Vec<OCELObject> |  |  |  |  |

| `object_type` | function | object_type(&self) -> &str |  |  |  |  |

| `object_types` | function | object_types(&self) -> impl Iterator<Item = &str> |  |  |  |  |

| `objects` | function | objects(&self) -> &[Object] |  |  |  |  |

| `place_id` | function | place_id(&self) -> &str |  |  |  |  |

| `place_to_transition` | function | place_to_transition(place: &str, transition: &str) -> Self |  |  |  |  |

| `places` | function | places(&self) -> &[Place] |  |  |  |  |

| `predecessors` | function | predecessors(&self, idx: usize) -> Vec<usize> |  |  |  |  |

| `process` | function | process(&self) -> &BpmnProcess |  |  |  |  |

| `projection` | function | projection(&self) -> ProjectionNameOwned |  |  |  |  |

| `push` | function | push(&mut self, event: Event) |  |  |  |  |

| `qualified` | function | qualified(mut self, q: &str) -> Self |  |  |  |  |

| `qualifier` | function | qualifier(&self) -> Option<&str> |  |  |  |  |

| `raw` | function | raw(value: T) -> Self |  |  |  |  |

| `resource` | function | resource(&self) -> Option<&str> |  |  |  |  |

| `root` | function | root(&self) -> Option<ProcessTreeNodeId> |  |  |  |  |

| `schema` | function | schema(&self) -> &'static str |  |  |  |  |

| `scope` | function | scope(&self) -> &ObjectScopeConst |  |  |  |  |

| `silent` | function | silent(id: &str) -> Self |  |  |  |  |

| `soundness_state` | function | soundness_state(&self) -> SoundnessState |  |  |  |  |

| `source` | function | source(&self) -> &str |  |  |  |  |

| `source_id` | function | source_id(&self) -> &str |  |  |  |  |

| `steps` | function | steps(&self) -> &[NamedLoss] |  |  |  |  |

| `string` | function | string(key: &str, value: &str) -> Self |  |  |  |  |

| `successors` | function | successors(&self, idx: usize) -> Vec<usize> |  |  |  |  |

| `summary` | function | summary(&self, category: &str) -> NamedLoss |  |  |  |  |

| `synchronized` | function | synchronized(constraint: DeclareConstraint, object_types: Vec<String>) -> Self |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `target` | function | target(&self) -> &str |  |  |  |  |

| `target_id` | function | target_id(&self) -> &str |  |  |  |  |

| `task` | function | task(id: &str, t: BpmnTask) -> Self |  |  |  |  |

| `timestamp_ns` | function | timestamp_ns(&self) -> Option<u64> |  |  |  |  |

| `tip` | function | tip(&self) -> &ReceiptEnvelope |  |  |  |  |

| `tokens` | function | tokens(&self) -> &HashMap<String, usize> |  |  |  |  |

| `tokens_on` | function | tokens_on(&self, place_id: &str) -> usize |  |  |  |  |

| `trace_count` | function | trace_count(&self) -> usize |  |  |  |  |

| `traces` | function | traces(&self) -> &[Trace] |  |  |  |  |

| `transition_id` | function | transition_id(&self) -> &str |  |  |  |  |

| `transition_to_place` | function | transition_to_place(transition: &str, place: &str) -> Self |  |  |  |  |

| `transitions` | function | transitions(&self) -> &[Transition] |  |  |  |  |

| `try_from_parts` | function | try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) -> Result<Self, ReceiptRefusal> |  |  |  |  |

| `try_new` | function | try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) -> Result<Self, ReceiptRefusal> |  |  |  |  |

| `unary` | function | unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) -> Self |  |  |  |  |

| `validate` | function | validate(&self) -> Result<(), OcelRefusal> |  |  |  |  |

| `value` | function | value(&self) -> &str |  |  |  |  |

| `verdict` | function | verdict(&self) -> CausalConsistency |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), ReceiptRefusal> |  |  |  |  |

| `weight` | function | weight(&self) -> W |  |  |  |  |

| `with_attribute` | function | with_attribute(mut self, a: OcelAttribute) -> Self |  |  |  |  |

| `with_duration_ns` | function | with_duration_ns(mut self, d: u64) -> Self |  |  |  |  |

| `with_generalization` | function | with_generalization(mut self, v: f64) -> Self |  |  |  |  |

| `with_lifecycle` | function | with_lifecycle(mut self, lc: &str) -> Self |  |  |  |  |

| `with_precision` | function | with_precision(mut self, v: f64) -> Self |  |  |  |  |

| `with_simplicity` | function | with_simplicity(mut self, v: f64) -> Self |  |  |  |  |

| `with_type_dfg` | function | with_type_dfg(mut self, obj_type: &str, dfg: Dfg) -> Self |  |  |  |  |

| `witness_soundness` | function | witness_soundness(self, _proof: SoundnessProof) -> WfNetConst<{ SoundnessState::Witnessed }> |  |  |  |  |

| `Activity` | struct | Activity { pub String } |  |  |  |  |

| `Admission` | struct | Admission { pub value: T, _witness: PhantomData<W> } |  |  |  |  |

| `Admitted` | struct | Admitted |  |  |  |  |

| `Arc` | struct | Arc { pub source: String, pub target: String, pub weight: u32, pub object_type: Option<(String, bool)>, _dir: ArcDirection } |  |  |  |  |

| `ArtifactGrounding` | struct | ArtifactGrounding { pub shape: Pm4pyShape, pub evidence_ref: String, _evidence: PhantomData<E> } |  |  |  |  |

| `Between01` | struct | Between01 { _priv: () } |  |  |  |  |

| `BipartiteArcConst` | struct | BipartiteArcConst { _place_id: String, _transition_id: String, _weight: W } |  |  |  |  |

| `BpmnEdge` | struct | BpmnEdge { source: String, target: String } |  |  |  |  |

| `BpmnLane` | struct | BpmnLane { id: String, name: String, node_ids: Vec<String> } |  |  |  |  |

| `BpmnNode` | struct | BpmnNode { id: String, kind: BpmnNodeKind } |  |  |  |  |

| `BpmnPool` | struct | BpmnPool { id: String, name: String, process: BpmnProcess, lanes: Vec<BpmnLane> } |  |  |  |  |

| `BpmnProcess` | struct | BpmnProcess { nodes: Vec<BpmnNode>, edges: Vec<BpmnEdge> } |  |  |  |  |

| `BpmnTask` | struct | BpmnTask { pub label: String } |  |  |  |  |

| `CancellationRegion` | struct | CancellationRegion { _members: Vec<String> } |  |  |  |  |

| `CausalBinding` | struct | CausalBinding { pub source_tasks: Vec<String>, pub target_tasks: Vec<String> } |  |  |  |  |

| `CausalChain` | struct |  |  |  |  |  |

| `CausalLink` | struct | CausalLink { PhantomData<(From, To)> } |  |  |  |  |

| `CausalNet` | struct | CausalNet { pub nodes: Vec<String>, pub dependency_measures: Vec<(String, String, f64)>, pub inputs: Vec<CausalBinding>, pub outputs: Vec<CausalBinding> } |  |  |  |  |

| `CausallyOrderedEvidence` | struct | CausallyOrderedEvidence { pub inner: T } |  |  |  |  |

| `ChoiceGraph` | struct | ChoiceGraph { nodes: Vec<StandaloneChoiceGraphNode>, edges: Vec<(usize, usize)> } |  |  |  |  |

| `ConditionCell` | struct | ConditionCell { _priv: () } |  |  |  |  |

| `ConformanceResult` | struct | ConformanceResult { pub fitness: f64, pub precision: Option<f64>, pub generalization: Option<f64>, pub simplicity: Option<f64>, pub total_traces: usize, pub fitting_traces: usize, pub deviating_traces: usize } |  |  |  |  |

| `ConformanceVerdict` | struct | ConformanceVerdict { pub fitness: Option<Fitness>, pub deviations: Vec<Deviation> } |  |  |  |  |

| `ConsistencyVerified` | struct | ConsistencyVerified { pub inner: T, verdict: CausalConsistency } |  |  |  |  |

| `ControlFlowPerspective` | struct |  |  |  |  |  |

| `CorrelatedLog` | struct | CorrelatedLog { PhantomData<(A, B)> } |  |  |  |  |

| `CorrelationKey` | struct |  |  |  |  |  |

| `DataPerspective` | struct |  |  |  |  |  |

| `DeclareConstraint` | struct | DeclareConstraint { pub template: DeclareTemplate, pub activation: Activity, pub target: Option<Activity>, pub scope: DeclareScope } |  |  |  |  |

| `DenseKernel` | struct | DenseKernel { pub size: usize } |  |  |  |  |

| `DependencyMeasure` | struct | DependencyMeasure { pub f64 } |  |  |  |  |

| `Deviation` | struct | Deviation { pub position: usize, pub label: String } |  |  |  |  |

| `Dfg` | struct | Dfg { nodes: Vec<DfgNode>, edges: Vec<DfgEdge> } |  |  |  |  |

| `DfgEdge` | struct | DfgEdge { source: String, target: String, count: u64 } |  |  |  |  |

| `DfgEdgeFull` | struct | DfgEdgeFull { pub source: String, pub target: String, _freq: u64, _dur: Option<u64> } |  |  |  |  |

| `DfgNode` | struct | DfgNode { pub String } |  |  |  |  |

| `DfgWeight` | struct | DfgWeight { pub u64 } |  |  |  |  |

| `DiagnosticReport` | struct | DiagnosticReport { pub kind: DiagnosticKind, pub message: String, pub stage: String } |  |  |  |  |

| `Digest` | struct | Digest { pub String } |  |  |  |  |

| `Event` | struct | Event { pub activity: String, pub attributes: HashMap<String, String>, _ts: Option<u64>, _resource: Option<String>, _lifecycle: Option<String> } |  |  |  |  |

| `EventLog` | struct | EventLog { pub traces: Vec<Trace> } |  |  |  |  |

| `EventObjectLink` | struct | EventObjectLink { pub event_id: String, pub object_id: String, _qualifier: Option<String> } |  |  |  |  |

| `EventStream` | struct | EventStream { events: Vec<Event> } |  |  |  |  |

| `EventTypeName` | struct | EventTypeName { inner: Cow<'static, str>, _kind: PhantomData<K> } |  |  |  |  |

| `Evidence` | struct | Evidence { pub value: T, _state: PhantomData<S>, _chain: PhantomData<C> } |  |  |  |  |

| `Exportable` | struct | Exportable |  |  |  |  |

| `F1` | struct | F1 { pub f64 } |  |  |  |  |

| `Fitness` | struct | Fitness { pub f64 } |  |  |  |  |

| `Generalization` | struct | Generalization { pub f64 } |  |  |  |  |

| `InitialFinalMarkingPair` | struct | InitialFinalMarkingPair { _initial: Marking, _final: Marking } |  |  |  |  |

| `InputBinding` | struct | InputBinding { pub &'a str, pub &'a str } |  |  |  |  |

| `LifecycleEvent` | struct | LifecycleEvent { pub event_type: String, pub object_id: String } |  |  |  |  |

| `LossChain` | struct | LossChain { steps: Vec<NamedLoss> } |  |  |  |  |

| `LossReport` | struct | LossReport { pub projection: ProjectionName, pub policy: LossPolicy, _dropped: Dropped, _ph: PhantomData<(A, B)> } |  |  |  |  |

| `Marking` | struct | Marking { pub HashMap<String, usize> } |  |  |  |  |

| `Metric` | struct | Metric { _priv: () } |  |  |  |  |

| `MultiPerspectiveEvidence` | struct | MultiPerspectiveEvidence { pub inner: T, _perspective: PhantomData<P> } |  |  |  |  |

| `MultiPerspectiveLog` | struct | MultiPerspectiveLog { pub traces: Vec<String> } |  |  |  |  |

| `MultipleInstanceSpec` | struct | MultipleInstanceSpec { pub min: usize, pub max: Option<usize>, pub threshold: Option<usize>, pub creation: InstanceCreationKind } |  |  |  |  |

| `MultipleInstanceSpecConst` | struct | MultipleInstanceSpecConst { _priv: () } |  |  |  |  |

| `NamedLoss` | struct | NamedLoss { pub projection_str: String, pub _category: String } |  |  |  |  |

| `OCEL` | struct | OCEL { pub events: Vec<OCELEvent>, pub objects: Vec<OCELObject> } |  |  |  |  |

| `OCELEvent` | struct | OCELEvent { pub id: std::string::String, pub event_type: std::string::String, pub relationships: Vec<OCELRelationship>, pub attributes: Vec<OCELEventAttribute> } |  |  |  |  |

| `OCELEventAttribute` | struct | OCELEventAttribute { pub name: std::string::String, pub value: OCELAttributeValue } |  |  |  |  |

| `OCELObject` | struct | OCELObject { pub id: std::string::String, pub object_type: std::string::String, pub attributes: Vec<OCELEventAttribute>, pub relationships: Vec<OCELRelationship> } |  |  |  |  |

| `OCELRelationship` | struct | OCELRelationship { pub object_id: std::string::String, pub qualifier: std::string::String } |  |  |  |  |

| `OCELType` | struct | OCELType { pub name: std::string::String, pub attributes: Vec<OCELTypeAttribute> } |  |  |  |  |

| `OCELTypeAttribute` | struct | OCELTypeAttribute { pub name: std::string::String, pub value_type: std::string::String } |  |  |  |  |

| `Object` | struct | Object { pub id: String, pub obj_type: String, _attrs: Vec<OcelAttribute> } |  |  |  |  |

| `ObjectCentricDfg` | struct | ObjectCentricDfg { HashMap<String, Dfg> } |  |  |  |  |

| `ObjectCentricPetriNet` | struct | ObjectCentricPetriNet { _net: PetriNet, _object_types: Vec<String> } |  |  |  |  |

| `ObjectChange` | struct | ObjectChange { _object_id: String, _attribute: String, _value: String, _ts: Option<u64> } |  |  |  |  |

| `ObjectLifecycle` | struct | ObjectLifecycle { pub events: Vec<LifecycleEvent> } |  |  |  |  |

| `ObjectObjectLink` | struct | ObjectObjectLink { pub source_id: String, pub target_id: String, _qualifier: Option<String> } |  |  |  |  |

| `ObjectScope` | struct | ObjectScope { pub object_types: Vec<String> } |  |  |  |  |

| `ObjectScopeConst` | struct | ObjectScopeConst { pub object_types: Vec<String> } |  |  |  |  |

| `ObjectTypeCardinality` | struct | ObjectTypeCardinality { pub min_count: Option<usize>, pub max_count: Option<usize>, pub created_by: Vec<std::string::String>, pub terminated_by: Vec<std::string::String> } |  |  |  |  |

| `ObjectTypeName` | struct | ObjectTypeName { inner: Cow<'static, str>, _kind: PhantomData<K> } |  |  |  |  |

| `OcDeclareConstraint` | struct | OcDeclareConstraint { pub constraint: DeclareConstraint, pub object_types: Vec<String>, _synchronized: bool } |  |  |  |  |

| `OcelAttribute` | struct | OcelAttribute { pub key: std::string::String, pub value: OcelAttributeValue } |  |  |  |  |

| `OcelEvent` | struct | OcelEvent { pub id: String, pub event_type: String, _ts: Option<u64>, _attrs: Vec<OcelAttribute> } |  |  |  |  |

| `OcelLog` | struct | OcelLog { objects: Vec<Object>, events: Vec<OcelEvent>, event_object_links: Vec<EventObjectLink>, object_object_links: Vec<ObjectObjectLink>, object_changes: Vec<ObjectChange> } |  |  |  |  |

| `OcpqQuery` | struct | OcpqQuery { pub scope: ObjectScope, pub predicates: Vec<Predicate>, pub sub_queries: Vec<OcpqQuery> } |  |  |  |  |

| `OcpqQueryConst` | struct | OcpqQueryConst { _scope: ObjectScopeConst } |  |  |  |  |

| `OrderEdge` | struct | OrderEdge { pub from: PowlNodeId, pub to: PowlNodeId } |  |  |  |  |

| `OutputBinding` | struct | OutputBinding { pub &'a str, pub &'a str } |  |  |  |  |

| `PackedKeyTable` | struct | PackedKeyTable { data: HashMap<u64, (String, V)> } |  |  |  |  |

| `ParityComparer` | struct |  |  |  |  |  |

| `Parsed` | struct | Parsed |  |  |  |  |

| `PerspectiveCombination` | struct | PerspectiveCombination { PhantomData<(A, B)> } |  |  |  |  |

| `PetriNet` | struct | PetriNet { pub places: Vec<Place>, pub transitions: Vec<Transition>, pub arcs: Vec<Arc>, pub initial_marking: Marking, pub final_marking: Marking } |  |  |  |  |

| `Place` | struct | Place { pub id: String } |  |  |  |  |

| `PlaceNodeMarker` | struct |  |  |  |  |  |

| `PlaceToTransitionArc` | struct | PlaceToTransitionArc { W, PhantomData<(P, T)> } |  |  |  |  |

| `Powl` | struct | Powl { pub nodes: Vec<PowlNode>, pub edges: Vec<OrderEdge>, pub root: Option<PowlNodeId> } |  |  |  |  |

| `PowlChoiceNode` | struct | PowlChoiceNode { branches: Vec<PowlNodeId> } |  |  |  |  |

| `PowlComposition` | struct | PowlComposition { pub inner: Inner } |  |  |  |  |

| `PowlNode` | struct | PowlNode { pub id: PowlNodeId, pub kind: PowlNodeKind } |  |  |  |  |

| `PowlNodeId` | struct | PowlNodeId { pub u64 } |  |  |  |  |

| `Precision` | struct | Precision { pub f64 } |  |  |  |  |

| `Predicate` | struct | Predicate { pub kind: PredicateKind, _data: std::marker::PhantomData<T> } |  |  |  |  |

| `ProcessCube` | struct | ProcessCube { pub dimensions: Vec<CubeDimensionKind> } |  |  |  |  |

| `ProcessSlice` | struct | ProcessSlice { pub dimension: CubeDimensionKind, pub value: String } |  |  |  |  |

| `ProcessTree` | struct | ProcessTree { pub nodes: Vec<ProcessTreeNode>, pub root: Option<ProcessTreeNodeId> } |  |  |  |  |

| `ProcessTreeNodeId` | struct | ProcessTreeNodeId { pub usize } |  |  |  |  |

| `Projected` | struct | Projected |  |  |  |  |

| `ProjectionName` | struct | ProjectionName { pub &'static str } |  |  |  |  |

| `ProjectionNameOwned` | struct | ProjectionNameOwned { pub String } |  |  |  |  |

| `QualityProfile` | struct | QualityProfile { pub fitness: Between01<FN, FD>, pub precision: Between01<PN, PD>, pub f1: Between01<F1N, F1D>, pub generalization: Between01<GN, GD>, pub simplicity: Between01<SN, SD> } |  |  |  |  |

| `Raw` | struct | Raw |  |  |  |  |

| `ReceiptChain` | struct | ReceiptChain { pub run_id: String, pub envelopes: Vec<ReceiptEnvelope> } |  |  |  |  |

| `ReceiptChainConst` | struct | ReceiptChainConst { pub run_id: String, pub links: Vec<ReceiptEnvelope> } |  |  |  |  |

| `ReceiptEnvelope` | struct | ReceiptEnvelope { pub subject: String, pub witness: String, pub digest: Digest, pub replay_hint: ReplayHint } |  |  |  |  |

| `Receipted` | struct | Receipted |  |  |  |  |

| `Refusal` | struct | Refusal { pub reason: R, _witness: PhantomData<W> } |  |  |  |  |

| `Refused` | struct | Refused |  |  |  |  |

| `ReplayHint` | struct | ReplayHint { pub String } |  |  |  |  |

| `Require` | struct |  |  |  |  |  |

| `ResourcePerspective` | struct |  |  |  |  |  |

| `RuntimeMarking` | struct | RuntimeMarking { table: &'a PackedKeyTable<usize> } |  |  |  |  |

| `SeparableWfNet` | struct | SeparableWfNet { pub net: WfNetConst<S> } |  |  |  |  |

| `Simplicity` | struct | Simplicity { pub f64 } |  |  |  |  |

| `SoundnessClaimed` | struct |  |  |  |  |  |

| `SoundnessProof` | struct | SoundnessProof |  |  |  |  |

| `SoundnessUnknown` | struct |  |  |  |  |  |

| `StableId` | struct | StableId { pub String } |  |  |  |  |

| `TemporalConstraint` | struct | TemporalConstraint { pub relation: TemporalRelation, pub bound_ms: u64 } |  |  |  |  |

| `TimePerspective` | struct |  |  |  |  |  |

| `Trace` | struct | Trace { pub events: Vec<Event>, pub case_id: String } |  |  |  |  |

| `Transition` | struct | Transition { pub id: String, pub label: String } |  |  |  |  |

| `TransitionNodeMarker` | struct |  |  |  |  |  |

| `TransitionToPlaceArc` | struct | TransitionToPlaceArc { W, PhantomData<(T, P)> } |  |  |  |  |

| `TypedEventPredicate` | struct | TypedEventPredicate { _expr: String } |  |  |  |  |

| `TypedId` | struct | TypedId { pub id: String, _type: PhantomData<T> } |  |  |  |  |

| `TypedLoopNode` | struct | TypedLoopNode { pub children: Children } |  |  |  |  |

| `TypedObjectPredicate` | struct | TypedObjectPredicate { _expr: String } |  |  |  |  |

| `TypedPowlLoopNode` | struct | TypedPowlLoopNode { pub children: Children } |  |  |  |  |

| `TypedRelationPredicate` | struct | TypedRelationPredicate { _expr: String } |  |  |  |  |

| `UnknownVerifier` | struct |  |  |  |  |  |

| `WfNet` | struct | WfNet { _net: PetriNet, _final: Marking, _state: PhantomData<S> } |  |  |  |  |

| `WfNetConst` | struct | WfNetConst { _priv: () } |  |  |  |  |

| `Witnessed` | struct | Witnessed |  |  |  |  |

| `IsTrue` | trait |  |  |  |  |  |

| `IsValidArc` | trait |  |  |  |  |  |

| `VerifyCausalConsistency` | trait |  |  |  |  |  |

| `Witness` | trait |  |  |  |  |  |

| `crate::models::PackedKeyTable` | use | crate::models::PackedKeyTable |  |  |  |  |

| `MAX_POWL_DEPTH` | const | MAX_POWL_DEPTH: usize |  |  |  |  |

| `ArcDirection` | enum | ArcDirection { PlaceToTransition, TransitionToPlace } |  |  |  |  |

| `ArcDirectionConst` | enum | ArcDirectionConst { PlaceToTransition, TransitionToPlace } |  |  |  |  |

| `BpmnEvent` | enum | BpmnEvent { Start, Intermediate, End, Boundary } |  |  |  |  |

| `BpmnGateway` | enum | BpmnGateway { Exclusive, Parallel, Inclusive, EventBased, Complex } |  |  |  |  |

| `BpmnNodeKind` | enum | BpmnNodeKind { Event(BpmnEvent), Task(BpmnTask), Gateway(BpmnGateway) } |  |  |  |  |

| `BpmnRefusal` | enum | BpmnRefusal { DanglingEdge, MissingStart, MissingStartEvent, MissingEndEvent, EmptyProcess, DuplicateNodeId, LaneNodeNotDeclared } |  |  |  |  |

| `CausalConsistency` | enum | CausalConsistency { Unknown, Consistent, HasCycles, HasContradictions } |  |  |  |  |

| `CausalNetRefusal` | enum | CausalNetRefusal { MissingActivity, InvalidDependencyScore, DisconnectedGraph } |  |  |  |  |

| `CompatDiagnostic` | enum | CompatDiagnostic { MissingWitness, MissingRoundTripFixture, RawEvidenceExportedAsAdmitted, LossyProjectionWithoutPolicy, HiddenFlattening, MissingRefusalPath, MissingReceiptShape, UnreachablePrimitive, MigrationRecommended } |  |  |  |  |

| `ComplianceKind` | enum | ComplianceKind { Monitoring, Audit, Certification } |  |  |  |  |

| `ConformanceRefusal` | enum | ConformanceRefusal { EmptyModel, TooManyDeviations, InvalidProfile, MissingLog, MissingModel, MissingDeviationPath, FitnessUnavailable, PrecisionUnavailable, F1Unavailable, GeneralizationUnavailable, SimplicityUnavailable } |  |  |  |  |

| `ConformanceVerdict` | enum | ConformanceVerdict { PerfectAlignment, FitnessDeficit, DeadlockEncountered } |  |  |  |  |

| `CorrelationSchema` | enum | CorrelationSchema { ByCase, ByObject, ByTimestamp, ByAttribute } |  |  |  |  |

| `CubeDimensionKind` | enum | CubeDimensionKind { Activity, Resource, Time, DataAttribute, ObjectType, CaseAttribute } |  |  |  |  |

| `DeclareRefusal` | enum | DeclareRefusal { EmptyActivity, MissingActivation, BinaryRequiresTarget, UnsupportedTemplate, MissingTarget, InvalidTemplateArity, EmptyObjectScope, SynchronizationViolation } |  |  |  |  |

| `DeclareScope` | enum | DeclareScope { SingleObjectScope(String), MultiObjectScope(Vec<String>), SynchronizedObjectScope(Vec<String>), CrossObjectScope(String, String), GlobalScope } |  |  |  |  |

| `DeclareTemplate` | enum | DeclareTemplate { Existence, Absence, Init, Existence2, Existence3, Absence2, Absence3, RespondedExistence, CoExistence, Response, Precedence, Succession, AlternateResponse, AlternatePrecedence, AlternateSuccession, ChainResponse, ChainPrecedence, ChainSuccession, NotSuccession, NotChainSuccession, NotCoExistence, ExclusiveChoice } |  |  |  |  |

| `DfgRefusal` | enum | DfgRefusal { EmptyGraph, DanglingEdge, IsolatedNode } |  |  |  |  |

| `DiagnosticKind` | enum | DiagnosticKind { Warning, Error, Info } |  |  |  |  |

| `DiagnosticSeverity` | enum | DiagnosticSeverity { Error, Warning, Info } |  |  |  |  |

| `EventLogRefusal` | enum | EventLogRefusal { EmptyLog, EmptyTrace, MissingActivity, NonMonotonicTrace } |  |  |  |  |

| `EventPredicateKind` | enum | EventPredicateKind { ActivityEquals, AttributeEquals, TimestampInRange } |  |  |  |  |

| `EvidenceMode` | enum | EvidenceMode { Raw, Parsed, Admitted, Refused, Projected, Exportable, Witnessed, Receipted } |  |  |  |  |

| `FilterShape` | enum | FilterShape { Activity, Timeframe, Variant, Attribute, ObjectType } |  |  |  |  |

| `FormatKind` | enum | FormatKind { OcelJson, OcelXml, OcelSqlite, XesXml, BpmnXml, PetriPnml, PowlJson } |  |  |  |  |

| `InstanceCreationKind` | enum | InstanceCreationKind { Static, Dynamic } |  |  |  |  |

| `InteropRefusal` | enum | InteropRefusal { UnsupportedShape, MissingGrounding, SchemaConflict, UngroundedArtifact, FlatClaimOverObjectCentric, DimensionShapeMismatch } |  |  |  |  |

| `KernelRefusal` | enum | KernelRefusal { ZeroSize, TooDense } |  |  |  |  |

| `LifecycleRefusal` | enum | LifecycleRefusal { EmptyLifecycle, InvalidTransition, OrphanEvent } |  |  |  |  |

| `LossFunction` | enum | LossFunction { MeanSquared, CrossEntropy, Hinge } |  |  |  |  |

| `LossPolicy` | enum | LossPolicy { RefuseLoss, AllowLossWithReport, AllowLossSilent, AllowNamedProjection } |  |  |  |  |

| `LossRefusal` | enum | LossRefusal { InvalidParameters, NumericalInstability } |  |  |  |  |

| `OCELAttributeValue` | enum | OCELAttributeValue { Integer(i64), Float(f64), Boolean(bool), String(std::string::String), Time(std::string::String), Null } |  |  |  |  |

| `ObjectCentricity` | enum | ObjectCentricity { CaseCentric, ObjectCentric, Mixed } |  |  |  |  |

| `ObjectLifecyclePhase` | enum | ObjectLifecyclePhase { Created, Active, Modified, Archived, Deleted } |  |  |  |  |

| `ObjectPredicateKind` | enum | ObjectPredicateKind { AttributeEquals, TypeEquals } |  |  |  |  |

| `ObjectTypeCardinality` | enum | ObjectTypeCardinality { One, ZeroOrOne, OneOrMany, ZeroOrMany } |  |  |  |  |

| `OcDeclareRefusal` | enum | OcDeclareRefusal { EmptyObjectTypeList, SynchronizationRequiresMultipleTypes, ScopeMismatch } |  |  |  |  |

| `OcelAttributeValue` | enum | OcelAttributeValue { Integer(i64), Float(f64), Boolean(bool), String(std::string::String), TimestampNs(u64), List(Vec<OcelAttributeValue>), Map(Vec<(std::string::String, OcelAttributeValue)>), Null } |  |  |  |  |

| `OcelRefusal` | enum | OcelRefusal { EmptyEventObjectLinks, DanglingEventObjectLink } |  |  |  |  |

| `OcpqRefusal` | enum | OcpqRefusal { EmptyFilter, UnsupportedScope } |  |  |  |  |

| `OcpqScopeKind` | enum | OcpqScopeKind { Open, Closed, SingleType } |  |  |  |  |

| `PerspectiveRefusal` | enum | PerspectiveRefusal { MissingPerspective, ConflictingPerspectives } |  |  |  |  |

| `PetriNetRefusal` | enum | PetriNetRefusal { IsolatedPlace, InvalidStructure, EmptyNet } |  |  |  |  |

| `PetriRefusal` | enum | PetriRefusal { IsolatedPlace, IsolatedTransition, MissingInitialMarking, InvalidWeight, MissingFinalMarking, UnsafeNet, InvalidInstanceBounds, ObjectTypeNotPreserved, DeadTransition } |  |  |  |  |

| `Pm4pyShape` | enum | Pm4pyShape { EventLog, ObjectCentricLog, PetriNet, ProcessTree, Bpmn, DirectlyFollowsGraph, Declare } |  |  |  |  |

| `Powl8Op` | enum | Powl8Op { NoOp = 0, Sequence = 1, Choice = 2, Parallel = 3, PartialOrder = 4, Loop = 5, Silent = 6, Or = 7, ChoiceGraph = 8 } |  |  |  |  |

| `Powl8OpError` | enum | Powl8OpError { InvalidDiscriminant } |  |  |  |  |

| `PowlNodeKind` | enum | PowlNodeKind { Atom(String), Silent, PartialOrder(Vec<PowlNodeId>), Choice(Vec<PowlNodeId>), Loop { body: PowlNodeId, redo: Option<PowlNodeId> }, ChoiceGraph { nodes: Vec<PowlNodeId>, edges: Vec<(usize, usize)> } } |  |  |  |  |

| `PowlProjectionState` | enum | PowlProjectionState { Unknown, ProcessTreeProjectable, ExceedsProcessTree, RefusedProjection } |  |  |  |  |

| `PowlRefusal` | enum | PowlRefusal { CyclicPartialOrder, InvalidLoop, InvalidChoiceArity { declared: usize, required_min: usize }, ChoiceGraphDisconnected } |  |  |  |  |

| `PredicateKind` | enum | PredicateKind { Event(String), Object(String), Relation(String), Temporal(String), Cardinality { min: usize, max: usize }, Nested, ChildSetBound { branch_label: String, min: usize, max: usize }, E2ORelation { event_var: String, object_var: String, qualifier: Option<String> }, O2ORelation { source_var: String, target_var: String, qualifier: Option<String> }, TimeBetweenEvents { from_var: String, to_var: String } } |  |  |  |  |

| `PredictionHorizon` | enum | PredictionHorizon { FullCase, Events(usize), TimeUnits(u64) } |  |  |  |  |

| `PredictionRefusal` | enum | PredictionRefusal { InsufficientData, InvalidModel, ConvergenceFailure, MissingPrefix, MissingTarget, EmptyPrefix, TargetUnsupported, NonPrefixTrace, ConstraintNotNamed } |  |  |  |  |

| `PredictionTarget` | enum | PredictionTarget { NextActivity, OutcomeLabel, RemainingTime, DriftSignal, Risk, ComplianceConstraint } |  |  |  |  |

| `ProcessPerspective` | enum | ProcessPerspective { ControlFlow, Data, Resource, Time } |  |  |  |  |

| `ProcessShapeKind` | enum | ProcessShapeKind { Event, Trace, EventLog, EventStream, XesLog, OcelLog, DirectlyFollowsGraph, ObjectCentricDfg, PetriNet, WorkflowNet, ObjectCentricPetriNet, ProcessTree, Powl, DeclareModel, ObjectCentricDeclareModel, LogSkeleton, OcpqQuery, Alignment, TokenReplay, ConformanceVerdict, PredictionProblem, Receipt } |  |  |  |  |

| `ProcessTreeNode` | enum | ProcessTreeNode { Activity(String), Operator { operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId> } } |  |  |  |  |

| `ProcessTreeOperator` | enum | ProcessTreeOperator { Sequence, Xor, Parallel, Or, Loop, Silent } |  |  |  |  |

| `ProcessTreeRefusal` | enum | ProcessTreeRefusal { EmptyTree, OrphanNode, InvalidLoopArity, UnsupportedOperator, MissingRoot, DanglingNodeReference, TauLeafWithChildren, BelowMinimumArity, InvalidArity, CycleDetected } |  |  |  |  |

| `QualityDimension` | enum | QualityDimension { Fitness, Precision, F1, Generalization, Simplicity } |  |  |  |  |

| `QualityMetricKind` | enum | QualityMetricKind { Fitness, Precision, F1, Generalization, Simplicity } |  |  |  |  |

| `ReceiptRefusal` | enum | ReceiptRefusal { EmptyChain, HashMismatch, InvalidDigest, MissingSubject, MissingWitness, MissingDigest, MissingReplayHint, BrokenChainLink(usize) } |  |  |  |  |

| `ReceiptVerdict` | enum | ReceiptVerdict { Admitted, Refused(ReceiptRefusal) } |  |  |  |  |

| `RelationLaw` | enum | RelationLaw { EventToObject, ObjectToObject, ObjectToEvent } |  |  |  |  |

| `RelationPredicateKind` | enum | RelationPredicateKind { E2O, O2O, TimeBetweenEvents } |  |  |  |  |

| `ReplayHintKind` | enum | ReplayHintKind { FromStart, FromSeq(u64), Latest } |  |  |  |  |

| `SoundnessState` | enum | SoundnessState { Unknown, Claimed, Witnessed } |  |  |  |  |

| `StandaloneChoiceGraphNode` | enum | StandaloneChoiceGraphNode { Start, End, Activity(String), SubModel(usize) } |  |  |  |  |

| `SummaryShape` | enum | SummaryShape { Counts, TraceVariants, ActivityDistribution, TimingProfile, ObjectTypeDistribution } |  |  |  |  |

| `TemporalOrder` | enum | TemporalOrder { Before, After, Concurrent, Unknown } |  |  |  |  |

| `TemporalRefusal` | enum | TemporalRefusal { ZeroBound, ConflictingConstraints } |  |  |  |  |

| `TemporalRelation` | enum | TemporalRelation { Before, After, During, Concurrent } |  |  |  |  |

| `WitnessFamily` | enum | WitnessFamily { Standard, ApiGrammar, Implementation, Paper } |  |  |  |  |

| `WorkflowPattern` | enum | WorkflowPattern { Sequence, ParallelSplit, Synchronization, ExclusiveChoice, SimpleMerge, MultiChoice, StructuredSynchronizingMerge, MultiMerge, StructuredDiscriminator, ArbitraryCycles, ImplicitTermination, MultipleInstancesWithoutSync, MultipleInstancesWithDesignTimeKnowledge, DeferredChoice, InterleavedParallelRouting, CancelActivity, CancelCase } |  |  |  |  |

| `activity` | function | activity(&self) -> &str |  |  |  |  |

| `add_arc` | function | add_arc(&mut self, a: Arc) |  |  |  |  |

| `add_place` | function | add_place(&mut self, p: Place) |  |  |  |  |

| `add_transition` | function | add_transition(&mut self, t: Transition) |  |  |  |  |

| `admit_flat` | function | admit_flat(&self) -> Result<(), InteropRefusal> |  |  |  |  |

| `admit_shape` | function | admit_shape(&self) -> Result<(), ProcessTreeRefusal> |  |  |  |  |

| `admits` | function | admits(&self, count: usize) -> bool |  |  |  |  |

| `arcs` | function | arcs(&self) -> &[Arc] |  |  |  |  |

| `arity` | function | arity(&self) -> u8 |  |  |  |  |

| `as_f64` | function | as_f64(&self) -> f64 |  |  |  |  |

| `as_str` | function | as_str(&self) -> &str |  |  |  |  |

| `assert_epsilon_close` | function | assert_epsilon_close(actual: f64, expected: f64) |  |  |  |  |

| `at_ns` | function | at_ns(mut self, ts: u64) -> Self |  |  |  |  |

| `attribute` | function | attribute(&self) -> &str |  |  |  |  |

| `attributes` | function | attributes(&self) -> &[OcelAttribute] |  |  |  |  |

| `binary` | function | binary(template: DeclareTemplate, activation: Activity, target: Activity, scope: DeclareScope) -> Self |  |  |  |  |

| `boolean` | function | boolean(key: &str, value: bool) -> Self |  |  |  |  |

| `by` | function | by(mut self, resource: &str) -> Self |  |  |  |  |

| `case_id` | function | case_id(&self) -> &str |  |  |  |  |

| `category` | function | category(&self) -> &str |  |  |  |  |

| `check_filter_shape` | function | check_filter_shape(shape: Pm4pyShape, filter: FilterShape) -> Result<(), InteropRefusal> |  |  |  |  |

| `claim_sound` | function | claim_sound(self) -> WfNet<SoundnessClaimed> |  |  |  |  |

| `conformance_rate` | function | conformance_rate(&self) -> f64 |  |  |  |  |

| `count` | function | count(&self) -> usize |  |  |  |  |

| `count_objects_of_type` | function | count_objects_of_type(&self, t: &str) -> usize |  |  |  |  |

| `declare_separable` | function | declare_separable(net: WfNetConst<S>) -> Self |  |  |  |  |

| `default` | function | default() -> Self |  |  |  |  |

| `den` | function | den(&self) -> u64 |  |  |  |  |

| `direction` | function | direction(&self) -> ArcDirection |  |  |  |  |

| `duration_ns` | function | duration_ns(&self) -> Option<u64> |  |  |  |  |

| `e2o` | function | e2o(&self, event_id: &str) -> Vec<(&str, &str)> |  |  |  |  |

| `edges` | function | edges(&self) -> &[BpmnEdge] |  |  |  |  |

| `empty` | function | empty() -> Self |  |  |  |  |

| `eval` | function | eval(&self, event_id: &str) -> Option<HashMap<std::string::String, OCELAttributeValue>> |  |  |  |  |

| `event` | function | event(id: &str, e: BpmnEvent) -> Self |  |  |  |  |

| `event_count` | function | event_count(&self) -> usize |  |  |  |  |

| `event_id` | function | event_id(&self) -> &str |  |  |  |  |

| `event_object_links` | function | event_object_links(&self) -> &[EventObjectLink] |  |  |  |  |

| `event_set` | function | event_set(&self) -> &Vec<OCELEvent> |  |  |  |  |

| `events` | function | events(&self) -> &[OcelEvent] |  |  |  |  |

| `expression` | function | expression(&self) -> &str |  |  |  |  |

| `extend_with` | function | extend_with(&mut self, env: ReceiptEnvelope) -> Result<(), ReceiptRefusal> |  |  |  |  |

| `final_marking` | function | final_marking(&self) -> Option<&Marking> |  |  |  |  |

| `float` | function | float(key: &str, value: f64) -> Self |  |  |  |  |

| `fnv1a_64` | function | fnv1a_64(bytes: &[u8]) -> u64 |  |  |  |  |

| `frequency` | function | frequency(&self) -> DfgWeight |  |  |  |  |

| `from_events` | function | from_events(events: impl IntoIterator<Item = Event>) -> Self |  |  |  |  |

| `from_owned` | function | from_owned(s: String) -> Self |  |  |  |  |

| `from_static` | function | from_static(s: &'static str) -> Self |  |  |  |  |

| `from_traces` | function | from_traces(traces: impl IntoIterator<Item = Trace>) -> Self |  |  |  |  |

| `gateway` | function | gateway(id: &str, g: BpmnGateway) -> Self |  |  |  |  |

| `get` | function | get(&self) -> f64 |  |  |  |  |

| `get_by_hash` | function | get_by_hash(&self, hash: u64) -> Option<&V> |  |  |  |  |

| `get_by_key` | function | get_by_key(&self, key: &str) -> Option<&V> |  |  |  |  |

| `id` | function | id(&self) -> &str |  |  |  |  |

| `initial_marking` | function | initial_marking(&self) -> &Marking |  |  |  |  |

| `inner` | function | inner(&self) -> &T |  |  |  |  |

| `insert` | function | insert(&mut self, hash: u64, key: String, value: V) |  |  |  |  |

| `integer` | function | integer(key: &str, value: i64) -> Self |  |  |  |  |

| `internal` | function | internal(id: ProcessTreeNodeId, operator: ProcessTreeOperator, children: Vec<ProcessTreeNodeId>) -> Self |  |  |  |  |

| `into_admitted` | function | into_admitted(self) -> Evidence<T, state::Admitted, C> |  |  |  |  |

| `into_evidence` | function | into_evidence(self) -> Evidence<T, state::Admitted, W> |  |  |  |  |

| `into_exportable` | function | into_exportable(self) -> Evidence<T, state::Exportable, C> |  |  |  |  |

| `into_inner` | function | into_inner(self) -> T |  |  |  |  |

| `into_lost` | function | into_lost(self) -> Dropped |  |  |  |  |

| `into_parsed` | function | into_parsed(self) -> Evidence<T, state::Parsed, C> |  |  |  |  |

| `into_projected` | function | into_projected(self) -> Evidence<T, state::Projected, C> |  |  |  |  |

| `into_reason` | function | into_reason(self) -> R |  |  |  |  |

| `into_receipted` | function | into_receipted(self) -> Evidence<T, state::Receipted, C> |  |  |  |  |

| `is_chain` | function | is_chain(&self) -> bool |  |  |  |  |

| `is_consistent` | function | is_consistent(&self) -> bool |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_grounded` | function | is_grounded(&self) -> bool |  |  |  |  |

| `is_lossless` | function | is_lossless(&self) -> bool |  |  |  |  |

| `is_negative` | function | is_negative(&self) -> bool |  |  |  |  |

| `is_object_centric` | function | is_object_centric(&self) -> bool |  |  |  |  |

| `is_perfect` | function | is_perfect(&self) -> bool |  |  |  |  |

| `is_silent` | function | is_silent(&self) -> bool |  |  |  |  |

| `is_synchronized` | function | is_synchronized(&self) -> bool |  |  |  |  |

| `is_well_shaped` | function | is_well_shaped(&self) -> bool |  |  |  |  |

| `iter` | function | iter(&self) -> impl Iterator<Item = &ReceiptEnvelope> |  |  |  |  |

| `kind` | function | kind(&self) -> QualityMetricKind |  |  |  |  |

| `label` | function | label(&self) -> &str |  |  |  |  |

| `lanes` | function | lanes(&self) -> &[BpmnLane] |  |  |  |  |

| `leaf` | function | leaf(id: ProcessTreeNodeId, label: &str) -> Self |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `length` | function | length(&self) -> usize |  |  |  |  |

| `lifecycle` | function | lifecycle(&self) -> Option<&str> |  |  |  |  |

| `mark` | function | mark(&mut self, place_id: &str, tokens: usize) |  |  |  |  |

| `max` | function | max(&self) -> usize |  |  |  |  |

| `members` | function | members(&self) -> &[String] |  |  |  |  |

| `min` | function | min(&self) -> usize |  |  |  |  |

| `name` | function | name(&self) -> &str |  |  |  |  |

| `net` | function | net(&self) -> &PetriNet |  |  |  |  |

| `new` | function | new(value: T) -> Self |  |  |  |  |

| `node_count` | function | node_count(&self) -> usize |  |  |  |  |

| `node_ids` | function | node_ids(&self) -> &[String] |  |  |  |  |

| `nodes` | function | nodes(&self) -> &[BpmnNode] |  |  |  |  |

| `num` | function | num(&self) -> u64 |  |  |  |  |

| `o2o` | function | o2o(&self, object_id: &str) -> Vec<(&str, &str)> |  |  |  |  |

| `object_changes` | function | object_changes(&self) -> &[ObjectChange] |  |  |  |  |

| `object_id` | function | object_id(&self) -> &str |  |  |  |  |

| `object_object_links` | function | object_object_links(&self) -> &[ObjectObjectLink] |  |  |  |  |

| `object_set` | function | object_set(&self) -> &Vec<OCELObject> |  |  |  |  |

| `object_type` | function | object_type(&self) -> &str |  |  |  |  |

| `object_types` | function | object_types(&self) -> impl Iterator<Item = &str> |  |  |  |  |

| `objects` | function | objects(&self) -> &[Object] |  |  |  |  |

| `place_id` | function | place_id(&self) -> &str |  |  |  |  |

| `place_to_transition` | function | place_to_transition(place: &str, transition: &str) -> Self |  |  |  |  |

| `places` | function | places(&self) -> &[Place] |  |  |  |  |

| `predecessors` | function | predecessors(&self, idx: usize) -> Vec<usize> |  |  |  |  |

| `process` | function | process(&self) -> &BpmnProcess |  |  |  |  |

| `projection` | function | projection(&self) -> ProjectionNameOwned |  |  |  |  |

| `push` | function | push(&mut self, event: Event) |  |  |  |  |

| `qualified` | function | qualified(mut self, q: &str) -> Self |  |  |  |  |

| `qualifier` | function | qualifier(&self) -> Option<&str> |  |  |  |  |

| `raw` | function | raw(value: T) -> Self |  |  |  |  |

| `resource` | function | resource(&self) -> Option<&str> |  |  |  |  |

| `root` | function | root(&self) -> Option<ProcessTreeNodeId> |  |  |  |  |

| `schema` | function | schema(&self) -> &'static str |  |  |  |  |

| `scope` | function | scope(&self) -> &ObjectScopeConst |  |  |  |  |

| `silent` | function | silent(id: &str) -> Self |  |  |  |  |

| `soundness_state` | function | soundness_state(&self) -> SoundnessState |  |  |  |  |

| `source` | function | source(&self) -> &str |  |  |  |  |

| `source_id` | function | source_id(&self) -> &str |  |  |  |  |

| `steps` | function | steps(&self) -> &[NamedLoss] |  |  |  |  |

| `string` | function | string(key: &str, value: &str) -> Self |  |  |  |  |

| `successors` | function | successors(&self, idx: usize) -> Vec<usize> |  |  |  |  |

| `summary` | function | summary(&self, category: &str) -> NamedLoss |  |  |  |  |

| `synchronized` | function | synchronized(constraint: DeclareConstraint, object_types: Vec<String>) -> Self |  |  |  |  |

| `tag` | function | tag(&self) -> &'static str |  |  |  |  |

| `target` | function | target(&self) -> &str |  |  |  |  |

| `target_id` | function | target_id(&self) -> &str |  |  |  |  |

| `task` | function | task(id: &str, t: BpmnTask) -> Self |  |  |  |  |

| `timestamp_ns` | function | timestamp_ns(&self) -> Option<u64> |  |  |  |  |

| `tip` | function | tip(&self) -> &ReceiptEnvelope |  |  |  |  |

| `tokens` | function | tokens(&self) -> &HashMap<String, usize> |  |  |  |  |

| `tokens_on` | function | tokens_on(&self, place_id: &str) -> usize |  |  |  |  |

| `trace_count` | function | trace_count(&self) -> usize |  |  |  |  |

| `traces` | function | traces(&self) -> &[Trace] |  |  |  |  |

| `transition_id` | function | transition_id(&self) -> &str |  |  |  |  |

| `transition_to_place` | function | transition_to_place(transition: &str, place: &str) -> Self |  |  |  |  |

| `transitions` | function | transitions(&self) -> &[Transition] |  |  |  |  |

| `try_from_parts` | function | try_from_parts(subject: &str, witness: &str, digest: Digest, replay_hint: ReplayHint) -> Result<Self, ReceiptRefusal> |  |  |  |  |

| `try_new` | function | try_new(run_id: &str, envelopes: Vec<ReceiptEnvelope>) -> Result<Self, ReceiptRefusal> |  |  |  |  |

| `unary` | function | unary(template: DeclareTemplate, activation: Activity, scope: DeclareScope) -> Self |  |  |  |  |

| `validate` | function | validate(&self) -> Result<(), OcelRefusal> |  |  |  |  |

| `value` | function | value(&self) -> &str |  |  |  |  |

| `verdict` | function | verdict(&self) -> CausalConsistency |  |  |  |  |

| `verify` | function | verify(&self) -> Result<(), ReceiptRefusal> |  |  |  |  |

| `weight` | function | weight(&self) -> W |  |  |  |  |

| `with_attribute` | function | with_attribute(mut self, a: OcelAttribute) -> Self |  |  |  |  |

| `with_duration_ns` | function | with_duration_ns(mut self, d: u64) -> Self |  |  |  |  |

| `with_generalization` | function | with_generalization(mut self, v: f64) -> Self |  |  |  |  |

| `with_lifecycle` | function | with_lifecycle(mut self, lc: &str) -> Self |  |  |  |  |

| `with_precision` | function | with_precision(mut self, v: f64) -> Self |  |  |  |  |

| `with_simplicity` | function | with_simplicity(mut self, v: f64) -> Self |  |  |  |  |

| `with_type_dfg` | function | with_type_dfg(mut self, obj_type: &str, dfg: Dfg) -> Self |  |  |  |  |

| `witness_soundness` | function | witness_soundness(self, _proof: SoundnessProof) -> WfNetConst<{ SoundnessState::Witnessed }> |  |  |  |  |

| `Activity` | struct | Activity { pub String } |  |  |  |  |

| `Admission` | struct | Admission { pub value: T, _witness: PhantomData<W> } |  |  |  |  |

| `Admitted` | struct | Admitted |  |  |  |  |

| `Arc` | struct | Arc { pub source: String, pub target: String, pub weight: u32, pub object_type: Option<(String, bool)>, _dir: ArcDirection } |  |  |  |  |

| `ArtifactGrounding` | struct | ArtifactGrounding { pub shape: Pm4pyShape, pub evidence_ref: String, _evidence: PhantomData<E> } |  |  |  |  |

| `Between01` | struct | Between01 { _priv: () } |  |  |  |  |

| `BipartiteArcConst` | struct | BipartiteArcConst { _place_id: String, _transition_id: String, _weight: W } |  |  |  |  |

| `BpmnEdge` | struct | BpmnEdge { source: String, target: String } |  |  |  |  |

| `BpmnLane` | struct | BpmnLane { id: String, name: String, node_ids: Vec<String> } |  |  |  |  |

| `BpmnNode` | struct | BpmnNode { id: String, kind: BpmnNodeKind } |  |  |  |  |

| `BpmnPool` | struct | BpmnPool { id: String, name: String, process: BpmnProcess, lanes: Vec<BpmnLane> } |  |  |  |  |

| `BpmnProcess` | struct | BpmnProcess { nodes: Vec<BpmnNode>, edges: Vec<BpmnEdge> } |  |  |  |  |

| `BpmnTask` | struct | BpmnTask { pub label: String } |  |  |  |  |

| `CancellationRegion` | struct | CancellationRegion { _members: Vec<String> } |  |  |  |  |

| `CausalBinding` | struct | CausalBinding { pub source_tasks: Vec<String>, pub target_tasks: Vec<String> } |  |  |  |  |

| `CausalChain` | struct |  |  |  |  |  |

| `CausalLink` | struct | CausalLink { PhantomData<(From, To)> } |  |  |  |  |

| `CausalNet` | struct | CausalNet { pub nodes: Vec<String>, pub dependency_measures: Vec<(String, String, f64)>, pub inputs: Vec<CausalBinding>, pub outputs: Vec<CausalBinding> } |  |  |  |  |

| `CausallyOrderedEvidence` | struct | CausallyOrderedEvidence { pub inner: T } |  |  |  |  |

| `ChoiceGraph` | struct | ChoiceGraph { nodes: Vec<StandaloneChoiceGraphNode>, edges: Vec<(usize, usize)> } |  |  |  |  |

| `ConditionCell` | struct | ConditionCell { _priv: () } |  |  |  |  |

| `ConformanceResult` | struct | ConformanceResult { pub fitness: f64, pub precision: Option<f64>, pub generalization: Option<f64>, pub simplicity: Option<f64>, pub total_traces: usize, pub fitting_traces: usize, pub deviating_traces: usize } |  |  |  |  |

| `ConformanceVerdict` | struct | ConformanceVerdict { pub fitness: Option<Fitness>, pub deviations: Vec<Deviation> } |  |  |  |  |

| `ConsistencyVerified` | struct | ConsistencyVerified { pub inner: T, verdict: CausalConsistency } |  |  |  |  |

| `ControlFlowPerspective` | struct |  |  |  |  |  |

| `CorrelatedLog` | struct | CorrelatedLog { PhantomData<(A, B)> } |  |  |  |  |

| `CorrelationKey` | struct |  |  |  |  |  |

| `DataPerspective` | struct |  |  |  |  |  |

| `DeclareConstraint` | struct | DeclareConstraint { pub template: DeclareTemplate, pub activation: Activity, pub target: Option<Activity>, pub scope: DeclareScope } |  |  |  |  |

| `DenseKernel` | struct | DenseKernel { pub size: usize } |  |  |  |  |

| `DependencyMeasure` | struct | DependencyMeasure { pub f64 } |  |  |  |  |

| `Deviation` | struct | Deviation { pub position: usize, pub label: String } |  |  |  |  |

| `Dfg` | struct | Dfg { nodes: Vec<DfgNode>, edges: Vec<DfgEdge> } |  |  |  |  |

| `DfgEdge` | struct | DfgEdge { source: String, target: String, count: u64 } |  |  |  |  |

| `DfgEdgeFull` | struct | DfgEdgeFull { pub source: String, pub target: String, _freq: u64, _dur: Option<u64> } |  |  |  |  |

| `DfgNode` | struct | DfgNode { pub String } |  |  |  |  |

| `DfgWeight` | struct | DfgWeight { pub u64 } |  |  |  |  |

| `DiagnosticReport` | struct | DiagnosticReport { pub kind: DiagnosticKind, pub message: String, pub stage: String } |  |  |  |  |

| `Digest` | struct | Digest { pub String } |  |  |  |  |

| `Event` | struct | Event { pub activity: String, pub attributes: HashMap<String, String>, _ts: Option<u64>, _resource: Option<String>, _lifecycle: Option<String> } |  |  |  |  |

| `EventLog` | struct | EventLog { pub traces: Vec<Trace> } |  |  |  |  |

| `EventObjectLink` | struct | EventObjectLink { pub event_id: String, pub object_id: String, _qualifier: Option<String> } |  |  |  |  |

| `EventStream` | struct | EventStream { events: Vec<Event> } |  |  |  |  |

| `EventTypeName` | struct | EventTypeName { inner: Cow<'static, str>, _kind: PhantomData<K> } |  |  |  |  |

| `Evidence` | struct | Evidence { pub value: T, _state: PhantomData<S>, _chain: PhantomData<C> } |  |  |  |  |

| `Exportable` | struct | Exportable |  |  |  |  |

| `F1` | struct | F1 { pub f64 } |  |  |  |  |

| `Fitness` | struct | Fitness { pub f64 } |  |  |  |  |

| `Generalization` | struct | Generalization { pub f64 } |  |  |  |  |

| `InitialFinalMarkingPair` | struct | InitialFinalMarkingPair { _initial: Marking, _final: Marking } |  |  |  |  |

| `InputBinding` | struct | InputBinding { pub &'a str, pub &'a str } |  |  |  |  |

| `LifecycleEvent` | struct | LifecycleEvent { pub event_type: String, pub object_id: String } |  |  |  |  |

| `LossChain` | struct | LossChain { steps: Vec<NamedLoss> } |  |  |  |  |

| `LossReport` | struct | LossReport { pub projection: ProjectionName, pub policy: LossPolicy, _dropped: Dropped, _ph: PhantomData<(A, B)> } |  |  |  |  |

| `Marking` | struct | Marking { pub HashMap<String, usize> } |  |  |  |  |

| `Metric` | struct | Metric { _priv: () } |  |  |  |  |

| `MultiPerspectiveEvidence` | struct | MultiPerspectiveEvidence { pub inner: T, _perspective: PhantomData<P> } |  |  |  |  |

| `MultiPerspectiveLog` | struct | MultiPerspectiveLog { pub traces: Vec<String> } |  |  |  |  |

| `MultipleInstanceSpec` | struct | MultipleInstanceSpec { pub min: usize, pub max: Option<usize>, pub threshold: Option<usize>, pub creation: InstanceCreationKind } |  |  |  |  |

| `MultipleInstanceSpecConst` | struct | MultipleInstanceSpecConst { _priv: () } |  |  |  |  |

| `NamedLoss` | struct | NamedLoss { pub projection_str: String, pub _category: String } |  |  |  |  |

| `OCEL` | struct | OCEL { pub events: Vec<OCELEvent>, pub objects: Vec<OCELObject> } |  |  |  |  |

| `OCELEvent` | struct | OCELEvent { pub id: std::string::String, pub event_type: std::string::String, pub relationships: Vec<OCELRelationship>, pub attributes: Vec<OCELEventAttribute> } |  |  |  |  |

| `OCELEventAttribute` | struct | OCELEventAttribute { pub name: std::string::String, pub value: OCELAttributeValue } |  |  |  |  |

| `OCELObject` | struct | OCELObject { pub id: std::string::String, pub object_type: std::string::String, pub attributes: Vec<OCELEventAttribute>, pub relationships: Vec<OCELRelationship> } |  |  |  |  |

| `OCELRelationship` | struct | OCELRelationship { pub object_id: std::string::String, pub qualifier: std::string::String } |  |  |  |  |

| `OCELType` | struct | OCELType { pub name: std::string::String, pub attributes: Vec<OCELTypeAttribute> } |  |  |  |  |

| `OCELTypeAttribute` | struct | OCELTypeAttribute { pub name: std::string::String, pub value_type: std::string::String } |  |  |  |  |

| `Object` | struct | Object { pub id: String, pub obj_type: String, _attrs: Vec<OcelAttribute> } |  |  |  |  |

| `ObjectCentricDfg` | struct | ObjectCentricDfg { HashMap<String, Dfg> } |  |  |  |  |

| `ObjectCentricPetriNet` | struct | ObjectCentricPetriNet { _net: PetriNet, _object_types: Vec<String> } |  |  |  |  |

| `ObjectChange` | struct | ObjectChange { _object_id: String, _attribute: String, _value: String, _ts: Option<u64> } |  |  |  |  |

| `ObjectLifecycle` | struct | ObjectLifecycle { pub events: Vec<LifecycleEvent> } |  |  |  |  |

| `ObjectObjectLink` | struct | ObjectObjectLink { pub source_id: String, pub target_id: String, _qualifier: Option<String> } |  |  |  |  |

| `ObjectScope` | struct | ObjectScope { pub object_types: Vec<String> } |  |  |  |  |

| `ObjectScopeConst` | struct | ObjectScopeConst { pub object_types: Vec<String> } |  |  |  |  |

| `ObjectTypeCardinality` | struct | ObjectTypeCardinality { pub min_count: Option<usize>, pub max_count: Option<usize>, pub created_by: Vec<std::string::String>, pub terminated_by: Vec<std::string::String> } |  |  |  |  |

| `ObjectTypeName` | struct | ObjectTypeName { inner: Cow<'static, str>, _kind: PhantomData<K> } |  |  |  |  |

| `OcDeclareConstraint` | struct | OcDeclareConstraint { pub constraint: DeclareConstraint, pub object_types: Vec<String>, _synchronized: bool } |  |  |  |  |

| `OcelAttribute` | struct | OcelAttribute { pub key: std::string::String, pub value: OcelAttributeValue } |  |  |  |  |

| `OcelEvent` | struct | OcelEvent { pub id: String, pub event_type: String, _ts: Option<u64>, _attrs: Vec<OcelAttribute> } |  |  |  |  |

| `OcelLog` | struct | OcelLog { objects: Vec<Object>, events: Vec<OcelEvent>, event_object_links: Vec<EventObjectLink>, object_object_links: Vec<ObjectObjectLink>, object_changes: Vec<ObjectChange> } |  |  |  |  |

| `OcpqQuery` | struct | OcpqQuery { pub scope: ObjectScope, pub predicates: Vec<Predicate>, pub sub_queries: Vec<OcpqQuery> } |  |  |  |  |

| `OcpqQueryConst` | struct | OcpqQueryConst { _scope: ObjectScopeConst } |  |  |  |  |

| `OrderEdge` | struct | OrderEdge { pub from: PowlNodeId, pub to: PowlNodeId } |  |  |  |  |

| `OutputBinding` | struct | OutputBinding { pub &'a str, pub &'a str } |  |  |  |  |

| `PackedKeyTable` | struct | PackedKeyTable { data: HashMap<u64, (String, V)> } |  |  |  |  |

| `ParityComparer` | struct |  |  |  |  |  |

| `Parsed` | struct | Parsed |  |  |  |  |

| `PerspectiveCombination` | struct | PerspectiveCombination { PhantomData<(A, B)> } |  |  |  |  |

| `PetriNet` | struct | PetriNet { pub places: Vec<Place>, pub transitions: Vec<Transition>, pub arcs: Vec<Arc>, pub initial_marking: Marking, pub final_marking: Marking } |  |  |  |  |

| `Place` | struct | Place { pub id: String } |  |  |  |  |

| `PlaceNodeMarker` | struct |  |  |  |  |  |

| `PlaceToTransitionArc` | struct | PlaceToTransitionArc { W, PhantomData<(P, T)> } |  |  |  |  |

| `Powl` | struct | Powl { pub nodes: Vec<PowlNode>, pub edges: Vec<OrderEdge>, pub root: Option<PowlNodeId> } |  |  |  |  |

| `PowlChoiceNode` | struct | PowlChoiceNode { branches: Vec<PowlNodeId> } |  |  |  |  |

| `PowlComposition` | struct | PowlComposition { pub inner: Inner } |  |  |  |  |

| `PowlNode` | struct | PowlNode { pub id: PowlNodeId, pub kind: PowlNodeKind } |  |  |  |  |

| `PowlNodeId` | struct | PowlNodeId { pub u64 } |  |  |  |  |

| `Precision` | struct | Precision { pub f64 } |  |  |  |  |

| `Predicate` | struct | Predicate { pub kind: PredicateKind, _data: std::marker::PhantomData<T> } |  |  |  |  |

| `ProcessCube` | struct | ProcessCube { pub dimensions: Vec<CubeDimensionKind> } |  |  |  |  |

| `ProcessSlice` | struct | ProcessSlice { pub dimension: CubeDimensionKind, pub value: String } |  |  |  |  |

| `ProcessTree` | struct | ProcessTree { pub nodes: Vec<ProcessTreeNode>, pub root: Option<ProcessTreeNodeId> } |  |  |  |  |

| `ProcessTreeNodeId` | struct | ProcessTreeNodeId { pub usize } |  |  |  |  |

| `Projected` | struct | Projected |  |  |  |  |

| `ProjectionName` | struct | ProjectionName { pub &'static str } |  |  |  |  |

| `ProjectionNameOwned` | struct | ProjectionNameOwned { pub String } |  |  |  |  |

| `QualityProfile` | struct | QualityProfile { pub fitness: Between01<FN, FD>, pub precision: Between01<PN, PD>, pub f1: Between01<F1N, F1D>, pub generalization: Between01<GN, GD>, pub simplicity: Between01<SN, SD> } |  |  |  |  |

| `Raw` | struct | Raw |  |  |  |  |

| `ReceiptChain` | struct | ReceiptChain { pub run_id: String, pub envelopes: Vec<ReceiptEnvelope> } |  |  |  |  |

| `ReceiptChainConst` | struct | ReceiptChainConst { pub run_id: String, pub links: Vec<ReceiptEnvelope> } |  |  |  |  |

| `ReceiptEnvelope` | struct | ReceiptEnvelope { pub subject: String, pub witness: String, pub digest: Digest, pub replay_hint: ReplayHint } |  |  |  |  |

| `Receipted` | struct | Receipted |  |  |  |  |

| `Refusal` | struct | Refusal { pub reason: R, _witness: PhantomData<W> } |  |  |  |  |

| `Refused` | struct | Refused |  |  |  |  |

| `ReplayHint` | struct | ReplayHint { pub String } |  |  |  |  |

| `Require` | struct |  |  |  |  |  |

| `ResourcePerspective` | struct |  |  |  |  |  |

| `RuntimeMarking` | struct | RuntimeMarking { table: &'a PackedKeyTable<usize> } |  |  |  |  |

| `SeparableWfNet` | struct | SeparableWfNet { pub net: WfNetConst<S> } |  |  |  |  |

| `Simplicity` | struct | Simplicity { pub f64 } |  |  |  |  |

| `SoundnessClaimed` | struct |  |  |  |  |  |

| `SoundnessProof` | struct | SoundnessProof |  |  |  |  |

| `SoundnessUnknown` | struct |  |  |  |  |  |

| `StableId` | struct | StableId { pub String } |  |  |  |  |

| `TemporalConstraint` | struct | TemporalConstraint { pub relation: TemporalRelation, pub bound_ms: u64 } |  |  |  |  |

| `TimePerspective` | struct |  |  |  |  |  |

| `Trace` | struct | Trace { pub events: Vec<Event>, pub case_id: String } |  |  |  |  |

| `Transition` | struct | Transition { pub id: String, pub label: String } |  |  |  |  |

| `TransitionNodeMarker` | struct |  |  |  |  |  |

| `TransitionToPlaceArc` | struct | TransitionToPlaceArc { W, PhantomData<(T, P)> } |  |  |  |  |

| `TypedEventPredicate` | struct | TypedEventPredicate { _expr: String } |  |  |  |  |

| `TypedId` | struct | TypedId { pub id: String, _type: PhantomData<T> } |  |  |  |  |

| `TypedLoopNode` | struct | TypedLoopNode { pub children: Children } |  |  |  |  |

| `TypedObjectPredicate` | struct | TypedObjectPredicate { _expr: String } |  |  |  |  |

| `TypedPowlLoopNode` | struct | TypedPowlLoopNode { pub children: Children } |  |  |  |  |

| `TypedRelationPredicate` | struct | TypedRelationPredicate { _expr: String } |  |  |  |  |

| `UnknownVerifier` | struct |  |  |  |  |  |

| `WfNet` | struct | WfNet { _net: PetriNet, _final: Marking, _state: PhantomData<S> } |  |  |  |  |

| `WfNetConst` | struct | WfNetConst { _priv: () } |  |  |  |  |

| `Witnessed` | struct | Witnessed |  |  |  |  |

| `IsTrue` | trait |  |  |  |  |  |

| `IsValidArc` | trait |  |  |  |  |  |

| `VerifyCausalConsistency` | trait |  |  |  |  |  |

| `Witness` | trait |  |  |  |  |  |

| `crate::models::PackedKeyTable` | use | crate::models::PackedKeyTable |  |  |  |  |

| `AttributeValue` | enum | AttributeValue { String(std::string::String), Int(i64), Float(f64), Date(std::string::String), Boolean(bool), List(Vec<AttributeValue>), Container(HashMap<std::string::String, AttributeValue>), Null } |  |  |  |  |

| `new` | function | new(attributes: HashMap<std::string::String, AttributeValue>) -> Self |  |  |  |  |

| `DFG` | struct | DFG { pub nodes: Vec<DFGNode>, pub edges: Vec<DirectlyFollowsRelation>, pub start_activities: HashMap<std::string::String, usize>, pub end_activities: HashMap<std::string::String, usize> } |  |  |  |  |

| `DFGNode` | struct | DFGNode { pub id: std::string::String, pub label: std::string::String, pub frequency: usize } |  |  |  |  |

| `DirectlyFollowsRelation` | struct | DirectlyFollowsRelation { pub from: std::string::String, pub to: std::string::String, pub frequency: usize } |  |  |  |  |

| `Event` | struct | Event { pub attributes: HashMap<std::string::String, AttributeValue> } |  |  |  |  |

| `Trace` | struct | Trace { pub events: Vec<Event>, pub attributes: HashMap<std::string::String, AttributeValue> } |  |  |  |  |

| `AttributeValue` | enum | AttributeValue { String(std::string::String), Int(i64), Float(f64), Date(std::string::String), Boolean(bool), List(Vec<AttributeValue>), Container(HashMap<std::string::String, AttributeValue>), Null } |  |  |  |  |

| `new` | function | new(attributes: HashMap<std::string::String, AttributeValue>) -> Self |  |  |  |  |

| `DFG` | struct | DFG { pub nodes: Vec<DFGNode>, pub edges: Vec<DirectlyFollowsRelation>, pub start_activities: HashMap<std::string::String, usize>, pub end_activities: HashMap<std::string::String, usize> } |  |  |  |  |

| `DFGNode` | struct | DFGNode { pub id: std::string::String, pub label: std::string::String, pub frequency: usize } |  |  |  |  |

| `DirectlyFollowsRelation` | struct | DirectlyFollowsRelation { pub from: std::string::String, pub to: std::string::String, pub frequency: usize } |  |  |  |  |

| `Event` | struct | Event { pub attributes: HashMap<std::string::String, AttributeValue> } |  |  |  |  |

| `Trace` | struct | Trace { pub events: Vec<Event>, pub attributes: HashMap<std::string::String, AttributeValue> } |  |  |  |  |

| `affidavit:chicago:bls` | str_key | CONTEXT = "affidavit:chicago:bls" |  |  |  |  |

| `affidavit:chicago:zk-range` | str_key | LABEL = "affidavit:chicago:zk-range" |  |  |  |  |

| `git:fb09f991fab7995c1b7564f668065ef379303800` | str_key | SUBJECT = "git:fb09f991fab7995c1b7564f668065ef379303800" |  |  |  |  |

| `sha256:abb-0001` | str_key | ABB = "sha256:abb-0001" |  |  |  |  |

| `sha256:contract-0001` | str_key | CONTRACT = "sha256:contract-0001" |  |  |  |  |

| `sha256:producer-autofde-lab` | str_key | PRODUCER = "sha256:producer-autofde-lab" |  |  |  |  |

| `sha256:sbb-0001` | str_key | SBB = "sha256:sbb-0001" |  |  |  |  |

| `cc1577a8de1e4e92625386ebbecb0c6df9abd66a` | str_key | SUBJECT = "cc1577a8de1e4e92625386ebbecb0c6df9abd66a" |  |  |  |  |

| `filter_fixtures` | function | filter_fixtures( fixtures: Vec<FixtureMeta>, name: Option<&str>, events: Option<usize>, ) -> Vec<FixtureMeta> |  |  |  |  |

| `FixtureMeta` | struct | FixtureMeta { pub name: String, pub event_count: usize, pub description: String, pub path: Option<PathBuf> } |  |  |  |  |

| `__affi_no_noun' -a ` | str_key | NOUN_GUARD = "__affi_no_noun' -a " |  |  |  |  |

| `__affi_no_verb' -a ` | str_key | VERB_GUARD = "__affi_no_verb' -a " |  |  |  |  |

| `__affi_using_noun ` | str_key | NOUN_SCOPE = "__affi_using_noun " |  |  |  |  |

| `fixtures/crypto_trust_acvp` | str_key | FIXTURE_DIR = "fixtures/crypto_trust_acvp" |  |  |  |  |

| `CTP-ENVELOPE-v1` | str_key | COURT_ENVELOPE_VERSION = "CTP-ENVELOPE-v1" |  |  |  |  |

| `affidavit.crypto-trust-plane.v1` | str_key | COURT_DOMAIN_TAG = "affidavit.crypto-trust-plane.v1" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `fixtures/crypto_trust_jcs_corpus.json` | str_key | CORPUS_RELPATH = "fixtures/crypto_trust_jcs_corpus.json" |  |  |  |  |

| `fixtures/crypto_trust_jcs_expected.json` | str_key | EXPECTED_RELPATH = "fixtures/crypto_trust_jcs_expected.json" |  |  |  |  |

| `AFFIDAVIT_KAT_EXPORT_FIXTURE` | str_key | EXPORT_ENV = "AFFIDAVIT_KAT_EXPORT_FIXTURE" |  |  |  |  |

| `affidavit::crypto_trust_kat::generate_corpus() -> export_json (JCS, RFC 8785); deterministic under KAT_DOMAIN_TAG=affidavit.crypto-trust-plane.v1; regenerate: cd <repo root> && AFFIDAVIT_KAT_EXPORT_FIXTURE=1 cargo test --features crypto-trust --test crypto_trust_kat_fixture export_fixture -- --exact --ignored` | str_key | GENERATED_FROM = "affidavit::crypto_trust_kat::generate_corpus() -> export_json (JCS, RFC 8785); deterministic under KAT_DOMAIN_TAG=affidavit.crypto-trust-plane.v1; regenerate: cd <repo root> && AFFIDAVIT_KAT_EXPORT_FIXTURE=1 cargo test --features crypto-trust --test crypto_trust_kat_fixture export_fixture -- --exact --ignored" |  |  |  |  |

| `corpus` | str_key | EXPECTED_HEADER_KEYS = "corpus" |  |  |  |  |

| `fixtures/crypto_trust_kat.json` | str_key | FIXTURE_RELPATH = "fixtures/crypto_trust_kat.json" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `48563e42e5c6f6aa0d600fc8baf0d7455c37f7ac` | str_key | GENERATING_COMMIT = "48563e42e5c6f6aa0d600fc8baf0d7455c37f7ac" |  |  |  |  |

| `AFFIDAVIT_KATVECTORS_EXPORT_FIXTURE` | str_key | EXPORT_ENV = "AFFIDAVIT_KATVECTORS_EXPORT_FIXTURE" |  |  |  |  |

| `CTP-KATVECTORS-v1` | str_key | SCHEMA_VERSION = "CTP-KATVECTORS-v1" |  |  |  |  |

| `PASS: 16/16 seed pre-images` | str_key | CROSS_CHECK_SENTINEL = "PASS: 16/16 seed pre-images" |  |  |  |  |

| `affidavit-kat-subject` | str_key | CUSTODIAN_SUBJECT = "affidavit-kat-subject" |  |  |  |  |

| `affidavit.kat` | str_key | AUDIENCE = "affidavit.kat" |  |  |  |  |

| `affidavit.seal` | str_key | SEAL_AUDIENCE = "affidavit.seal" |  |  |  |  |

| `fixtures/crypto_trust_kat_vectors.json` | str_key | FIXTURE_RELPATH = "fixtures/crypto_trust_kat_vectors.json" |  |  |  |  |

| `0123456789abcdef` | str_key | HEX = "0123456789abcdef" |  |  |  |  |

| `3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f` | str_key | PAYLOAD_HASH = "3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f" |  |  |  |  |

| `subject-a` | str_key | SUBJECT = "subject-a" |  |  |  |  |

| `a` | str_key | POOL = "a" |  |  |  |  |

| `../fixtures/crypto_trust_wire_corpus.json` | str_key | CORPUS = "../fixtures/crypto_trust_wire_corpus.json" |  |  |  |  |

| `verify_maximalist` | function | verify_maximalist(receipt: &Receipt, tracer: &T) -> Verdict |  |  |  |  |

| `ArbitraryOperationEvent` | struct | ArbitraryOperationEvent { pub OperationEvent } |  |  |  |  |

| `ArbitraryReceipt` | struct | ArbitraryReceipt { pub Receipt } |  |  |  |  |

| `3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f` | str_key | PAYLOAD_HASH = "3f7a1b0c9d8e2f4153647a1b0c9d8e2f4153647a8b9c0d1e2f3a4b5c6d7e8f" |  |  |  |  |

| `subject-a` | str_key | SUBJECT = "subject-a" |  |  |  |  |

| `CARGO_PKG_VERSION` | str_key | PKG_VERSION = "CARGO_PKG_VERSION" |  |  |  |  |

| `from_receipt_dfg` | function | from_receipt_dfg(receipt: &Receipt) -> Self |  |  |  |  |

| `to_dot` | function | to_dot(&self) -> String |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `GraphEdge` | struct | GraphEdge { pub source: String, pub target: String, pub label: String } |  |  |  |  |

| `GraphNode` | struct | GraphNode { pub id: String, pub label: String, pub node_type: String } |  |  |  |  |

| `ReceiptGraph` | struct | ReceiptGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> } |  |  |  |  |

| `CspAc3` | struct |  |  |  |  |  |

| `CspAc3` | struct |  |  |  |  |  |

| `Engine` | enum | Engine { SatCdcl, CspAc3 } |  |  |  |  |

| `breed_id` | function | breed_id(self) -> &'static str |  |  |  |  |

| `encode_csp` | function | encode_csp(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> |  |  |  |  |

| `encode_sat` | function | encode_sat(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> |  |  |  |  |

| `forbid` | function | forbid(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `from_id` | function | from_id(s: &str) -> Option<Engine> |  |  |  |  |

| `generate_config` | function | generate_config( space: &FeatureSpace, query: &ConfigQuery, engine: Engine, ) -> Result<SemanticConfig, String> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `require` | function | require(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `ConfigQuery` | struct | ConfigQuery { pub require: Vec<String>, pub forbid: Vec<String> } |  |  |  |  |

| `SemanticConfig` | struct | SemanticConfig { pub engine_id: String, pub feasible: bool, pub genome: Option<Genome>, pub clash: Vec<String>, pub explanation: String, pub result: BreedResult } |  |  |  |  |

| `Engine` | enum | Engine { SatCdcl, CspAc3 } |  |  |  |  |

| `breed_id` | function | breed_id(self) -> &'static str |  |  |  |  |

| `encode_csp` | function | encode_csp(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> |  |  |  |  |

| `encode_sat` | function | encode_sat(space: &FeatureSpace, query: &ConfigQuery) -> Result<Contract, String> |  |  |  |  |

| `forbid` | function | forbid(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `from_id` | function | from_id(s: &str) -> Option<Engine> |  |  |  |  |

| `generate_config` | function | generate_config( space: &FeatureSpace, query: &ConfigQuery, engine: Engine, ) -> Result<SemanticConfig, String> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `require` | function | require(mut self, f: impl Into<String>) -> Self |  |  |  |  |

| `ConfigQuery` | struct | ConfigQuery { pub require: Vec<String>, pub forbid: Vec<String> } |  |  |  |  |

| `SemanticConfig` | struct | SemanticConfig { pub engine_id: String, pub feasible: bool, pub genome: Option<Genome>, pub clash: Vec<String>, pub explanation: String, pub result: BreedResult } |  |  |  |  |

| `CSP_AC3` | const | CSP_AC3: CspAc3 |  |  |  |  |

| `SAT_CDCL` | const | SAT_CDCL: SatCdcl |  |  |  |  |

| `Verdict` | enum | Verdict { Sat, Unsat, Unknown } |  |  |  |  |

| `from_json` | function | from_json(text: &str) -> Result<Contract, String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_sat` | function | is_sat(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(key: impl Into<String>, value: impl Into<String>) -> Self |  |  |  |  |

| `push` | function | push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) |  |  |  |  |

| `run_named` | function | run_named(id: &str, contract: &Contract) -> Result<BreedResult, String> |  |  |  |  |

| `supported_breeds` | function | supported_breeds() -> &'static [&'static str] |  |  |  |  |

| `tag` | function | tag(self) -> &'static str |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `BreedResult` | struct | BreedResult { pub breed: String, pub status: String, pub facts: Vec<Fact>, pub selected: String, pub explanation: String, pub inference_trace: Vec<TraceStep>, pub run_id: String, pub output_hash: String, pub inference_step_count: usize, pub rules_evaluated: usize } |  |  |  |  |

| `Contract` | struct | Contract { pub intent: String, pub facts: Vec<Fact> } |  |  |  |  |

| `Fact` | struct | Fact { pub key: String, pub value: String } |  |  |  |  |

| `Trace` | struct | Trace { steps: Vec<TraceStep> } |  |  |  |  |

| `TraceStep` | struct | TraceStep { pub step: usize, pub kind: String, pub detail: String, pub depth: usize } |  |  |  |  |

| `Breed` | trait |  |  |  |  |  |

| `csp::CspAc3` | use | csp::CspAc3 |  |  |  |  |

| `sat::SatCdcl` | use | sat::SatCdcl |  |  |  |  |

| `CSP_AC3` | const | CSP_AC3: CspAc3 |  |  |  |  |

| `SAT_CDCL` | const | SAT_CDCL: SatCdcl |  |  |  |  |

| `Verdict` | enum | Verdict { Sat, Unsat, Unknown } |  |  |  |  |

| `from_json` | function | from_json(text: &str) -> Result<Contract, String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `is_sat` | function | is_sat(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(key: impl Into<String>, value: impl Into<String>) -> Self |  |  |  |  |

| `push` | function | push(&mut self, kind: &str, detail: impl Into<String>, depth: usize) |  |  |  |  |

| `run_named` | function | run_named(id: &str, contract: &Contract) -> Result<BreedResult, String> |  |  |  |  |

| `supported_breeds` | function | supported_breeds() -> &'static [&'static str] |  |  |  |  |

| `tag` | function | tag(self) -> &'static str |  |  |  |  |

| `to_json` | function | to_json(&self) -> String |  |  |  |  |

| `BreedResult` | struct | BreedResult { pub breed: String, pub status: String, pub facts: Vec<Fact>, pub selected: String, pub explanation: String, pub inference_trace: Vec<TraceStep>, pub run_id: String, pub output_hash: String, pub inference_step_count: usize, pub rules_evaluated: usize } |  |  |  |  |

| `Contract` | struct | Contract { pub intent: String, pub facts: Vec<Fact> } |  |  |  |  |

| `Fact` | struct | Fact { pub key: String, pub value: String } |  |  |  |  |

| `Trace` | struct | Trace { steps: Vec<TraceStep> } |  |  |  |  |

| `TraceStep` | struct | TraceStep { pub step: usize, pub kind: String, pub detail: String, pub depth: usize } |  |  |  |  |

| `Breed` | trait |  |  |  |  |  |

| `csp::CspAc3` | use | csp::CspAc3 |  |  |  |  |

| `sat::SatCdcl` | use | sat::SatCdcl |  |  |  |  |

| `SatCdcl` | struct |  |  |  |  |  |

| `SatCdcl` | struct |  |  |  |  |  |

| `run_ga` | function | run_ga( eval: &mut impl Evaluator, space: &FeatureSpace, cfg: &GaConfig, ) -> Result<GaResult, GaError> |  |  |  |  |

| `GaConfig` | struct | GaConfig { pub population: usize, pub generations: usize, pub seed: u64, pub mutation_rate: f64, pub crossover_rate: f64, pub elitism: usize, pub tournament_k: usize } |  |  |  |  |

| `GaError` | struct | GaError { pub String } |  |  |  |  |

| `GaResult` | struct | GaResult { pub best_genome: Genome, pub best_eval: EvalResult, pub history: Vec<GenerationRecord>, pub evaluations: usize } |  |  |  |  |

| `GenerationRecord` | struct | GenerationRecord { pub index: usize, pub best_score: f64, pub mean_score: f64, pub best_features: Vec<String> } |  |  |  |  |

| `run_ga` | function | run_ga( eval: &mut impl Evaluator, space: &FeatureSpace, cfg: &GaConfig, ) -> Result<GaResult, GaError> |  |  |  |  |

| `GaConfig` | struct | GaConfig { pub population: usize, pub generations: usize, pub seed: u64, pub mutation_rate: f64, pub crossover_rate: f64, pub elitism: usize, pub tournament_k: usize } |  |  |  |  |

| `GaError` | struct | GaError { pub String } |  |  |  |  |

| `GaResult` | struct | GaResult { pub best_genome: Genome, pub best_eval: EvalResult, pub history: Vec<GenerationRecord>, pub evaluations: usize } |  |  |  |  |

| `GenerationRecord` | struct | GenerationRecord { pub index: usize, pub best_score: f64, pub mean_score: f64, pub best_features: Vec<String> } |  |  |  |  |

| `cargo_available` | function | cargo_available() -> bool |  |  |  |  |

| `count_compiler_messages` | function | count_compiler_messages(stdout: &str) -> (u64, u64) |  |  |  |  |

| `generic` | function | generic() -> Self |  |  |  |  |

| `new` | function | new(base_errors: u64, poison: I) -> Self |  |  |  |  |

| `resolves_from_output` | function | resolves_from_output(combined: &str) -> bool |  |  |  |  |

| `score_from` | function | score_from( weights: &ScoreWeights, builds: bool, resolves: bool, error_count: u64, n_features: usize, elapsed_s: f64, ) -> f64 |  |  |  |  |

| `with_extra_args` | function | with_extra_args(mut self, args: I) -> Self |  |  |  |  |

| `with_weights` | function | with_weights(mut self, weights: ScoreWeights) -> Self |  |  |  |  |

| `failed to select a version` | str_key | RESOLVE_FAIL_MARKERS = "failed to select a version" |  |  |  |  |

| `CargoEvaluator` | struct | CargoEvaluator { manifest_dir: PathBuf, weights: ScoreWeights, extra_args: Vec<String> } |  |  |  |  |

| `EvalResult` | struct | EvalResult { pub features: Vec<String>, pub resolves: bool, pub builds: bool, pub error_count: u64, pub warn_count: u64, pub elapsed_s: f64, pub score: f64, pub from_cache: bool } |  |  |  |  |

| `ScoreWeights` | struct | ScoreWeights { pub build_bonus: f64, pub resolve_bonus: f64, pub per_feature: f64, pub error_penalty: f64, pub time_penalty: f64 } |  |  |  |  |

| `SyntheticEvaluator` | struct | SyntheticEvaluator { weights: ScoreWeights, base_errors: u64, poison: BTreeMap<String, u64> } |  |  |  |  |

| `Evaluator` | trait |  |  |  |  |  |

| `cargo_available` | function | cargo_available() -> bool |  |  |  |  |

| `count_compiler_messages` | function | count_compiler_messages(stdout: &str) -> (u64, u64) |  |  |  |  |

| `generic` | function | generic() -> Self |  |  |  |  |

| `new` | function | new(base_errors: u64, poison: I) -> Self |  |  |  |  |

| `resolves_from_output` | function | resolves_from_output(combined: &str) -> bool |  |  |  |  |

| `score_from` | function | score_from( weights: &ScoreWeights, builds: bool, resolves: bool, error_count: u64, n_features: usize, elapsed_s: f64, ) -> f64 |  |  |  |  |

| `with_extra_args` | function | with_extra_args(mut self, args: I) -> Self |  |  |  |  |

| `with_weights` | function | with_weights(mut self, weights: ScoreWeights) -> Self |  |  |  |  |

| `failed to select a version` | str_key | RESOLVE_FAIL_MARKERS = "failed to select a version" |  |  |  |  |

| `CargoEvaluator` | struct | CargoEvaluator { manifest_dir: PathBuf, weights: ScoreWeights, extra_args: Vec<String> } |  |  |  |  |

| `EvalResult` | struct | EvalResult { pub features: Vec<String>, pub resolves: bool, pub builds: bool, pub error_count: u64, pub warn_count: u64, pub elapsed_s: f64, pub score: f64, pub from_cache: bool } |  |  |  |  |

| `ScoreWeights` | struct | ScoreWeights { pub build_bonus: f64, pub resolve_bonus: f64, pub per_feature: f64, pub error_penalty: f64, pub time_penalty: f64 } |  |  |  |  |

| `SyntheticEvaluator` | struct | SyntheticEvaluator { weights: ScoreWeights, base_errors: u64, poison: BTreeMap<String, u64> } |  |  |  |  |

| `Evaluator` | trait |  |  |  |  |  |

| `canonical` | function | canonical(&self, space: &FeatureSpace) -> Genome |  |  |  |  |

| `cargo_features_arg` | function | cargo_features_arg(&self) -> String |  |  |  |  |

| `contains` | function | contains(&self, feature: &str) -> bool |  |  |  |  |

| `empty` | function | empty() -> Self |  |  |  |  |

| `feature_set` | function | feature_set(&self) -> &BTreeSet<String> |  |  |  |  |

| `features` | function | features(&self) -> Vec<String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `key` | function | key(&self, space: &FeatureSpace) -> String |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(feats: I) -> Self |  |  |  |  |

| `random` | function | random(rng: &mut Rng, space: &FeatureSpace, p: f64) -> Self |  |  |  |  |

| `Genome` | struct | Genome { feats: BTreeSet<String> } |  |  |  |  |

| `canonical` | function | canonical(&self, space: &FeatureSpace) -> Genome |  |  |  |  |

| `cargo_features_arg` | function | cargo_features_arg(&self) -> String |  |  |  |  |

| `contains` | function | contains(&self, feature: &str) -> bool |  |  |  |  |

| `empty` | function | empty() -> Self |  |  |  |  |

| `feature_set` | function | feature_set(&self) -> &BTreeSet<String> |  |  |  |  |

| `features` | function | features(&self) -> Vec<String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `key` | function | key(&self, space: &FeatureSpace) -> String |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(feats: I) -> Self |  |  |  |  |

| `random` | function | random(rng: &mut Rng, space: &FeatureSpace, p: f64) -> Self |  |  |  |  |

| `Genome` | struct | Genome { feats: BTreeSet<String> } |  |  |  |  |

| `breeds::encode::{generate_config, ConfigQuery, Engine, SemanticConfig}` | use | breeds::encode::{generate_config, ConfigQuery, Engine, SemanticConfig} |  |  |  |  |

| `breeds::{run_named, Breed, BreedResult, Contract, Fact, Verdict}` | use | breeds::{run_named, Breed, BreedResult, Contract, Fact, Verdict} |  |  |  |  |

| `evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord}` | use | evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord} |  |  |  |  |

| `fitness::{ score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator, }` | use | fitness::{ score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator, } |  |  |  |  |

| `genome::Genome` | use | genome::Genome |  |  |  |  |

| `manifest::{feature_space_from_cargo_toml, ManifestError}` | use | manifest::{feature_space_from_cargo_toml, ManifestError} |  |  |  |  |

| `rng::Rng` | use | rng::Rng |  |  |  |  |

| `space::{FeatureSpace, SpaceError}` | use | space::{FeatureSpace, SpaceError} |  |  |  |  |

| `breeds::encode::{generate_config, ConfigQuery, Engine, SemanticConfig}` | use | breeds::encode::{generate_config, ConfigQuery, Engine, SemanticConfig} |  |  |  |  |

| `breeds::{run_named, Breed, BreedResult, Contract, Fact, Verdict}` | use | breeds::{run_named, Breed, BreedResult, Contract, Fact, Verdict} |  |  |  |  |

| `evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord}` | use | evolve::{run_ga, GaConfig, GaError, GaResult, GenerationRecord} |  |  |  |  |

| `fitness::{ score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator, }` | use | fitness::{ score_from, CargoEvaluator, EvalResult, Evaluator, ScoreWeights, SyntheticEvaluator, } |  |  |  |  |

| `genome::Genome` | use | genome::Genome |  |  |  |  |

| `manifest::{feature_space_from_cargo_toml, ManifestError}` | use | manifest::{feature_space_from_cargo_toml, ManifestError} |  |  |  |  |

| `rng::Rng` | use | rng::Rng |  |  |  |  |

| `space::{FeatureSpace, SpaceError}` | use | space::{FeatureSpace, SpaceError} |  |  |  |  |

| `ManifestError` | enum | ManifestError { Io(std::io::Error), NoFeaturesTable, Space(SpaceError) } |  |  |  |  |

| `feature_space_from_cargo_toml` | function | feature_space_from_cargo_toml( path: impl AsRef<Path>, include_default: bool, ) -> Result<FeatureSpace, ManifestError> |  |  |  |  |

| `feature_space_from_str` | function | feature_space_from_str( toml: &str, include_default: bool, ) -> Result<FeatureSpace, ManifestError> |  |  |  |  |

| `ManifestError` | enum | ManifestError { Io(std::io::Error), NoFeaturesTable, Space(SpaceError) } |  |  |  |  |

| `feature_space_from_cargo_toml` | function | feature_space_from_cargo_toml( path: impl AsRef<Path>, include_default: bool, ) -> Result<FeatureSpace, ManifestError> |  |  |  |  |

| `feature_space_from_str` | function | feature_space_from_str( toml: &str, include_default: bool, ) -> Result<FeatureSpace, ManifestError> |  |  |  |  |

| `Mode` | enum | Mode { DryRun, Real } |  |  |  |  |

| `to_json` | function | to_json( result: &GaResult, cfg: &GaConfig, mode: Mode, manifest: &str, universe: usize, ) -> String |  |  |  |  |

| `to_markdown` | function | to_markdown( result: &GaResult, cfg: &GaConfig, mode: Mode, manifest: &str, universe: usize, ) -> String |  |  |  |  |

| `Mode` | enum | Mode { DryRun, Real } |  |  |  |  |

| `to_json` | function | to_json( result: &GaResult, cfg: &GaConfig, mode: Mode, manifest: &str, universe: usize, ) -> String |  |  |  |  |

| `to_markdown` | function | to_markdown( result: &GaResult, cfg: &GaConfig, mode: Mode, manifest: &str, universe: usize, ) -> String |  |  |  |  |

| `below` | function | below(&mut self, n: usize) -> usize |  |  |  |  |

| `gen_bool` | function | gen_bool(&mut self, p: f64) -> bool |  |  |  |  |

| `new` | function | new(seed: u64) -> Self |  |  |  |  |

| `next_f64` | function | next_f64(&mut self) -> f64 |  |  |  |  |

| `next_u64` | function | next_u64(&mut self) -> u64 |  |  |  |  |

| `Rng` | struct | Rng { state: u64 } |  |  |  |  |

| `below` | function | below(&mut self, n: usize) -> usize |  |  |  |  |

| `gen_bool` | function | gen_bool(&mut self, p: f64) -> bool |  |  |  |  |

| `new` | function | new(seed: u64) -> Self |  |  |  |  |

| `next_f64` | function | next_f64(&mut self) -> f64 |  |  |  |  |

| `next_u64` | function | next_u64(&mut self) -> u64 |  |  |  |  |

| `Rng` | struct | Rng { state: u64 } |  |  |  |  |

| `SpaceError` | enum | SpaceError { DuplicateFeature(String), UnknownImplicationTarget(String, String), UnknownImplicationSource(String) } |  |  |  |  |

| `closure` | function | closure(&self, feats: &BTreeSet<String>) -> BTreeSet<String> |  |  |  |  |

| `contains` | function | contains(&self, name: &str) -> bool |  |  |  |  |

| `features` | function | features(&self) -> &[String] |  |  |  |  |

| `implications_of` | function | implications_of(&self, name: &str) -> impl Iterator<Item = &String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(features: F, implications: I) -> Result<Self, SpaceError> |  |  |  |  |

| `FeatureSpace` | struct | FeatureSpace { features: Vec<String>, implications: BTreeMap<String, BTreeSet<String>>, member: BTreeSet<String> } |  |  |  |  |

| `SpaceError` | enum | SpaceError { DuplicateFeature(String), UnknownImplicationTarget(String, String), UnknownImplicationSource(String) } |  |  |  |  |

| `closure` | function | closure(&self, feats: &BTreeSet<String>) -> BTreeSet<String> |  |  |  |  |

| `contains` | function | contains(&self, name: &str) -> bool |  |  |  |  |

| `features` | function | features(&self) -> &[String] |  |  |  |  |

| `implications_of` | function | implications_of(&self, name: &str) -> impl Iterator<Item = &String> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `new` | function | new(features: F, implications: I) -> Result<Self, SpaceError> |  |  |  |  |

| `FeatureSpace` | struct | FeatureSpace { features: Vec<String>, implications: BTreeMap<String, BTreeSet<String>>, member: BTreeSet<String> } |  |  |  |  |

| `diff_json_receipts` | function | diff_json_receipts(old_json: &str, new_json: &str) -> anyhow::Result<DiffResult> |  |  |  |  |

| `diff_receipts` | function | diff_receipts(old: &Receipt, new: &Receipt) -> DiffResult |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `DiffResult` | struct | DiffResult { pub added: Vec<OperationEvent>, pub removed: Vec<OperationEvent>, pub modified: Vec<ModifiedEvent> } |  |  |  |  |

| `ModifiedEvent` | struct | ModifiedEvent { pub id: String, pub old: OperationEvent, pub new: OperationEvent } |  |  |  |  |

| `DiffError` | enum | DiffError { Io( Json( FormatMismatch { left: String, right: String, }, ChainMismatch { left: Blake3Hash, right: Blake3Hash, }, Generic(String), } |  |  |  |  |

| `trace_diff` | function | trace_diff(left_path: &str, right_path: &str, f: F) -> DiffResult<T> |  |  |  |  |

| `DiffSummary` | struct | DiffSummary { pub left_event_count: usize, pub right_event_count: usize, pub total_differences: usize, pub added_count: usize, pub removed_count: usize, pub modified_count: usize } |  |  |  |  |

| `InstrumentedDiff` | trait |  |  |  |  |  |

| `build_graph` | function | build_graph(receipt: &Receipt) -> ReceiptGraph |  |  |  |  |

| `to_dot` | function | to_dot(graph: &ReceiptGraph) -> String |  |  |  |  |

| `to_json` | function | to_json(graph: &ReceiptGraph) -> anyhow::Result<String> |  |  |  |  |

| `GraphEdge` | struct | GraphEdge { pub from: String, pub to: String, pub weight: usize } |  |  |  |  |

| `GraphNode` | struct | GraphNode { pub id: String, pub label: String, pub event_count: usize } |  |  |  |  |

| `ReceiptGraph` | struct | ReceiptGraph { pub nodes: Vec<GraphNode>, pub edges: Vec<GraphEdge> } |  |  |  |  |

| `new` | function | new(receipt_path: &str, format: &str) -> Self |  |  |  |  |

| `trace_build` | function | trace_build(&self, event_count: usize, f: F) -> T |  |  |  |  |

| `trace_render` | function | trace_render(&self, node_count: usize, edge_count: usize, f: F) -> T |  |  |  |  |

| `VisualizeInstrumentation` | struct | VisualizeInstrumentation { pub receipt_path: String, pub format: String, pub start_time: Instant } |  |  |  |  |

| `VisualizeExt` | trait |  |  |  |  |  |

| `trace_catalog` | function | trace_catalog( filter_name: Option<&str>, filter_events: Option<usize>, f: F, ) -> T |  |  |  |  |

| `trace_catalog_scan` | function | trace_catalog_scan(source: &str, f: F) -> T |  |  |  |  |

| `generate_completions` | function | generate_completions(shell_name: &str) -> Result<()> |  |  |  |  |

| `try_dispatch_completion` | function | try_dispatch_completion() -> Result<bool> |  |  |  |  |

| `CompletionError` | enum | CompletionError { UnsupportedShell(String), Io( Generation(String), } |  |  |  |  |

| `Shell` | enum | Shell { Bash, Zsh, Fish, PowerShell, Elvish } |  |  |  |  |

| `completion` | function | completion(shell_name: String) -> anyhow::Result<()> |  |  |  |  |

| `trace_completion` | function | trace_completion(shell: Shell, f: F) -> T |  |  |  |  |

| `MiningError` | enum | MiningError { Him(String), Admission(String), OcelConversion(String) } |  |  |  |  |

| `discover_him` | function | discover_him(admitted: &AdmittedReceipt) -> Result<PetriNet, MiningError> |  |  |  |  |

| `receipt_to_ocel` | function | receipt_to_ocel(receipt: &Receipt) -> Result<OCEL, MiningError> |  |  |  |  |

| `PredictionError` | enum | PredictionError { InvalidTopK(usize), Wasm4pm(String) } |  |  |  |  |

| `predict_next` | function | predict_next( admitted: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, PredictionError> |  |  |  |  |

| `predict_next_with_model` | function | predict_next_with_model( model: &AdmittedReceipt, current_trace: &AdmittedReceipt, top_k: usize, ) -> Result<PredictionReport, PredictionError> |  |  |  |  |

| `ActivityPrediction` | struct | ActivityPrediction { pub activity: String, pub confidence: f64 } |  |  |  |  |

| `PredictionReport` | struct | PredictionReport { pub predictions: Vec<ActivityPrediction>, pub context_length: usize, pub model_type: String } |  |  |  |  |

| `from_receipt` | function | from_receipt(receipt: &Receipt, uri: &Url, text: &str) -> Self |  |  |  |  |

| `handle_definition` | function | handle_definition(pos: Position, index: &ReceiptIndex) -> Option<Vec<Location>> |  |  |  |  |

| `handle_hover` | function | handle_hover(pos: Position, index: &ReceiptIndex) -> Option<Hover> |  |  |  |  |

| `ObjectRefLocation` | struct | ObjectRefLocation { pub event_idx: usize, pub seq: u64 } |  |  |  |  |

| `ReceiptIndex` | struct | ReceiptIndex { pub receipt: Receipt, pub uri: Url, pub text: String, pub events: Vec<ReceiptSymbol>, pub object_refs: HashMap<String, Vec<ObjectRefLocation>> } |  |  |  |  |

| `ReceiptSymbol` | struct | ReceiptSymbol { pub event_id: String, pub seq: u64, pub event_type: String, pub range: Range, pub objects: Vec<ObjectRef>, pub payload_commitment: Blake3Hash } |  |  |  |  |

| `MutationKind` | enum | MutationKind { EventDrop, EventReorder, TypeChange, PayloadFlip } |  |  |  |  |

| `all_operators` | function | all_operators() -> Vec<Box<dyn MutationOperator>> |  |  |  |  |

| `AppliedMutation` | struct | AppliedMutation { pub kind: MutationKind, pub target_seq: u64, pub mutated_receipt: Receipt } |  |  |  |  |

| `EventDropOperator` | struct |  |  |  |  |  |

| `EventReorderOperator` | struct |  |  |  |  |  |

| `PayloadFlipOperator` | struct |  |  |  |  |  |

| `TypeChangeOperator` | struct |  |  |  |  |  |

| `MutationOperator` | trait |  |  |  |  |  |

| `DEFAULT_SNIPPETS` | const | DEFAULT_SNIPPETS: &str |  |  |  |  |

| `TEST_FN_TEMPLATE` | const | TEST_FN_TEMPLATE: &str |  |  |  |  |

| `TEST_MODULE_TEMPLATE` | const | TEST_MODULE_TEMPLATE: &str |  |  |  |  |

| `filter_by_tag` | function | filter_by_tag(&self, tag: &str) -> Vec<&Snippet> |  |  |  |  |

| `find_by_name` | function | find_by_name(&self, pattern: &str) -> Vec<&Snippet> |  |  |  |  |

| `format_snippet` | function | format_snippet(&self, snippet: &Snippet) -> String |  |  |  |  |

| `from_json` | function | from_json(json: &str) -> Result<Self> |  |  |  |  |

| `generate_test_function` | function | generate_test_function(&self, pattern_name: &str, events: Vec<serde_json::Value>, expected_verdict: &str, expected_failure_stage: Option<&str> ) -> Result<String> |  |  |  |  |

| `generate_test_module` | function | generate_test_module(&self, fixture_set_name: &str, tests: Vec<String>) -> Result<String> |  |  |  |  |

| `main` | function | main() -> Result<()> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `CodegenEngine` | struct | CodegenEngine { tera: Tera } |  |  |  |  |

| `Snippet` | struct | Snippet { pub name: String, pub tags: Vec<String>, pub description: String, pub language: String, pub imports: Vec<String>, pub code: String } |  |  |  |  |

| `SnippetRegistry` | struct | SnippetRegistry { pub snippets: Vec<Snippet> } |  |  |  |  |

| `ArbitraryOperationEvent` | struct | ArbitraryOperationEvent { pub OperationEvent } |  |  |  |  |

| `ArbitraryReceipt` | struct | ArbitraryReceipt { pub Receipt } |  |  |  |  |

| `all` | function | all(&self) -> Vec<Fixture> |  |  |  |  |

| `delete_by_name` | function | delete_by_name(&mut self, name: &str) -> Result<bool> |  |  |  |  |

| `get_by_chain_hash` | function | get_by_chain_hash(&self, chain_hash: &str) -> Option<Fixture> |  |  |  |  |

| `get_by_name` | function | get_by_name(&self, name: &str) -> Option<Fixture> |  |  |  |  |

| `insert` | function | insert(&mut self, name: &str, tags: &[&str], receipt: Receipt) -> Result<Fixture> |  |  |  |  |

| `is_empty` | function | is_empty(&self) -> bool |  |  |  |  |

| `len` | function | len(&self) -> usize |  |  |  |  |

| `open` | function | open(path: impl Into<PathBuf>) -> Result<Self> |  |  |  |  |

| `reindex` | function | reindex(&mut self) -> Result<()> |  |  |  |  |

| `save` | function | save(&self) -> Result<()> |  |  |  |  |

| `search` | function | search(&self, query: &FixtureQuery) -> Vec<Fixture> |  |  |  |  |

| `Fixture` | struct | Fixture { pub id: String, pub name: String, pub tags: Vec<String>, pub event_count: usize, pub event_types: Vec<String>, pub chain_hash: String, pub inserted_at: String, pub receipt: Receipt } |  |  |  |  |

| `FixtureDatabase` | struct | FixtureDatabase { path: PathBuf, db: JsonDb, index_by_name: BTreeMap<String, usize>, index_by_event_count: BTreeMap<usize, Vec<usize>>, index_by_chain_hash: BTreeMap<String, usize> } |  |  |  |  |

| `FixtureQuery` | struct | FixtureQuery { pub name_contains: Option<String>, pub tag: Option<String>, pub min_events: Option<usize>, pub max_events: Option<usize>, pub event_type: Option<String>, pub limit: Option<usize> } |  |  |  |  |

| `SLO_AVAILABILITY_PCT` | const | SLO_AVAILABILITY_PCT: f64 |  |  |  |  |

| `SLO_ERROR_RATE_PCT` | const | SLO_ERROR_RATE_PCT: f64 |  |  |  |  |

| `SLO_LATENCY_P99_MS` | const | SLO_LATENCY_P99_MS: f64 |  |  |  |  |

| `SloViolation` | enum | SloViolation { LatencyP99 { observed_ms: f64, threshold_ms: f64 }, ErrorRate { observed_pct: f64, threshold_pct: f64 }, Availability { observed_pct: f64, threshold_pct: f64 } } |  |  |  |  |

| `check_slo` | function | check_slo(&self) -> anyhow::Result<(), SloViolation> |  |  |  |  |

| `compute_sli` | function | compute_sli(&self) -> anyhow::Result<ServiceLevelIndicators> |  |  |  |  |

| `new` | function | new() -> Self |  |  |  |  |

| `new_noop` | function | new_noop() -> Self |  |  |  |  |

| `record_receipt_verified` | function | record_receipt_verified(&self, accepted: bool) |  |  |  |  |

| `record_stage_error` | function | record_stage_error(&self, _stage: &str) |  |  |  |  |

| `record_stage_latency` | function | record_stage_latency(&self, _stage: &str, duration: Duration) |  |  |  |  |

| `render` | function | render(&self) -> String |  |  |  |  |

| `with_window` | function | with_window(window_duration: Duration) -> Self |  |  |  |  |

| `MetricsCollector` | struct | MetricsCollector { inner: Mutex<CollectorState>, window_duration: Duration } |  |  |  |  |

| `PrometheusExporter` | struct | PrometheusExporter { collector: &'a MetricsCollector } |  |  |  |  |

| `ServiceLevelIndicators` | struct | ServiceLevelIndicators { pub latency_p99_ms: f64, pub error_rate_pct: f64, pub availability_pct: f64 } |  |  |  |  |

| `run` | function | run() -> Result<()> |  |  |  |  |

| `compare_baseline` | function | compare_baseline(&mut self, bench_id: &str, current: f64, baseline: f64) -> anyhow::Result<()> |  |  |  |  |

| `load_baseline_value` | function | load_baseline_value(path: impl AsRef<Path>) -> anyhow::Result<f64> |  |  |  |  |

| `new` | function | new(assembler: &'a mut ChainAssembler, counter: &'a mut SeqCounter) -> Self |  |  |  |  |

| `observe_criterion_bench` | function | observe_criterion_bench(&mut self, criterion_root: &str, bench_id: &str) -> anyhow::Result<f64> |  |  |  |  |

| `record_throughput` | function | record_throughput(&mut self, bench_id: &str, ops_per_sec: f64) -> anyhow::Result<()> |  |  |  |  |

| `ThroughputObserver` | struct | ThroughputObserver { pub assembler: &'a mut ChainAssembler, pub counter: &'a mut SeqCounter } |  |  |  |  |

| `CRITERION_CUSTOM_CSS` | const | CRITERION_CUSTOM_CSS: &str |  |  |  |  |

| `create` | str_key | SEQUENTIAL_ACTIVITIES = "create" |  |  |  |  |

| `release` | str_key | INTERLEAVED_ACTIVITIES = "release" |  |  |  |  |

| `to_noun_verb_error` | function | to_noun_verb_error(err: AffidavitError) -> NounVerbError |  |  |  |  |


<!-- ============================================================= -->
<!-- AGENT-FORBIDDEN-END: nothing below this line may describe     -->
<!-- code behavior.                                                -->
<!-- ============================================================= -->
