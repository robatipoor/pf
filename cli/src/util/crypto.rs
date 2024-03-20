use super::progress::progress_bar;
use pf_sdk::util::{
  crypto::{decrypt, decrypt_file, encrypt, encrypt_file, KeyNonce},
  file::{add_extension, add_parent_dir, rm_extra_extension},
  random::generate_random_string_with_prefix,
};
use std::path::{Path, PathBuf};
use tokio::fs::File;

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
) -> anyhow::Result<PathBuf> {
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
  tokio::fs::rename(&destination_file, &decrypted_file)
    .await
    .unwrap();
  tokio::fs::remove_dir(destination_file.parent().unwrap())
    .await
    .unwrap();
  Ok(decrypted_file)
}

pub async fn decrypt_download_file_with_progress_bar(
  key_nonce: &KeyNonce,
  encrypted_file: impl AsRef<Path>,
) -> anyhow::Result<PathBuf> {
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
  tokio::fs::rename(&destination_file, &decrypted_file)
    .await
    .unwrap();
  tokio::fs::remove_dir(destination_file.parent().unwrap())
    .await
    .unwrap();
  Ok(decrypted_file)
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
  encrypt(
    key_nonce,
    pb.wrap_async_read(reader)
      .with_finish(indicatif::ProgressFinish::WithMessage(
        "Encrypt completed successfully.".into(),
      )),
    writer,
  )
  .await?;
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
  decrypt(
    key_nonce,
    pb.wrap_async_read(reader)
      .with_finish(indicatif::ProgressFinish::WithMessage(
        "Decrypt completed successfully.".into(),
      )),
    writer,
  )
  .await?;
  Ok(())
}

#[cfg(test)]
mod tests {

  use fake::{Fake, Faker};
  use test_context::test_context;

  use pf_sdk::util::{
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
