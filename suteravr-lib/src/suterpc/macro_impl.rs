/// SuteRPCのワンショットリクエストのスキーマを定義するマクロ
///
/// (ワンショットリクエストとは、リクエストとレスポンスが1対1で対応するリクエストのことです。)
///
/// - `variant` には、このワンショットリクエストの種類を表す[`OneshotVariants`]のバリアントを指定します。
/// - `Request` には、リクエストのスキーマを表すenumまたはstructを指定します。
/// - `Response` には、レスポンスのスキーマを表すenumまたはstructを指定します。
///
/// [`suterpc_oneshot_variants!`]: crate::suterpc_oneshot_variants!
/// [`OneshotVariants`]: crate::schema::oneshot::OneshotVariants
/// [`Formula`]: alkahest::Formula
/// [`SerializeRef`]: alkahest::SerializeRef
/// [`Deserialize`]: alkahest::Deserialize
/// [`Oneshot`]: crate::suterpc::Oneshot
///
/// # 例
/// ```no_run
/// use suteravr_lib::suterpc_oneshot_schema;
/// suterpc_oneshot_schema! {
///     /// サーバーのバージョンを要求するワンショット。
///     variant: GetVersion,
///     enum Request {
///         SocialServer,
///         ClockingServer,
///     },
///     struct Response {
///         pub version: String,
///     },
/// }
/// ```
/// このマクロは、`suteravr_lib::schema::oneshot`に、  
/// `GetVersion`, `GetVersionRequest`, `GetVersionResponse`を公開します。
///
/// # 実装上の注意
/// #### Variantの定義
/// `variant`には、[`OneshotVariants`]のバリアントを指定します。  
/// このバリアントはマクロによって定義されます。詳しくは[`suterpc_oneshot_variants!`]を参照してください。
///
/// #### 可視性について
/// スキーマを利用するのは`clocking-server`, `client`などの外部クレートなので、
/// 可視性に注意しなけばなりません。
///
/// 以下のコードはコンパイルエラーにはなりませんが、
/// `clocking-server`や`client`など実際に利用するコードから`version`にアクセスできません。
/// ```no_run
/// use suteravr_lib::suterpc_oneshot_schema;
/// suterpc_oneshot_schema! {
///   /// サーバーのバージョンを要求するワンショット。
///   variant: GetVersion,
///   struct Request {},
///   struct Response {
///     // pub version: String, とすべき
///     version: String,
///   },
/// }
/// ```
/// スキーマ内のフィールドを非公開にする意味は基本的にないので、
/// `pub`を付けておきましょう。
///
/// #### ドキュメンテーションについて
/// 各スキーマは、**必ず** `variant`に対してドキュメンテーションを与える必要があります。  
/// これが欠如しているとコンパイルエラーになります。
/// ```compile_fail
/// use suteravr_lib::suterpc_oneshot_schema;
/// suterpc_oneshot_schema! {
///    // ただのコメントはドキュメンテーションにならない
///    variant: GetVersion,
///    struct Request {},
///    struct Response {},
/// }
/// ```
/// ドキュメンテーションとして認識されるには、Rustdocの形式で書く必要があります。  
/// 手軽なのはスラッシュ3つ`///`から始め, Markdownに従って書くことです。
/// ```no_run
/// use suteravr_lib::suterpc_oneshot_schema;
/// suterpc_oneshot_schema! {
///    /// サーバーのバージョンを要求するワンショット。
///    variant: GetVersion,
///    struct Request {},
///    struct Response {},
/// }
/// ```
/// 必須ではありませんが、リクエストとレスポンスそれぞれに対してドキュメンテーションを与えることができます。
/// ```no_run
/// use suteravr_lib::suterpc_oneshot_schema;
/// suterpc_oneshot_schema! {
///     /// サーバーのバージョンを要求するワンショット。(ここは必須)
///     variant: GetVersion,
///     /// どのサーバーのバージョンを要求するか指定する。(ここのドキュメンテーションは任意)
///     enum Request {
///         SocialServer,
///         ClockingServer,
///     },
///     /// サーバーのバージョンを返す。(ここのドキュメンテーションは任意)
///     struct Response {
///         /// 指定されたサーバーのバージョン (ここのドキュメンテーションは任意)
///         pub version: String,
///     },
/// }
/// ```
/// しかし、この例のドキュメンテーションは過剰であり、フィールド名から明らかなものに日本語で注釈をつける必要性はありません。
///
///
///
/// #### トレイト境界について
/// Request, Responseの型は、[`Formula`], [`SerializeRef`], [`Deserialize`], [`Send`] トレイト境界を要求します。  
/// (この要求は、このマクロが[`Oneshot`]トレイトを実装する型を生成するためです。)
///
/// [`Send`]トレイトはほとんどの型に実装されていますが、  
/// [`Formula`], [`SerializeRef`], [`Deserialize`] トレイトは[`alkahest`]クレートによって導出されるものです。
///
/// このマクロは、`Request`, `Response`として与えられたenum, structに対して自動でこれらを導出しますが、  
/// 外部のenum, structが含まれる場合は、既にこれらのトレイトが導出されている必要があります。
///
/// **以下のコードはコンパイルエラーになります:**
/// ```compile_fail
/// use suteravr_lib::suterpc_oneshot_schema;
///
/// pub struct MyStruct {
///   pub clocking_server: String,
///   pub social_server: String,
/// }
///
/// suterpc_oneshot_schema! {
///   /// サーバーのバージョンを要求するワンショット。
///   variant: GetVersion,
///   struct Request {},
///   struct Response {
///     pub version: MyStruct,
///     // -> MyStructはFormulaを実装しておらず、
///     //    バイナリにシリアライズ/デシリアライズできない
///   },
/// }
///
/// ```
///
/// この場合、`MyStruct`に対して、[`alkahest::alkahest`]マクロを使って、トレイトを導出すれば問題ありません。
/// ```no_run
/// use alkahest::{alkahest, Formula, SerializeRef, Deserialize};
/// use suteravr_lib::suterpc_oneshot_schema;
///
/// //↓ これで`MyStruct`をバイナリにシリアライズ/デシリアライズできるようになる
/// #[alkahest(Formula, SerializeRef, Deserialize)]
/// struct MyStruct {
///   pub clocking_server: String,
///   pub social_server: String,
/// }
///
/// suterpc_oneshot_schema! {
///   /// サーバーのバージョンを要求するワンショット。
///   variant: GetVersion,
///   struct Request {},
///   struct Response {
///     pub version: MyStruct,
///   },
/// }
/// ```
///
#[macro_export]
macro_rules! suterpc_oneshot_schema {
    (
        $(#[doc = $doc:expr])+
        variant: $variant:ident,
        $(#[doc = $req_doc:expr])*
        struct Request $req: tt,
        $(#[doc = $res_doc:expr])*
        struct Response $res: tt$(,)?
    ) => {
        $crate::suterpc_oneshot_schema!( @safe [$($doc)+], $variant, struct [$($req_doc)*] $req, struct [$($res_doc)*] $res);
    };
    (
        $(#[doc = $doc:expr])+
        variant: $variant:ident,
        $(#[doc = $req_doc:expr])*
        enum Request $req: tt,
        $(#[doc = $res_doc:expr])*
        struct Response $res: tt$(,)?
    ) => {
        $crate::suterpc_oneshot_schema!( @safe [$($doc)+], $variant, enum [$($req_doc)*] $req, struct [$($res_doc)*] $res);
    };
    (
        $(#[doc = $doc:expr])+
        variant: $variant:ident,
        $(#[doc = $req_doc:expr])*
        struct Request $req: tt,
        $(#[doc = $res_doc:expr])*
        enum Response $res: tt$(,)?
    ) => {
        $crate::suterpc_oneshot_schema!( @safe [$($doc)+], $variant, struct [$($req_doc)*] $req, enum [$($res_doc)*] $res);
    };
    (
        $(#[doc = $doc:expr])+
        variant: $variant:ident,
        $(#[doc = $req_doc:expr])*
        enum Request $req: tt,
        $(#[doc = $res_doc:expr])*
        enum Response $res: tt$(,)?
    ) => {
        $crate::suterpc_oneshot_schema!( @safe [$($doc)+], $variant, enum [$($req_doc)*] $req, enum [$($res_doc)*] $res);
    };
    (
        @safe [$($doc:expr)+], $variant:ident,
        $req_type:ident [$($req_doc:expr)*] $req:tt,
        $res_type:ident [$($res_doc:expr)*] $res:tt
    ) => {
        ::paste::paste! {
            $(#[doc = $doc])+
            #[doc = ""]
            #[doc = "# Schema"]
            #[doc = concat!(
                "リクエストスキーマ: **[`", ::std::stringify!($variant), "Request`]",
                "[crate::schema_oneshot::requests::", ::std::stringify!($variant), "`]**  ",
            )]
            #[doc = concat!(
                "レスポンススキーマ: **[`", ::std::stringify!($variant), "Response`]",
                "[crate::schema_oneshot::responses::", ::std::stringify!($variant), "`]**  ",
            )]
            pub struct $variant {}
            impl $crate::suterpc::Oneshot<'_> for $variant {
                type Request = [<$variant Request>];
                type Response = [<$variant Response>];
            }

            impl $crate::suterpc::OneshotRequestMarker for [<$variant Request>] {}
            impl $crate::suterpc::OneshotResponseMarker for [<$variant Response>] {}

            #[doc(hidden)]
            #[doc = concat!("[`", ::std::stringify!($variant), "`]のリクエスト用スキーマ。  ")]
            #[doc = ""]
            $(#[doc = $req_doc])*
            #[::alkahest::alkahest(Formula, SerializeRef, Deserialize)]
            pub $req_type [<$variant Request>] $req

            #[doc(hidden)]
            #[doc = concat!("[`", ::std::stringify!($variant), "`]のレスポンス用スキーマ。  ")]
            #[doc = ""]
            $(#[doc = $res_doc])*
            #[::alkahest::alkahest(Formula, SerializeRef, Deserialize)]
            pub $res_type [<$variant Response>] $res

            #[::async_trait::async_trait]
            impl $crate::suterpc::OneshotRequest<[<$variant Response>]> for [<$variant Request>] {
                async fn send<T: $crate::suterpc::Sender + Send>(self, server: T)
                  -> [<$variant Response>] {
                    server
                        .send_oneshot::<$variant>($crate::schema::oneshot::OneshotVariants::$variant, self)
                        .await
                }
            }
        }
    };
}

/// SuteRPCのワンショットリクエストの種類を定義するマクロ
///
/// (ワンショットリクエストとは、リクエストとレスポンスが1対1で対応するリクエストのことです。)
///
/// ここで定義されたバリアントは、そのスキーマが定義されている必要があります。  
/// **スキーマを定義したモジュールをここで公開しておく必要があります。**
///
/// # 例
/// `GetVersion`というワンショットリクエストの種類を定義します。
///
/// `suteravr-lib/src/schema/oneshot.rs`
/// ```ignore
/// use suteravr_lib::suterpc_oneshot_variants;
///
/// pub mod schema_version;
/// // <- 必ずpubが必要。
/// //    mod schema_version; とすると、ドキュメンテーションが生成されない。
///
/// suterpc_oneshot_variants! {
///     [schema_version] GetVersion = 0,
/// }
/// ```
///
/// `suteravr-lib/src/schema/oneshot/schema_version.rs`
/// ```ignore
/// use suteravr_lib::suterpc_oneshot_schema;
/// suterpc_oneshot_schema! {
///   ...
///   variant: GetVersion,
///   ...
/// }
/// ``````
///
#[macro_export]
macro_rules! suterpc_oneshot_variants {
    (
        $([$locate:path]$name:ident = $value:expr),* $(,)?
    ) => {
        #[doc = "ワンショットリクエストの種類を定義します。  "]
        #[doc = "[`suterpc_oneshot_schema!`][crate::suterpc_oneshot_schema!]マクロを使って、このトレイトを実装する型を定義します。"]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, ::num_derive::FromPrimitive)]
        pub enum OneshotVariants {
            $(
                #[doc = concat!("[`", ::std::stringify!($name), "`]")]
                $name = $value
            ),*
        }

        $(
            ::paste::paste!{
                #[doc(inline)]
                #[doc = concat!(
                  "ID: `", ::std::stringify!($value), "`,  ",
                  "バリアント:[`OneshotVariants::", ::std::stringify!($name), "`]  ",
                  "(in [`", ::std::stringify!($locate) ,"`])  "
                )]
                pub use $locate::{$name};
                #[doc(hidden)]
                pub use $locate::{[<$name Request>], [<$name Response>]};
            }
        )*

        /// **リクエストスキーマを全て列挙したモジュール**
        pub mod requests {
            ::paste::paste! {
                $(
                    #[doc(inline)]
                    pub use super::[<$name Request>] as $name;
                )*
            }
        }

        /// **レスポンススキーマを全て列挙したモジュール**
        pub mod responses {
            ::paste::paste! {
                $(
                    #[doc(inline)]
                    pub use super::[<$name Response>] as $name;
                )*
            }
        }
    };
}
