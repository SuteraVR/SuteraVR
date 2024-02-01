use derivative::Derivative;
use suteravr_lib::messaging::id::PlayerId;
use tokio::sync::mpsc;

use crate::tcp::requests::Response;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Player {
    pub id: PlayerId,
    #[derivative(Debug = "ignore")]
    pub sender: mpsc::Sender<Response>,
}
