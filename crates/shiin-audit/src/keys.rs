//! Persistent signing and replay keys in the OS keyring, following ADR-0032.

use crate::receipts;
use ed25519_dalek::SigningKey;
use keyring::{Entry, Error};
use zeroize::Zeroizing;

/// In-memory key material. Private bytes are never serialized or printed.
pub struct Keys {
    /// Signing key whose drop implementation zeroes secret material.
    pub signing: SigningKey,
    replay: Zeroizing<[u8; 32]>,
}

impl Keys {
    /// Loads the selected signing key without creating a replacement.
    ///
    /// # Errors
    /// Missing or unavailable keys require explicit operator initialization.
    pub fn active() -> Result<Self, String> {
        let id = entry("active-signing-key")?
            .get_password()
            .map_err(|_| "active OS keyring key unavailable")?;
        Self::load(&id)
    }
    /// Loads the active OS-keyring key, creating it only if no entry exists.
    ///
    /// # Errors
    /// A locked, unavailable, malformed, or unsupported keyring fails without a file fallback.
    pub fn initialize() -> Result<Self, String> {
        let active = entry("active-signing-key")?;
        match active.get_password() {
            Ok(id) => Self::load(&id),
            Err(Error::NoEntry) => Self::rotate(),
            Err(_) => Err("OS keyring is unavailable or locked".to_owned()),
        }
    }

    /// Generates a new key while retaining old keys for historical replay.
    ///
    /// # Errors
    /// Entropy or keyring failures leave the previous active key selected.
    pub fn rotate() -> Result<Self, String> {
        let mut secret = Zeroizing::new([0_u8; 64]);
        getrandom::fill(secret.as_mut()).map_err(|_| "OS randomness failed")?;
        let keys = Self::from_secret(secret.as_ref())?;
        let id = keys.id();
        entry(&id)?
            .set_secret(secret.as_ref())
            .map_err(|_| "OS keyring could not store signing key")?;
        entry("active-signing-key")?
            .set_password(&id)
            .map_err(|_| "OS keyring could not select active signing key")?;
        Ok(keys)
    }

    /// Loads a retained key by its public fingerprint.
    ///
    /// # Errors
    /// Unknown IDs and corrupt key material are rejected.
    pub fn load(id: &str) -> Result<Self, String> {
        if !id.strip_prefix("sha256:").is_some_and(|suffix| {
            suffix.len() == 64
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        }) {
            return Err("invalid signing key identifier".to_owned());
        }
        let secret = Zeroizing::new(
            entry(id)?
                .get_secret()
                .map_err(|_| "OS keyring key unavailable")?,
        );
        let keys = Self::from_secret(&secret)?;
        if keys.id() != id {
            return Err("OS keyring key identifier mismatch".to_owned());
        }
        Ok(keys)
    }

    fn from_secret(secret: &[u8]) -> Result<Self, String> {
        if secret.len() != 64 {
            return Err("invalid OS keyring key length".to_owned());
        }
        let seed = Zeroizing::new(
            <[u8; 32]>::try_from(&secret[..32]).map_err(|_| "invalid signing seed")?,
        );
        let replay = secret[32..].try_into().map_err(|_| "invalid replay key")?;
        Ok(Self {
            signing: SigningKey::from_bytes(&seed),
            replay: Zeroizing::new(replay),
        })
    }

    /// Public fingerprint suitable for audit records.
    #[must_use]
    pub fn id(&self) -> String {
        receipts::key_id(&self.signing.verifying_key())
    }

    /// Public key bytes for offline verification.
    #[must_use]
    pub fn public(&self) -> [u8; 32] {
        self.signing.verifying_key().to_bytes()
    }

    /// Encrypts replay input with a fresh 192-bit nonce.
    ///
    /// # Errors
    /// Entropy or encryption failures prevent recording.
    pub fn encrypt(
        &self,
        request_id: &str,
        plaintext: &[u8],
    ) -> Result<receipts::EncryptedReplay, String> {
        let mut nonce = [0_u8; 24];
        getrandom::fill(&mut nonce).map_err(|_| "OS randomness failed")?;
        receipts::encrypt_replay(&self.replay, &self.id(), request_id, nonce, plaintext)
    }

    /// Authenticates and decrypts a previously verified replay artifact.
    ///
    /// # Errors
    /// Wrong keys, corrupted ciphertext, or changed bindings are rejected.
    pub fn decrypt(&self, artifact: &receipts::EncryptedReplay) -> Result<Vec<u8>, String> {
        if artifact.key_id != self.id() {
            return Err("replay key identifier mismatch".to_owned());
        }
        receipts::decrypt_replay(&self.replay, artifact)
    }
}

fn entry(username: &str) -> Result<Entry, String> {
    Entry::new("shiin", username).map_err(|_| "OS keyring is unavailable or unsupported".to_owned())
}
