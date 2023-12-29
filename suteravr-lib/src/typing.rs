//! SuteraVR全体で使う構造体, 列挙体を定義するモジュール

pub mod id;

/// バイナリにシリアライズされることを想定している構造体, 列挙体に実装するトレイト
pub trait SizedForBinary {
    /// バイナリにシリアライズされたときのサイズを示します。   
    /// [u8] の要素数ともいえます。
    const SIZE: usize;
}
