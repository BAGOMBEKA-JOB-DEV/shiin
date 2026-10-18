//! Persistent signing and replay keys in the OS keyring, following ADR-0032.

use crate::receipts;
use ed25519_dalek::SigningKey;
use keyring_core::{Entry, Error};
use std::sync::LazyLock;
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
        KeyStore::PRODUCTION.active()
    }
    /// Loads the active OS-keyring key, creating it only if no entry exists.
    ///
    /// # Errors
    /// A locked, unavailable, malformed, or unsupported keyring fails without a file fallback.
    pub fn initialize() -> Result<Self, String> {
        KeyStore::PRODUCTION.initialize()
    }

    /// Generates a new key while retaining old keys for historical replay.
    ///
    /// # Errors
    /// Entropy or keyring failures leave the previous active key selected.
    pub fn rotate() -> Result<Self, String> {
        KeyStore::PRODUCTION.rotate()
    }

    /// Loads a retained key by its public fingerprint.
    ///
    /// # Errors
    /// Unknown IDs and corrupt key material are rejected.
    pub fn load(id: &str) -> Result<Self, String> {
        KeyStore::PRODUCTION.load(id)
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

/// The production namespace is fixed; only in-module tests select an isolated namespace.
struct KeyStore<'a> {
    service: &'a str,
}

impl KeyStore<'_> {
    const PRODUCTION: Self = Self { service: "shiin" };

    fn entry(&self, username: &str) -> Result<Entry, String> {
        if NATIVE_STORE.is_err() {
            return Err("OS keyring is unavailable or unsupported".to_owned());
        }
        Entry::new(self.service, username)
            .map_err(|_| "OS keyring is unavailable or unsupported".to_owned())
    }

    fn active(&self) -> Result<Keys, String> {
        let id = self
            .entry("active-signing-key")?
            .get_password()
            .map_err(|_| "active OS keyring key unavailable")?;
        self.load(&id)
    }

    fn initialize(&self) -> Result<Keys, String> {
        match self.entry("active-signing-key")?.get_password() {
            Ok(id) => self.load(&id),
            Err(Error::NoEntry) => self.rotate(),
            Err(_) => Err("OS keyring is unavailable or locked".to_owned()),
        }
    }

    fn rotate(&self) -> Result<Keys, String> {
        let mut secret = Zeroizing::new([0_u8; 64]);
        getrandom::fill(secret.as_mut()).map_err(|_| "OS randomness failed")?;
        let keys = Keys::from_secret(secret.as_ref())?;
        let id = keys.id();
        self.entry(&id)?
            .set_secret(secret.as_ref())
            .map_err(|_| "OS keyring could not store signing key")?;
        self.entry("active-signing-key")?
            .set_password(&id)
            .map_err(|_| "OS keyring could not select active signing key")?;
        Ok(keys)
    }

    fn load(&self, id: &str) -> Result<Keys, String> {
        if !id.strip_prefix("sha256:").is_some_and(|suffix| {
            suffix.len() == 64
                && suffix
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        }) {
            return Err("invalid signing key identifier".to_owned());
        }
        let secret = Zeroizing::new(
            self.entry(id)?
                .get_secret()
                .map_err(|_| "OS keyring key unavailable")?,
        );
        let keys = Keys::from_secret(&secret)?;
        if keys.id() != id {
            return Err("OS keyring key identifier mismatch".to_owned());
        }
        Ok(keys)
    }
}

// Initialize exactly one native store. Store failures never select a file or mock backend.
static NATIVE_STORE: LazyLock<Result<(), Error>> = LazyLock::new(initialize_native_store);

fn initialize_native_store() -> Result<(), Error> {
    #[cfg(target_os = "macos")]
    let store = apple_native_keyring_store::keychain::Store::new()?;
    #[cfg(target_os = "windows")]
    let store = windows_native_keyring_store::Store::new()?;
    #[cfg(all(
        unix,
        not(any(target_os = "macos", target_os = "ios", target_os = "android"))
    ))]
    let store = dbus_secret_service_keyring_store::Store::new()?;
    #[cfg(all(any(unix, windows), not(any(target_os = "ios", target_os = "android"))))]
    {
        keyring_core::set_default_store(store);
        Ok(())
    }
    #[cfg(not(all(any(unix, windows), not(any(target_os = "ios", target_os = "android")))))]
    Err(Error::NoDefaultStore)
}

