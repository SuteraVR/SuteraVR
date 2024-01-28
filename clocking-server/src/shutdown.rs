#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownReason {
    SIGINT,
    SIGTERM,
}
