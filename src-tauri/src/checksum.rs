use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Compute the SHA-256 checksum of a file, returning the hex-encoded digest.
pub fn compute_sha256(path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::with_capacity(64 * 1024, file);
    let mut hasher = Sha256::new();

    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

/// Verify that a file matches an expected SHA-256 checksum.
/// Returns `Ok(true)` if the file matches, `Ok(false)` if it does not.
pub fn verify_sha256(path: &Path, expected: &str) -> Result<bool, std::io::Error> {
    let actual = compute_sha256(path)?;
    Ok(actual == expected)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_sha256_produces_consistent_digest() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut tmp.as_file().try_clone().unwrap(), b"hello amber").unwrap();

        let hash1 = compute_sha256(tmp.path()).unwrap();
        let hash2 = compute_sha256(tmp.path()).unwrap();
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex is 64 chars
    }

    #[test]
    fn verify_sha256_returns_true_for_matching_hash() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut tmp.as_file().try_clone().unwrap(), b"test data").unwrap();

        let hash = compute_sha256(tmp.path()).unwrap();
        assert!(verify_sha256(tmp.path(), &hash).unwrap());
    }

    #[test]
    fn verify_sha256_returns_false_for_mismatched_hash() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        std::io::Write::write_all(&mut tmp.as_file().try_clone().unwrap(), b"test data").unwrap();

        assert!(!verify_sha256(
            tmp.path(),
            "0000000000000000000000000000000000000000000000000000000000000000"
        )
        .unwrap());
    }

    #[test]
    fn compute_sha256_fails_for_missing_file() {
        let result = compute_sha256(Path::new("/nonexistent/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn empty_file_produces_known_sha256() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        // SHA-256 of empty input is e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        let hash = compute_sha256(tmp.path()).unwrap();
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
