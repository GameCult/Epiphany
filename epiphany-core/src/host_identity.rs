use anyhow::{Result, anyhow, bail};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const HOST_IDENTITY_TRUST_ANCHOR_TYPE: &str = "epiphany.host_identity_trust_anchor.v0";

const ID_DOMAIN: &[u8] = b"epiphany.host-incarnation.identity.v0\0";
const SIGNATURE_DOMAIN: &[u8] = b"epiphany.host-incarnation.signature.v0\0";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostIdentitySignature {
    pub identity_id: String,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostIdentityTrustAnchorEntry {
    pub schema_version: String,
    pub identity_id: String,
    pub public_key: Vec<u8>,
    pub assurance: String,
    pub identity_created_at: String,
    pub source_identity_record_sha256: String,
}

pub fn verify_host_identity_trust_anchor_signature(
    anchor: &HostIdentityTrustAnchorEntry,
    purpose: &str,
    payload: &[u8],
    proof: &HostIdentitySignature,
) -> Result<()> {
    if anchor.schema_version != HOST_IDENTITY_TRUST_ANCHOR_TYPE
        || anchor.public_key.len() != 32
        || identity_id(&anchor.public_key) != anchor.identity_id
        || proof.identity_id != anchor.identity_id
    {
        bail!("host identity trust anchor or signature identity is invalid");
    }
    let public_key: [u8; 32] = anchor
        .public_key
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("host identity trust anchor public key has invalid length"))?;
    let signature = Signature::from_slice(&proof.signature)
        .map_err(|_| anyhow!("host identity signature has invalid length"))?;
    VerifyingKey::from_bytes(&public_key)?
        .verify(&signing_message(purpose, payload)?, &signature)
        .map_err(|_| anyhow!("host identity signature verification failed"))
}

fn identity_id(public_key: &[u8]) -> String {
    format!("{:x}", Sha256::digest([ID_DOMAIN, public_key].concat()))
}

fn signing_message(purpose: &str, payload: &[u8]) -> Result<Vec<u8>> {
    if purpose.trim().is_empty() {
        bail!("host identity signature purpose must not be empty");
    }
    let mut message =
        Vec::with_capacity(SIGNATURE_DOMAIN.len() + purpose.len() + payload.len() + 16);
    message.extend_from_slice(SIGNATURE_DOMAIN);
    message.extend_from_slice(&(purpose.len() as u64).to_be_bytes());
    message.extend_from_slice(purpose.as_bytes());
    message.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    message.extend_from_slice(payload);
    Ok(message)
}

#[cfg(test)]
mod fixtures {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use std::path::Path;

    pub(crate) struct HostIdentitySigner {
        identity_id: String,
        public_key: Vec<u8>,
        signing_key: SigningKey,
    }

    impl HostIdentitySigner {
        pub(crate) fn identity_id(&self) -> &str {
            &self.identity_id
        }

        pub(crate) fn sign(&self, purpose: &str, payload: &[u8]) -> Result<HostIdentitySignature> {
            let message = signing_message(purpose, payload)?;
            Ok(HostIdentitySignature {
                identity_id: self.identity_id.clone(),
                signature: self.signing_key.sign(&message).to_bytes().to_vec(),
            })
        }
    }

    pub(crate) fn test_host_identity(label: &str) -> HostIdentitySigner {
        let seed: [u8; 32] = Sha256::digest(
            [b"epiphany.test-host-identity.v0\0".as_slice(), label.as_bytes()].concat(),
        )
        .into();
        let signing_key = SigningKey::from_bytes(&seed);
        let public_key = signing_key.verifying_key().to_bytes().to_vec();
        HostIdentitySigner {
            identity_id: identity_id(&public_key),
            public_key,
            signing_key,
        }
    }

    pub(crate) fn enroll_host_identity_at(path: &Path) -> Result<HostIdentitySigner> {
        Ok(test_host_identity(&path.display().to_string()))
    }

    pub(crate) fn write_test_host_identity_anchor(
        signer: &HostIdentitySigner,
        output: &Path,
    ) -> Result<HostIdentityTrustAnchorEntry> {
        let anchor = HostIdentityTrustAnchorEntry {
            schema_version: HOST_IDENTITY_TRUST_ANCHOR_TYPE.into(),
            identity_id: signer.identity_id.clone(),
            public_key: signer.public_key.clone(),
            assurance: "test-only".into(),
            identity_created_at: "2026-08-23T00:00:00Z".into(),
            source_identity_record_sha256: format!(
                "sha256-{:x}",
                Sha256::digest(&signer.public_key)
            ),
        };
        if let Some(parent) = output.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(output, rmp_serde::to_vec(&anchor)?)?;
        Ok(anchor)
    }

    pub(crate) fn export_host_identity_trust_anchor(
        signer: &HostIdentitySigner,
        output: &Path,
    ) -> Result<HostIdentityTrustAnchorEntry> {
        write_test_host_identity_anchor(signer, output)
    }
}

#[cfg(test)]
pub(crate) use fixtures::{
    HostIdentitySigner, enroll_host_identity_at, export_host_identity_trust_anchor,
    test_host_identity, write_test_host_identity_anchor,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_binds_purpose_payload_and_pinned_identity() -> Result<()> {
        let signer = test_host_identity("bifrost");
        let anchor = write_test_host_identity_anchor(
            &signer,
            &tempfile::tempdir()?.path().join("anchor.msgpack"),
        )?;
        let proof = signer.sign("feedback", b"payload")?;
        verify_host_identity_trust_anchor_signature(&anchor, "feedback", b"payload", &proof)?;
        assert!(
            verify_host_identity_trust_anchor_signature(&anchor, "other", b"payload", &proof)
                .is_err()
        );
        let substituted = test_host_identity("substituted").sign("feedback", b"payload")?;
        assert!(
            verify_host_identity_trust_anchor_signature(
                &anchor,
                "feedback",
                b"payload",
                &substituted,
            )
            .is_err()
        );
        Ok(())
    }
}
