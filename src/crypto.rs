use base16ct::HexDisplay;
use sha2::{Digest, Sha256};

pub fn sha256sum(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    sha256sum_hex(hasher)
}

pub fn sha256sum_hex(hasher: Sha256) -> String {
    format!("{:x}", HexDisplay(&hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256sum(b"ohai!");
        assert_eq!(
            hash,
            "f66b9e95324778cbc291d16cc30a950a0cacfe1c06e72cd9743d474c5e3e6b99"
        );
    }
}
