use std::mem::size_of;

use bytes::Buf;
use enum_map::enum_map;
use enum_map::{Enum, EnumMap};
use once_cell::sync::Lazy;

use crate::messaging::id::MessageId;
use crate::util::search_from_enum;

use super::traits::{ClockingFrame, MessageAuthor};

#[derive(Enum, PartialEq, Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum OneshotTypes {
    Connection_HealthCheck_Push,
    Connection_HealthCheck_Pull,
    Authentication_Login_Pull,
    TextChat_SendMessage_Pull,
    VoiceChat_SubVoiceTopic_Pull,
    VoiceChat_UnsubVoiceTopic_Pull,
    VoiceChat_SubAllVoiceTopic_Pull,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum OneshotDirection {
    Push,
    Pull,
}

#[derive(Enum, PartialEq, Debug, Clone, Copy)]
pub enum OneshotStep {
    Request,
    Response,
}

static ONESHOT_TYPES_MAP: Lazy<
    EnumMap<OneshotTypes, [u8; OneshotHeader::MESSAGE_TYPE_DISTINCTOR_SIZE]>,
> = Lazy::new(|| {
    enum_map! {
        OneshotTypes::Connection_HealthCheck_Push     => [0x00, 0x00, 0x00, 0x00],
        OneshotTypes::Connection_HealthCheck_Pull     => [0x00, 0x00, 0x00, 0x01],
        OneshotTypes::Authentication_Login_Pull       => [0x00, 0x01, 0x00, 0x00],
        OneshotTypes::TextChat_SendMessage_Pull       => [0x00, 0x03, 0x00, 0x00],
        OneshotTypes::VoiceChat_SubVoiceTopic_Pull    => [0x00, 0x03, 0x01, 0x00],
        OneshotTypes::VoiceChat_UnsubVoiceTopic_Pull  => [0x00, 0x03, 0x01, 0x01],
        OneshotTypes::VoiceChat_SubAllVoiceTopic_Pull => [0x00, 0x03, 0x01, 0x02],
    }
});

pub static ONESHOT_DIRECTION_MAP: Lazy<EnumMap<OneshotTypes, OneshotDirection>> = Lazy::new(|| {
    enum_map! {
        OneshotTypes::Connection_HealthCheck_Push     => OneshotDirection::Push,
        OneshotTypes::Connection_HealthCheck_Pull     => OneshotDirection::Pull,
        OneshotTypes::Authentication_Login_Pull       => OneshotDirection::Pull,
        OneshotTypes::TextChat_SendMessage_Pull       => OneshotDirection::Pull,
        OneshotTypes::VoiceChat_SubVoiceTopic_Pull    => OneshotDirection::Pull,
        OneshotTypes::VoiceChat_UnsubVoiceTopic_Pull  => OneshotDirection::Pull,
        OneshotTypes::VoiceChat_SubAllVoiceTopic_Pull => OneshotDirection::Pull,
    }
});

pub static ONESHOT_STEP_MAP: Lazy<
    EnumMap<OneshotStep, [u8; OneshotHeader::MESSAGE_STEP_DISTINCTOR_SIZE]>,
> = Lazy::new(|| {
    enum_map! {
        OneshotStep::Request  => [0x01, 0x00],
        OneshotStep::Response => [0x01, 0x01],
    }
});

#[derive(Debug, PartialEq, Clone)]
pub struct OneshotHeader {
    pub step: OneshotStep,
    pub message_type: OneshotTypes,
    pub message_id: MessageId,
}

impl OneshotHeader {
    pub const MESSAGE_STEP_DISTINCTOR_SIZE: usize = 2;
    pub const MESSAGE_TYPE_DISTINCTOR_SIZE: usize = 4;
}

impl ClockingFrame for OneshotHeader {
    type Context = MessageAuthor;

    const MIN_FRAME_SIZE: usize = Self::MESSAGE_STEP_DISTINCTOR_SIZE
        + size_of::<MessageId>()
        + Self::MESSAGE_TYPE_DISTINCTOR_SIZE;

