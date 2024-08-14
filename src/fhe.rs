use fhe::{
    bfv::{BfvParameters, BfvParametersBuilder, SecretKey as FheRsSecretKey},
    mbfv::{CommonRandomPoly, PublicKeyShare as FheRsPublicKeyShare},
};
use fhe_traits::Serialize;
use rand::{CryptoRng, RngCore};
use std::{
    mem,
    sync::{Arc, Mutex},
};

// Some loose error/result stuff we can use for this module
pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

// Define a trait for Rng which we use below
pub trait Rng: RngCore + CryptoRng {}
impl<T: RngCore + CryptoRng> Rng for T {}

/// Wrapped PublicKeyShare. This is wrapped to provide an inflection point
/// as we use this library elsewhere we only implement traits as we need them
/// and avoid exposing underlying structures from fhe.rs
#[derive(Debug, Clone)]
pub struct PublicKeyShare(pub FheRsPublicKeyShare);

impl PublicKeyShare {
    pub fn as_bytes(&self) -> Vec<u8> {
        self.0.to_bytes()
    }
}

impl From<PublicKeyShare> for Vec<u8> {
    fn from(share: PublicKeyShare) -> Vec<u8> {
        share.as_bytes()
    }
}

/// Our wrapped SecretKey
#[derive(PartialEq)] // Avoid adding debugging and copy traits as this is a secret key and we want
                     // Underlying struct is a Box<[i64]> so Copy will do a memory copy although
                     // the key is zeroized on drop
pub struct SecretKey(pub FheRsSecretKey);
impl From<SecretKey> for Vec<u8> {
    fn from(key: SecretKey) -> Vec<u8> {
        serialize_box_i64(key.0.coeffs)
    }
}

// Serialize Box<[i64]> to Vec<u8>
fn serialize_box_i64(boxed: Box<[i64]>) -> Vec<u8> {
    let vec = boxed.into_vec();
    let mut bytes = Vec::with_capacity(vec.len() * mem::size_of::<i64>());
    for &num in &vec {
        bytes.extend_from_slice(&num.to_le_bytes());
    }
    bytes
}

// Deserialize Vec<u8> to Box<[i64]>
fn deserialize_to_box_i64(bytes: Vec<u8>) -> Option<Box<[i64]>> {
    if bytes.len() % mem::size_of::<i64>() != 0 {
        return None; // Input length is not a multiple of i64 size
    }

    let mut result = Vec::with_capacity(bytes.len() / mem::size_of::<i64>());
    let mut chunks = bytes.chunks_exact(mem::size_of::<i64>());

    for chunk in &mut chunks {
        let num = i64::from_le_bytes(chunk.try_into().unwrap());
        result.push(num);
    }

    Some(result.into_boxed_slice())
}
/// Fhe is the accessor crate for our Fhe encryption lib. We should use this as an inflection point.
/// Underlying internal types and errors should not be leaked. We should aim to maintain a simple
/// API in line with our needs not the underlying library and what this does should be pretty
/// lightweight
pub struct Fhe {
    params: Arc<BfvParameters>,
    crp: CommonRandomPoly,
}

impl Fhe {
    pub fn new<R: Rng>(
        rng: Arc<Mutex<R>>,
        moduli: Vec<u64>,
        degree: usize,
        plaintext_modulus: u64,
    ) -> Result<Fhe> {
        let params = BfvParametersBuilder::new()
            .set_degree(degree)
            .set_plaintext_modulus(plaintext_modulus)
            .set_moduli(&moduli)
            .build_arc()?;
        let crp = CommonRandomPoly::new(&params, &mut *rng.lock().unwrap())?;
        Ok(Fhe { params, crp })
    }

    pub fn get_params(&self) -> (&Arc<BfvParameters>, &CommonRandomPoly) {
        (&self.params, &self.crp)
    }

    pub fn generate_keyshare<R: Rng>(
        &self,
        rng: Arc<Mutex<R>>,
    ) -> Result<(SecretKey, PublicKeyShare)> {
        let sk_share = {
            let mut r1 = rng.lock().unwrap();
            FheRsSecretKey::random(&self.params, &mut *r1)
        };
        let pk_share = {
            let mut r2 = rng.lock().unwrap();
            FheRsPublicKeyShare::new(&sk_share, self.crp.clone(), &mut *r2)?
        };
        Ok((SecretKey(sk_share), PublicKeyShare(pk_share)))
    }
}
