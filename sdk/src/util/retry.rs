pub const MAX_RETRY: u32 = 10;
pub const MINIMUM_DELAY_TIME: std::time::Duration = std::time::Duration::from_millis(100);

#[macro_export]
macro_rules! retry {
  ($func:expr) => {{
    let mut remaining_attempts = $crate::util::retry::MAX_RETRY;
    let mut delay = $crate::util::retry::MINIMUM_DELAY_TIME;
    loop {
      remaining_attempts -= 1;
      let result = $func().await;
      if result.is_ok() {
        break result;
      } else if remaining_attempts == 0 {
        tracing::warn!("Maximum number of attempts exceeded.");
        break result;
      }
      tokio::time::sleep(delay).await;
      delay += $crate::util::retry::MINIMUM_DELAY_TIME;
    }
  }};
  ($func:expr,$predicate:expr) => {{
    let mut remaining_attempts = $crate::util::retry::MAX_RETRY;
    let mut delay = $crate::util::retry::MINIMUM_DELAY_TIME;
    loop {
      remaining_attempts -= 1;
      let result = $func().await;
      if $predicate(&result) {
        break result;
      } else if remaining_attempts == 0 {
        tracing::warn!("Maximum number of attempts exceeded.");
        break result;
      }
      tokio::time::sleep(delay).await;
      delay += $crate::util::retry::MINIMUM_DELAY_TIME;
    }
  }};
}
