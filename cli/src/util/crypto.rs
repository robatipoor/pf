use sdk::util::{
  crypto::{decrypt_file, encrypt_file, KeyNonce},
  file::{add_extension, add_parent_dir, rm_extra_extension},
  random::generate_random_string_with_prefix,
};
use std::path::{Path, PathBuf};

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

#[cfg(test)]
mod tests {

  use fake::{Fake, Faker};
  use test_context::test_context;

  use sdk::util::{
    crypto::{KeyType, NonceType},
    test::FileTestContext,
  };

  use super::*;

  #[test_context(FileTestContext)]
  #[tokio::test]
  pub async fn test_encrypt_upload_file_and_decrypt_download_file(ctx: &mut FileTestContext) {
    let key_nonce = KeyNonce {
      key: KeyType::new("01234567890123456789012345678912").unwrap(),
      nonce: NonceType::new("1234567891213141516").unwrap(),
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
