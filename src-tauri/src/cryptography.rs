use aes_gcm::{
    aead::{generic_array::typenum::U12, Aead, AeadCore, KeyInit}, Aes256Gcm, Key, Nonce
};
use sharks::{Sharks, Share};
use rand::{rngs::OsRng,RngCore};
use opaque_ke::CipherSuite;
use argon2::Argon2;
use secrecy::zeroize::Zeroize;

pub(crate) struct DefaultCipherSuite;

impl CipherSuite for DefaultCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeGroup = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::key_exchange::tripledh::TripleDh;
    type Ksf = Argon2<'static>;
}

pub(crate) fn decrypt_using_nonce(key: &[u8], ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, ()> {
    let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&nonce[..12]);
    cipher.decrypt(nonce,ciphertext).map_err(|_| ())
}

pub(crate) fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, ()> {
    let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).map_err(|_| ())?;
    Ok([nonce.to_vec(), ciphertext].concat())
}

pub(crate) fn encrypt_with_nonce(key: &[u8], plaintext: &[u8], nonce: Nonce<U12>) -> Result<Vec<u8>, ()> {
    let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
    let cipher = Aes256Gcm::new(&key);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_ref()).map_err(|_| ())?;
    Ok(ciphertext)
}

pub(crate) fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, ()> {
    let key = Key::<Aes256Gcm>::from_slice(&key[..32]);
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    cipher.decrypt(nonce, &ciphertext[12..]).map_err(|_| ())
}

pub(crate) fn secret_padded_shares(password: &str) -> (Vec<Share>,usize) {
    let sharks = Sharks(2);
    let padding_characters_count = 128 - password.len();
    let mut rng = OsRng;
    let mut random_padding = vec![0u8; padding_characters_count];
    rng.fill_bytes(&mut random_padding);
    let dealer = sharks.dealer([password.as_bytes(), random_padding.as_slice()].concat().as_slice());
    random_padding.zeroize();
    (dealer.take(2).collect(), padding_characters_count)
}