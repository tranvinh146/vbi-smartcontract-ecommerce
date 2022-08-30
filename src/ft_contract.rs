use near_sdk::serde::{Deserialize, Serialize};

use crate::*;

pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "near_sdk::serde")]
struct FTMessage {
    order_id: OrderId,
    order_amount: U128,
}

#[near_bindgen]
impl FungibleTokenReceiver for EcommerceContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_eq!(env::predecessor_account_id(), self.ft_contract_id);
        let FTMessage {
            order_id,
            order_amount,
        } = near_sdk::serde_json::from_str(&msg).expect("ERROR_NOT_INVALID_MESSAGE");

        assert!(amount.0 >= order_amount.0);

        let order_optional = self.orders.get(&order_id);
        match order_optional {
            Some(order) => {
                assert!(!order.is_completed)
            }
            None => {}
        }

        let order = Order {
            order_id: order_id.clone(),
            payer_id: sender_id,
            payment_method: PaymentMethod::FungibleToken,
            amount: order_amount.into(),
            received_amount: env::attached_deposit(),
            is_completed: true,
            is_refund: false,
            created_at: env::block_timestamp(),
        };

        self.orders.insert(&order_id, &order);

        if amount.0 > order_amount.0 {
            PromiseOrValue::Value(U128(amount.0 - order_amount.0))
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }
}

#[ext_contract(ext_ft)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}
