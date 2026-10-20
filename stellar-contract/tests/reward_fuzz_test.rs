#![cfg(test)]
// fuzz tests for reward distribution math
use proptest::prelude::*;

/// Pure reward-distribution math extracted from `distribute_rewards`.
/// Returns `(total_distributed, recycler_amount)`.
fn calc_reward(
    reward_points: u64,
    weight_grams: u64,
    collector_pct: u32,
    owner_pct: u32,
    num_collectors: u32,
) -> (i128, i128) {
    let weight_kg = (weight_grams / 1000) as i128;
    let total_reward = (reward_points as i128) * weight_kg;

    let collector_share = (total_reward * collector_pct as i128) / 100;
    let owner_share = (total_reward * owner_pct as i128) / 100;

    // Each collector gets `collector_share` individually (matches contract behaviour).
    // Total collector pool is capped at `total_reward - owner_share` to prevent overflow.
    let total_collector_pool = collector_share * num_collectors as i128;
    let total_distributed = total_collector_pool.min(total_reward - owner_share) + owner_share;
    let recycler_amount = total_reward - total_distributed;

    (total_distributed, recycler_amount)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    /// Invariant 1: total distributed never exceeds total reward.
    /// Invariant 2: recycler always receives a non-negative amount.
    /// Invariant 3: total_distributed is never negative (no share underflows).
    /// Invariant 4: zero weight (below 1 kg) always yields zero reward and
    ///              zero distribution, matching the contract's floor-division
    ///              behaviour.
    ///
    /// Percentages are constrained so collector_pct + owner_pct <= 100,
    /// matching the guard in `ScavengerContract::initialize`.
    ///
    /// Corpus broadened (issue #929): weight_grams now includes the 0–999 g
    /// sub-kilogram range, reward_points extends to u64::MAX, and
    /// num_collectors extends to 500 to stress the accumulation loop.
    #[test]
    fn fuzz_reward_distribution(
        reward_points in 0u64..=u64::MAX,
        weight_grams in 0u64..=1_000_000_000u64,  // 0 g – 1 000 000 kg
        collector_pct in 0u32..=100u32,
        owner_pct_offset in 0u32..=100u32,
        num_collectors in 0u32..=500u32,
    ) {
        // Mirror the contract's initialize invariant: sum <= 100
        let owner_pct = owner_pct_offset.min(100 - collector_pct);

        let (total_distributed, recycler_amount) =
            calc_reward(reward_points, weight_grams, collector_pct, owner_pct, num_collectors);

        let total_reward = (reward_points as i128) * ((weight_grams / 1000) as i128);

        prop_assert!(
            total_distributed <= total_reward,
            "total_distributed ({total_distributed}) > total_reward ({total_reward})"
        );
        prop_assert!(
            total_distributed >= 0,
            "total_distributed ({total_distributed}) is negative"
        );
        prop_assert!(
            recycler_amount >= 0,
            "recycler_amount ({recycler_amount}) is negative"
        );
        if weight_grams < 1000 {
            prop_assert_eq!(total_reward, 0);
            prop_assert_eq!(total_distributed, 0);
            prop_assert_eq!(recycler_amount, 0);
        }
    }

    /// Fuzz target mirroring the percentage-sum guard in
    /// `ScavengerContract::set_percentages` / `set_collector_percentage` /
    /// `set_owner_percentage` (issue #926's checked-add fix): `checked_add`
    /// must report overflow exactly when the true (widened) sum exceeds
    /// `u32::MAX`, and must otherwise report the exact widened sum — never a
    /// silently wrapped value.
    #[test]
    fn fuzz_percentage_sum_never_silently_wraps(
        collector_pct in 0u32..=u32::MAX,
        owner_pct in 0u32..=u32::MAX,
    ) {
        let true_sum = collector_pct as u64 + owner_pct as u64;
        match collector_pct.checked_add(owner_pct) {
            Some(sum) => {
                prop_assert!(true_sum <= u32::MAX as u64);
                prop_assert_eq!(sum as u64, true_sum);
            }
            None => prop_assert!(true_sum > u32::MAX as u64),
        }
    }
}
