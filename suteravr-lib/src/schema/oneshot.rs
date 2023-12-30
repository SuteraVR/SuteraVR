//! ワンショットリクエストのスキーマを定義するモジュール ([`schema::oneshot`][crate::schema::oneshot])  
//! (ワンショットリクエストとは、リクエストとレスポンスが1対1で対応するリクエストのことです。)
//!
//! クライアントからサーバーへ送信するenumまたはstruct (**リクエストスキーマ**)を、[`Request`]に指定します。  
//! サーバーからクライアントへ送信するenumまたはstruct (**レスポンススキーマ**)を、[`Response`]に指定します。
//!
//! [`Request`]: crate::suterpc::Oneshot::Request
//! [`Response`]: crate::suterpc::Oneshot::Response

pub mod schema_auth;
pub mod schema_version;

pub(crate) use crate::suterpc_oneshot_variants;

suterpc_oneshot_variants! {
    [schema_version] GetVersion = 0,
    [schema_auth]    RequestPlayerAuth = 2,
}
