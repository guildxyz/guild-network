use sha3::Digest;

pub type Hash = [u8; 32];

pub fn keccak256<T: AsRef<[u8]>>(input: T) -> Hash {
    let mut output = [0u8; 32];
    let mut hasher = sha3::Keccak256::new();
    hasher.update(input.as_ref());
    hasher.finalize_into((&mut output).into());
    output
}

pub fn blake2256<T: AsRef<[u8]>>(input: T) -> Hash {
    let mut output = [0u8; 32];
    let mut hasher = blake2::Blake2s256::new();
    hasher.update(input.as_ref());
    hasher.finalize_into((&mut output).into());
    output
}

pub struct Keccak256;

impl hash_db::Hasher for Keccak256 {
    type Out = Hash;
    type StdHasher = PlaceholderHasher;
    const LENGTH: usize = 32;

    fn hash(x: &[u8]) -> Self::Out {
        keccak256(x)
    }
}

/// This hasher is never used but it is required as a placeholder.
#[derive(Default)]
pub struct PlaceholderHasher;

impl core::hash::Hasher for PlaceholderHasher {
    fn write(&mut self, _bytes: &[u8]) {
        unimplemented!()
    }

    fn finish(&self) -> u64 {
        unimplemented!()
    }
}
