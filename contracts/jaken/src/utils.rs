use rand_chacha::ChaChaRng;
use rand_core::{RngCore, SeedableRng};
use sha2::{Digest, Sha256};

pub const SHA256_HASH_SIZE: usize = 32;

pub fn sha_256(data: &[u8]) -> [u8; SHA256_HASH_SIZE] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();

    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_slice());
    result
}
pub struct Prng {
    rng: ChaChaRng,
}
impl Prng {
    pub fn new(seed: &[u8], entropy: &[u8]) -> Self {
        let mut hasher = Sha256::new();

        // write input message
        hasher.update(&seed);
        hasher.update(&entropy);
        let hash = hasher.finalize();

        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(hash.as_slice());

        let rng = ChaChaRng::from_seed(hash_bytes);

        Self { rng }
    }

    pub fn rand_bytes(&mut self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        self.rng.fill_bytes(&mut bytes);

        bytes
    }
    pub fn new_rand_bytes(seed: &[u8], entropy: &[u8]) -> Vec<u8> {
        let mut rng = Self::new(seed, (entropy).as_ref());
        let rand_slice = rng.rand_bytes();
        sha_256(&rand_slice).to_vec()
    }
}
