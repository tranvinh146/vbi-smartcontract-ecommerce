use crate::*;

pub async fn test_ft_payment(
    alice: &Account,
    ft_contract: &Contract,
    ecommerce_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    println!("\tTest FT payment method");

    // 1. storage deposit & transfer FT
    storage_deposit_ft_contract(&alice, &ft_contract, &worker).await?;
    storage_deposit_ft_contract(&ecommerce_contract.as_account(), &ft_contract, &worker).await?;
    add_ft_token(&alice, &ft_contract, &worker).await?;

    // 2. pay order with FT
    pay_order_with_ft(&alice, &ft_contract, &ecommerce_contract, &worker).await?;

    // 3, cancel order & refund FT
    cancel_order_and_refund(&alice, &ft_contract, &ecommerce_contract, &worker).await?;

    Ok(())
}

async fn storage_deposit_ft_contract(
    account: &Account,
    contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let res: serde_json::Value = account
        .call(&worker, contract.id(), "storage_deposit")
        .deposit(parse_near!("0.00125 N"))
        .args_json(json!({"account_id": account.id()}))?
        .transact()
        .await?
        .json()?;

    let expected = json!(
        {
            "total": parse_near!("0.00125 N").to_string(),
            "available": "0"
        }
    );

    assert_eq!(res, expected);
    println!(
        "\t\tPassed ✅ storage_deposit {:?} on FT contract",
        account.id()
    );
    Ok(())
}

async fn add_ft_token(
    alice: &Account,
    ft_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    ft_contract
        .call(&worker, "ft_transfer")
        .args_json(json!({
            "receiver_id": alice.id(),
            "amount": parse_near!("10 N").to_string()
        }))?
        .deposit(1)
        .transact()
        .await?;

    let res: serde_json::Value = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": alice.id()}))?
        .view()
        .await?
        .json()?;

    assert_eq!(res, parse_near!("10 N").to_string());
    println!("\t\tPassed ✅ transfer FT to {:?}", alice.id());
    Ok(())
}

async fn pay_order_with_ft(
    alice: &Account,
    ft_contract: &Contract,
    ecommerce_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    let order_info = "{\"order_id\": \"order_1\", \"order_amount\": \"1000000000000000000000000\"}";

    alice
        .call(&worker, ft_contract.id(), "ft_transfer_call")
        .args_json(json!({
          "receiver_id": ecommerce_contract.id(),
          "amount": parse_near!("5 N").to_string(),
          "msg": order_info
        }))?
        .deposit(1)
        .max_gas()
        .transact()
        .await?;

    let res: serde_json::Value = ecommerce_contract
        .call(&worker, "get_order")
        .args_json(json!({"order_id": "order_1"}))?
        .view()
        .await?
        .json()?;

    assert_eq!(*res.get("order_id").unwrap(), json!("order_1"));
    assert_eq!(*res.get("payment_method").unwrap(), json!("FungibleToken"));
    assert_eq!(*res.get("amount").unwrap(), json!(parse_near!("1 N")));

    println!("\t\tPassed ✅ pay order with FT method");
    Ok(())
}

async fn cancel_order_and_refund(
    alice: &Account,
    ft_contract: &Contract,
    ecommerce_contract: &Contract,
    worker: &Worker<Sandbox>,
) -> anyhow::Result<()> {
    ecommerce_contract
        .call(&worker, "cancel_order")
        .args_json(json!({"order_id": "order_1"}))?
        .max_gas()
        .transact()
        .await?;

    let res: serde_json::Value = ecommerce_contract
        .call(&worker, "get_order")
        .args_json(json!({"order_id": "order_1"}))?
        .view()
        .await?
        .json()?;

    let balance_ft_of_alice: serde_json::Value = ft_contract
        .call(&worker, "ft_balance_of")
        .args_json(json!({"account_id": alice.id()}))?
        .view()
        .await?
        .json()?;

    assert_eq!(*res.get("order_id").unwrap(), json!("order_1"));
    assert_eq!(*res.get("payment_method").unwrap(), json!("FungibleToken"));
    assert_eq!(*res.get("amount").unwrap(), json!(parse_near!("1 N")));
    assert_eq!(*res.get("is_refund").unwrap(), json!(true));
    assert_eq!(balance_ft_of_alice, json!(parse_near!("10 N").to_string()));

    println!("\t\tPassed ✅ cancel order and refund FT");
    Ok(())
}
