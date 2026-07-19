use anyhow::{Context, Result, anyhow, bail};
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const HOST_IDENTITY_TYPE: &str = "epiphany.host_incarnation_identity.v0";
pub const HOST_IDENTITY_SCHEMA_VERSION: &str = "epiphany.host_incarnation_identity.v0";
pub const HOST_IDENTITY_KEY: &str = "host-incarnation";
pub const HOST_IDENTITY_TRUST_ANCHOR_TYPE: &str = "epiphany.host_identity_trust_anchor.v0";
pub const HOST_IDENTITY_TRUST_ANCHOR_KEY: &str = "host-incarnation-public";
pub const WINDOWS_HOST_IDENTITY_ASSURANCE: &str = "os_user_installation_bound_best_effort";
pub const LINUX_HOST_IDENTITY_ASSURANCE: &str = "os_installation_file_bound_cloneable_baseline";

const ID_DOMAIN: &[u8] = b"epiphany.host-incarnation.identity.v0\0";
const SIGNATURE_DOMAIN: &[u8] = b"epiphany.host-incarnation.signature.v0\0";

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.host_incarnation_identity.v0",
    schema = "HostIncarnationIdentityEntry"
)]
pub struct HostIncarnationIdentityEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub identity_id: String,
    #[cultcache(key = 2)]
    pub public_key: Vec<u8>,
    #[cultcache(key = 3)]
    pub protected_private_seed: Vec<u8>,
    #[cultcache(key = 4)]
    pub protector_kind: String,
    #[cultcache(key = 5)]
    pub protector_binding: String,
    #[cultcache(key = 6)]
    pub protector_version: String,
    #[cultcache(key = 7)]
    pub assurance: String,
    #[cultcache(key = 8)]
    pub created_at: String,
    #[cultcache(key = 9)]
    pub enrollment_nonce: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostIdentitySignature {
    pub identity_id: String,
    pub signature: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.host_identity_trust_anchor.v0",
    schema = "HostIdentityTrustAnchorEntry"
)]
pub struct HostIdentityTrustAnchorEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub identity_id: String,
    #[cultcache(key = 2)]
    pub public_key: Vec<u8>,
    #[cultcache(key = 3)]
    pub assurance: String,
    #[cultcache(key = 4)]
    pub identity_created_at: String,
    #[cultcache(key = 5)]
    pub source_identity_record_sha256: String,
}

/// A deliberately narrow signing handle. The private seed is never exposed by
/// the public API and is unprotected only while this value exists.
pub struct HostIdentitySigner {
    entry: HostIncarnationIdentityEntry,
    signing_key: SigningKey,
}

impl HostIdentitySigner {
    pub fn entry(&self) -> &HostIncarnationIdentityEntry {
        &self.entry
    }

    pub fn sign(&self, purpose: &str, payload: &[u8]) -> Result<HostIdentitySignature> {
        let message = signing_message(purpose, payload)?;
        Ok(HostIdentitySignature {
            identity_id: self.entry.identity_id.clone(),
            signature: self.signing_key.sign(&message).to_bytes().to_vec(),
        })
    }
}

pub fn default_host_identity_store_path() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        let root = std::env::var_os("LOCALAPPDATA")
            .ok_or_else(|| anyhow!("LOCALAPPDATA is unavailable; host identity path is unknown"))?;
        return Ok(PathBuf::from(root)
            .join("GameCult")
            .join("Epiphany")
            .join("host-identity.ccmp"));
    }
    #[cfg(target_os = "linux")]
    {
        let root = if let Some(root) = std::env::var_os("XDG_STATE_HOME") {
            PathBuf::from(root)
        } else {
            let home = std::env::var_os("HOME")
                .ok_or_else(|| anyhow!("HOME is unavailable; host identity path is unknown"))?;
            PathBuf::from(home).join(".local").join("state")
        };
        return Ok(root
            .join("gamecult")
            .join("epiphany")
            .join("host-identity.ccmp"));
    }
    #[allow(unreachable_code)]
    Err(anyhow!(
        "host identity enrollment is unsupported on this operating system"
    ))
}

