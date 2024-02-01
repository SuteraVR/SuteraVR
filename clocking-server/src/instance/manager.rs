use suteravr_lib::{
    info,
    messaging::id::{InstanceId, WorldId},
    util::logger::EnvLogger,
};
use tokio::{
    sync::mpsc,
    task::{self, JoinHandle},
};

use crate::{
    errors::{ClockingServerError, InstanceError},
    shutdown::ShutdownReason,
};

use super::Instance;

pub enum InstancesControl {
    Shutdown(ShutdownReason),
    SpawnNew { id: InstanceId, world: WorldId },
}

pub struct InstanceManager {
    instances: Vec<Instance>,
    pub handle: JoinHandle<Result<(), InstanceError>>,
    logger: EnvLogger,
}

impl InstanceManager {
    pub fn new(
        mut command_receiver: mpsc::Receiver<InstancesControl>,
    ) -> Result<Self, ClockingServerError> {
        let logger = EnvLogger {
            target: "instance-manager".to_string(),
        };
        let logger_fut = logger.clone();
        let fut = async move {
            let logger = logger_fut;
            loop {
                tokio::select! {
                    Some(command) = command_receiver.recv() => {
                        match command {
                            InstancesControl::Shutdown(_) => {
                                break;
                            },
                            InstancesControl::SpawnNew { id, world } => todo!(),
                        }
                    },
                }
            }
            info!(logger, "Shutting down...");
            Ok::<(), InstanceError>(())
        };

        let handle = task::Builder::new()
            .name("Instance Manager")
            .spawn(fut)
            .map_err(ClockingServerError::SpawnError)?;

        Ok(Self {
            instances: Vec::new(),
            logger,
            handle,
        })
    }

    pub fn spawn_instance(&mut self, id: InstanceId, world: WorldId) {
        let instance = Instance {
            id,
            world,
            players: Vec::new(),
            chat_history: Vec::new(),
        };
        self.instances.push(instance);
    }
}
