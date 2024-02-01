use crate::messaging::id::InstanceId;
use alkahest::alkahest;

#[alkahest(Formula, Serialize, Deserialize)]
pub struct LoginRequest {
    // FIXME:
    // このトークンは、Balancing-serverに問い合わせるか、データベースから正常なトークンを貰っているかを確認する必要があります。
    // この実装は、インスタンスIDさえ分かれば、誰でもインスタンスに入れてしまうので、セキュリティ上の問題があります。
    pub join_token: InstanceId,
}

#[alkahest(Formula, Serialize, Deserialize)]
pub enum LoginResponse {
    Ok,
    BadToken,
}
