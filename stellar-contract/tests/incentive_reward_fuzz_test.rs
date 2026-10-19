#![cfg(test)]
// Fuzz target for `calculate_incentive_reward` (issue #929: new fuzz target).
// Unlike reward_fuzz_test.rs (which fuzzes extracted pure math), this drives
// the real contract entry point end-to-end.

use proptest::prelude::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env};
use stellar_scavngr_contract::{ParticipantRole, ScavengerContract, ScavengerContractClient, WasteType};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Invariant: for any reward_points/budget/weight combination that stays
    /// within u64 (no overflow panic expected), `calculate_incentive_reward`
    /// returns exactly `floor(weight_grams / 1000) * reward_points`,
    /// regardless of the incentive's remaining budget (the function does not
    /// cap by budget — that's `distribute_rewards`'s job).
    #[test]
    fn fuzz_calculate_incentive_reward_matches_formula(
        reward_points in 0u64..=1_000_000u64,
        total_budget in 1u64..=u64::MAX,
        weight_grams in 0u64..=1_000_000_000u64,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let client = ScavengerContractClient::new(&env, &env.register_contract(None, ScavengerContract));

        let manufacturer = Address::generate(&env);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer, &symbol_short!("Mfr"), &0, &0);

        let incentive = client.create_incentive(&manufacturer, &WasteType::Metal, &reward_points, &total_budget);

        let expected = (weight_grams / 1000) * reward_points;
        let actual = client.calculate_incentive_reward(&incentive.id, &weight_grams);

        prop_assert_eq!(actual, expected);
    }

    /// Invariant: deactivating an incentive makes every subsequent reward
    /// calculation return exactly 0, regardless of weight or reward_points.
    #[test]
    fn fuzz_inactive_incentive_always_yields_zero_reward(
        reward_points in 0u64..=1_000_000u64,
        weight_grams in 0u64..=1_000_000_000u64,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let client = ScavengerContractClient::new(&env, &env.register_contract(None, ScavengerContract));

        let manufacturer = Address::generate(&env);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer, &symbol_short!("Mfr"), &0, &0);

        let incentive = client.create_incentive(&manufacturer, &WasteType::Metal, &reward_points, &1_000_000);
        client.deactivate_incentive(&incentive.id, &manufacturer);

        prop_assert_eq!(client.calculate_incentive_reward(&incentive.id, &weight_grams), 0);
    }
}
