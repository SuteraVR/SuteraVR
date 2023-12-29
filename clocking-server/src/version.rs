use once_cell::sync::Lazy;
use suteravr_lib::semver::Semver;

const VERSION_STR: &str = env!("CARGO_PKG_VERSION");
pub static SERVER_VERSION: Lazy<Semver> = Lazy::new(|| VERSION_STR.into());
