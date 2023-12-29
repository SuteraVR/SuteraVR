use alkahest::alkahest;

/// プレイヤーを識別するための構造体
///
/// 0 - 255までの値を取ります。  
/// 実質的にワールド内の最大人数を256人に制限しています。
#[derive(Debug, PartialEq, Eq)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct PlayerIdentifier(pub u8);