pub fn enroll_default_host_identity() -> Result<HostIdentitySigner> {
    enroll_host_identity_at(&default_host_identity_store_path()?)
}

pub fn open_default_host_identity() -> Result<HostIdentitySigner> {
    open_host_identity_at(&default_host_identity_store_path()?)
}

pub fn export_host_identity_trust_anchor(
    signer: &HostIdentitySigner,
    output: &Path,
) -> Result<HostIdentityTrustAnchorEntry> {
    let anchor = host_identity_trust_anchor(signer)?;
    let entry = signer.entry();
    prepare_parent(output)?;
    let envelope = CultCacheEnvelope {
        key: HOST_IDENTITY_TRUST_ANCHOR_KEY.into(),
        r#type: HOST_IDENTITY_TRUST_ANCHOR_TYPE.into(),
        payload: rmp_serde::to_vec(&anchor)?,
        stored_at: entry.created_at.clone(),
        schema_id: Some(HOST_IDENTITY_TRUST_ANCHOR_TYPE.into()),
    };
    let mut backing = SingleFileMessagePackBackingStore::new(output);
    let existing = backing.pull_all()?;
    match existing.as_slice() {
        [] => backing.push(&envelope)?,
        [current] if current == &envelope => {}
        _ => bail!("host identity trust anchor output already contains different state"),
    }
    Ok(anchor)
}

pub fn export_raw_host_identity_trust_anchor(
    signer: &HostIdentitySigner,
    output: &Path,
) -> Result<HostIdentityTrustAnchorEntry> {
    let anchor = host_identity_trust_anchor(signer)?;
    let bytes = rmp_serde::to_vec(&anchor)?;
    prepare_parent(output)?;
    if output.exists() {
        if std::fs::read(output)? != bytes {
            bail!("raw host identity trust anchor output already contains different state");
        }
        return Ok(anchor);
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(anchor)
}

fn host_identity_trust_anchor(signer: &HostIdentitySigner) -> Result<HostIdentityTrustAnchorEntry> {
    let entry = signer.entry();
    let anchor = HostIdentityTrustAnchorEntry {
        schema_version: HOST_IDENTITY_TRUST_ANCHOR_TYPE.into(),
        identity_id: entry.identity_id.clone(),
        public_key: entry.public_key.clone(),
        assurance: entry.assurance.clone(),
        identity_created_at: entry.created_at.clone(),
        source_identity_record_sha256: format!(
            "sha256-{:x}",
            Sha256::digest(rmp_serde::to_vec(entry)?)
        ),
    };
    Ok(anchor)
}

/// Performs one immutable enrollment. Existing state is never reused by this
/// operation and malformed state is never replaced.
pub fn enroll_host_identity_at(store_path: &Path) -> Result<HostIdentitySigner> {
    if store_path.exists() {
        bail!(
            "host identity store {} already exists; enrollment is immutable",
            store_path.display()
        );
    }
    prepare_parent(store_path)?;

    let mut seed = [0_u8; 32];
    rand_core::RngCore::fill_bytes(&mut OsRng, &mut seed);
    let signing_key = SigningKey::from_bytes(&seed);
    let mut nonce = [0_u8; 32];
    rand_core::RngCore::fill_bytes(&mut OsRng, &mut nonce);
    let public_key = signing_key.verifying_key().to_bytes();
    let identity_id = identity_id(&public_key);
    let binding = platform_binding()?;
    let protected_private_seed = protect_seed(signing_key.as_bytes(), &binding)?;
    let entry = HostIncarnationIdentityEntry {
        schema_version: HOST_IDENTITY_SCHEMA_VERSION.to_string(),
        identity_id,
        public_key: public_key.to_vec(),
        protected_private_seed,
        protector_kind: platform_protector_kind().to_string(),
        protector_binding: binding,
        protector_version: "v1".to_string(),
        assurance: platform_assurance().to_string(),
        created_at: chrono::Utc::now().to_rfc3339(),
        enrollment_nonce: nonce.to_vec(),
    };
    validate_entry(&entry)?;
    let envelope = entry_envelope(&entry)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if !backing.insert_entry_if_absent(envelope)? {
        bail!("host identity enrollment lost an atomic create race; refusing to reuse state");
    }
    harden_store_permissions(store_path)?;
    open_host_identity_at(store_path)
}

pub fn open_host_identity_at(store_path: &Path) -> Result<HostIdentitySigner> {
    if !store_path.is_file() {
        bail!(
            "host identity store {} does not exist",
            store_path.display()
        );
    }
    let entries = SingleFileMessagePackBackingStore::new(store_path).pull_all()?;
    if entries.len() != 1 {
        bail!("host identity store must contain exactly one immutable envelope");
    }
    let envelope = &entries[0];
    if envelope.r#type != HOST_IDENTITY_TYPE || envelope.key != HOST_IDENTITY_KEY {
        bail!("host identity store contains an unexpected type or key");
    }
    let entry: HostIncarnationIdentityEntry = rmp_serde::from_slice(&envelope.payload)
        .context("host identity payload is malformed MessagePack")?;
    validate_entry(&entry)?;
    if entry.protector_kind != platform_protector_kind() || entry.assurance != platform_assurance()
    {
        bail!("host identity protector does not belong to this platform implementation");
    }
    let current_binding = platform_binding()?;
    if entry.protector_binding != current_binding {
        bail!("host identity protector binding does not match this OS installation");
    }
    let seed = unprotect_seed(&entry.protected_private_seed, &current_binding)?;
    let seed: [u8; 32] = seed
        .try_into()
        .map_err(|_| anyhow!("unprotected host identity seed has invalid length"))?;
    let signing_key = SigningKey::from_bytes(&seed);
    if signing_key.verifying_key().to_bytes().as_slice() != entry.public_key.as_slice() {
        bail!("host identity private seed does not match the enrolled public key");
    }
    Ok(HostIdentitySigner { entry, signing_key })
}

