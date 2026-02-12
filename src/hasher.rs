use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{Read, Result};
use std::path::Path;

/// Hash raw bytes and return hex string
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Hash a string
pub fn hash_string(input: &str) -> String {
    hash_bytes(input.as_bytes())
}

/// Hash a file
pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_hash_string() {
        let hash = hash_string("hello");
        assert_eq!(
            hash,
            "2cf24dba5fb0a030e...replace_with_full_hash"
        );
    }

    #[test]
    fn test_hash_bytes() {
        let data = b"hello";
        let hash = hash_bytes(data);
        assert_eq!(
            hash,
            "2cf24dba5fb0a030e...replace_with_full_hash"
        );
    }

    #[test]
    fn test_hash_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "hello").unwrap();

        let hash = hash_file(temp_file.path()).unwrap();
        assert_eq!(
            hash,
            "2cf24dba5fb0a030e...replace_with_full_hash"
        );
    }
}
