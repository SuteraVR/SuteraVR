#[macro_export]
macro_rules! suterpc_oneshot_schema {
    (
        variant: $variant:ident,
        struct Request $req: tt,
        struct Response $res: tt,
    ) => {
        $crate::suterpc_oneshot_schema!( @safe $variant, struct $req, struct $res);
    };
    (
        variant: $variant:ident,
        enum Request $req: tt,
        struct Response $res: tt,
    ) => {
        $crate::suterpc_oneshot_schema!( @safe $variant, enum $req, struct $res);
    };
    (
        variant: $variant:ident,
        struct Request $req: tt,
        enum Response $res: tt,
    ) => {
        $crate::suterpc_oneshot_schema!( @safe $variant, struct $req, enum $res);
    };
    (
        variant: $variant:ident,
        enum Request $req: tt,
        enum Response $res: tt,
    ) => {
        $crate::suterpc_oneshot_schema!( @safe $variant, enum $req, enum $res);
    };
    (
        @safe $variant:ident, $req_struct_or_enum:ident $req: tt, $res_struct_or_enum:ident $res: tt
    ) => {
        pub struct $variant {}
        ::paste::paste! {
            impl $crate::suterpc::Oneshot<'_> for $variant {
                type Request = [<$variant Request>];
                type Response = [<$variant Response>];
            }

            #[::alkahest::alkahest(Formula, SerializeRef, Deserialize)]
            pub(crate) $req_struct_or_enum [<$variant Request>] $req
            #[::alkahest::alkahest(Formula, SerializeRef, Deserialize)]
            pub(crate) $res_struct_or_enum [<$variant Response>] $res

            #[::async_trait::async_trait]
            impl $crate::suterpc::OneshotRequest<[<$variant Response>]> for [<$variant Request>] {
                async fn send<T: $crate::suterpc::Sender + Send>(self, server: T) -> $crate::suterpc::OneshotResult<[<$variant Response>]> {
                    server
                        .send_oneshot::<$variant>($crate::suterpc::OneshotVariants::$variant, self)
                        .await
                }
            }
        }
    };
}
