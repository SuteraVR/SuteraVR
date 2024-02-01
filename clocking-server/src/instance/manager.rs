use std::collections::{HashMap, hash_map};

use suteravr_lib::{
    error, info, messaging::id::{InstanceId, WorldId}, util::logger::EnvLogger
};
use tokio::{
    sync::{mpsc, oneshot}, task::JoinSet}
;

use crate::{
    errors::{ClockingServerError, InstanceError}, instance::launch_instance, shutdown::ShutdownReason
};

use super::InstanceControl;

pub enum InstancesControl {
    Shutdown(ShutdownReason),
    SpawnNew { id: InstanceId, world: WorldId, reply: oneshot::Sender<Option<mpsc::Sender<InstanceControl>>> },
}

pub struct InstanceManager {
    instances: HashMap<InstanceId, mpsc::Sender<InstanceControl>>,
    handles: JoinSet<Result<(), InstanceError>>,
}


impl InstanceManager {
    pub fn new(
    ) -> Result<Self, ClockingServerError> {
        Ok(Self {
            instances: HashMap::new(),
            handles: JoinSet::new(),
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
                    InstancesControl::SpawnNew { id, world, reply } => {    
                        let instance_connection = if let hash_map::Entry::Vacant(o) = mng.instances.entry(id) {
                            let (instance_tx, instance_rx) = mpsc::channel::<InstanceControl>(32);
                            o.insert(instance_tx.clone());
                            mng.handles
                                .build_task()
                                .name(format!("Instance {:?}", id).as_str())
                                .spawn(
                                    launch_instance(
                                        id,
                                        world,
                                        instance_rx,
                                    )
                                )?;
                            Some(instance_tx)
                        } else {
                            error!(logger, "Failed to spawn the instance {:?}. Hashmap had been occupied.", id);
                            None  
                        };
                        reply.send(instance_connection)
                            .map_err(|_| ClockingServerError::CannotSendReply)?;
                    },
                }
            }
        }
    }
    info!(logger, "Shutting down...");
    Ok::<(), ClockingServerError>(())

}