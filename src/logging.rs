use env_logger::{Builder, Env};
use log::LevelFilter;
use chrono::Local;
use std::io::Write;

pub fn init_logger() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {}",
                // %Y-%m-%d
                Local::now().format("%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Debug)
        .init();
}
