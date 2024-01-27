use once_cell::sync::Lazy;
use std::env;

pub enum SuteraEnv {
    Development,
    Production,
}

pub static PORT: Lazy<u16> = Lazy::new(|| match env::var("PORT") {
    Ok(val) => val.parse().unwrap(),
    Err(_) => 3501,
});
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub static ENV: Lazy<SuteraEnv> = Lazy::new(|| match env::var("ENV") {
    Ok(val) => match val.to_lowercase().as_str() {
        "development" => SuteraEnv::Development,
        "production" => SuteraEnv::Production,
        _ => SuteraEnv::Development,
    },
    Err(_) => SuteraEnv::Development,
});
