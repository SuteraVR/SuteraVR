use once_cell::sync::Lazy;
use std::env;

pub static PORT: Lazy<u16> = Lazy::new(|| match env::var("PORT") {
    Ok(val) => val.parse().unwrap(),
    Err(_) => 3501,
});
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
