//! 識別子に関する型定義を行うモジュール

use alkahest::alkahest;

use super::SizedForBinary;

/// プレイヤーを識別するための構造体
///
/// 0 - 255までの値を取ります。  
/// 実質的にワールド内の最大人数を256人に制限しています。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct PlayerIdentifier(pub InstanceIdentifier, pub u8);
impl SizedForBinary for PlayerIdentifier {
    const SIZE: usize = InstanceIdentifier::SIZE + 1;
}

/// インスタンスを識別するための構造体
///
/// 0 - 255までの値を取ります。
/// 実質的に1つのクロッキングサーバーがホストできる最大インスタンス数を256個に制限しています。
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct InstanceIdentifier(pub u8);
impl SizedForBinary for InstanceIdentifier {
    const SIZE: usize = 1;
}

/// リクエストを識別するための構造体
///
/// u64, ビッグエンディアンにエンコードされます。
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
#[alkahest(Formula, SerializeRef, Deserialize)]
pub struct RequestIdentifier(pub u64);
impl SizedForBinary for RequestIdentifier {
    const SIZE: usize = 8;
}
