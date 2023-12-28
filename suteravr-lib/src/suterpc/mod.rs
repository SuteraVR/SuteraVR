pub(crate) mod macro_impl;

use alkahest::{advanced::BareFormula, Deserialize, Formula, SerializeRef};
use async_trait::async_trait;

use crate::schema::oneshot::OneshotVariants;

// TODO: OneshotError type is need.
pub type OneshotResult<Response> = Result<Response, ()>;

#[async_trait]
pub trait Sender {
    async fn send_oneshot<'de, T: Oneshot<'de>>(
        &self,
        variant: OneshotVariants,
        payload: T::Request,
    ) -> OneshotResult<T::Response>;
}

#[async_trait]
pub trait OneshotRequest<Response> {
    async fn send<T: Sender + Send>(self, server: T) -> OneshotResult<Response>;
}

/// SuteRPCのワンショットリクエストの、リクエストとレスポンスの型を扱うトレイトです。
///
/// (ワンショットリクエストとは、リクエストとレスポンスが1対1で対応するリクエストのことです。)
///
/// **ほとんどの場合、[`suterpc_oneshot_schema!`]マクロを使って、このトレイトを実装する型を定義します。**  
/// **`impl Oneshot<'_> for ...`のような実装を書く必要はありません。**
///
/// [`Request`]: Self::Request
/// [`Response`]: Self::Response
/// [`suterpc_oneshot_schema!`]: crate::suterpc_oneshot_schema!
pub trait Oneshot<'de> {
    /// **リクエスト(クライアントからサーバーへ送信するデータ)** のスキーマを保持する関連型です。
    ///
    /// - [`Formula`], [`SerializeRef`], [`Deserialize`] トレイト境界を要求します  
    ///   (TCP通信のために、どちらもバイナリにシリアライズ/デシリアライズできる必要があるため)  
    /// - [`Send`]マーカートレイト境界を要求します  
    ///   (どちらもスレッド間で安全に所有権を移動できる必要があるため)
    /// - [`OneshotRequest`]トレイト境界を要求します
    ///   これを強制することで、[`OneshotRequest::send`]メソッドを呼び出すだけでリクエストを送信できるようにしています。。
    type Request: Send
        + Formula
        + BareFormula
        + SerializeRef<Self::Request>
        + Deserialize<'de, Self::Request>
        + OneshotRequest<Self::Response>;
    /// **レスポンス(サーバーからクライアントへ送信するデータ)** のスキーマを保持する関連型です。
    ///
    /// - [`Formula`], [`SerializeRef`], [`Deserialize`] トレイト境界を要求します  
    ///   (TCP通信のために、どちらもバイナリにシリアライズ/デシリアライズできる必要があるため)  
    /// - [`Send`]マーカートレイト境界を要求します  
    ///   (どちらもスレッド間で安全に所有権を移動できる必要があるため)
    type Response: Send
        + Formula
        + BareFormula
        + SerializeRef<Self::Response>
        + Deserialize<'de, Self::Response>;
}
