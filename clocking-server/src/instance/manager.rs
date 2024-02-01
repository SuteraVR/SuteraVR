use suteravr_lib::{
    info,
    messaging::id::{InstanceId, WorldId},
    util::logger::EnvLogger,
};
use tokio::
    sync::mpsc
;

use crate::{
    errors::ClockingServerError,
    shutdown::ShutdownReason,
};

use super::Instance;

pub enum InstancesControl {
    Shutdown(ShutdownReason),
    SpawnNew { id: InstanceId, world: WorldId },
}

pub struct InstanceManager {
    instances: Vec<Instance>,
}


impl InstanceManager {
    pub fn new(
    ) -> Result<Self, ClockingServerError> {

        Ok(Self {
            instances: Vec::new(),
        })
    }
}

pub async fn launch_instance_manager(
    mut command_receiver: mpsc::Receiver<InstancesControl>
) -> Result<(), ClockingServerError> {
    let logger = EnvLogger {
        target: "instance-manager".to_string(),
    };
    let mut mng = InstanceManager::new()?;
    info!(logger, "Manager spawned. Ready!");
    loop {
        tokio::select! {
            Some(command) = command_receiver.recv() => {
                match command {
                    InstancesControl::Shutdown(_) => {
                        break;
                    },
                    InstancesControl::SpawnNew { id, world } => {            
                        let instance = Instance {
                            id,
                            world,
                            players: Vec::new(),
                            chat_history: Vec::new(),
                        };
                        mng.instances.push(instance);
                    },
                }
            }
        }
    }
    info!(logger, "Shutting down...");
    Ok::<(), ClockingServerError>(())

}