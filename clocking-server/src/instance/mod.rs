use derivative::Derivative;
use suteravr_lib::{
    clocking::schemas::chat_entry::ChatEntry,
    info,
    messaging::id::{InstanceId, WorldId},
    util::logger::EnvLogger,
};
use tokio::sync::mpsc;

use crate::{errors::InstanceError, shutdown::ShutdownReason};

use self::player::Player;

pub mod manager;
pub mod player;

pub enum InstanceControl {
    Shutdown(ShutdownReason),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Instance {
    pub id: InstanceId,
    pub world: WorldId,
    pub players: Vec<Player>,
    pub chat_history: Vec<ChatEntry>,

    #[derivative(Debug = "ignore")]
    pub logger: EnvLogger,
}

impl Instance {
    fn new(
        id: InstanceId,
        world: WorldId,
        players: Vec<Player>,
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
    let _instance = Instance::new(id, world, Vec::new(), Vec::new(), logger.clone());
    info!(logger, "Instance started.");

    loop {
        tokio::select! {
            Some(command) = command_receiver.recv() => {
                match command {
                    InstanceControl::Shutdown(_) => {
                        break;
                    },
                }
            }
        }
    }
    Ok::<(), InstanceError>(())
}
