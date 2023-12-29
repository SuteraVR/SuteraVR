//! 認証を行うリクエストのスキーマを定義するモジュール

use crate::{suterpc_oneshot_schema, typing::id::PlayerIdentifier};
suterpc_oneshot_schema! {
    /// セッションをインスタンスのプレイヤーとして認証することを求めるワンショット
    ///
    /// # Examples
    /// ```no_run
    /// use suteravr_lib::schema_oneshot::{requests, responses};
    /// use suteravr_lib::typing::id::{PlayerIdentifier, InstanceIdentifier};
    ///
    /// // リクエスト
    /// let request = requests::RequestPlayerAuth {
    ///   player: PlayerIdentifier(InstanceIdentifier(6), 3),
    ///   token: String::from("SUTERAVRTOKEN-1a2b3c..."),
    /// };
    ///
    /// // レスポンス
    /// let response = responses::RequestPlayerAuth::Ok;
    /// ```
    variant: RequestPlayerAuth,
    struct Request {
        /// 認証されるプレイヤーの識別子。
        /// Headerのものと一致する必要がある。
        pub player: PlayerIdentifier,
        /// Balancing-serverから得たトークン
        pub token: String,
    },
    enum Response {
        /// 認証に成功した場合
        Ok,
        /// ペイロード内とヘッダー内でPlayerが一致しない場合
        PlayerIdentifierMismatch,
        /// トークンが期限切れの場合
        TokenExpired,
        /// トークンが不正な場合
        BadToken,
    }
}
