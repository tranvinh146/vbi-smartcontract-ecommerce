use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise,
    PromiseOrValue, PromiseResult,
};

mod ft_contract;
mod order;

use ft_contract::*;
use order::*;

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[near_bindgen]
pub struct EcommerceContract {
    pub owner_id: AccountId,
    pub ft_contract_id: AccountId,
    pub orders: LookupMap<OrderId, Order>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    OrderKey,
}

pub const ERROR_DEPOSIT_NOT_ENOUGH: &str = "ERROR_DEPOSIT_NOT_ENOUGH";
pub const ERROR_NOT_FOUND_ORDER_ID: &str = "ERROR_NOT_FOUND_ORDER_ID";
pub const ERROR_TOO_MANY_RESULTS: &str = "ERROR_TOO_MANY_RESULTS";

pub const SET_REFUND_GAS: Gas = Gas(10_000_000_000_000);

#[near_bindgen]
impl EcommerceContract {
    #[init]
    pub fn new(owner_id: AccountId, ft_contract_id: AccountId) -> Self {
        Self {
            owner_id,
            ft_contract_id,
            orders: LookupMap::new(StorageKey::OrderKey),
        }
    }

    #[payable]
    pub fn pay_order(&mut self, order_id: OrderId, order_amount: U128) -> PromiseOrValue<U128> {
        assert!(
            env::attached_deposit() >= order_amount.0,
            "{}",
            ERROR_DEPOSIT_NOT_ENOUGH
        );

        let order_optional = self.orders.get(&order_id);
        match order_optional {
            Some(order) => {
                assert!(!order.is_completed)
            }
            None => {}
        }

        let order = Order {
            order_id: order_id.clone(),
            payer_id: env::predecessor_account_id(),
            payment_method: PaymentMethod::NEAR,
            amount: order_amount.into(),
            received_amount: env::attached_deposit(),
            is_completed: true,
            is_refund: false,
            created_at: env::block_timestamp(),
        };

        self.orders.insert(&order_id, &order);

        if env::attached_deposit() > order_amount.into() {
            let promise = Promise::new(env::predecessor_account_id())
                .transfer(env::attached_deposit() - order_amount.0);
            PromiseOrValue::Promise(promise)
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }

    pub fn get_order(&self, order_id: OrderId) -> Order {
        self.orders.get(&order_id).expect(ERROR_NOT_FOUND_ORDER_ID)
    }

    pub fn cancel_order(&mut self, order_id: OrderId) -> PromiseOrValue<U128> {
        assert_eq!(env::predecessor_account_id(), self.owner_id);

        let order = self.get_order(order_id.clone());
        assert!(order.is_completed && !order.is_refund);

        if order.amount > 0 {
            match order.payment_method {
                PaymentMethod::NEAR => {
                    let promise = Promise::new(order.payer_id).transfer(order.amount).then(
                        ext_self::ext(env::current_account_id())
                            .with_attached_deposit(0)
                            .with_static_gas(SET_REFUND_GAS)
                            .set_is_refunded(order_id),
                    );
                    PromiseOrValue::Promise(promise)
                }
                PaymentMethod::FungibleToken => {
                    let promise = ext_ft::ext(self.ft_contract_id.clone())
                        .with_attached_deposit(1)
                        .with_static_gas(SET_REFUND_GAS)
                        .ft_transfer(
                            order.payer_id,
                            U128(order.amount),
                            Some("Refund order from FT contract".to_owned()),
                        )
                        .then(
                            ext_self::ext(env::current_account_id())
                                .with_attached_deposit(0)
                                .with_static_gas(SET_REFUND_GAS)
                                .set_is_refunded(order_id),
                        );
                    PromiseOrValue::Promise(promise)
                }
            }
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn set_is_refunded(&mut self, order_id: OrderId) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl ExtSelf for EcommerceContract {
    #[private]
    fn set_is_refunded(&mut self, order_id: OrderId) -> PromiseOrValue<U128> {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            ERROR_TOO_MANY_RESULTS
        );
        match env::promise_result(0) {
            PromiseResult::Successful(_) => {
                let mut order = self.orders.get(&order_id).expect(ERROR_NOT_FOUND_ORDER_ID);
                order.is_refund = true;
                self.orders.insert(&order_id, &order);

                PromiseOrValue::Value(U128(order.amount))
            }
            _ => PromiseOrValue::Value(U128(0)),
        }
    }
}

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

        let mut contract = EcommerceContract::new(alice(), /*Change later */ alice());
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

        let mut contract = EcommerceContract::new(alice(), /*Change later */ alice());
        let order_id: OrderId = String::from("order_2");
        let order_amount = U128::from(2);
        contract.pay_order(order_id.clone(), order_amount);
    }
}
