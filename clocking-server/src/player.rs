use suteravr_lib::typing::id::PlayerIdentifier;

use crate::{instance::WorldInstance, typing::Arw};

pub struct Player {
    /// プレイヤーの識別子
    pub identifier: PlayerIdentifier,
    /// プレイヤーが所属するインスタンス/
    pub instance: Arw<WorldInstance>,
}
