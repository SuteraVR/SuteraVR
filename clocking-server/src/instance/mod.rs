use derivative::Derivative;
use suteravr_lib::{
    clocking::schemas::oneshot::chat_entry::ChatEntry,
    info,
    messaging::id::{InstanceId, PlayerId, WorldId},
    util::logger::EnvLogger,
};
use tokio::sync::mpsc;

use crate::{errors::InstanceError, shutdown::ShutdownReason};

pub mod manager;
pub enum InstanceControl {
    Shutdown(ShutdownReason),
    Join(PlayerId),
    Leave(PlayerId),
    ChatMesasge(ChatEntry),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Instance {
    pub id: InstanceId,
    pub world: WorldId,
    pub players: Vec<PlayerId>,
    pub chat_history: Vec<ChatEntry>,

    #[derivative(Debug = "ignore")]
    pub logger: EnvLogger,
}

impl Instance {
    fn new(
        id: InstanceId,
        world: WorldId,
        players: Vec<PlayerId>,
        chat_history: Vec<ChatEntry>,
        logger: EnvLogger,
    ) -> Self {
        Self {
            id,
            world,
            players,
            chat_history,
            logger,
        }
    }
}

pub async fn launch_instance(
    id: InstanceId,
    world: WorldId,
    mut command_receiver: mpsc::Receiver<InstanceControl>,
) -> Result<(), InstanceError> {
    let logger = EnvLogger {
        target: format!("instance-{:?}", id),
    };
    let mut instance = Instance::new(id, world, Vec::new(), Vec::new(), logger.clone());
    info!(logger, "Instance started.");

    loop {
        tokio::select! {
            Some(command) = command_receiver.recv() => {
                match command {
                    InstanceControl::Shutdown(_) => {
                        break;
                    },
                    InstanceControl::Join(player_id) => {
                        instance.players.push(player_id);
                        info!(logger, "Player joined (id: {:?}), currently {} player(s) in instance.", player_id, instance.players.len());
                    },
                    InstanceControl::Leave(player_id) => {
                        instance.players.retain(|&id| id != player_id);
                        info!(logger, "Player left: (id: {:?}), currently {} player(s) in instance.", player_id, instance.players.len());
                    },
                    InstanceControl::ChatMesasge(chat_entry) => {
                        info!(logger, "TextChat: {:?}", chat_entry);
                        instance.chat_history.push(chat_entry);
                    },

                }
            }
        }
    }
    info!(logger, "Instance stopping...");
    Ok::<(), InstanceError>(())
}
