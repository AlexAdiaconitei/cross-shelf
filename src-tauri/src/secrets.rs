//! Secret storage: one DPAPI-encrypted file per connection (spec 4.4).
//! Scope is the current Windows user. Writes are atomic (temp file + rename).

use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum SecretError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("dpapi: {0}")]
    Dpapi(String),
    #[error("invalid secret name")]
    BadName,
}

pub struct SecretStore {
    dir: PathBuf,
}

impl SecretStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    fn path(&self, name: &str) -> Result<PathBuf, SecretError> {
        let ok = !name.is_empty()
            && name.len() <= 64
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if !ok {
            return Err(SecretError::BadName);
        }
        Ok(self.dir.join(format!("{name}.bin")))
    }

    pub fn put(&self, name: &str, plain: &[u8]) -> Result<(), SecretError> {
        let target = self.path(name)?;
        std::fs::create_dir_all(&self.dir)?;
        let enc = protect(plain)?;
        write_atomic(&target, &enc)
    }

    pub fn get(&self, name: &str) -> Result<Option<Vec<u8>>, SecretError> {
        let p = self.path(name)?;
        match std::fs::read(&p) {
            Ok(enc) => Ok(Some(unprotect(&enc)?)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn delete(&self, name: &str) -> Result<(), SecretError> {
        match std::fs::remove_file(self.path(name)?) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

fn write_atomic(target: &Path, data: &[u8]) -> Result<(), SecretError> {
    let tmp = target.with_extension("tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, target)?;
    Ok(())
}

#[cfg(windows)]
fn protect(plain: &[u8]) -> Result<Vec<u8>, SecretError> {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptProtectData};
    unsafe {
        let input = CRYPT_INTEGER_BLOB {
            cbData: plain.len() as u32,
            pbData: plain.as_ptr() as *mut u8,
        };
        let mut out = CRYPT_INTEGER_BLOB::default();
        CryptProtectData(&input, None, None, None, None, 0, &mut out)
            .map_err(|e| SecretError::Dpapi(e.to_string()))?;
        let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out.pbData as *mut _)));
        Ok(v)
    }
}

#[cfg(windows)]
fn unprotect(enc: &[u8]) -> Result<Vec<u8>, SecretError> {
    use windows::Win32::Foundation::{HLOCAL, LocalFree};
    use windows::Win32::Security::Cryptography::{CRYPT_INTEGER_BLOB, CryptUnprotectData};
    unsafe {
        let input = CRYPT_INTEGER_BLOB {
            cbData: enc.len() as u32,
            pbData: enc.as_ptr() as *mut u8,
        };
        let mut out = CRYPT_INTEGER_BLOB::default();
        CryptUnprotectData(&input, None, None, None, None, 0, &mut out)
            .map_err(|e| SecretError::Dpapi(e.to_string()))?;
        let v = std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(out.pbData as *mut _)));
        Ok(v)
    }
}

#[cfg(not(windows))]
fn protect(_: &[u8]) -> Result<Vec<u8>, SecretError> {
    Err(SecretError::Dpapi("DPAPI only exists on Windows".into()))
}
#[cfg(not(windows))]
fn unprotect(_: &[u8]) -> Result<Vec<u8>, SecretError> {
    Err(SecretError::Dpapi("DPAPI only exists on Windows".into()))
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_ciphertext_hides_plaintext() {
        let dir = tempfile::tempdir().unwrap();
        let s = SecretStore::new(dir.path());
        let secret = b"SENTINEL_REFRESH_TOKEN_12345";
        s.put("gog", secret).unwrap();
        let raw = std::fs::read(dir.path().join("gog.bin")).unwrap();
        assert!(!raw.windows(secret.len()).any(|w| w == secret));
        assert_eq!(s.get("gog").unwrap().unwrap(), secret);
    }

    #[test]
    fn missing_is_none_and_delete_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let s = SecretStore::new(dir.path());
        assert!(s.get("epic").unwrap().is_none());
        s.delete("epic").unwrap();
        s.put("epic", b"x").unwrap();
        s.delete("epic").unwrap();
        assert!(s.get("epic").unwrap().is_none());
    }

    #[test]
    fn rejects_path_traversal_names() {
        let dir = tempfile::tempdir().unwrap();
        let s = SecretStore::new(dir.path());
        assert!(matches!(s.put("../x", b"1"), Err(SecretError::BadName)));
    }
}
