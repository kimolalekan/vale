use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use hex;
use rand::RngCore;

#[derive(Debug, Clone)]
pub struct Crypto {
    pub data: Vec<u8>,
    pub key: Option<String>,
}

impl Crypto {
    pub fn encrypt(data: Vec<u8>, encryption_key: Option<String>) -> Result<Crypto, String> {
        let mut rng = rand::thread_rng();
        let key = match encryption_key {
            Some(key_str) => {
                let key_bytes =
                    hex::decode(key_str).map_err(|_| "Invalid key encoding".to_string())?;
                if key_bytes.len() != 32 {
                    return Err("Key must be 32 bytes long for ChaCha20-Poly1305.".into());
                }
                key_bytes
            }
            None => {
                let mut generated_key = [0u8; 32];
                rng.fill_bytes(&mut generated_key);
                generated_key.to_vec()
            }
        };

        let mut nonce_bytes = [0u8; 12];
        rng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key));
        let ciphertext = cipher
            .encrypt(nonce, data.as_ref())
            .map_err(|_| "Encryption failed".to_string())?;

        // Combine nonce with ciphertext
        let combined_data = [nonce.as_slice(), ciphertext.as_slice()].concat();

        Ok(Crypto {
            data: combined_data,
            key: Some(hex::encode(key)),
        })
    }

    pub fn decrypt(encrypted_data: Vec<u8>, key: &str) -> Result<Crypto, String> {
        let key_bytes = hex::decode(key).map_err(|_| "Invalid key encoding".to_string())?;
        if key_bytes.len() != 32 {
            return Err("Key must be 32 bytes long for ChaCha20-Poly1305.".into());
        }

        if encrypted_data.len() < 12 {
            return Err("Invalid encrypted data length.".into());
        }

        let (nonce_bytes, ciphertext) = encrypted_data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&key_bytes));

        let decrypted_data = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| "Decryption failed".to_string())?;

        Ok(Crypto {
            data: decrypted_data,
            key: None,
        })
    }
}

// fn main() {
//     let data = b"Hello, ChaCha20-Poly1305!".to_vec();
//     let encryption_key = Some("f5dbde0d4aa09c31033608cbf4ab8f9530a9f00d07f837".to_string()); // Or set to some specific hex-encoded key

//     match Crypto::encrypt(data.clone(), encryption_key) {
//         Ok(encrypted) => {
//             println!("Encrypted Data: {}", hex::encode(&encrypted.data));
//             println!("Encryption Key: {}", encrypted.key.as_ref().unwrap());

//             match Crypto::decrypt(encrypted.data, &encrypted.key.unwrap()) {
//                 Ok(decrypted) => {
//                     println!(
//                         "Decrypted Data: {}",
//                         String::from_utf8_lossy(&decrypted.data)
//                     );
//                 }
//                 Err(e) => println!("Decryption failed: {}", e),
//             }
//         }
//         Err(e) => println!("Encryption failed: {}", e),
//     }
// }
