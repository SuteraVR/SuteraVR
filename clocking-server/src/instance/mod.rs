use suteravr_lib::{
    clocking::schemas::chat_entry::ChatEntry,
    messaging::id::{InstanceId, WorldId},
};

use self::player::Player;

pub mod manager;
pub mod player;

#[derive(Debug)]
pub struct Instance {
    pub id: InstanceId,
    pub world: WorldId,
    pub players: Vec<Player>,
    pub chat_history: Vec<ChatEntry>,
}
