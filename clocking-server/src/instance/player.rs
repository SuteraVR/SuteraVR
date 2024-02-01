use derivative::Derivative;
use suteravr_lib::messaging::id::PlayerId;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Player {
    pub id: PlayerId,
}