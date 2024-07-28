use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct InitiateTxn {
    pub sender_id: String,
    pub receiver_id: String,
    pub type_: String,
    pub amount: i64,
}
