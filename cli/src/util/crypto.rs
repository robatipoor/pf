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
use sdk::util::random::generate_random_string_with_prefix;
use std::path::{Path, PathBuf};
use tokio::{
  fs::File,
  io::{AsyncReadExt, AsyncWriteExt},
};

use crate::parse::KeyAndNonce;

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
  plaintext_file: PathBuf,
  key_and_nonce: &KeyAndNonce,
) -> anyhow::Result<PathBuf> {
  let encrypt_file = sdk::util::file::add_extension(&plaintext_file, "enc");
  encrypt(key_and_nonce, &plaintext_file, &encrypt_file).await?;
  Ok(encrypt_file)
}

pub async fn decrypt_file(
  encrypted_file: PathBuf,
  key_and_nonce: &KeyAndNonce,
) -> anyhow::Result<()> {
  let decrypted_file = sdk::util::file::rm_extra_extension(sdk::util::file::add_parent_dir(
    &encrypted_file,
    &generate_random_string_with_prefix("tmp"),
  )?)?;
  tokio::fs::create_dir(&decrypted_file.parent().unwrap()).await?;
  decrypt(key_and_nonce, &encrypted_file, &decrypted_file).await?;
  tokio::fs::remove_file(&encrypted_file).await.unwrap();
  tokio::fs::rename(decrypted_file, encrypted_file).await?;

  Ok(())
}

async fn encrypt(
  KeyAndNonce { key, nonce }: &KeyAndNonce,
  input_file: &Path,
  output_file: &Path,
) -> anyhow::Result<()> {
  let mut reader = File::open(input_file).await?;
  let mut writer = File::create(output_file).await?;
  let mut buffer = [0u8; DEFAULT_BUF_SIZE];
  let mut stream_encryptor =
    EncryptorBE32::from_aead(XChaCha20Poly1305::new(&*key), (*nonce).as_ref().into());
  loop {
    let read_count = reader.read(&mut buffer).await?;
    if read_count == DEFAULT_BUF_SIZE {
      let ciphertext = stream_encryptor
        .encrypt_next(buffer.as_slice())
        .map_err(|err| anyhow!("Encrypting file failed, Error: {err}"))?;
      writer.write_all(&ciphertext).await?;
    } else {
      let ciphertext = stream_encryptor
        .encrypt_last(&buffer[..read_count])
        .map_err(|err| anyhow!("Encrypting file failed, Error: {err}"))?;
      writer.write_all(&ciphertext).await?;
      break;
    }
  }
  writer.flush().await?;

  Ok(())
}

async fn decrypt(
  KeyAndNonce { key, nonce }: &KeyAndNonce,
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
      writer.write_all(&plaintext).await?;
    } else if read_count == 0 {
      break;
    } else {
      let plaintext = stream_decryptor
        .decrypt_last(&buffer[..read_count])
        .map_err(|err| anyhow!("Decrypting file failed, Error: {err}"))?;
      writer.write_all(&plaintext).await?;
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
    let key_and_nonce = KeyAndNonce {
      key: KeyType::new("01234567890123456789012345678912").unwrap(),
      nonce: NonceType::new("1234567891213141516").unwrap(),
    };
    let contents = Faker.fake::<String>();

    let plaintext_file = ctx.temp_path.join("file.txt");
    tokio::fs::write(&plaintext_file, &contents).await.unwrap();
    let ciphertext_file = ctx.temp_path.join("file.bin");
    encrypt(&key_and_nonce, &plaintext_file, &ciphertext_file)
      .await
      .unwrap();
    let result_file = ctx.temp_path.join("result_file.txt");
    decrypt(&key_and_nonce, &ciphertext_file, &result_file)
      .await
      .unwrap();
    let actual_contents = tokio::fs::read_to_string(result_file).await.unwrap();
    assert_eq!(contents, actual_contents)
  }
}
