use anyhow::anyhow;
use chacha20poly1305::{
  aead::{
    stream::{DecryptorBE32, EncryptorBE32},
    KeyInit,
  },
  XChaCha20Poly1305,
};
use indicatif::ProgressBar;
use sdk::util::{
  crypto::{decrypt_file, encrypt_file, KeyNonce, DECRYPT_BUFFER_LEN, ENCRYPT_BUFFER_LEN},
  file::{add_extension, add_parent_dir, rm_extra_extension},
  random::generate_random_string_with_prefix,
};

use std::path::{Path, PathBuf};
use tokio::{
  fs::File,
  io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
};

use super::progress::progress_bar;

pub async fn encrypt_upload_file(
  key_nonce: &KeyNonce,
  plaintext_file: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
  let encrypted_file = add_extension(plaintext_file.as_ref(), "bin");
  encrypt_file(key_nonce, plaintext_file, encrypted_file.as_path()).await?;
  Ok(encrypted_file)
}

pub async fn decrypt_download_file(
  key_nonce: &KeyNonce,
  encrypted_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
  let decrypted_file = rm_extra_extension(&encrypted_file).unwrap();
  let destination_file =
    add_parent_dir(&decrypted_file, &generate_random_string_with_prefix("tmp")).unwrap();
  tokio::fs::create_dir(&destination_file.parent().unwrap())
    .await
    .unwrap();
  decrypt_file(key_nonce, &encrypted_file, destination_file.as_path())
    .await
    .unwrap();
  tokio::fs::remove_file(&encrypted_file).await.unwrap();
  tokio::fs::rename(&destination_file, decrypted_file)
    .await
    .unwrap();
  tokio::fs::remove_dir(destination_file.parent().unwrap())
    .await
    .unwrap();
  Ok(())
}

pub async fn encrypt_upload_file_with_progress_bar(
  key_nonce: &KeyNonce,
  plaintext_file: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
  let encrypted_file = add_extension(plaintext_file.as_ref(), "bin");
  encrypt_file_with_progress_bar(key_nonce, plaintext_file, encrypted_file.as_path()).await?;
  Ok(encrypted_file)
}

pub async fn decrypt_download_file_with_progress_bar(
  key_nonce: &KeyNonce,
  encrypted_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
  let decrypted_file = rm_extra_extension(&encrypted_file).unwrap();
  let destination_file =
    add_parent_dir(&decrypted_file, &generate_random_string_with_prefix("tmp")).unwrap();
  tokio::fs::create_dir(&destination_file.parent().unwrap())
    .await
    .unwrap();
  decrypt_file_with_progress_bar(key_nonce, &encrypted_file, destination_file.as_path())
    .await
    .unwrap();
  tokio::fs::remove_file(&encrypted_file).await.unwrap();
  tokio::fs::rename(&destination_file, decrypted_file)
    .await
    .unwrap();
  tokio::fs::remove_dir(destination_file.parent().unwrap())
    .await
    .unwrap();
  Ok(())
}

pub async fn encrypt_file_with_progress_bar(
  key_nonce: &KeyNonce,
  plaintext_file: impl AsRef<Path>,
  destination_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
  let reader = File::open(plaintext_file).await?;
  let writer = File::create(destination_file).await?;
  let total_size = reader.metadata().await?.len();
  let pb = progress_bar(total_size)?;
  encrypt_with_progress_bar(key_nonce, reader, writer, pb).await?;
  Ok(())
}

pub async fn decrypt_file_with_progress_bar(
  key_nonce: &KeyNonce,
  encrypted_file: impl AsRef<Path>,
  destination_file: impl AsRef<Path>,
) -> anyhow::Result<()> {
  let reader = File::open(encrypted_file).await?;
  let writer = File::create(destination_file).await?;
  let total_size = reader.metadata().await?.len();
  let pb = progress_bar(total_size)?;
  decrypt_with_progress_bar(key_nonce, reader, writer, pb).await?;
  Ok(())
}

pub async fn encrypt_with_progress_bar<R, W>(
  KeyNonce { key, nonce }: &KeyNonce,
  mut reader: R,
  mut writer: W,
  pb: ProgressBar,
) -> anyhow::Result<()>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  let mut buffer = [0u8; ENCRYPT_BUFFER_LEN];
  let mut stream_encryptor =
    EncryptorBE32::from_aead(XChaCha20Poly1305::new(key), (*nonce).as_ref().into());
  let mut total_read = 0;
  loop {
    let read_count = reader.read(&mut buffer).await?;
    total_read += read_count;
    pb.set_position(total_read as u64);
    if read_count == ENCRYPT_BUFFER_LEN {
      let ciphertext = stream_encryptor
        .encrypt_next(buffer.as_slice())
        .map_err(|err| anyhow!("Encrypting file failed, Error: {err}"))?;
      writer.write_all(&ciphertext).await?;
    } else if read_count == 0 {
      pb.finish_with_message("Encrypt completed successfully.");
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

pub async fn decrypt_with_progress_bar<R, W>(
  KeyNonce { key, nonce }: &KeyNonce,
  mut reader: R,
  mut writer: W,
  pb: ProgressBar,
) -> anyhow::Result<()>
where
  R: AsyncRead + Unpin,
  W: AsyncWrite + Unpin,
{
  let mut buffer = [0u8; DECRYPT_BUFFER_LEN];
  let mut stream_decryptor =
    DecryptorBE32::from_aead(XChaCha20Poly1305::new(key), nonce.as_ref().into());
  let mut total_read = 0;
  loop {
    let read_count = reader.read(&mut buffer).await?;
    total_read += read_count;
    pb.set_position(total_read as u64);
    if read_count == DECRYPT_BUFFER_LEN {
      let plaintext = stream_decryptor
        .decrypt_next(buffer.as_slice())
        .map_err(|err| anyhow!("Decrypting file failed, Error: {err}"))?;
      writer.write_all(&plaintext).await?;
    } else if read_count == 0 {
      pb.finish_with_message("Decrypt completed successfully.");
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

  use sdk::util::{
    crypto::{KeyType, NonceType},
    random::generate_random_string,
    test::FileTestContext,
  };

  use super::*;

  #[test_context(FileTestContext)]
  #[tokio::test]
  pub async fn test_encrypt_upload_file_and_decrypt_download_file(ctx: &mut FileTestContext) {
    let key_nonce = KeyNonce {
      key: KeyType::new(&generate_random_string(32)).unwrap(),
      nonce: NonceType::new(&generate_random_string(19)).unwrap(),
    };
    let contents: String = Faker.fake::<String>();
    let plaintext_file = ctx.temp_path.join("file.txt");
    tokio::fs::write(&plaintext_file, &contents).await.unwrap();
    let ciphertext_file = encrypt_upload_file(&key_nonce, &plaintext_file)
      .await
      .unwrap();
    tokio::fs::remove_file(&plaintext_file).await.unwrap();
    let exist = tokio::fs::try_exists(&ciphertext_file).await.unwrap();
    assert!(exist, "ciphertext file {ciphertext_file:?} should be exist");
    decrypt_download_file(&key_nonce, &ciphertext_file)
      .await
      .unwrap();
    let exist = tokio::fs::try_exists(&ciphertext_file).await.unwrap();
    assert!(!exist, "ciphertext file should not be exist");
    let actual_contents = tokio::fs::read_to_string(plaintext_file).await.unwrap();
    assert_eq!(contents, actual_contents)
  }
}
