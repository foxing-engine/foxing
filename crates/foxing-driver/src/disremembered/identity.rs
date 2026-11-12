//! A passphrase per name. Set on first login, checked after, stored as an Argon2id hash
//! on the citizen's account component, which only admins can view.
//!
//! This is the minimum that lets a third person play without being able to play as you.
//! It is not an account system: no recovery, no email, no sessions beyond the connection.

use crate::world::World;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::Argon2;
use foxing_store::EntityId;
use rand_core::OsRng;

/// Why a login was refused.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum LoginError {
    /// Name exists and the passphrase does not match.
    #[error("{name} is already someone's{}, and that is not its passphrase. If the name is yours, send the passphrase it was set with; if not, choose another name (a passing name can collide with a live one, so an uncommon one is safer).", if *.passing { ": a passing name, forgotten an hour after it goes quiet" } else { ", a kept name" })]
    WrongPassphrase { name: String, passing: bool },
    /// First login with a passphrase that is too short.
    #[error("A passphrase is at least {min} characters; it is set the first time a name logs in.")]
    TooShort { min: usize },
    /// A new name that is an old name in other letters.
    #[error("That name is taken, as {taken}. Log in as {taken}, or choose another name.")]
    Taken { taken: String },
    /// A new name the world uses for something else (R7).
    #[error("{name} is {what} here, and a name that is also one reads wrong in every line; choose another.")]
    Reserved { name: String, what: String },
    /// A passphrase longer than anyone types.
    #[error("A passphrase is at most {max} bytes.")]
    TooLong { max: usize },
    /// A passphrase is required.
    #[error("login <name> <passphrase>: the passphrase is set on first login and required after.")]
    Missing,
    /// Too many wrong passphrases for this name recently.
    #[error("Too many wrong passphrases for {name}; try again in {minutes} minute{}.", if *.minutes == 1 { "" } else { "s" })]
    Locked { name: String, minutes: i64 },
    /// The world's fault.
    #[error("internal: {0}")]
    Internal(String),
}

pub const MIN_PASSPHRASE: usize = 8;

/// Reject a first-login passphrase that is empty or short.
pub fn acceptable(passphrase: &str) -> Result<(), LoginError> {
    if passphrase.is_empty() {
        return Err(LoginError::Missing);
    }
    if passphrase.chars().count() < MIN_PASSPHRASE {
        return Err(LoginError::TooShort {
            min: MIN_PASSPHRASE,
        });
    }
    if passphrase.len() > MAX_PASSPHRASE_BYTES {
        return Err(LoginError::TooLong {
            max: MAX_PASSPHRASE_BYTES,
        });
    }
    Ok(())
}

/// The most a passphrase may be: hashing is work, and a kilobyte is nobody's secret.
pub const MAX_PASSPHRASE_BYTES: usize = 256;

/// Hash a passphrase for storage.
pub fn hash(passphrase: &str) -> Result<String, LoginError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(passphrase.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| LoginError::Internal(e.to_string()))
}

/// Check a passphrase against the citizen's stored hash.
pub fn check(world: &World, citizen: EntityId, passphrase: &str) -> Result<(), LoginError> {
    let name = super::plain_str(world, citizen, super::CITIZEN, "name")
        .unwrap_or("that name")
        .to_string();
    if passphrase.is_empty() {
        return Err(LoginError::Missing);
    }
    let Some(stored) = super::plain_str(world, citizen, super::ACCOUNT, "passphrase_hash") else {
        // A citizen from before passphrases existed: the first passphrase offered claims
        // the name. The pre-alpha world is reset before anyone else can.
        return Ok(());
    };
    // S1: a passphrase this process has already verified against this stored hash is
    // known by a quick keyed digest. The full check is twenty milliseconds of work
    // on a fast machine (more on the world's own), it ran on every command that
    // came over HTTP or MCP, and it ran with the world held.
    let quick = quick_digest(passphrase);
    if let (Some(d), Ok(seen)) = (quick, VERIFIED.lock()) {
        if seen.get(stored).is_some_and(|k| same(k, &d)) {
            return Ok(());
        }
    }
    let parsed = PasswordHash::new(stored).map_err(|e| LoginError::Internal(e.to_string()))?;
    Argon2::default()
        .verify_password(passphrase.as_bytes(), &parsed)
        .map_err(|_| LoginError::WrongPassphrase {
            passing: super::is_passing(world, citizen),
            name,
        })?;
    if let (Some(d), Ok(mut seen)) = (quick, VERIFIED.lock()) {
        // Keyed by the stored hash, so a changed passphrase is a new key and the old
        // digest answers nothing. Bounded: past ten thousand, start again.
        if seen.len() > 10_000 {
            seen.clear();
        }
        seen.insert(stored.to_string(), d);
    }
    Ok(())
}

/// Passphrases verified in this process: the stored hash, and a quick digest of the
/// passphrase that passed it. In memory only, and keyed with a secret drawn at start,
/// so the digests mean nothing outside this process.
static VERIFIED: std::sync::Mutex<std::collections::BTreeMap<String, [u8; 32]>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());

/// A digest of a passphrase under this process's own key: Argon2id at its smallest
/// (eight kibibytes, one pass), which is BLAKE2b underneath and takes microseconds.
fn quick_digest(passphrase: &str) -> Option<[u8; 32]> {
    static KEY: std::sync::OnceLock<[u8; 16]> = std::sync::OnceLock::new();
    let key = KEY.get_or_init(|| {
        let mut k = [0u8; 16];
        rand_core::RngCore::fill_bytes(&mut OsRng, &mut k);
        k
    });
    let params = argon2::Params::new(8, 1, 1, Some(32)).ok()?;
    let mut out = [0u8; 32];
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
        .hash_password_into(passphrase.as_bytes(), key, &mut out)
        .ok()?;
    Some(out)
}

/// Equal, in time that does not depend on where they differ.
fn same(a: &[u8; 32], b: &[u8; 32]) -> bool {
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}
