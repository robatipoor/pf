use std::path::{Path, PathBuf};

pub fn config_log() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(path_log_file()).unwrap())
        .apply()
        .unwrap();
}

pub fn path_log_file() -> PathBuf {
    dirs::home_dir().unwrap().join(Path::new(".pf.log"))
}