pub fn verify_host_identity_signature(
    entry: &HostIncarnationIdentityEntry,
    purpose: &str,
    payload: &[u8],
    proof: &HostIdentitySignature,
) -> Result<()> {
    validate_entry(entry)?;
    if proof.identity_id != entry.identity_id {
        bail!("host identity signature names a different identity");
    }
    let public_key: [u8; 32] = entry
        .public_key
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("host identity public key has invalid length"))?;
    let signature = Signature::from_slice(&proof.signature)
        .map_err(|_| anyhow!("host identity signature has invalid length"))?;
    VerifyingKey::from_bytes(&public_key)?
        .verify(&signing_message(purpose, payload)?, &signature)
        .map_err(|_| anyhow!("host identity signature verification failed"))
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

fn validate_entry(entry: &HostIncarnationIdentityEntry) -> Result<()> {
    if entry.schema_version != HOST_IDENTITY_SCHEMA_VERSION
        || entry.public_key.len() != 32
        || entry.enrollment_nonce.len() != 32
        || entry.protected_private_seed.is_empty()
        || entry.protector_version != "v1"
    {
        bail!("host identity entry violates its fixed schema");
    }
    chrono::DateTime::parse_from_rfc3339(&entry.created_at)
        .map_err(|_| anyhow!("host identity created_at is not RFC3339"))?;
    if identity_id(&entry.public_key) != entry.identity_id {
        bail!("host identity id does not match its public key");
    }
    Ok(())
}

