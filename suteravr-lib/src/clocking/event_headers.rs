use bytes::Buf;
use derivative::Derivative;
use derive_new::new;
use enum_map::enum_map;
use enum_map::{Enum, EnumMap};
use once_cell::sync::Lazy;

use crate::util::search_from_enum;

use super::sutera_header::SuteraHeader;
use super::sutera_status::SuteraStatus;
use super::traits::{ClockingFrame, MessageAuthor};

#[derive(Enum, PartialEq, Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum EventTypes {
    Instance_PlayerJoined_Push,
    Instance_PlayerLeft_Push,
    Instance_PubPlayerMove_Pull,
    Instance_PushPlayerMove_Push,
    TextChat_ReceiveChatMessage_Push,
}

#[derive(Enum, PartialEq, Debug, Clone, Copy)]
pub enum EventDirection {
    Push,
    Pull,
}

static EVENT_TYPES_MAP: Lazy<EnumMap<EventTypes, [u8; EventHeader::MESSAGE_TYPE_DISTINCTOR_SIZE]>> =
    Lazy::new(|| {
        enum_map! {
            EventTypes::Instance_PlayerJoined_Push       => [0x00, 0x02, 0x00, 0x01],
            EventTypes::Instance_PlayerLeft_Push         => [0x00, 0x02, 0x00, 0x02],
            EventTypes::Instance_PubPlayerMove_Pull      => [0x00, 0x02, 0x01, 0x01],
            EventTypes::Instance_PushPlayerMove_Push     => [0x00, 0x02, 0x01, 0x02],
            EventTypes::TextChat_ReceiveChatMessage_Push => [0x00, 0x03, 0x00, 0x01],
        }
    });

pub static EVENT_TYPES_DIRECTION_MAP: Lazy<EnumMap<EventTypes, EventDirection>> = Lazy::new(|| {
    enum_map! {
        EventTypes::Instance_PlayerJoined_Push       => EventDirection::Push,
        EventTypes::Instance_PlayerLeft_Push         => EventDirection::Push,
        EventTypes::Instance_PubPlayerMove_Pull      => EventDirection::Pull,
        EventTypes::Instance_PushPlayerMove_Push     => EventDirection::Push,
        EventTypes::TextChat_ReceiveChatMessage_Push => EventDirection::Push,
    }
});

pub static EVENT_DIRECTION_MAP: Lazy<
    EnumMap<EventDirection, [u8; EventHeader::MESSAGE_DIRECTION_DISTINCTOR_SIZE]>,
> = Lazy::new(|| {
    enum_map! {
        EventDirection::Push => [0x02, 0x00],
        EventDirection::Pull => [0x03, 0x00],
    }
});

#[derive(Debug, PartialEq, Clone)]
pub struct EventHeader {
    pub direction: EventDirection,
    pub message_type: EventTypes,
}

impl EventHeader {
    pub const MESSAGE_DIRECTION_DISTINCTOR_SIZE: usize = 2;
    pub const MESSAGE_TYPE_DISTINCTOR_SIZE: usize = 4;
}

impl ClockingFrame for EventHeader {
    type Context = MessageAuthor;

    const MIN_FRAME_SIZE: usize =
        Self::MESSAGE_DIRECTION_DISTINCTOR_SIZE + Self::MESSAGE_TYPE_DISTINCTOR_SIZE;

    fn parse_frame_unchecked(
        cursor: &mut std::io::Cursor<&[u8]>,
        ctx: &Self::Context,
    ) -> Option<Self> {
        let dir = [cursor.get_u8(), cursor.get_u8()];
        match search_from_enum(*EVENT_DIRECTION_MAP, &dir) {
            Some(dir) => {
                let message_type = [
                    cursor.get_u8(),
                    cursor.get_u8(),
                    cursor.get_u8(),
                    cursor.get_u8(),
                ];
                let message_type = search_from_enum(*EVENT_TYPES_MAP, &message_type)?;
                if EVENT_TYPES_DIRECTION_MAP[message_type] != dir {
                    return None;
                }

                if dir
                    != match ctx {
                        MessageAuthor::Client => EventDirection::Pull,
                        MessageAuthor::Server => EventDirection::Push,
                    }
                {
                    return None;
                }
                Some(Self {
                    direction: dir,
                    message_type,
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
        stream
            .write_all(&EVENT_DIRECTION_MAP[self.direction])
            .await?;
        stream
            .write_all(&EVENT_TYPES_MAP[self.message_type])
            .await?;
        Ok(())
    }
}

#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct EventRequest {
    pub sutera_header: SuteraHeader,
    pub event_header: EventHeader,
    pub payload: Vec<u8>,
}
#[derive(Derivative, new)]
#[derivative(Debug)]
pub struct EventResponse {
    pub sutera_header: SuteraHeader,
    pub sutera_status: SuteraStatus,
    pub event_header: EventHeader,
    pub payload: Vec<u8>,
}