    fn parse_frame_unchecked(
        cursor: &mut std::io::Cursor<&[u8]>,
        ctx: &Self::Context,
    ) -> Option<Self> {
        let step = [cursor.get_u8(), cursor.get_u8()];
        match search_from_enum(*ONESHOT_STEP_MAP, &step) {
            Some(step) => {
                let message_id = cursor.get_u64();
                let message_type = [
                    cursor.get_u8(),
                    cursor.get_u8(),
                    cursor.get_u8(),
                    cursor.get_u8(),
                ];
                let Some(message_type) = search_from_enum(*ONESHOT_TYPES_MAP, &message_type) else {
                    return None;
                };
                if ONESHOT_DIRECTION_MAP[message_type]
                    != match (ctx, step) {
                        (MessageAuthor::Client, OneshotStep::Request) => OneshotDirection::Pull,
                        (MessageAuthor::Client, OneshotStep::Response) => OneshotDirection::Push,
                        (MessageAuthor::Server, OneshotStep::Request) => OneshotDirection::Push,
                        (MessageAuthor::Server, OneshotStep::Response) => OneshotDirection::Pull,
                    }
                {
                    return None;
                }
                Some(Self {
                    step,
                    message_type,
                    message_id,
                })
            }
            None => None,
        }
    }

    async fn write_frame<W: tokio::io::AsyncWriteExt + Unpin>(
        &self,
        stream: &mut W,
        _ctx: &Self::Context,
    ) -> std::io::Result<()> {
        stream.write_all(&ONESHOT_STEP_MAP[self.step]).await?;
        stream.write_u64(self.message_id).await?;
        stream
            .write_all(&ONESHOT_TYPES_MAP[self.message_type])
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::clocking::traits::test_util::{
        encode, test_clockingframe_mustfail, test_clockingframe_reflective,
    };

    use super::*;

    #[tokio::test]
    async fn clockingserver_oneshot_header() {
        test_clockingframe_reflective(
            OneshotHeader {
                step: OneshotStep::Request,
                message_type: OneshotTypes::Connection_HealthCheck_Push,
                message_id: 0x1234567890abcdef,
            },
            MessageAuthor::Server,
        )
        .await;
        test_clockingframe_reflective(
            OneshotHeader {
                step: OneshotStep::Request,
                message_type: OneshotTypes::Connection_HealthCheck_Pull,
                message_id: 0x1234567890abcdef,
            },
            MessageAuthor::Client,
        )
        .await;
        test_clockingframe_reflective(
            OneshotHeader {
                step: OneshotStep::Response,
                message_type: OneshotTypes::Connection_HealthCheck_Push,
                message_id: 0x1234567890abcdef,
            },
            MessageAuthor::Client,
        )
        .await;
        test_clockingframe_reflective(
            OneshotHeader {
                step: OneshotStep::Response,
                message_type: OneshotTypes::Connection_HealthCheck_Pull,
                message_id: 0x1234567890abcdef,
            },
            MessageAuthor::Server,
        )
        .await;
    }

    #[tokio::test]
    async fn clockingserver_oneshot_header_mismatch_direction_pull() {
        let payload = encode(
            &OneshotHeader {
                step: OneshotStep::Request,
                message_type: OneshotTypes::Connection_HealthCheck_Pull,
                message_id: 0x1234567890abcdef,
            },
            &MessageAuthor::Server,
        )
        .await;
        test_clockingframe_mustfail::<OneshotHeader>(
            &payload,
            &MessageAuthor::Server,
            Some(OneshotHeader::MIN_FRAME_SIZE),
        )
        .await;
    }

    #[tokio::test]
    async fn clockingserver_oneshot_header_mismatch_direction_push() {
        let payload = encode(
            &OneshotHeader {
                step: OneshotStep::Request,
                message_type: OneshotTypes::Connection_HealthCheck_Push,
                message_id: 0x1234567890abcdef,
            },
            &MessageAuthor::Client,
        )
        .await;
        test_clockingframe_mustfail::<OneshotHeader>(
            &payload,
            &MessageAuthor::Client,
            Some(OneshotHeader::MIN_FRAME_SIZE),
        )
        .await;
    }
}
