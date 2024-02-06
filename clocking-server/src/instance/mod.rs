use std::collections::{hash_map::Entry, HashMap};

use derivative::Derivative;
use suteravr_lib::{
    clocking::schemas::oneshot::chat_entry::ChatEntry,
    error, info,
    messaging::id::{InstanceId, PlayerId, WorldId},
    util::logger::EnvLogger,
    warn,
};
use tokio::{sync::mpsc, task};

use crate::{errors::InstanceError, shutdown::ShutdownReason};

pub mod manager;
pub enum InstanceControl {
    Shutdown(ShutdownReason),
    Join(PlayerId, mpsc::Sender<PlayerControl>),
    Leave(PlayerId),
    ChatMesasge(ChatEntry),
}
pub enum PlayerControl {
    PlayerJoined(PlayerId),
    PlayerLeft(PlayerId),
    NewChatMessage(ChatEntry),
}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Instance {
    pub id: InstanceId,
    pub world: WorldId,
    pub players: HashMap<PlayerId, mpsc::Sender<PlayerControl>>,
    pub chat_history: Vec<ChatEntry>,

    #[derivative(Debug = "ignore")]
    pub logger: EnvLogger,
}

impl Instance {
    fn new(
        id: InstanceId,
        world: WorldId,
        players: HashMap<PlayerId, mpsc::Sender<PlayerControl>>,
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
    let mut instance = Instance::new(id, world, HashMap::new(), Vec::new(), logger.clone());
    info!(logger, "Instance started.");

    loop {
        tokio::select! {
            Some(command) = command_receiver.recv() => {
                match command {
                    InstanceControl::Shutdown(_) => {
                        break;
                    },
                    InstanceControl::Join(player_id, sender) => {
                        match instance.players.entry(player_id) {
                            Entry::Vacant(o) => {
                                o.insert(sender);
                                info!(logger, "Player joined (id: {:?}), currently {} player(s) in instance.", player_id, instance.players.len());

                                let iter = instance.players.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>();
                                for (target_player_id, sender) in iter {
                                    let id = player_id;
                                    if id == target_player_id { break; }
                                    let logger = logger.clone();
                                    task::Builder::new()
                                        .name("PlayerJoined Notify")
                                        .spawn(async move {
                                            if let Err(e) = sender.send(PlayerControl::PlayerJoined(id)).await {
                                                error!(logger, "Failed to PlayerJoined Notify to Player {}: {:?}", target_player_id, e);
                                            }
                                        }).map_err(InstanceError::SpawnError)?;
                                }
                            },
                            Entry::Occupied(mut o) => {
                                o.insert(sender);
                                warn!(logger, "Join request received but already in instance (id: {:?}).", player_id);
                            }
                        }
                    },
                    InstanceControl::Leave(player_id) => {
                        instance.players.remove(&player_id);
                        info!(logger, "Player left: (id: {:?}), currently {} player(s) in instance.", player_id, instance.players.len());
                        let iter = instance.players.iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>();
                        for (target_player_id, sender) in iter {
                            let id = player_id;
                            if id == target_player_id { break; }
                            let logger = logger.clone();
                            task::Builder::new()
                                .name("PlayerLeft Notify")
                                .spawn(async move {
                                    if let Err(e) = sender.send(PlayerControl::PlayerLeft(id)).await {
                                        error!(logger, "Failed to PlayerLeft Notify to Player {}: {:?}", target_player_id, e);
                                    }
                                }).map_err(InstanceError::SpawnError)?;
                        }
                    },
                    InstanceControl::ChatMesasge(chat_entry) => {
                        instance.chat_history.push(chat_entry.clone());
                        info!(logger, "TextChat: {:?}", chat_entry);
                        for (_, sender) in instance.players.iter() {
                            sender.send(PlayerControl::NewChatMessage(chat_entry.clone())).await?;
                        }
                    },

                }
            }
        }
    }
    info!(logger, "Instance stopping...");
    Ok::<(), InstanceError>(())
}
