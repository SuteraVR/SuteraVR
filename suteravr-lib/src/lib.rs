pub mod schema;
pub mod suterpc;
pub mod typing;

#[doc(inline)]
pub use schema::oneshot as schema_oneshot;

/// このクレートが他からインポートできることを確認するテスト用の構造体
pub struct Foo {}
