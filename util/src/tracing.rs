use once_cell::sync::Lazy;

pub static INIT_SUBSCRIBER: Lazy<()> = Lazy::new(|| tracing_subscriber::fmt().init());
