use clocking_server::version::SCHEMA_SEMVER;

#[tokio::main]
async fn main() {
    dbg!(*SCHEMA_SEMVER);
}
