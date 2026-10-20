#![cfg(test)]
// Property-based invariant tests for the recycling supply chain (issue #928).
//
// Invariant 1 (weight conservation): a waste item's recorded weight never
// changes as it moves through the supply chain — only ownership changes.
// Invariant 2 (ownership): after a successful transfer, the recipient is the
// sole current owner — the waste appears in the recipient's owned-list and no
// longer in the sender's.

use proptest::prelude::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};
use stellar_scavngr_contract::{ParticipantRole, ScavengerContract, ScavengerContractClient, WasteType};

const MAX_WASTE_WEIGHT: u128 = 1_000_000_000;

fn waste_type_from_index(i: u8) -> WasteType {
    match i % 5 {
        0 => WasteType::Paper,
        1 => WasteType::PetPlastic,
        2 => WasteType::Plastic,
        3 => WasteType::Metal,
        _ => WasteType::Glass,
    }
}

fn assert_owner_and_membership(
    client: &ScavengerContractClient<'_>,
    waste_id: u128,
    expected_owner: &Address,
    previous_owner: &Address,
) {
    let waste = client.get_waste_v2(&waste_id).expect("waste must exist after transfer");
    assert_eq!(&waste.current_owner, expected_owner, "ownership invariant violated");

    let owner_list = client.get_participant_wastes_v2(expected_owner);
    assert!(
        owner_list.iter().any(|id| id == waste_id),
        "recipient's owned-waste list must contain the transferred waste_id"
    );

    let previous_list = client.get_participant_wastes_v2(previous_owner);
    assert!(
        !previous_list.iter().any(|id| id == waste_id),
        "sender's owned-waste list must no longer contain the transferred waste_id"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    /// Single-hop chain: Recycler -> Manufacturer.
    /// Weight must be identical before and after the transfer; ownership must
    /// move cleanly from recycler to manufacturer.
    #[test]
    fn prop_weight_and_ownership_preserved_recycler_to_manufacturer(
        weight in 1u128..=MAX_WASTE_WEIGHT,
        waste_type_idx in 0u8..5,
        lat in -90_000_000i128..=90_000_000i128,
        lon in -180_000_000i128..=180_000_000i128,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let client = ScavengerContractClient::new(&env, &env.register_contract(None, ScavengerContract));

        let recycler = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &symbol_short!("R"), &0, &0);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer, &symbol_short!("M"), &0, &0);

        let waste_id = client.recycle_waste(&waste_type_from_index(waste_type_idx), &weight, &recycler, &0, &0);
        let before = client.get_waste_v2(&waste_id).unwrap();
        prop_assert_eq!(before.weight, weight);

        client.transfer_waste_v2(&waste_id, &recycler, &manufacturer, &lat, &lon);

        let after = client.get_waste_v2(&waste_id).unwrap();
        prop_assert_eq!(after.weight, weight, "weight must be conserved across transfer");
        assert_owner_and_membership(&client, waste_id, &manufacturer, &recycler);
    }

    /// Two-hop chain: Recycler -> Collector -> Manufacturer.
    /// Weight and exclusive ownership must hold after *each* hop, not just
    /// at the end of the chain.
    #[test]
    fn prop_weight_and_ownership_preserved_through_collector_chain(
        weight in 1u128..=MAX_WASTE_WEIGHT,
        waste_type_idx in 0u8..5,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let client = ScavengerContractClient::new(&env, &env.register_contract(None, ScavengerContract));

        let recycler = Address::generate(&env);
        let collector = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &symbol_short!("R"), &0, &0);
        client.register_participant(&collector, &ParticipantRole::Collector, &symbol_short!("C"), &0, &0);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer, &symbol_short!("M"), &0, &0);

        let waste_id = client.recycle_waste(&waste_type_from_index(waste_type_idx), &weight, &recycler, &0, &0);

        client.transfer_waste_v2(&waste_id, &recycler, &collector, &0, &0);
        prop_assert_eq!(client.get_waste_v2(&waste_id).unwrap().weight, weight);
        assert_owner_and_membership(&client, waste_id, &collector, &recycler);

        client.transfer_waste_v2(&waste_id, &collector, &manufacturer, &0, &0);
        prop_assert_eq!(client.get_waste_v2(&waste_id).unwrap().weight, weight);
        assert_owner_and_membership(&client, waste_id, &manufacturer, &collector);
    }

    /// Invalid routes (e.g. skipping backwards, or re-transferring from a
    /// terminal Manufacturer) must be rejected without mutating state:
    /// weight and ownership must remain exactly as they were before the
    /// rejected call.
    #[test]
    fn prop_invalid_route_leaves_weight_and_ownership_unchanged(
        weight in 1u128..=MAX_WASTE_WEIGHT,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let client = ScavengerContractClient::new(&env, &env.register_contract(None, ScavengerContract));

        let collector = Address::generate(&env);
        let recycler = Address::generate(&env);
        client.register_participant(&collector, &ParticipantRole::Collector, &symbol_short!("C"), &0, &0);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &symbol_short!("R"), &0, &0);

        // Collector -> Recycler is not a valid route.
        let waste_id = client.recycle_waste(&WasteType::Metal, &weight, &collector, &0, &0);
        let result = client.try_transfer_waste_v2(&waste_id, &collector, &recycler, &0, &0);
        prop_assert!(result.is_err());

        let waste = client.get_waste_v2(&waste_id).unwrap();
        prop_assert_eq!(waste.weight, weight);
        prop_assert_eq!(&waste.current_owner, &collector);
    }
}
