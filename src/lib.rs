use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise,
    PromiseOrValue,
};

mod order;
use order::*;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[near_bindgen]
struct EcommerceContract {
    pub owner_id: AccountId,
    pub orders: LookupMap<OrderId, Order>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    OrderKey,
}

enum Error {
    ERROR_NOT_FOUND_ORDER_ID,
}

#[near_bindgen]
impl EcommerceContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            orders: LookupMap::new(StorageKey::OrderKey),
        }
    }

    #[payable]
    pub fn pay_order(&mut self, order_id: OrderId, order_amount: U128) -> PromiseOrValue<U128> {
        assert!(
            env::attached_deposit() >= order_amount.0,
            "ERROR_DEPOSIT_NOT_ENOUGH"
        );

        let order = Order {
            order_id: order_id.clone(),
            payer_id: env::predecessor_account_id(),
            amount: order_amount.into(),
            received_amount: env::attached_deposit(),
            is_completed: true,
            is_refund: false,
            created_at: env::block_timestamp(),
        };

        self.orders.insert(&order_id, &order);

        if env::attached_deposit() > order_amount.into() {
            let promise =
                Promise::new(env::predecessor_account_id()).transfer(env::attached_deposit());
            PromiseOrValue::Promise(promise)
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }

    pub fn get_order(&self, order_id: OrderId) -> Order {
        self.orders
            .get(&order_id)
            .expect("ERROR_NOT_FOUND_ORDER_ID")
    }

    pub fn cancel_order(&mut self, order_id: OrderId) {
        let mut order = self.get_order(order_id);
        let payer_id = order.payer_id;
        assert_eq!(payer_id, env::predecessor_account_id());
        Promise::new(env::predecessor_account_id()).transfer(env::attached_deposit());
        // .then(ext_self::set_refunded());
        self.set_refunded(order.order_id);
    }

    fn set_refunded(&mut self, order_id: OrderId) {
        let mut order = self.get_order(order_id);
        order.is_refund = true;
    }
}

// #[ext_contract(ext_self)]
// pub trait ExtSelf {
//     fn set_refunded(&mut self, order_id: OrderId);
// }

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .account_balance(10)
            .current_account_id(alice())
            .signer_account_id(alice())
            .predecessor_account_id(alice())
            .is_view(is_view);
        builder
    }

    #[test]
    fn test_pay_order() {
        let mut context = get_context(false);
        context.attached_deposit(1);

        testing_env!(context.build());

        let mut contract = EcommerceContract::new(alice());
        let order_id: OrderId = String::from("order_1");
        let order_amount = U128::from(1);
        contract.pay_order(order_id.clone(), order_amount);

        let order = contract.get_order(order_id.clone());

        assert_eq!(order.order_id, order_id.clone());
        assert_eq!(order.amount, order_amount.into());
        assert_eq!(order.payer_id, env::predecessor_account_id());
        assert_eq!(order.is_completed, true);
    }

    #[test]
    #[should_panic(expected = "ERROR_DEPOSIT_NOT_ENOUGH")]
    fn test_pay_order_with_lack_balance() {
        let mut context = get_context(false);
        context.attached_deposit(1);

        testing_env!(context.build());

        let mut contract = EcommerceContract::new(alice());
        let order_id: OrderId = String::from("order_2");
        let order_amount = U128::from(2);
        contract.pay_order(order_id.clone(), order_amount);
    }
}
