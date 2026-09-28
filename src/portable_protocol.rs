//! Structural verification for foreign portable-consequence receipts.
//!
//! This module certifies relationships only. It does not decide whether an
//! authority issuer is trusted, whether an action should execute, or whether a
//! foreign system is honest.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeignAuthority<'a> {
    pub decision_id: &'a str,
    pub consequence_digest: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeignObservation<'a> {
    pub consequence_digest: &'a str,
    pub observation_digest: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeignReceipt<'a> {
    pub authority_decision_id: &'a str,
    pub consequence_digest: &'a str,
    pub observation_digest: &'a str,
    pub replay_key: &'a str,
}

pub fn verify_foreign_receipt(
    consequence_digest: &str,
    authority: ForeignAuthority<'_>,
    observation: ForeignObservation<'_>,
    receipt: ForeignReceipt<'_>,
) -> Result<(), &'static str> {
    if consequence_digest.is_empty() { return Err("REFUSED:CONSEQUENCE_IDENTITY_REQUIRED"); }
    if authority.decision_id.is_empty() { return Err("REFUSED:AUTHORITY_DECISION_ID_REQUIRED"); }
    if authority.consequence_digest != consequence_digest { return Err("REFUSED:AUTHORITY_SCOPE_MISMATCH"); }
    if observation.consequence_digest != consequence_digest { return Err("REFUSED:OBSERVATION_CONSEQUENCE_MISMATCH"); }
    if receipt.authority_decision_id != authority.decision_id { return Err("REFUSED:RECEIPT_AUTHORITY_MISMATCH"); }
    if receipt.consequence_digest != consequence_digest { return Err("REFUSED:RECEIPT_CONSEQUENCE_MISMATCH"); }
    if receipt.observation_digest != observation.observation_digest { return Err("REFUSED:RECEIPT_OBSERVATION_MISMATCH"); }
    if receipt.replay_key.is_empty() { return Err("REFUSED:REPLAY_KEY_REQUIRED"); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn authority<'a>(d: &'a str) -> ForeignAuthority<'a> { ForeignAuthority { decision_id: "a", consequence_digest: d } }
    fn observation<'a>(d: &'a str) -> ForeignObservation<'a> { ForeignObservation { consequence_digest: d, observation_digest: "o" } }
    fn receipt<'a>(d: &'a str) -> ForeignReceipt<'a> { ForeignReceipt { authority_decision_id: "a", consequence_digest: d, observation_digest: "o", replay_key: "r" } }
    #[test] fn foreign_identity_is_not_required() { assert_eq!(verify_foreign_receipt("d",authority("d"),observation("d"),receipt("d")),Ok(())); }
    #[test] fn forged_authority_binding_is_refused() { let mut r=receipt("d"); r.authority_decision_id="other"; assert_eq!(verify_foreign_receipt("d",authority("d"),observation("d"),r),Err("REFUSED:RECEIPT_AUTHORITY_MISMATCH")); }
    #[test] fn receipt_for_another_consequence_is_refused() { assert_eq!(verify_foreign_receipt("d",authority("d"),observation("d"),receipt("other")),Err("REFUSED:RECEIPT_CONSEQUENCE_MISMATCH")); }
}
