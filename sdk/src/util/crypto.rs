use chacha20poly1305::{
  aead::{generic_array::GenericArray, Aead, KeyInit},
  ChaCha20Poly1305, Nonce,
};

pub fn encrypt(data: &[u8], key: &String, nonce: &String) -> anyhow::Result<Vec<u8>> {
  let key: [u8; 32] = key
    .as_bytes()
    .try_into()
    .map_err(|_e| anyhow::anyhow!("The key length should be 32 characters."))?;
  let nonce: [u8; 12] = nonce
    .as_bytes()
    .try_into()
    .map_err(|_e| anyhow::anyhow!("The nonce length should be 12 characters."))?;
  let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(&key));
  let cipher_data = cipher
    .encrypt(Nonce::from_slice(&nonce), data)
    .map_err(|e| anyhow::anyhow!("Encrypt data failed. Error: {e}"))?;
  Ok(cipher_data)
}

pub fn decrypt(cipher_data: &[u8], key: &String, nonce: &String) -> anyhow::Result<Vec<u8>> {
  let key: [u8; 32] = key
    .as_bytes()
    .try_into()
    .map_err(|_e| anyhow::anyhow!("The key length should be 32 characters."))?;
  let nonce: [u8; 12] = nonce
    .as_bytes()
    .try_into()
    .map_err(|_e| anyhow::anyhow!("The nonce length should be 12 characters."))?;
  let cipher = ChaCha20Poly1305::new(GenericArray::from_slice(&key));
  let data = cipher
    .decrypt(Nonce::from_slice(&nonce), cipher_data)
    .map_err(|e| anyhow::anyhow!("Decrypt data failed. Error: {e}"))?;
  Ok(data)
}

#[cfg(test)]
mod tests {
  use fake::{Fake, Faker};

  use crate::util::crypto::decrypt;

  use super::encrypt;

  #[test]
  pub fn test_encrypt_decrypt() {
    let data = Faker.fake::<String>();
    let data = data.as_bytes();
    let password = String::from("01234567890123456789012345678912");
    let nonce = String::from("012345678912");
    let cipher_data = encrypt(data, &password, &nonce).unwrap();
    let actual_data = decrypt(&cipher_data, &password, &nonce).unwrap();
    assert_eq!(data, actual_data)
  }
}
