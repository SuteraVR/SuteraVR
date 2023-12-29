use alkahest::{advanced::BareFormula, Deserialize, Formula, SerializeRef};
use async_trait::async_trait;

use crate::schema::oneshot::OneshotVariants;

// TODO: OneshotError type is need.
pub type OneshotResult<Response> = Result<Response, ()>;

/// SuteRPCで扱う全てのリクエストを送信するトレイト
///
/// [`OneshotRequest`]トレイトの[`send`][OneshotRequest::send]メソッドなどから利用されます。
#[async_trait]
pub trait Sender {
    async fn send_oneshot<'de, T: Oneshot<'de>>(
        &self,
        variant: OneshotVariants,
        payload: T::Request,
    ) -> OneshotResult<T::Response>;
}

/// ワンショットリクエストのリクエストスキーマであることを示すマーカートレイト
pub trait OneshotRequestMarker {}

/// ワンショットリクエストのレスポンススキーマであることを示すマーカートレイト
pub trait OneshotResponseMarker {}

/// ワンショットリクエストを、リクエストスキーマから直接送信するためのトレイト
///
/// このトレイトを実装していると、[`send`][OneshotRequest::send]メソッドを呼び出すだけでリクエストを送信できます。
/// ```no_run
/// use suteravr_lib::schema_oneshot::{requests, responses, OneshotVariants};
/// use suteravr_lib::suterpc::{Oneshot, OneshotRequest, OneshotResult, Sender};
/// use async_trait::async_trait;
///
/// struct MockSender {}
///
/// #[async_trait]
/// impl Sender for MockSender {
///   async fn send_oneshot<'de, T: Oneshot<'de>>(
///     &self,
///     variant: OneshotVariants,
///     payload: T::Request,
///   ) -> OneshotResult<T::Response> {
///     unimplemented!()
///   }
/// }
///
/// // Senderトレイトを持った構造体をsendメゾットに渡すだけで送信できる
/// #[tokio::main]
/// async fn main() {
///   let response = requests::GetVersion::ClockingServer
///     .send(MockSender {})
///     .await
///     .unwrap();
///   assert_eq!(response.version, "v0.1.0");
/// }
/// ```
///
/// [`Sender`]トレイトは、Oneshotのペイロードを[`Sender::send_oneshot`]メソッドで送信できることを示すトレイトです。
///
/// `Sender`はふつう`server.sender()`のようにして得られますが、  
/// この例では、テスト用Senderを作成しています。
#[async_trait]
pub trait OneshotRequest<Response> {
    async fn send<T: Sender + Send>(self, server: T) -> OneshotResult<Response>;
}

/// SuteRPCのワンショットリクエストの、リクエストとレスポンスの型を扱うトレイト
///
/// (ワンショットリクエストとは、リクエストとレスポンスが1対1で対応するリクエストのことです。)
///
/// **ほとんどの場合、[`suterpc_oneshot_schema!`]マクロを使って、このトレイトを実装する型を定義します。**  
/// **`impl Oneshot<'_> for ...`のような実装を書く必要はありません。**
///
/// ライフタイムパラメータ-`'de`は、[`Deserialize`]トレイトの実装に必要なものです。
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
    ///   これを強制することで、[`OneshotRequest::send`]メソッドを呼び出すだけでリクエストを送信できるようにしています。
    type Request: Send
        + Formula
        + BareFormula
        + SerializeRef<Self::Request>
        + Deserialize<'de, Self::Request>
        + OneshotRequest<Self::Response>
        + OneshotRequestMarker;
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
        + Deserialize<'de, Self::Response>
        + OneshotResponseMarker;
}
