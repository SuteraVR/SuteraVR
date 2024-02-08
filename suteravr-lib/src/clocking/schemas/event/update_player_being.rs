use alkahest::alkahest;

use crate::messaging::id::PlayerId;

#[derive(Debug)]
#[alkahest(Formula, Serialize, Deserialize)]
pub struct PlayerJoined {
    pub joined_player: PlayerId,
}

#[derive(Debug)]
#[alkahest(Formula, Serialize, Deserialize)]
pub struct PlayerLeft {
    pub left_player: PlayerId,
}
