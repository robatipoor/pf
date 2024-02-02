use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use std::fmt::Write;

pub fn progress_bar(total_size: u64) -> anyhow::Result<ProgressBar> {
  let pb = ProgressBar::new(total_size);
  pb.set_style(
    ProgressStyle::with_template(
      "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}",
    )
    .map_err(|e| anyhow::anyhow!("Invalid template progress bar. Error: {e}"))?
    .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
      write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
    })
    .progress_chars("#>-"),
  );
  Ok(pb)
}