fn identity_id(public_key: &[u8]) -> String {
    hex(&Sha256::digest([ID_DOMAIN, public_key].concat()))
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

fn entry_envelope(entry: &HostIncarnationIdentityEntry) -> Result<CultCacheEnvelope> {
    Ok(CultCacheEnvelope {
        key: HOST_IDENTITY_KEY.to_string(),
        r#type: HOST_IDENTITY_TYPE.to_string(),
        payload: rmp_serde::to_vec(entry)?,
        stored_at: entry.created_at.clone(),
        schema_id: Some(HOST_IDENTITY_SCHEMA_VERSION.to_string()),
    })
}

fn prepare_parent(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("host identity path has no parent"))?;
    std::fs::create_dir_all(parent)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn harden_store_permissions(_path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(_path, std::fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

#[cfg(windows)]
fn platform_protector_kind() -> &'static str {
    "windows_dpapi_current_user"
}
#[cfg(target_os = "linux")]
fn platform_protector_kind() -> &'static str {
    "linux_file_mode_machine_id_binding"
}
#[cfg(windows)]
fn platform_assurance() -> &'static str {
    WINDOWS_HOST_IDENTITY_ASSURANCE
}
#[cfg(target_os = "linux")]
fn platform_assurance() -> &'static str {
    LINUX_HOST_IDENTITY_ASSURANCE
}

#[cfg(windows)]
fn platform_binding() -> Result<String> {
    Ok("dpapi-current-user:epiphany-host-identity-v1".to_string())
}

#[cfg(target_os = "linux")]
fn platform_binding() -> Result<String> {
    let machine_id = std::fs::read_to_string("/etc/machine-id")
        .or_else(|_| std::fs::read_to_string("/var/lib/dbus/machine-id"))
        .context("Linux machine-id is unavailable")?;
    let machine_id = machine_id.trim();
    if machine_id.is_empty() {
        bail!("Linux machine-id is empty");
    }
    Ok(format!(
        "machine-id-sha256:{}",
        hex(&Sha256::digest(machine_id.as_bytes()))
    ))
}

#[cfg(windows)]
fn protect_seed(seed: &[u8; 32], binding: &str) -> Result<Vec<u8>> {
    dpapi(seed, binding, true)
}

#[cfg(windows)]
fn unprotect_seed(protected: &[u8], binding: &str) -> Result<Vec<u8>> {
    dpapi(protected, binding, false)
}

