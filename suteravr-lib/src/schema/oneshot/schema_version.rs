//! バージョン情報を共有するリクエストのスキーマを定義するモジュール

use crate::{semver::Semver, suterpc_oneshot_schema};

suterpc_oneshot_schema! {
    /// サーバーのバージョンを要求するワンショット。
    ///
    /// # Examples
    /// ```no_run
    /// use suteravr_lib::schema_oneshot::{requests, responses};
    ///
    /// // リクエスト
    /// let request = requests::GetVersion::ClockingServer;
    ///
    /// // レスポンス
    /// let response = responses::GetVersion {
    ///   version: "v0.1.0".into(),
    /// };
    /// ```
    variant: GetVersion,
    /// どのサーバーのバージョンを要求するか指定する。
    enum Request {
        SocialServer,
        ClockingServer,
    },
    struct Response {
        pub version: Semver,
    },
}
