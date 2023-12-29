use num_derive::FromPrimitive;

/// SuteRPC上のエラーの種類を表す列挙体
///
/// 基本的なエラーを、[`ErrorVariants`]によって**返却してはいけません**。  
/// 代わりに、列挙体によってエラーを表現してください。  
/// 例えば[`OneshotRequest`][crate::packet::request_type::RequestType::Oneshot]のようなリクエストに対しては、
/// 以下のようにしてクライアントにエラーを通知すべきです。
/// ```no_run
/// # type Reason = u8;
/// enum Response {
///   Success,
///   Error(Reason)
/// }
/// ```
///
/// [`ErrorVariants`]を使用するのは、そもそもクライアントのリクエストが正常ではなく、   
/// 上記のような方法でエラーを返してもクライアントが解釈できると期待されない場合に限ります。
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum ErrorVariants {
    /// クライアントの渡すヘッダーが存在しないか正しくない、  
    /// バイナリのペイロードがデシリアライズできない場合など
    BadRequest = 0,

    /// クライアントのバージョンがサーバーのバージョンと一致しない場合
    VersionMismatch = 1,

    /// クライアントとのセッションが認証されていない場合
    Unauthorized = 2,
}
