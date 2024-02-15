use std::collections::{hash_map::Entry, HashMap};

use derivative::Derivative;
use suteravr_lib::{
    clocking::schemas::{
        event::player_move::{PubPlayerMove, PushPlayerMove},
        oneshot::chat_entry::ChatEntry,
    },
    debug, error, info,
    messaging::id::{InstanceId, PlayerId, WorldId},
    util::logger::EnvLogger,
    warn,
};
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

use crate::{errors::InstanceError, shutdown::ShutdownReason};

pub mod manager;
pub enum InstanceControl {
    Shutdown(ShutdownReason),
    Join(
        PlayerId,
        mpsc::Sender<PlayerControl>,
        oneshot::Sender<Vec<PlayerId>>,
    ),
    Leave(PlayerId),
    ChatMesasge(ChatEntry),
    PlayerMoved(PlayerId, PubPlayerMove),
}
#[derive(Clone)]
pub enum PlayerControl {
    PlayerJoined(PlayerId),
    PlayerLeft(PlayerId),
    NewChatMessage(ChatEntry),
    PlayerMoved(PushPlayerMove),
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
                    InstanceControl::Join(player_id, sender, reply_pos) => {
                        match instance.players.entry(player_id) {
                            Entry::Vacant(o) => {
                                o.insert(sender);
                                info!(logger, "Player joined (id: {:?}), currently {} player(s) in instance.", player_id, instance.players.len());
                                notify(&instance, "PlayerJoined".to_string(), &logger, player_id,  PlayerControl::PlayerJoined(player_id))?;
                                reply_pos.send(instance.players.keys().cloned().filter(|p| *p != player_id).collect()).map_err(|_| InstanceError::CannotSendReply)?;
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
                        notify(&instance, "PlayerLeft".to_string(), &logger, player_id,  PlayerControl::PlayerLeft(player_id))?;
                    },
                    InstanceControl::PlayerMoved(player_id, pub_player_move) => {
                        debug!(logger, "PlayerMoved: {:?}", pub_player_move);
                        notify(
                            &instance, "PlayerMoved".to_string(), &logger, player_id,
                            PlayerControl::PlayerMoved(
                                PushPlayerMove { player: player_id, now: pub_player_move.now }
                            )
                        )?;
                    }
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

fn notify(
    instance: &Instance,
    mut name: String,
    logger: &EnvLogger,
    author: PlayerId,
    content: PlayerControl,
) -> Result<(), InstanceError> {
    let iter = instance
        .players
        .iter()
        .filter(|(k, _)| **k != author)
        .map(|(k, v)| (*k, v.clone()))
        .collect::<Vec<_>>();
    name.push_str(" Notify");
    for (target_player_id, sender) in iter {
        let logger = logger.clone();
        let content = content.clone();
        task::Builder::new()
            .name(&name)
            .spawn(async move {
                if let Err(e) = sender.send(content).await {
                    error!(
                        logger,
                        "Failed to PlayerLeft Notify to Player {}: {:?}", target_player_id, e
                    );
                }
            })
            .map_err(InstanceError::SpawnError)?;
    }
    Ok(())
}
