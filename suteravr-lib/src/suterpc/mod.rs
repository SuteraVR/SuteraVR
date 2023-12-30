//! `clocking-server`と`client`の間でやり取りされるデータのスキーマを構成するモジュール
mod oneshot;

pub(crate) mod macro_impl;
pub use oneshot::*;
