
use suteravr_lib::messaging::id::{InstanceId, WorldId};
use tokio::{sync::{mpsc, oneshot}, task::{self, JoinHandle}};

use crate::{errors::{ClockingServerError, InstanceError}, shutdown::ShutdownReason};

use super::Instance;

pub enum InstancesControl {
    SpawnNew { id: InstanceId, world: WorldId },
}

pub struct InstanceManager {
    instances: Vec<Instance>,
}

impl InstanceManager {
    pub fn new(
        mut command_receiver: mpsc::Receiver<InstancesControl>,
        mut shutdown: oneshot::Receiver<ShutdownReason>
    ) -> Result<(Self, JoinHandle<Result<(), InstanceError>>), ClockingServerError> {
        let fut = async move {
            loop {
                tokio::select! {
                    Some(_command) = command_receiver.recv() => {

                    },
                    _ = &mut shutdown => {
                        break;
                    }
                }
            }
            Ok::<(), InstanceError>(())
        };

        let handle = task::Builder::new()
            .name("Instance Manager")
            .spawn(fut)
            .map_err(ClockingServerError::SpawnError)?;


        Ok((Self {
            instances: Vec::new(),
        }, handle))
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
