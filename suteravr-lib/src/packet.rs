//! SuteRPCの通信に必要な情報を扱うモジュール

pub mod header;
pub mod request_type;
pub mod semver;

#[doc(inline)]
pub use header::RequestHeader;
#[doc(inline)]
pub use request_type::RequestType;
#[doc(inline)]
pub use semver::Semver;
