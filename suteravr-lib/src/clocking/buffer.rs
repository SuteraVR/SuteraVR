use crate::clocking::ClockingFrameUnit;
use crate::util::logger::Logger;
use crate::{debug, info, warn, error};

use super::oneshot_headers::OneshotHeader;
use super::sutera_header::SuteraHeader;
use super::sutera_status::SuteraStatus;
use super::traits::MessageAuthor;

pub struct FrameBuffer<T: Logger> {
    pub buffer: Vec<ClockingFrameUnit>,
    logger: T,
}

#[derive(Debug, Clone)]
pub enum ContentHeader {
    Oneshot(OneshotHeader)
}

#[derive(Debug, Clone)]
pub struct ReceivePayload {
    pub sutera_header: SuteraHeader,
    pub sutera_status: Option<SuteraStatus>,
    pub content_header: ContentHeader,
    pub payload: Vec<u8>,
}

impl<T: Logger> FrameBuffer<T> {
    #[inline]
    pub fn new(logger: T) -> Self {
        (&logger as &dyn Logger).write_debug("data".to_string());
        Self {
            buffer: Vec::with_capacity(4),
            logger,
        }
    }

    #[inline]
    fn push(&mut self, unit: ClockingFrameUnit) {
        debug!(self.logger, "Frame: {:?}", unit);
        self.buffer.push(unit);
    }

    #[inline]
    fn len(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    fn clear(&mut self) {
        self.buffer.clear()
    }

    #[inline]
    fn get_into(&self, index: usize) -> Option<ClockingFrameUnit> {
        self.buffer.get(index).cloned()
    }

    #[inline]
    pub fn append(
        &mut self,
        payload: ClockingFrameUnit,
        author: MessageAuthor,
    ) -> Option<ReceivePayload> {
        match payload {
            ClockingFrameUnit::SuteraStatus(_) => {
                error!(self.logger, "Unexpected SuteraStatus of ClockingConnection!");
                unreachable!();
            }
            ClockingFrameUnit::SuteraHeader(_) => {
                let len = self.len();
                if self.len() != 0 {
                    warn!(self.logger, "Skipped {} frame(s).", len);
                    self.clear();

                }
                self.push(payload);
            },
            ClockingFrameUnit::Unfragmented(c) => {
                warn!(self.logger, "Receive {} unfragmented byte(s)", c.len());
            },
            ClockingFrameUnit::Content(payload) => {
                let len = self.len();
                if len != match author {
                    MessageAuthor::Client => 2,
                    MessageAuthor::Server => 3,
                } {
                    warn!(self.logger, "Unexpected content, Skipped {} frame(s).", len);
                    self.clear();
                    return None;
                }
                let Some(ClockingFrameUnit::SuteraHeader(sutera_header)) = self.get_into(0) else {
                    return None;
                };
                let Ok(sutera_status) = (if author == MessageAuthor::Client { Ok(None) } else {
                    match self.get_into(1) {
                        Some(ClockingFrameUnit::SuteraStatus(sutera_status)) => Ok(Some(sutera_status)),
                        _ => {
                            Err(())
                        }
                    }
                }) else {
                    return None
                };
                match self.get_into(match author {
                    MessageAuthor::Client => 1,
                    MessageAuthor::Server => 2,
                }) {
                    Some(ClockingFrameUnit::OneshotHeaders(oneshot_header)) => {
                        let request = ReceivePayload {
                            sutera_header,
                            sutera_status,
                            content_header: ContentHeader::Oneshot(oneshot_header),
                            payload,
                        };
                        info!(self.logger, "Receive: {:?}", &request);
                        self.clear();
                        return Some(request);
                    },
                    Some(_) | None => {
                        return None;
                    },
                }
            }
            _ => {
                self.push(payload);
            }
        }
        None
    }
}