#[cfg(windows)]
fn dpapi(input: &[u8], binding: &str, protect: bool) -> Result<Vec<u8>> {
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
    };
    let mut input = input.to_vec();
    let mut entropy = Sha256::digest(
        [
            b"epiphany-host-identity-dpapi-v1\0".as_slice(),
            binding.as_bytes(),
        ]
        .concat(),
    )
    .to_vec();
    let input_blob = CRYPT_INTEGER_BLOB {
        cbData: input.len() as u32,
        pbData: input.as_mut_ptr(),
    };
    let entropy_blob = CRYPT_INTEGER_BLOB {
        cbData: entropy.len() as u32,
        pbData: entropy.as_mut_ptr(),
    };
    let mut output = CRYPT_INTEGER_BLOB {
        cbData: 0,
        pbData: std::ptr::null_mut(),
    };
    let ok = unsafe {
        if protect {
            CryptProtectData(
                &input_blob,
                std::ptr::null(),
                &entropy_blob,
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input_blob,
                std::ptr::null_mut(),
                &entropy_blob,
                std::ptr::null(),
                std::ptr::null(),
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut output,
            )
        }
    };
    if ok == 0 {
        return Err(std::io::Error::last_os_error())
            .context("DPAPI host identity operation failed");
    }
    let bytes =
        unsafe { std::slice::from_raw_parts(output.pbData, output.cbData as usize).to_vec() };
    unsafe {
        LocalFree(output.pbData.cast());
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
fn protect_seed(seed: &[u8; 32], binding: &str) -> Result<Vec<u8>> {
    // This is intentionally labeled a cloneable baseline, not encryption. File
    // modes protect the seed locally; the binding detects a copied installation.
    let mask = Sha256::digest(
        [
            b"epiphany-linux-host-seed-v1\0".as_slice(),
            binding.as_bytes(),
        ]
        .concat(),
    );
    Ok(seed
        .iter()
        .zip(mask)
        .map(|(byte, mask)| byte ^ mask)
        .collect())
}

#[cfg(target_os = "linux")]
fn unprotect_seed(protected: &[u8], binding: &str) -> Result<Vec<u8>> {
    if protected.len() != 32 {
        bail!("protected Linux host seed has invalid length");
    }
    let mask = Sha256::digest(
        [
            b"epiphany-linux-host-seed-v1\0".as_slice(),
            binding.as_bytes(),
        ]
        .concat(),
    );
    Ok(protected
        .iter()
        .zip(mask)
        .map(|(byte, mask)| byte ^ mask)
        .collect())
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(DIGITS[(byte >> 4) as usize] as char);
        output.push(DIGITS[(byte & 0xf) as usize] as char);
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enrollment_is_immutable_and_signatures_are_purpose_bound() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("host-identity.ccmp");
        let signer = enroll_host_identity_at(&store)?;
        let proof = signer.sign("test-receipt", b"payload")?;
        verify_host_identity_signature(signer.entry(), "test-receipt", b"payload", &proof)?;
        assert!(
            verify_host_identity_signature(signer.entry(), "other", b"payload", &proof).is_err()
        );
        assert!(enroll_host_identity_at(&store).is_err());
        let reopened = open_host_identity_at(&store)?;
        assert_eq!(reopened.entry(), signer.entry());
        Ok(())
    }

    #[test]
    fn malformed_existing_store_fails_closed_without_regeneration() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("host-identity.ccmp");
        std::fs::write(&store, b"not a CultCache store")?;
        let before = std::fs::read(&store)?;
        assert!(open_host_identity_at(&store).is_err());
        assert!(enroll_host_identity_at(&store).is_err());
        assert_eq!(std::fs::read(&store)?, before);
        Ok(())
    }

    #[test]
    fn exported_trust_anchor_is_public_only_and_immutable() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let identity_store = temp.path().join("host-identity.ccmp");
        let anchor_store = temp.path().join("host-identity-public.ccmp");
        let signer = enroll_host_identity_at(&identity_store)?;
        let anchor = export_host_identity_trust_anchor(&signer, &anchor_store)?;
        assert_eq!(anchor.identity_id, signer.entry().identity_id);
        assert_eq!(anchor.public_key, signer.entry().public_key);

        let envelopes = SingleFileMessagePackBackingStore::new(&anchor_store).pull_all()?;
        assert_eq!(envelopes.len(), 1);
        assert_eq!(envelopes[0].r#type, HOST_IDENTITY_TRUST_ANCHOR_TYPE);
        assert_eq!(envelopes[0].key, HOST_IDENTITY_TRUST_ANCHOR_KEY);
        let decoded: HostIdentityTrustAnchorEntry = rmp_serde::from_slice(&envelopes[0].payload)?;
        assert_eq!(decoded, anchor);
        assert!(
            !envelopes[0]
                .payload
                .windows(signer.entry().protected_private_seed.len())
                .any(|window| window == signer.entry().protected_private_seed.as_slice())
        );

        let before = std::fs::read(&anchor_store)?;
        export_host_identity_trust_anchor(&signer, &anchor_store)?;
        assert_eq!(std::fs::read(&anchor_store)?, before);
        Ok(())
    }

    #[test]
    fn exported_raw_trust_anchor_is_the_six_field_crossing_artifact() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let signer = enroll_host_identity_at(&temp.path().join("identity.ccmp"))?;
        let output = temp.path().join("anchor.msgpack");
        let anchor = export_raw_host_identity_trust_anchor(&signer, &output)?;
        let decoded: HostIdentityTrustAnchorEntry =
            rmp_serde::from_slice(&std::fs::read(&output)?)?;
        assert_eq!(decoded, anchor);
        let before = std::fs::read(&output)?;
        export_raw_host_identity_trust_anchor(&signer, &output)?;
        assert_eq!(std::fs::read(&output)?, before);
        Ok(())
    }
}