#[cfg(test)]
mod tests {
    use super::KeyStore;
    use crate::receipts;
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use serde_json::json;

    struct TestStore {
        service: String,
        ids: Vec<String>,
    }

    impl Drop for TestStore {
        fn drop(&mut self) {
            let store = KeyStore {
                service: &self.service,
            };
            for id in self
                .ids
                .iter()
                .map(String::as_str)
                .chain(["active-signing-key"])
            {
                if let Ok(entry) = store.entry(id) {
                    let _ = entry.delete_credential();
                }
            }
        }
    }

    // Explicit opt-in prevents the ordinary test suite from touching a native key store.
    // This test uses the actual platform backend, with neither mocks nor file fallbacks.
    #[test]
    #[ignore = "requires a native key store; run cargo xtask native-keyring-check on a hosted runner"]
    fn native_key_store_lifecycle() -> Result<(), String> {
        if std::env::var("SHIIN_TEST_NATIVE_KEYRING").as_deref() != Ok("1") {
            return Err("native key-store test requires explicit opt-in".to_owned());
        }
        let mut nonce = [0_u8; 16];
        getrandom::fill(&mut nonce).map_err(|_| "test namespace randomness failed")?;
        let suffix = URL_SAFE_NO_PAD.encode(nonce);
        let mut cleanup = TestStore {
            service: format!("shiin-EXAMPLE-native-test-{suffix}"),
            ids: Vec::new(),
        };
        let store = KeyStore {
            service: &cleanup.service,
        };
        assert!(store.active().is_err());
        let first = store.initialize()?;
        cleanup.ids.push(first.id());
        assert_eq!(store.active()?.public(), first.public());
        assert_eq!(store.initialize()?.id(), first.id());
        let checkpoint = json!({
            "schema": "shiin.checkpoint/v1-draft.1",
            "key_id": first.id(),
            "seq": 0,
            "head": format!("sha256:{}", "0".repeat(64)),
            "signed_at": "2026-10-16T00:00:00Z",
        });
        let envelope = receipts::sign(&first.signing, receipts::CHECKPOINT_TYPE, &checkpoint)?;
        assert_eq!(
            receipts::verify(&envelope, &first.public(), receipts::CHECKPOINT_TYPE)?,
            checkpoint
        );
        let encrypted = first.encrypt(
            "00000000-0000-4000-8000-000000000000",
            b"EXAMPLE private replay inputs",
        )?;
        let second = store.rotate()?;
        cleanup.ids.push(second.id());
        assert_ne!(second.id(), first.id());
        assert_eq!(store.active()?.id(), second.id());
        let retained = store.load(&first.id())?;
        assert_eq!(
            retained.decrypt(&encrypted)?,
            b"EXAMPLE private replay inputs"
        );
        assert!(second.decrypt(&encrypted).is_err());
        assert!(receipts::verify(&envelope, &second.public(), receipts::CHECKPOINT_TYPE).is_err());

        // A corrupt selected key never causes implicit regeneration or selection of another key.
        store
            .entry(&second.id())?
            .set_secret(b"EXAMPLE corrupt key")
            .map_err(|_| "test could not corrupt isolated key")?;
        assert!(store.load(&second.id()).is_err());
        assert!(store.active().is_err());
        assert!(store.initialize().is_err());
        assert_eq!(
            store
                .entry("active-signing-key")?
                .get_password()
                .map_err(|_| "test could not read active pointer")?,
            second.id()
        );
        // A correctly sized secret stored under a different fingerprint is also rejected.
        store
            .entry(&second.id())?
            .set_secret(&[0_u8; 64])
            .map_err(|_| "test could not replace isolated key")?;
        assert!(store.load(&second.id()).is_err());
        store
            .entry(&first.id())?
            .delete_credential()
            .map_err(|_| "test could not delete isolated key")?;
        assert!(store.load(&first.id()).is_err());
        Ok(())
    }
}
