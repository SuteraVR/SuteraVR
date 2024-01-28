use clocking_server::clocking_server;
use clocking_server::errors::ClockingServerError;

#[tokio::main]
async fn main() -> Result<(), ClockingServerError> {
    clocking_server().await
}
