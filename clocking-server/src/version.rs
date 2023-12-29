use suteravr_lib::semver::Semver;

const VERSION_STR: &str = env!("CARGO_PKG_VERSION");
pub const SERVER_VERSION: Semver = VERSION_STR.into();
