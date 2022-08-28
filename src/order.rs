use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, Timestamp};

use crate::*;

pub type OrderId = String;

#[derive(BorshDeserialize, BorshSerialize, Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    pub order_id: OrderId,
    pub payer_id: AccountId,
    pub amount: Balance,
    pub received_amount: Balance,
    pub is_completed: bool,
    pub is_refund: bool,
    pub created_at: Timestamp,
}
