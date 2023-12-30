//! Clocking-server
//!
//! [`suteravr-lib`][suteravr_lib]が使用できます。
//! ```no_run
//! use suteravr_lib::Foo;
//! ```
//!

use alkahest::{deserialize, DeserializeError};
use async_trait::async_trait;
use instance::WorldInstance;
use player::Player;
use std::collections::HashMap;
use suteravr_lib::{
    packet::{
        OneshotImplementer, RequestHeader, RequestType, ResponseHeader, SuterpcRequestPayload,
        SuterpcResponsePayload,
    },
    schema::error::ErrorVariants,
    schema_oneshot::{requests, responses, GetVersion, OneshotVariants},
    suterpc::Oneshot,
    typing::id::{InstanceIdentifier, PlayerIdentifier, RequestIdentifier},
};
use typing::Arw;
use version::SCHEMA_SEMVER;

pub mod instance;
pub mod player;
pub mod typing;
pub mod version;

pub struct ReqCtx {}

pub struct Server {
    pub instances: HashMap<InstanceIdentifier, Arw<WorldInstance>>,
    pub players: HashMap<PlayerIdentifier, Arw<Player>>,
}

impl Server {
    pub async fn binary_encode(&self, request: &[u8]) -> Vec<u8> {
        match RequestHeader::parse_binary(request) {
            // リクエストのパースに成功した場合
            Ok((header, payload)) => {
                // リクエストのスキーマバージョンがサーバーのものと一致しない場合
                // SchemaVersionMismatchを返却
                if header.schema_version != *SCHEMA_SEMVER {
                    return SuterpcResponsePayload::<()> {
                        header: ResponseHeader {
                            schema_version: *SCHEMA_SEMVER,
                            request_id: header.request_id,
                            request_type: RequestType::Error(ErrorVariants::VersionMismatch),
                        },
                        payload: (),
                    }
                    .into();
                }
                // リクエストの種類に応じて処理を行う
                let context = ReqCtx {};
                self.routing(header, context, payload).await.unwrap_or(
                    SuterpcResponsePayload::<()> {
                        header: ResponseHeader {
                            schema_version: *SCHEMA_SEMVER,
                            request_id: header.request_id,
                            request_type: RequestType::Error(ErrorVariants::BadRequest),
                        },
                        payload: (),
                    }
                    .into(),
                )
            }

            // リクエストのパースに失敗した場合 BadRequestを返却
            Err(_) => SuterpcResponsePayload::<()> {
                header: ResponseHeader {
                    schema_version: *SCHEMA_SEMVER,
                    request_id: RequestIdentifier::default(),
                    request_type: RequestType::Error(ErrorVariants::BadRequest),
                },
                payload: (),
            }
            .into(),
        }
    }
}

impl Server {
    async fn routing(
        &self,
        header: RequestHeader,
        ctx: ReqCtx,
        payload: &[u8],
    ) -> Result<Vec<u8>, DeserializeError> {
        Ok(match header.request_type {
            RequestType::Oneshot(OneshotVariants::GetVersion) => self
                .handle(ctx, self.parse::<GetVersion>(header, payload)?)
                .await
                .into(),
            _ => {
                // どの処理にも該当してなければ、未実装としてUnimplementedを返却
                SuterpcResponsePayload::<()> {
                    header: ResponseHeader {
                        schema_version: *SCHEMA_SEMVER,
                        request_id: header.request_id,
                        request_type: RequestType::Error(ErrorVariants::Unimplemented),
                    },
                    payload: (),
                }
                .into()
            }
        })
    }
}

impl Server {
    pub fn parse<'de, T: Oneshot<'de>>(
        &self,
        header: RequestHeader,
        payload: &'de [u8],
    ) -> Result<SuterpcRequestPayload<T::Request>, DeserializeError> {
        match deserialize::<'de, T::Request, T::Request>(payload) {
            Ok(payload) => Ok(SuterpcRequestPayload::<T::Request> { header, payload }),
            Err(e) => Err(e),
        }
    }
}

#[async_trait]
impl OneshotImplementer<'_, GetVersion, ReqCtx> for Server {
    async fn handle(
        &self,
        _ctx: ReqCtx,
        req: SuterpcRequestPayload<requests::GetVersion>,
    ) -> SuterpcResponsePayload<responses::GetVersion> {
        SuterpcResponsePayload::<responses::GetVersion> {
            header: ResponseHeader {
                schema_version: *SCHEMA_SEMVER,
                request_id: req.header.request_id,
                request_type: RequestType::Oneshot(OneshotVariants::GetVersion),
            },
            payload: responses::GetVersion {
                version: "0.1.0".into(),
            },
        }
    }
}
