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
  io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};

pub const DECRYPT_BUFFER_LEN: usize = 500 + 16;
pub const ENCRYPT_BUFFER_LEN: usize = 500;

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

#[derive(Debug, Clone, Copy)]
pub struct KeyNonce {
  pub key: KeyType,
  pub nonce: NonceType,
}

pub async fn encrypt_file(
  key_nonce: &KeyNonce,
  plaintext_file: impl AsRef<Path>,
  destination_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
  let reader = File::open(plaintext_file).await?;
  let writer = File::create(destination_file).await?;
  encrypt(key_nonce, reader, writer).await?;
  Ok(())
}

pub async fn decrypt_file(
  key_nonce: &KeyNonce,
  encrypted_file: impl AsRef<Path>,
  destination_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
  let reader = File::open(encrypted_file).await?;
  let writer = File::create(destination_file).await?;
  decrypt(key_nonce, reader, writer).await?;
  Ok(())
}

pub async fn encrypt<R, W>(
  KeyNonce { key, nonce }: &KeyNonce,
  mut reader: R,
  mut writer: W,
) -> anyhow::Result<()>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  let mut buffer = [0u8; ENCRYPT_BUFFER_LEN];
  let mut stream_encryptor =
    EncryptorBE32::from_aead(XChaCha20Poly1305::new(key), (*nonce).as_ref().into());
  loop {
    let read_count = reader.read(&mut buffer).await?;
    if read_count == ENCRYPT_BUFFER_LEN {
      let ciphertext = stream_encryptor
        .encrypt_next(buffer.as_slice())
        .map_err(|err| anyhow!("Encrypting file failed, Error: {err}"))?;
      writer.write_all(&ciphertext).await?;
    } else if read_count == 0 {
      break;
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

pub async fn decrypt<R, W>(
  KeyNonce { key, nonce }: &KeyNonce,
  mut reader: R,
  mut writer: W,
) -> anyhow::Result<()>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  let mut buffer = [0u8; DECRYPT_BUFFER_LEN];
  let mut stream_decryptor =
    DecryptorBE32::from_aead(XChaCha20Poly1305::new(key), nonce.as_ref().into());

  loop {
    let read_count = reader.read(&mut buffer).await?;
    if read_count == DECRYPT_BUFFER_LEN {
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
  writer.flush().await?;

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
  pub async fn test_encrypt_file_and_decrypt_file(ctx: &mut FileTestContext) {
    let key_nonce = KeyNonce {
      key: KeyType::new("01234567890123456789012345678912").unwrap(),
      nonce: NonceType::new("1234567891213141516").unwrap(),
    };
    let contents = Faker.fake::<String>();

    let plaintext_file = ctx.temp_path.join("file.txt");
    tokio::fs::write(&plaintext_file, &contents).await.unwrap();
    let ciphertext_file = ctx.temp_path.join("file.bin");
    encrypt_file(&key_nonce, &plaintext_file, &ciphertext_file)
      .await
      .unwrap();
    let result_file = ctx.temp_path.join("result_file.txt");
    decrypt_file(&key_nonce, &ciphertext_file, &result_file)
      .await
      .unwrap();
    let actual_contents = tokio::fs::read_to_string(result_file).await.unwrap();
    assert_eq!(contents, actual_contents)
  }
}
