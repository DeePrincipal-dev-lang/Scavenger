use proptest::prelude::*;
use soroban_sdk::{testutils::Address as _, Address, Env};
use stellar_scavngr_contract::ScavngrContract;

proptest! {
    #[test]
    fn fuzz_participant_registration(
        role in 0u32..3,
        lat in -90i32..90i32,
        lon in -180i32..180i32,
    ) {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavngrContract);
        let client = ScavngrContract::new(&env, &contract_id);

        let admin = Address::random(&env);
        let participant = Address::random(&env);

        client.initialize_admin(&admin);

        // Should not panic with any valid role and coordinates
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.register_participant(&participant, &role, &"Test".into(), &lat, &lon);
        }));

        prop_assert!(result.is_ok(), "Registration should not panic");
    }

    #[test]
    fn fuzz_waste_submission(
        weight in 1u128..u128::MAX,
        lat in -90i32..90i32,
        lon in -180i32..180i32,
    ) {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavngrContract);
        let client = ScavngrContract::new(&env, &contract_id);

        let admin = Address::random(&env);
        let participant = Address::random(&env);

        client.initialize_admin(&admin);
        client.register_participant(&participant, &0, &"Recycler".into(), &0, &0);

        // Should handle any weight value
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.submit_material(&participant, &0, &weight, &lat, &lon);
        }));

        prop_assert!(result.is_ok(), "Waste submission should not panic");
    }

    #[test]
    fn fuzz_waste_transfer(
        waste_id in 0u64..1000,
        lat in -90i32..90i32,
        lon in -180i32..180i32,
    ) {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavngrContract);
        let client = ScavngrContract::new(&env, &contract_id);

        let admin = Address::random(&env);
        let from = Address::random(&env);
        let to = Address::random(&env);

        client.initialize_admin(&admin);
        client.register_participant(&from, &0, &"Recycler".into(), &0, &0);
        client.register_participant(&to, &1, &"Collector".into(), &0, &0);

        // Should handle invalid waste IDs gracefully
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = client.transfer_waste(&waste_id, &from, &to, &lat, &lon, &"Transfer".into());
        }));

        prop_assert!(result.is_ok(), "Waste transfer should not panic");
    }

    #[test]
    fn fuzz_incentive_creation(
        reward_points in 1u128..u128::MAX,
        budget in 1u128..u128::MAX,
    ) {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavngrContract);
        let client = ScavngrContract::new(&env, &contract_id);

        let admin = Address::random(&env);
        let manufacturer = Address::random(&env);

        client.initialize_admin(&admin);
        client.register_participant(&manufacturer, &2, &"Manufacturer".into(), &0, &0);

        // Should handle any reward and budget values
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.create_incentive(&manufacturer, &0, &reward_points, &budget);
        }));

        prop_assert!(result.is_ok(), "Incentive creation should not panic");
    }
}
