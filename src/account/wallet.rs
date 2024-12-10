use crate::vault::KeyPair;
use bs58::{decode, encode};
use curve25519_dalek::ristretto::RistrettoPoint;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallet {
    pub private_key: String,
    pub public_key: String,
    pub address: String,
}

impl Wallet {
    pub fn new() -> Self {
        let key = KeyPair::generate();

        let private_key = key.private_key;
        let public_key = key.public_key;
        let address = Self::generate_address(&public_key);

        Wallet {
            private_key: hex::encode(private_key.to_bytes()),
            public_key: hex::encode(public_key.compress().to_bytes()),
            address,
        }
    }

    pub fn generate_address(public_key: &RistrettoPoint) -> String {
        let key = KeyPair::generate();
        let one_time_public_key = key.public_key;

        let stealth_address_point = one_time_public_key + public_key;

        let mut address_bytes = Vec::new();
        address_bytes.extend_from_slice(&stealth_address_point.compress().to_bytes());

        let checksum = Self::calculate_checksum(&address_bytes);
        address_bytes.extend_from_slice(&checksum);

        encode(address_bytes).into_string()
    }

    fn calculate_checksum(data: &[u8]) -> [u8; 4] {
        let hash = blake3::hash(data);
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&hash.as_bytes()[..4]);
        checksum
    }

    pub fn verify_address(address: &str) -> Result<bool, &'static str> {
        let decoded = decode(address)
            .into_vec()
            .map_err(|_| "Invalid Base58 encoding")?;
        if decoded.len() < 4 {
            return Err("Invalid address length");
        }

        let (address_bytes, checksum) = decoded.split_at(decoded.len() - 4);

        let expected_checksum = Self::calculate_checksum(address_bytes);

        Ok(checksum == expected_checksum)
    }

    pub fn verify(private_key: &str) -> Result<String, &'static str> {
        let public_key = KeyPair::verify(private_key)?;

        Ok(public_key)
    }
}
