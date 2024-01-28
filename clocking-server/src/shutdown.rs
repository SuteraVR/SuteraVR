#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownReason {
    Sigint,
    Sigterm,
    SignalChannelClosed,
}
