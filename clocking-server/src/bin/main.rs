use clocking_server::{server::clocking_server, tcp::error::ClockingServerError};

#[tokio::main]
async fn main() -> Result<(), ClockingServerError> {
    clocking_server().await
}
