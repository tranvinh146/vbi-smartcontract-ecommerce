use anyhow::Ok;
use near_units::parse_near;
use serde_json::json;
use workspaces::prelude::*;
use workspaces::{network::Sandbox, Account, Contract, Worker};

mod test_ft_payment;
use test_ft_payment::*;

const ECOMMERCE_WASM_FILEPATH: &str = "../out/ecommerce.wasm";
const FT_WASM_FILEPATH: &str = "../out/fungible_token.wasm";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let ecommerce_wasm = std::fs::read(ECOMMERCE_WASM_FILEPATH)?;
    let ft_wasm = std::fs::read(FT_WASM_FILEPATH)?;
    let ecommerce_contract = worker.dev_deploy(&ecommerce_wasm).await?;
    let ft_contract = worker.dev_deploy(&ft_wasm).await?;

    // initialize FT Contract
    // FT has decimals 24 like NEAR
    ft_contract
        .call(&worker, "new_default_meta")
        .args_json(json!({
          "owner_id": ft_contract.id(),
          "total_supply": parse_near!("100 N").to_string()
        }))?
        .transact()
        .await?;

    // initialize Ecommerce Contract
    ecommerce_contract
        .call(&worker, "new")
        .args_json(
            json!({"owner_id": ecommerce_contract.id(), "ft_contract_id": ft_contract.id()}),
        )?
        .transact()
        .await?;

    // create accounts
    let owner = worker.root_account();
    let alice = owner
        .create_subaccount(&worker, "alice")
        .initial_balance(parse_near!("30 N"))
        .transact()
        .await?
        .into_result()?;

    // begin tests
    test_ft_payment(&alice, &ft_contract, &ecommerce_contract, &worker).await?;

    Ok(())
}
