use near_sdk::serde_json::{json, Value};
use workspaces::prelude::*;
use workspaces::{Account, AccountId, Contract, Network, Worker};

const MULTI_OWNABLE_FILEPATH: &str = "../out/test_contract.wasm";

#[tokio::test]
async fn test_multi_ownable() -> anyhow::Result<()> {
    let worker = workspaces::sandbox();

    let root = worker.root_account();

    let alice = worker.dev_create_account().await?;

    let wasm2 = std::fs::read(MULTI_OWNABLE_FILEPATH)?;
    let contract = worker.dev_deploy(wasm2).await?;
    init(&worker, &contract, &root).await?;

    let number_1 = 123;
    let number_args = json!({ "number": number_1 }).to_string();
    multi_ownable_call(
        &worker,
        &contract,
        &root,
        "update_number",
        number_args.as_str(),
    )
    .await?;

    let owners = get_owners(&worker, &contract).await?;
    println!("OWNERS {:?}", owners);
    assert!(owners.len()==1, "wrong number of owners");

    let thresh = get_threshold(&worker, &contract).await?;
    println!("THRESHOLD {:?}", thresh);

    let rr = get_number(&worker, &contract).await?;
    println!("NUMBER {:?}", rr);
    assert_eq!(number_1, rr, "wrong number");

    update_multi_ownable(
        &worker,
        &contract,
        &root,
        vec![root.id().clone(), alice.id().clone()],
        2,
    )
    .await?;

    let owners = get_owners(&worker, &contract).await?;
    println!("OWNERS 2 {:?}", owners);
    assert!(owners.len()==2, "wrong number of owners");

    let number_2 = 246;
    let number_args = json!({ "number": number_2 }).to_string();
    // root updates
    multi_ownable_call(
        &worker,
        &contract,
        &root,
        "update_number",
        number_args.as_str(),
    )
    .await?;

    let rr = get_number(&worker, &contract).await?;
    println!("NUMBER {:?}", rr);
    assert_eq!(number_1, rr, "wrong number");
    // alice updates
    multi_ownable_call(
        &worker,
        &contract,
        &alice,
        "update_number",
        number_args.as_str(),
    )
    .await?;

    let rr = get_number(&worker, &contract).await?;
    println!("RR {:?}", rr);
    assert_eq!(number_2, rr, "wrong number");

    Ok(())
}
pub async fn multi_ownable_call(
    worker: &Worker<impl Network>,
    park: &Contract,
    account: &Account,
    call: &str,
    args: &str,
) -> anyhow::Result<()> {
    let outcome = account
        .call(&worker, park.id().clone(), "multi_ownable_call")
        .args_json(json!({ "call_name": call, "arguments": args }))?
        .transact()
        .await?;
    println!("park multi_ownable_call: {:#?}", outcome);
    Ok(())
}

pub async fn update_multi_ownable(
    worker: &Worker<impl Network>,
    park: &Contract,
    account: &Account,
    new_owners: Vec<AccountId>,
    threshold: u16,
) -> anyhow::Result<()> {
    let outcome = account
        .call(&worker, park.id().clone(), "update_multi_ownable")
        .args_json(json!({ "owners": new_owners, "threshold": threshold }))?
        .transact()
        .await?;
    println!("park update_multi_ownable: {:#?}", outcome);
    Ok(())
}

async fn get_threshold(worker: &Worker<impl Network>, park: &Contract) -> anyhow::Result<Value> {
    let rr: Value = worker
        .view(park.id().clone(), "get_threshold", Vec::new())
        .await?
        .json()?;
    Ok(rr)
}

async fn get_owners(worker: &Worker<impl Network>, park: &Contract) -> anyhow::Result<Vec<String>> {
    let rr: Vec<String> = worker
        .view(park.id().clone(), "get_owners", Vec::new())
        .await?
        .json()?;
    Ok(rr)
}

async fn get_number(worker: &Worker<impl Network>, park: &Contract) -> anyhow::Result<u16> {
    let rr: u16 = worker
        .view(park.id().clone(), "get_number", Vec::new())
        .await?
        .json()?;
    Ok(rr)
}


pub async fn init(
    worker: &Worker<impl Network>,
    park: &Contract,
    root: &Account,
) -> anyhow::Result<()> {
    let outcome = root
        .call(&worker, park.id().clone(), "new")
        .args_json(json!({
            "owner_id": root.id(),
        }))?
        .transact()
        .await?;
    println!("new: {:#?}", outcome);

    Ok(())
}