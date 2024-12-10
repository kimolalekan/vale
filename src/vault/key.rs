use arrayref::array_ref;
use curve25519_dalek::constants;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::scalar::Scalar;
use rand::rngs::OsRng;
use rand::RngCore;

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: RistrettoPoint,
    pub private_key: Scalar,
}

impl KeyPair {
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let mut random_bytes = [0u8; 32];
        csprng.fill_bytes(&mut random_bytes);

        let private_key = Scalar::from_bytes_mod_order(random_bytes);
        let public_key = private_key * &constants::RISTRETTO_BASEPOINT_POINT;

        KeyPair {
            public_key,
            private_key,
        }
    }

    pub fn verify(private_key: &str) -> Result<String, &'static str> {
        let private_key_bytes =
            hex::decode(private_key).map_err(|_| "Invalid private key encoding")?;
        let private_key_array = array_ref![private_key_bytes, 0, 32];
        let private_key_scalar = Scalar::from_bytes_mod_order(*private_key_array);
        let public_key = &private_key_scalar * &constants::RISTRETTO_BASEPOINT_POINT;
        let public_key = hex::encode(public_key.compress().to_bytes());

        Ok(public_key)
    }
}
