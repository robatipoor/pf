use anyhow::anyhow;
use chacha20poly1305::{
  aead::{
    generic_array::GenericArray,
    stream::{DecryptorBE32, EncryptorBE32},
    KeyInit,
  },
  consts::U32,
  XChaCha20Poly1305,
};
use std::path::Path;
use tokio::{
  fs::File,
  io::{AsyncReadExt, AsyncWriteExt},
};

const DEFAULT_BUF_SIZE: usize = 4096; // 4KB chunk size

#[derive(Debug, Clone, Copy)]
pub struct KeyType(GenericArray<u8, U32>);

impl KeyType {
  pub fn new(key: &str) -> anyhow::Result<Self> {
    let key: [u8; 32] = key
      .as_bytes()
      .try_into()
      .map_err(|_e| anyhow::anyhow!("The key length should be 32 characters."))?;

    Ok(Self(GenericArray::from_iter(key)))
  }
}

impl std::ops::Deref for KeyType {
  type Target = GenericArray<u8, U32>;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

#[derive(Debug, Clone, Copy)]
pub struct NonceType([u8; 19]);

impl NonceType {
  pub fn new(nonce: &str) -> anyhow::Result<Self> {
    let nonce: [u8; 19] = nonce
      .as_bytes()
      .try_into()
      .map_err(|_e| anyhow::anyhow!("The nonce length should be 19 characters."))?;

    Ok(Self(nonce))
  }
}

impl std::ops::Deref for NonceType {
  type Target = [u8; 19];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub async fn encrypt_file(
  key: KeyType,
  nonce: NonceType,
  input_file: &Path,
  output_file: &Path,
) -> anyhow::Result<()> {
  let mut reader = File::open(input_file).await?;
  let mut writer = File::create(output_file).await?;
  let mut buffer = [0u8; DEFAULT_BUF_SIZE];
  let mut stream_encryptor =
    EncryptorBE32::from_aead(XChaCha20Poly1305::new(&*key), (&*nonce).as_ref().into());
  loop {
    let read_count = reader.read(&mut buffer).await?;
    if read_count == DEFAULT_BUF_SIZE {
      let ciphertext = stream_encryptor
        .encrypt_next(buffer.as_slice())
        .map_err(|err| anyhow!("Encrypting file failed, Error: {err}"))?;
      writer.write(&ciphertext).await?;
    } else {
      let ciphertext = stream_encryptor
        .encrypt_last(&buffer[..read_count])
        .map_err(|err| anyhow!("Encrypting file failed, Error: {err}"))?;
      writer.write(&ciphertext).await?;
      break;
    }
  }
  writer.flush().await?;

  Ok(())
}

pub async fn decrypt_file(
  key: KeyType,
  nonce: NonceType,
  input_file: &Path,
  output_file: &Path,
) -> anyhow::Result<()> {
  let mut reader = File::open(input_file).await?;
  let mut writer = File::create(output_file).await?;
  let mut buffer = [0u8; DEFAULT_BUF_SIZE];
  let mut stream_decryptor =
    DecryptorBE32::from_aead(XChaCha20Poly1305::new(&*key), nonce.as_ref().into());

  loop {
    let read_count = reader.read(&mut buffer).await?;
    if read_count == DEFAULT_BUF_SIZE {
      let plaintext = stream_decryptor
        .decrypt_next(buffer.as_slice())
        .map_err(|err| anyhow!("Decrypting file failed, Error: {err}"))?;
      writer.write(&plaintext).await?;
    } else if read_count == 0 {
      break;
    } else {
      let plaintext = stream_decryptor
        .decrypt_last(&buffer[..read_count])
        .map_err(|err| anyhow!("Decrypting file failed, Error: {err}"))?;
      writer.write(&plaintext).await?;
      break;
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {

  use fake::{Fake, Faker};
  use test_context::test_context;

  use crate::util::test::FileTestContext;

  use super::*;

  #[test_context(FileTestContext)]
  #[tokio::test]
  pub async fn test_encrypt_and_decrypt_file(ctx: &mut FileTestContext) {
    let key = KeyType::new("01234567890123456789012345678912").unwrap();
    let nonce = NonceType::new("1234567891213141516").unwrap();
    let contents = Faker.fake::<String>();

    let plaintext_file = ctx.temp_path.join("file.txt");
    tokio::fs::write(&plaintext_file, &contents).await.unwrap();
    let ciphertext_file = ctx.temp_path.join("file.bin");
    encrypt_file(key, nonce, &plaintext_file, &ciphertext_file)
      .await
      .unwrap();
    let result_file = ctx.temp_path.join("result_file.txt");
    decrypt_file(key, nonce, &ciphertext_file, &result_file)
      .await
      .unwrap();
    let actual_contents = tokio::fs::read_to_string(result_file).await.unwrap();
    assert_eq!(contents, actual_contents)
  }
}
