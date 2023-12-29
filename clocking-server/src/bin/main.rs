use clocking_server::version::SERVER_VERSION;

#[tokio::main]
async fn main() {
    dbg!(*SERVER_VERSION);
}
