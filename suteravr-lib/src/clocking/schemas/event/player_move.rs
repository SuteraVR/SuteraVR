use alkahest::alkahest;

use crate::messaging::{id::PlayerId, player::StandingTransform};

#[derive(Debug)]
#[alkahest(Formula, Serialize, Deserialize)]
pub struct PubPlayerMove {
    pub now: StandingTransform,
}

#[derive(Debug)]
#[alkahest(Formula, Serialize, Deserialize)]
pub struct PushPlayerMove {
    pub player: PlayerId,
    pub now: StandingTransform,
}
