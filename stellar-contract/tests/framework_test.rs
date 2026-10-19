#![cfg(test)]
mod test_helpers;
use soroban_sdk::Env;
use stellar_scavngr_contract::WasteType;

#[test]
fn test_full_setup_helper() {
    let env = Env::default();
    let (client, _admin, recycler, collector, manufacturer) = test_helpers::setup_full(&env);
    assert!(client.is_participant_registered(&recycler));
    assert!(client.is_participant_registered(&collector));
    assert!(client.is_participant_registered(&manufacturer));
}

#[test]
fn test_submit_waste_helper() {
    let env = Env::default();
    let (client, _admin, recycler, _collector, _manufacturer) = test_helpers::setup_full(&env);
    let waste_id = test_helpers::submit_waste(&client, &recycler, WasteType::Plastic, 5000);
    assert!(waste_id > 0);
    test_helpers::assert_waste_owner(&client, waste_id, &recycler);
}

#[test]
fn test_create_recyclers_helper() {
    let env = Env::default();
    let (client, _admin, _recycler, _collector, _manufacturer) = test_helpers::setup_full(&env);
    let recyclers = test_helpers::create_recyclers(&env, &client, 3);
    assert_eq!(recyclers.len(), 3);
    for addr in &recyclers {
        assert!(client.is_participant_registered(addr));
    }
}

#[test]
fn test_create_incentive_helper() {
    let env = Env::default();
    let (client, _admin, _recycler, _collector, manufacturer) = test_helpers::setup_full(&env);
    let incentive_id = test_helpers::create_test_incentive(&client, &manufacturer, WasteType::Metal, 100, 5000);
    assert!(incentive_id > 0);
    let incentive = client.get_incentive_by_id(&incentive_id).expect("incentive not found");
    assert!(incentive.active);
}
