use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook::iterator::Signals;

async fn listen_signal() {
    let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
    for sig in signals.forever() {}
}
