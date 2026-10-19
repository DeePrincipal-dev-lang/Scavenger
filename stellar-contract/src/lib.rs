#![no_std]

mod errors;
mod events;
mod test_transfer_path_validation;
mod test_factories;
mod types;
mod validation;

#[cfg(test)]
mod validation_properties;

pub use errors::Error;
pub use types::{
    Challenge, ChallengeProgress, ChallengeStatus, CertificationLevel, GlobalMetrics, GradeRecord, Auction, Incentive,
    LeaderboardEntry, Material, Milestone, ParticipantRole, PendingTransfer,
    PendingTransferStatus, ProcessingRecord, ProcessingStatus, RecyclingStats, SeasonalMultiplier,
    TransferItemType, TransferRecord, TransferStatus, Waste, WasteGrade, WasteTransfer, WasteType,
};
pub use types::calculate_carbon_credits;

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String, Symbol, Vec,
};

// Storage keys
const ADMINS: Symbol = symbol_short!("ADMINS");
const CHARITY: Symbol = symbol_short!("CHARITY");
const REWARD_CFG: Symbol = symbol_short!("RWD_CFG");
const TOTAL_WEIGHT: Symbol = symbol_short!("TOT_WGT");
const TOTAL_TOKENS: Symbol = symbol_short!("TOT_TKN");
const REENTRANCY_GUARD: Symbol = symbol_short!("RE_GUARD");
const TOKEN_ADDR: Symbol = symbol_short!("TKN_ADDR");
const PART_INDEX: Symbol = symbol_short!("PART_IDX");
const PAUSED: Symbol = symbol_short!("PAUSED");
const MULTISIG_THRESHOLD: Symbol = symbol_short!("MS_THRESH");
const PROPOSAL_COUNT: Symbol = symbol_short!("PROP_CNT");

// New feature storage keys
const CHALLENGE_COUNT: Symbol = symbol_short!("CHAL_CNT");
const PENDING_XFR_CNT: Symbol = symbol_short!("PXFR_CNT");
const MILESTONES_KEY: Symbol = symbol_short!("MLSTONES");
const AUCTION_COUNT: Symbol = symbol_short!("AUC_CNT");

// Pre-existing missing constants
const SEASONAL_MUL: Symbol = symbol_short!("SEAS_MUL");
const TOTAL_CARBON: Symbol = symbol_short!("TOT_CARB");
const CONTAMINATED_LIST: Symbol = symbol_short!("CONT_LST");

// Reputation delta constants
const REP_TRANSFER: i128 = 5;
const REP_CONFIRM: i128 = 3;
const REP_VERIFY: i128 = 10;

/// 7 days in seconds
const PROPOSAL_TTL_SECS: u64 = 7 * 24 * 60 * 60;

/// 24 hours in seconds (transfer approval expiry)
const TRANSFER_EXPIRY_SECS: u64 = 24 * 60 * 60;

/// Max active challenges
const MAX_ACTIVE_CHALLENGES: u32 = 10;

/// Min auction duration: 1 hour in seconds
const MIN_AUCTION_DURATION: u64 = 60 * 60;

/// Max auction duration: 7 days in seconds
const MAX_AUCTION_DURATION: u64 = 7 * 24 * 60 * 60;

/// Min bid increment: 5%
const MIN_BID_INCREMENT_PERCENT: u128 = 5;

/// Predefined milestone thresholds in grams
const MILESTONE_THRESHOLDS: [u128; 7] = [
    100_000,
    500_000,
    1_000_000,
    5_000_000,
    10_000_000,
    50_000_000,
    100_000_000,
];

/// Actions that require multi-sig approval.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminAction {
    TransferAdmin(Vec<Address>),
    SetPercentages(u32, u32),
    DeactivateWaste(u128),
}

/// A pending multi-sig proposal.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdminProposal {
    pub id: u64,
    pub action: AdminAction,
    pub proposer: Address,
    pub approvers: Vec<Address>,
    pub executed: bool,
    pub created_at: u64,
}

/// Maximum allowed waste weight per submission (1 000 000 kg in grams).
const MAX_WASTE_WEIGHT: u128 = 1_000_000_000;

/// Reward distribution percentages stored as a single instance-storage entry.
///
/// Consolidating `collector_percentage` and `owner_percentage` into one struct
/// means a single `storage.get` call fetches both values, halving the number
/// of instance-storage lookups on every `_reward_tokens` invocation.
///
/// Migration note: contracts deployed with the old two-key layout
/// (`COL_PCT` / `OWN_PCT`) should call `set_percentages` once after upgrade
/// to write the new `RWD_CFG` key; the old keys are then unused and will
/// expire with the instance TTL.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardConfig {
    /// Percentage of total reward distributed to each collector in the transfer chain.
    pub collector_percentage: u32,
    /// Percentage of total reward distributed to the current waste owner.
    pub owner_percentage: u32,
}

/// On-chain record for a registered supply-chain participant.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participant {
    pub address: Address,
    pub role: ParticipantRole,
    pub name: soroban_sdk::Symbol,
    /// Latitude in microdegrees (e.g. 1_000_000 = 1.0°).
    pub latitude: i128,
    /// Longitude in microdegrees.
    pub longitude: i128,
    pub is_registered: bool,
    /// Cumulative grams of waste processed.
    pub total_waste_processed: u128,
    /// Cumulative reward tokens earned.
    pub total_tokens_earned: u128,
    /// Ledger timestamp at registration.
    pub registered_at: u64,
    /// Reputation score in range [-1000, 10000].
    pub reputation_score: i128,
    /// Ledger timestamp of last activity (for decay).
    pub last_active_at: u64,
    /// Certification level based on activity and accuracy
    pub certification: CertificationLevel,
}

/// Combined view of a participant and their recycling statistics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantInfo {
    pub participant: Participant,
    pub stats: RecyclingStats,
}

#[contract]
pub struct ScavengerContract;

#[contractimpl]
impl ScavengerContract {
    // ========== Reentrancy Guard Functions ==========

    /// Acquire reentrancy lock
    fn lock(env: &Env) {
        if env.storage().instance().has(&REENTRANCY_GUARD) {
            panic!("Reentrant call detected");
        }
        env.storage().instance().set(&REENTRANCY_GUARD, &true);
    }

    /// Release reentrancy lock
    fn unlock(env: &Env) {
        env.storage().instance().remove(&REENTRANCY_GUARD);
    }

    // ========== Admin Functions ==========

    /// Initialise the contract administrator.
    ///
    /// Must be called exactly once, immediately after deployment.
    /// Subsequent calls panic with `"Admin already initialized"`.
    ///
    /// # Parameters
    /// - `admin`: Address that will hold admin privileges. Must sign the transaction.
    ///
    /// # Errors
    /// - Panics `"Admin already initialized"` if called more than once.
    pub fn initialize_admin(env: Env, admin: Address) {
        admin.require_auth();

        // Check if admin is already set
        if env.storage().instance().has(&ADMINS) {
            panic!("Admin already initialized");
        }

        let mut admins = Vec::new(&env);
        admins.push_back(admin);
        env.storage().instance().set(&ADMINS, &admins);
    }

    /// Get the current admin addresses.
    ///
    /// # Returns
    /// A vector of `Address`es that hold admin privileges.
    ///
    /// # Errors
    /// - Panics `"Admin not set"` if [`initialize_admin`] has not been called.
    pub fn get_admins(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&ADMINS)
            .expect("Admin not set")
    }

    /// Get the primary admin address (first in the list).
    ///
    /// # Returns
    /// The `Address` of the primary contract administrator.
    ///
    /// # Errors
    /// - Panics `"Admin not set"` if [`initialize_admin`] has not been called.
    pub fn get_admin(env: Env) -> Address {
        Self::get_admins(env)
            .first()
            .expect("No admin found")
            .clone()
    }

    /// Transfer admin rights to new addresses (current admin only)
    /// Replaces the entire admin list with the new list.
    pub fn transfer_admin(env: Env, current_admin: Address, new_admins: Vec<Address>) {
        Self::require_admin(&env, &current_admin);
        // Validate new_admins is not empty
        if new_admins.is_empty() {
            panic!("Admin list cannot be empty");
        }
        env.storage().instance().set(&ADMINS, &new_admins);
        events::emit_admin_transferred(&env, &current_admin);
    }

    /// Add a new admin address (current admin only)
    pub fn add_admin(env: Env, current_admin: Address, new_admin: Address) {
        Self::require_admin(&env, &current_admin);
        let mut admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&ADMINS)
            .expect("Admin not set");
        if !admins.contains(&new_admin) {
            admins.push_back(new_admin);
            env.storage().instance().set(&ADMINS, &admins);
        }
    }

    /// Remove an admin address (current admin only)
    /// Cannot remove the last admin.
    pub fn remove_admin(env: Env, current_admin: Address, admin_to_remove: Address) {
        Self::require_admin(&env, &current_admin);
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&ADMINS)
            .expect("Admin not set");
        if admins.len() <= 1 {
            panic!("Cannot remove the last admin");
        }
        // Find and remove the admin
        let mut new_admins = Vec::new(&env);
        for admin in admins.iter() {
            if admin != admin_to_remove {
                new_admins.push_back(admin);
            }
        }
        if new_admins.len() == admins.len() {
            panic!("Admin to remove not found");
        }
        env.storage().instance().set(&ADMINS, &new_admins);
    }

    /// Check if caller is admin
    fn require_admin(env: &Env, caller: &Address) {
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&ADMINS)
            .expect("Admin not set");

        if !admins.contains(caller) {
            panic!("Unauthorized: caller is not admin");
        }

        caller.require_auth();
    }

    // ========== Access Control Helper Functions ==========

    /// Verify that the caller is a registered participant
    /// Panics with "Caller is not a registered participant" if not registered
    fn only_registered(env: &Env, caller: &Address) {
        caller.require_auth();

        let key = (caller.clone(),);
        let participant: Option<Participant> = env.storage().instance().get(&key);

        match participant {
            Some(p) if p.is_registered => {}
            Some(_) => panic!("Caller is not a registered participant"),
            None => panic!("Caller is not a registered participant"),
        }
    }

    /// Verify that the caller is a registered manufacturer
    /// Panics with "Caller is not a manufacturer" if not a manufacturer
    /// Panics with "Caller is not a registered participant" if not registered
    fn only_manufacturer(env: &Env, caller: &Address) {
        caller.require_auth();

        let key = (caller.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Caller is not a registered participant");

        if !participant.is_registered {
            panic!("Caller is not a registered participant");
        }

        if !participant.role.can_manufacture() {
            panic!("Caller is not a manufacturer");
        }
    }

    fn require_registered(env: &Env, address: &Address) {
        let key = (address.clone(),);
        let participant: Option<Participant> = env.storage().instance().get(&key);

        match participant {
            Some(p) if p.is_registered => {}
            Some(_) => panic!("Participant is not registered"),
            None => panic!("Participant not found"),
        }
    }

    /// Verify that the caller is the contract administrator
    /// Panics with "Caller is not the contract admin" if not admin
    /// Panics with "Contract admin has not been set" if admin not configured
    fn only_admin(env: &Env, caller: &Address) {
        caller.require_auth();

        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&ADMINS)
            .expect("Contract admin has not been set");

        if !admins.contains(caller) {
            panic!("Caller is not the contract admin");
        }
    }

    /// Verify that the caller owns the specified waste item
    /// Panics with "Caller is not the owner of this waste item" if not owner
    /// Panics with "Waste item not found" if waste doesn't exist
    fn only_waste_owner(env: &Env, caller: &Address, waste_id: u128) {
        caller.require_auth();

        let waste: Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste item not found");

        if &waste.current_owner != caller {
            panic!("Caller is not the owner of this waste item");
        }
    }

    // ========== Reentrancy Guard Helper Functions ==========

    fn require_addresses_different(from: &Address, to: &Address) {
        if from == to {
            panic!("Self-transfer is not allowed");
        }
    }

    /// Apply a reputation delta to a participant (clamped to [-1000, 10000]).
    fn apply_reputation(env: &Env, address: &Address, delta: i128) {
        let key = (address.clone(),);
        if let Some(mut p) = env.storage().instance().get::<_, Participant>(&key) {
            p.reputation_score = (p.reputation_score + delta).max(-1000).min(10000);
            env.storage().instance().set(&key, &p);
        }
    }

    // ========== Charity Contract Functions ==========

    /// Set the charity contract address that receives donations.
    ///
    /// # Parameters
    /// - `admin`: Must be the current contract admin and sign the transaction.
    /// - `charity_address`: Target charity contract. Must differ from `admin`.
    ///
    /// # Errors
    /// - Panics `"Charity address cannot be the same as admin"`.
    /// - Panics `"Caller is not the contract admin"` if `admin` is not the admin.
    pub fn set_charity_contract(env: Env, admin: Address, charity_address: Address) {
        Self::only_admin(&env, &admin);

        // Validate address (basic check - address should not be the zero address)
        // In Soroban, we can't easily check for zero address, but we can ensure it's different from admin
        if charity_address == admin {
            panic!("Charity address cannot be the same as admin");
        }

        env.storage().instance().set(&CHARITY, &charity_address);
    }

    /// Get the configured charity contract address.
    ///
    /// # Returns
    /// `Some(Address)` if set, `None` otherwise.
    pub fn get_charity_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&CHARITY)
    }

    /// Donate tokens from the caller's earned balance to the charity contract.
    ///
    /// Deducts `amount` from `donor.total_tokens_earned` and emits a
    /// `donation_made` event. Protected by a reentrancy guard.
    ///
    /// # Parameters
    /// - `donor`: Registered participant making the donation. Must sign.
    /// - `amount`: Number of tokens to donate (must be > 0).
    ///
    /// # Errors
    /// - Panics `"Donation amount must be greater than zero"`.
    /// - Panics `"Insufficient balance"` if donor has fewer tokens than `amount`.
    /// - Panics `"Charity contract not set"` if no charity address is configured.
    pub fn donate_to_charity(env: Env, donor: Address, amount: i128) {
        // Reentrancy guard
        Self::lock(&env);
        Self::require_not_paused(&env);
        Self::only_registered(&env, &donor);

        // Validate amount
        if amount <= 0 {
            Self::unlock(&env);
            panic!("Donation amount must be greater than zero");
        }

        // Validate donor has enough earned token balance.
        let donor_key = (donor.clone(),);
        let mut participant: Participant = env
            .storage()
            .instance()
            .get(&donor_key)
            .expect("Caller is not a registered participant");
        let donation_amount = amount as u128;
        if participant.total_tokens_earned < donation_amount {
            Self::unlock(&env);
            panic!("Insufficient balance");
        }
        participant.total_tokens_earned -= donation_amount;
        env.storage().instance().set(&donor_key, &participant);

        // Get charity contract address
        let charity_contract = env
            .storage()
            .instance()
            .get::<Symbol, Address>(&CHARITY)
            .expect("Charity contract not set");

        // Emit donation event
        events::emit_donation_made(&env, &donor, amount, &charity_contract);

        // Release lock
        Self::unlock(&env);
    }

    // ========== Percentage Configuration Functions ==========

    /// Read the reward config in one storage lookup (internal helper).
    fn get_reward_config(env: &Env) -> RewardConfig {
        env.storage()
            .instance()
            .get(&REWARD_CFG)
            .unwrap_or(RewardConfig {
                collector_percentage: 5,
                owner_percentage: 50,
            })
    }

    /// Set both collector and owner reward percentages atomically.
    ///
    /// Persists a single [`RewardConfig`] entry so that every subsequent
    /// `_reward_tokens` call costs one storage lookup instead of two.
    /// Defaults before first call: collector = 5 %, owner = 50 %.
    ///
    /// # Parameters
    /// - `admin`: Contract admin. Must sign.
    /// - `collector_percentage`: Share (0–100) given to each collector in the chain.
    /// - `owner_percentage`: Share (0–100) given to the current waste owner.
    ///
    /// # Errors
    /// - Panics `"Total percentages cannot exceed 100"` if the sum > 100.
    ///
    /// # Example
    /// ```text
    /// set_percentages(admin, 10, 40)
    /// // collectors get 10 %, owner gets 40 %, remainder goes to recycler
    /// ```
    pub fn set_percentages(
        env: Env,
        admin: Address,
        collector_percentage: u32,
        owner_percentage: u32,
    ) {
        Self::only_admin(&env, &admin);

        if collector_percentage + owner_percentage > 100 {
            panic!("Total percentages cannot exceed 100");
        }

        env.storage().instance().set(
            &REWARD_CFG,
            &RewardConfig {
                collector_percentage,
                owner_percentage,
            },
        );
    }

    /// Get the current collector reward percentage.
    ///
    /// Always returns `Some`; defaults to `5` if never explicitly set.
    pub fn get_collector_percentage(env: Env) -> Option<u32> {
        Some(Self::get_reward_config(&env).collector_percentage)
    }

    /// Get the current owner reward percentage.
    ///
    /// Always returns `Some`; defaults to `50` if never explicitly set.
    pub fn get_owner_percentage(env: Env) -> Option<u32> {
        Some(Self::get_reward_config(&env).owner_percentage)
    }

    /// Update only the collector percentage, preserving the owner percentage.
    ///
    /// # Parameters
    /// - `admin`: Contract admin. Must sign.
    /// - `new_percentage`: New collector share (0–100).
    ///
    /// # Errors
    /// - Panics `"Total percentages cannot exceed 100"` if `new_percentage + owner_pct > 100`.
    pub fn set_collector_percentage(env: Env, admin: Address, new_percentage: u32) {
        Self::only_admin(&env, &admin);

        let mut cfg = Self::get_reward_config(&env);
        if new_percentage + cfg.owner_percentage > 100 {
            panic!("Total percentages cannot exceed 100");
        }
        cfg.collector_percentage = new_percentage;
        env.storage().instance().set(&REWARD_CFG, &cfg);
    }

    /// Update only the owner percentage, preserving the collector percentage.
    ///
    /// # Parameters
    /// - `admin`: Contract admin. Must sign.
    /// - `new_percentage`: New owner share (0–100).
    ///
    /// # Errors
    /// - Panics `"Total percentages cannot exceed 100"` if `collector_pct + new_percentage > 100`.
    pub fn set_owner_percentage(env: Env, admin: Address, new_percentage: u32) {
        Self::only_admin(&env, &admin);

        let mut cfg = Self::get_reward_config(&env);
        if cfg.collector_percentage + new_percentage > 100 {
            panic!("Total percentages cannot exceed 100");
        }
        cfg.owner_percentage = new_percentage;
        env.storage().instance().set(&REWARD_CFG, &cfg);
    }

    // ========== Token Management Functions ==========

    /// Set the SEP-41 token contract address used for reward transfers.
    ///
    /// # Parameters
    /// - `admin`: Contract admin. Must sign.
    /// - `token_address`: Address of the token contract.
    pub fn set_token_address(env: Env, admin: Address, token_address: Address) {
        Self::require_admin(&env, &admin);
        env.storage().instance().set(&TOKEN_ADDR, &token_address);
    }

    /// Get the configured token contract address.
    ///
    /// # Returns
    /// `Some(Address)` if set via [`set_token_address`], `None` otherwise.
    pub fn get_token_address(env: Env) -> Option<Address> {
        env.storage().instance().get(&TOKEN_ADDR)
    }

    /// Set a seasonal reward multiplier (admin only).
    ///
    /// The multiplier is expressed in basis points: 100 = 1x, 150 = 1.5x, 500 = 5x (max).
    /// It is active for rewards distributed between `start` and `end` (inclusive, Unix timestamps).
    ///
    /// # Errors
    /// - Panics if `multiplier` is 0, < 100, or > 500.
    /// - Panics if `start >= end`.
    pub fn set_seasonal_multiplier(env: Env, admin: Address, multiplier: u32, start: u64, end: u64) {
        Self::require_admin(&env, &admin);
        assert!(multiplier >= 100 && multiplier <= 500, "Multiplier must be between 100 and 500 basis points");
        assert!(start < end, "start must be before end");
        let seasonal = SeasonalMultiplier { multiplier, start, end };
        env.storage().instance().set(&SEASONAL_MUL, &seasonal);
        events::emit_seasonal_multiplier_set(&env, multiplier, start, end);
    }

    /// Return the active seasonal multiplier in basis points, or 100 (1x) if none is active.
    pub fn get_current_multiplier(env: Env) -> u32 {
        let now = env.ledger().timestamp();
        if let Some(s) = env.storage().instance().get::<_, SeasonalMultiplier>(&SEASONAL_MUL) {
            if now >= s.start && now <= s.end {
                return s.multiplier;
            }
        }
        100
    }

    /// Manually reward tokens to a registered recipient.
    ///
    /// Increments `recipient.total_tokens_earned` and the global token counter,
    /// then emits a `tokens_rewarded` event. Protected by a reentrancy guard.
    ///
    /// # Parameters
    /// - `rewarder`: Caller authorising the reward. Must sign.
    /// - `recipient`: Registered participant receiving the tokens.
    /// - `amount`: Token amount (must be > 0).
    /// - `waste_id`: Associated waste record ID (used in the emitted event).
    ///
    /// # Errors
    /// - Panics `"Reward amount must be greater than zero"`.
    /// - Panics `"Recipient not registered"`.
    /// - Panics `"Token address not set"` if [`set_token_address`] was not called.
    pub fn reward_tokens(
        env: Env,
        rewarder: Address,
        recipient: Address,
        amount: i128,
        waste_id: u64,
    ) {
        // Reentrancy guard
        Self::lock(&env);

        rewarder.require_auth();
        Self::require_not_paused(&env);

        // Validate amount
        if amount <= 0 {
            Self::unlock(&env);
            panic!("Reward amount must be greater than zero");
        }

        // Validate recipient is registered
        if !Self::is_participant_registered(env.clone(), recipient.clone()) {
            Self::unlock(&env);
            panic!("Recipient not registered");
        }

        // Get token address
        let token_address = env.storage().instance().get::<Symbol, Address>(&TOKEN_ADDR);

        if token_address.is_none() {
            Self::unlock(&env);
            panic!("Token address not set");
        }

        // Update recipient's total tokens earned
        let recipient_key = (recipient.clone(),);
        if let Some(mut participant) = env
            .storage()
            .instance()
            .get::<_, Participant>(&recipient_key)
        {
            participant.total_tokens_earned = participant
                .total_tokens_earned
                .checked_add(amount as u128)
                .expect("Token overflow");
            env.storage().instance().set(&recipient_key, &participant);
        }

        // Update global total tokens
        let total_tokens: u128 = env.storage().instance().get(&TOTAL_TOKENS).unwrap_or(0);
        let new_total = total_tokens
            .checked_add(amount as u128)
            .expect("Total tokens overflow");
        env.storage().instance().set(&TOTAL_TOKENS, &new_total);

        // Emit token reward event
        events::emit_tokens_rewarded(&env, &recipient, amount as u128, waste_id);

        // Release lock
        Self::unlock(&env);
    }

    // ========== Participant Storage Functions ==========

    /// Store a participant record
    /// Internal helper function for efficient participant storage
    fn set_participant(env: &Env, address: &Address, participant: &Participant) {
        let key = (address.clone(),);
        env.storage().instance().set(&key, participant);
    }

    /// Check whether an address is a registered participant.
    ///
    /// # Returns
    /// `true` if the address exists in storage and `is_registered == true`.
    pub fn is_participant_registered(env: Env, address: Address) -> bool {
        let key = (address,);
        if let Some(p) = env.storage().instance().get::<_, Participant>(&key) {
            p.is_registered
        } else {
            false
        }
    }

    /// Register a new supply-chain participant.
    ///
    /// Stores the participant on-chain and emits a `participant_registered` event.
    ///
    /// # Parameters
    /// - `address`: Participant's Stellar address. Must sign.
    /// - `role`: [`ParticipantRole`] — `Recycler`, `Collector`, or `Manufacturer`.
    /// - `name`: Short display name (Soroban `Symbol`, max 32 chars).
    /// - `latitude`: Location in microdegrees (e.g. `52_520_000` = 52.52°N).
    /// - `longitude`: Location in microdegrees.
    ///
    /// # Returns
    /// The newly created [`Participant`] record.
    ///
    /// # Errors
    /// - Panics `"Participant already registered"` on duplicate registration.
    /// - Panics on invalid coordinates (see [`validation::validate_coordinates`]).
    ///
    /// # Example
    /// ```text
    /// register_participant(alice, ParticipantRole::Recycler, "alice", 52_520_000, 13_405_000)
    /// ```
    pub fn register_participant(
        env: Env,
        address: Address,
        role: ParticipantRole,
        name: soroban_sdk::Symbol,
        latitude: i128,
        longitude: i128,
    ) -> Participant {
        Self::require_not_paused(&env);
        address.require_auth();

        // Validate coordinates
        validation::validate_coordinates(latitude, longitude);

        // Check if already registered
        if Self::is_participant_registered(env.clone(), address.clone()) {
            panic!("Participant already registered");
        }

        let participant = Participant {
            address: address.clone(),
            role,
            name: name.clone(),
            latitude,
            longitude,
            is_registered: true,
            total_waste_processed: 0,
            total_tokens_earned: 0,
            registered_at: env.ledger().timestamp(),
            reputation_score: 0,
            last_active_at: env.ledger().timestamp(),
            certification: CertificationLevel::Beginner,
        };

        // Store participant using helper function
        Self::set_participant(&env, &address, &participant);

        // Add to participant index
        let mut participant_index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));
        participant_index.push_back(address.clone());
        env.storage()
            .instance()
            .set(&PART_INDEX, &participant_index);

        // Emit event
        events::emit_participant_registered(
            &env,
            &address,
            role,
            name.clone(),
            latitude,
            longitude,
        );

        participant
    }

    /// Update participant statistics after processing waste
    /// Uses checked arithmetic to prevent overflow
    fn update_participant_stats(
        env: &Env,
        address: &Address,
        waste_weight: u64,
        tokens_earned: u64,
    ) {
        let key = (address.clone(),);
        if let Some(mut participant) = env.storage().instance().get::<_, Participant>(&key) {
            // Use checked arithmetic to prevent overflow
            participant.total_waste_processed = participant
                .total_waste_processed
                .checked_add(waste_weight as u128)
                .expect("Overflow in total_waste_processed");

            participant.total_tokens_earned = participant
                .total_tokens_earned
                .checked_add(tokens_earned as u128)
                .expect("Overflow in total_tokens_earned");

            // Update certification based on waste processed
            let old_certification = participant.certification;
            participant.certification = CertificationLevel::from_waste_count(participant.total_waste_processed);

            env.storage().instance().set(&key, &participant);

            // Emit certification granted event if level upgraded
            if participant.certification != old_certification {
                events::emit_certification_granted(env, address, participant.certification);
            }

            // Update global total tokens if tokens were earned
            if tokens_earned > 0 {
                Self::add_to_total_tokens(env, tokens_earned as u128);
            }
        }
    }

    /// Helper to distribute token rewards and emit events through the supply chain.
    /// Batches reads and merges writes to minimise storage round-trips.
    fn _reward_tokens(env: &Env, waste_id: u64, total_reward: u128) {
        if total_reward == 0 {
            return;
        }

        // One storage lookup fetches both percentages (was two separate gets)
        let cfg = Self::get_reward_config(env);
        let collector_pct = cfg.collector_percentage;
        let owner_pct = cfg.owner_percentage;

        let collector_share = (total_reward * (collector_pct as u128)) / 100;
        let owner_share = (total_reward * (owner_pct as u128)) / 100;

        // Read transfer history once
        let transfers = Self::get_transfer_history(env.clone(), waste_id);

        let mut total_distributed: u128 = 0;

        // Reward collectors — one read per unique transfer recipient
        for transfer in transfers.iter() {
            let key = (transfer.to.clone(),);
            let participant: Option<Participant> = env.storage().instance().get(&key);
            if let Some(p) = participant {
                if matches!(p.role, ParticipantRole::Collector) {
                    let base_share = collector_share;
                    let multiplier = p.certification.reward_multiplier() as u128;
                    let share = (base_share * multiplier) / 100;
                    total_distributed += share;
                    Self::update_participant_stats(env, &transfer.to, 0, share as u64);
                    events::emit_tokens_rewarded(env, &transfer.to, share, waste_id);
                }
            }
        }

        // Reward the current owner (submitter) — merge owner_share + remainder into one
        // read-modify-write instead of two separate calls.
        if let Some(material) = Self::get_waste_internal(env, waste_id) {
            let recycler_amount = total_reward.saturating_sub(total_distributed + owner_share);
            let submitter_total = owner_share + recycler_amount; // = total_reward - total_distributed

            if submitter_total > 0 {
                let key = (material.submitter.clone(),);
                if let Some(participant) = env.storage().instance().get::<_, Participant>(&key) {
                    let multiplier = participant.certification.reward_multiplier() as u128;
                    let adjusted_total = (submitter_total * multiplier) / 100;

                    let mut participant = participant; // make mutable
                    participant.total_tokens_earned = participant
                        .total_tokens_earned
                        .checked_add(adjusted_total)
                        .expect("Overflow in total_tokens_earned");
                    env.storage().instance().set(&key, &participant);

                    // Single global-tokens update for the submitter's combined share
                    Self::add_to_total_tokens(env, adjusted_total);

                    // Emit two events to preserve existing behaviour / test expectations
                    let adjusted_owner_share = (owner_share * multiplier) / 100;
                    let adjusted_recycler_amount = (recycler_amount * multiplier) / 100;
                    events::emit_tokens_rewarded(env, &material.submitter, adjusted_owner_share, waste_id);
                    if adjusted_recycler_amount > 0 {
                        events::emit_tokens_rewarded(
                            env,
                            &material.submitter,
                            adjusted_recycler_amount,
                            waste_id,
                        );
                    }
                }
            }
        }
    }

    /// Store a waste record by ID
    /// Internal helper function for efficient waste storage
    fn set_waste(env: &Env, waste_id: u64, material: &Material) {
        let key = ("waste", waste_id);
        env.storage().instance().set(&key, material);
    }

    /// Retrieve a waste record by ID (internal helper)
    /// Returns None if waste doesn't exist
    fn get_waste_internal(env: &Env, waste_id: u64) -> Option<Material> {
        let key = ("waste", waste_id);
        env.storage().instance().get(&key)
    }

    /// Check whether a waste record with the given ID exists.
    ///
    /// # Returns
    /// `true` if the record is present in storage.
    pub fn waste_exists(env: Env, waste_id: u64) -> bool {
        let key = ("waste", waste_id);
        env.storage().instance().has(&key)
    }

    /// Convert a [`WasteType`] variant to its human-readable string label.
    ///
    /// # Returns
    /// A Soroban `String` such as `"Plastic"` or `"Metal"`.
    pub fn get_waste_type_string(env: Env, waste_type: WasteType) -> String {
        String::from_str(&env, waste_type.as_str())
    }

    /// Convert a [`ParticipantRole`] variant to its human-readable string label.
    ///
    /// # Returns
    /// A Soroban `String` such as `"Recycler"`, `"Collector"`, or `"Manufacturer"`.
    pub fn get_participant_role_string(env: Env, role: ParticipantRole) -> String {
        String::from_str(&env, role.as_str())
    }

    /// Check whether a transfer from `from` to `to` is a permitted role transition.
    ///
    /// Valid routes:
    /// - `Recycler → Collector`
    /// - `Recycler → Manufacturer`
    /// - `Collector → Manufacturer`
    ///
    /// # Returns
    /// `true` if both participants are registered and the role transition is allowed.
    pub fn is_valid_transfer(env: &Env, from: Address, to: Address) -> bool {
        let from_participant: Option<Participant> = env.storage().instance().get(&(from,));
        let to_participant: Option<Participant> = env.storage().instance().get(&(to,));

        let (Some(from_p), Some(to_p)) = (from_participant, to_participant) else {
            return false;
        };

        if !from_p.is_registered || !to_p.is_registered {
            return false;
        }

        // Invalid if transferring to the same role
        if from_p.role == to_p.role {
            return false;
        }

        matches!(
            (from_p.role, to_p.role),
            (ParticipantRole::Recycler, ParticipantRole::Collector)
                | (ParticipantRole::Recycler, ParticipantRole::Manufacturer)
                | (ParticipantRole::Collector, ParticipantRole::Manufacturer)
        )
    }

    /// Get the total count of waste records
    fn get_waste_count(env: &Env) -> u64 {
        env.storage().instance().get(&("waste_count",)).unwrap_or(0)
    }

    /// Increment and return the next waste ID
    fn next_waste_id(env: &Env) -> u64 {
        let count = Self::get_waste_count(env);
        let next_id = count + 1;
        env.storage().instance().set(&("waste_count",), &next_id);
        next_id
    }

    /// Get the total count of incentive records
    #[allow(dead_code)]
    fn get_incentive_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&("incentive_count",))
            .unwrap_or(0)
    }

    /// Increment and return the next incentive ID
    #[allow(dead_code)]
    fn next_incentive_id(env: &Env) -> u64 {
        let count = Self::get_incentive_count(env);
        let next_id = count + 1;
        env.storage()
            .instance()
            .set(&("incentive_count",), &next_id);
        next_id
    }

    /// Store an incentive record by ID
    /// Internal helper function for efficient incentive storage
    fn set_incentive(env: &Env, incentive_id: u64, incentive: &Incentive) {
        let key = ("incentive", incentive_id);
        env.storage().instance().set(&key, incentive);
    }

    /// Retrieve an incentive record by ID
    /// Returns None if incentive doesn't exist
    fn get_incentive_internal(env: &Env, incentive_id: u64) -> Option<Incentive> {
        let key = ("incentive", incentive_id);
        env.storage().instance().get(&key)
    }

    /// Retrieve an incentive by ID (internal compatibility alias).
    fn get_incentive(env: &Env, incentive_id: u64) -> Option<Incentive> {
        Self::get_incentive_internal(env, incentive_id)
    }

    /// Check whether an incentive record with the given ID exists.
    ///
    /// # Returns
    /// `true` if the record is present in storage.
    pub fn incentive_exists(env: Env, incentive_id: u64) -> bool {
        let key = ("incentive", incentive_id);
        env.storage().instance().has(&key)
    }

    /// Get global total weight
    fn get_total_weight(env: &Env) -> u64 {
        env.storage().instance().get(&TOTAL_WEIGHT).unwrap_or(0)
    }

    /// Add to global total weight
    fn add_to_total_weight(env: &Env, weight: u64) {
        let current = Self::get_total_weight(env);
        let new_total = current
            .checked_add(weight)
            .expect("Overflow in total weight");
        env.storage().instance().set(&TOTAL_WEIGHT, &new_total);
    }

    /// Get global total tokens earned
    fn get_total_tokens(env: &Env) -> u128 {
        env.storage().instance().get(&TOTAL_TOKENS).unwrap_or(0)
    }

    /// Add to global total tokens
    fn add_to_total_tokens(env: &Env, tokens: u128) {
        let current = Self::get_total_tokens(env);
        let new_total = current
            .checked_add(tokens)
            .expect("Overflow in total tokens");
        env.storage().instance().set(&TOTAL_TOKENS, &new_total);
    }

    /// Calculate total weight for active waste entries in v2 storage.
    /// Iterates once across the waste ID range to keep reads linear and allocation-free.
    fn get_total_active_waste_weight(env: &Env) -> u64 {
        let mut total_weight: u64 = 0;
        let total_wastes = Self::get_waste_count(env);

        for waste_id in 1..=total_wastes {
            if let Some(waste) = env
                .storage()
                .instance()
                .get::<_, Waste>(&("waste_v2", waste_id as u128))
            {
                if waste.is_active {
                    let weight =
                        u64::try_from(waste.weight).expect("Waste weight exceeds u64 range");
                    total_weight = total_weight
                        .checked_add(weight)
                        .expect("Overflow in active waste total weight");
                }
            }
        }

        total_weight
    }

    /// Retrieve an incentive by its numeric ID.
    ///
    /// # Returns
    /// `Some(Incentive)` if found, `None` otherwise.
    pub fn get_incentive_by_id(env: Env, incentive_id: u64) -> Option<Incentive> {
        Self::get_incentive(&env, incentive_id)
    }

    /// Toggle the active status of an incentive.
    ///
    /// Only the original `rewarder` who created the incentive may call this.
    ///
    /// # Parameters
    /// - `incentive_id`: ID of the incentive to update.
    /// - `is_active`: `true` to re-enable, `false` to pause.
    ///
    /// # Returns
    /// The updated [`Incentive`].
    ///
    /// # Errors
    /// - Panics `"Incentive not found"`.
    pub fn update_incentive_status(env: Env, incentive_id: u64, is_active: bool) -> Incentive {
        Self::require_not_paused(&env);
        let mut incentive: Incentive =
            Self::get_incentive(&env, incentive_id).expect("Incentive not found");

        // Require auth from the rewarder
        incentive.rewarder.require_auth();
        Self::require_registered(&env, &incentive.rewarder);

        incentive.active = is_active;
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }

    /// Update the reward points and total budget of an active incentive.
    ///
    /// Remaining budget is recalculated as `new_total_budget - budget_already_used`.
    /// If `new_total_budget` is less than the amount already spent, the incentive
    /// is automatically deactivated.
    ///
    /// # Parameters
    /// - `incentive_id`: ID of the incentive to update.
    /// - `new_reward_points`: New points per kg (must be > 0).
    /// - `new_total_budget`: New total token budget (must be > 0).
    ///
    /// # Returns
    /// The updated [`Incentive`].
    ///
    /// # Errors
    /// - Panics `"Incentive not found"`.
    /// - Panics `"Incentive is not active"`.
    /// - Panics `"Reward must be greater than zero"`.
    /// - Panics `"Total budget must be greater than zero"`.
    pub fn update_incentive(
        env: Env,
        incentive_id: u64,
        new_reward_points: u64,
        new_total_budget: u64,
    ) -> Incentive {
        Self::require_not_paused(&env);
        // Step 1: Retrieve incentive (existence check)
        let mut incentive: Incentive =
            Self::get_incentive(&env, incentive_id).expect("Incentive not found");

        // Step 2: Authorization check
        incentive.rewarder.require_auth();
        Self::require_registered(&env, &incentive.rewarder);

        // Step 3: Active status check
        if !incentive.active {
            panic!("Incentive is not active");
        }

        // Step 4: Input validation
        if new_reward_points == 0 {
            panic!("Reward must be greater than zero");
        }
        if new_total_budget == 0 {
            panic!("Total budget must be greater than zero");
        }

        // Calculate how much budget has been used
        let budget_used = incentive.total_budget - incentive.remaining_budget;

        // Step 5: Update fields (atomic)
        incentive.reward_points = new_reward_points;
        incentive.total_budget = new_total_budget;

        // Adjust remaining budget based on new total budget
        if new_total_budget > budget_used {
            incentive.remaining_budget = new_total_budget - budget_used;
        } else {
            incentive.remaining_budget = 0;
            incentive.active = false;
        }

        // Step 6: Persist to storage
        Self::set_incentive(&env, incentive_id, &incentive);

        // Step 7: Emit event
        env.events().publish(
            (symbol_short!("inc_upd"), incentive_id),
            (
                incentive.rewarder.clone(),
                new_reward_points,
                new_total_budget,
            ),
        );

        incentive
    }

    /// Calculate the token reward for a given waste amount under a specific incentive.
    ///
    /// Formula: `floor(waste_amount / 1000) * reward_points`, capped at
    /// `incentive.remaining_budget`. Returns `0` if the incentive is inactive.
    ///
    /// # Parameters
    /// - `incentive_id`: ID of the incentive to use.
    /// - `waste_amount`: Waste weight in grams.
    ///
    /// # Returns
    /// Token reward amount (`u64`). Returns `0` for inactive incentives.
    ///
    /// # Errors
    /// - Panics `"Incentive not found"`.
    pub fn calculate_incentive_reward(env: Env, incentive_id: u64, waste_amount: u64) -> u64 {
        let incentive: Incentive =
            Self::get_incentive(&env, incentive_id).expect("Incentive not found");

        // Check if incentive is active
        if !incentive.active {
            return 0;
        }

        // Calculate reward: (weight in kg) * reward_points
        let weight_kg = waste_amount / 1000;
        weight_kg * incentive.reward_points
    }

    /// Get all active incentives for a specific waste type, sorted by reward descending.
    ///
    /// # Parameters
    /// - `waste_type`: The [`WasteType`] to filter by.
    ///
    /// # Returns
    /// A `Vec<Incentive>` ordered highest `reward_points` first.
    pub fn get_incentives_by_waste_type(
        env: Env,
        waste_type: WasteType,
    ) -> soroban_sdk::Vec<Incentive> {
        let mut results: soroban_sdk::Vec<Incentive> = soroban_sdk::Vec::new(&env);
        let count = Self::get_incentive_count(&env);

        for i in 1..=count {
            if let Some(incentive) = Self::get_incentive(&env, i) {
                if incentive.waste_type == waste_type && Self::incentive_in_window(&incentive, env.ledger().timestamp()) {
                    // Keep results sorted by reward_points descending.
                    let mut inserted = false;
                    for idx in 0..results.len() {
                        if incentive.reward_points > results.get(idx).unwrap().reward_points {
                            results.insert(idx, incentive.clone());
                            inserted = true;
                            break;
                        }
                    }
                    if !inserted {
                        results.push_back(incentive);
                    }
                }
            }
        }

        results
    }

    /// Alias for [`get_incentives_by_waste_type`].
    pub fn get_incentives(env: Env, waste_type: WasteType) -> soroban_sdk::Vec<Incentive> {
        Self::get_incentives_by_waste_type(env, waste_type)
    }

    /// Get all currently active incentives across all waste types.
    ///
    /// # Returns
    /// A `Vec<Incentive>` in insertion order.
    pub fn get_active_incentives(env: Env) -> soroban_sdk::Vec<Incentive> {
        let mut results = soroban_sdk::Vec::new(&env);
        let count = Self::get_incentive_count(&env);
        let now = env.ledger().timestamp();

        for i in 1..=count {
            if let Some(incentive) = Self::get_incentive(&env, i) {
                if Self::incentive_in_window(&incentive, now) {
                    results.push_back(incentive);
                }
            }
        }

        results
    }

    /// Retrieve a participant record by address.
    ///
    /// # Returns
    /// `Some(Participant)` if the address is registered, `None` otherwise.
    pub fn get_participant(env: Env, address: Address) -> Option<Participant> {
        let key = (address,);
        env.storage().instance().get(&key)
    }

    /// Get total tokens earned by a specific participant
    /// Returns the total tokens earned, or 0 for unregistered participants
    pub fn get_participant_earnings(env: Env, address: Address) -> i128 {
        let key = (address,);
        if let Some(participant) = env.storage().instance().get::<_, Participant>(&key) {
            participant.total_tokens_earned as i128
        } else {
            0
        }
    }

    /// Get participant information with current statistics
    /// Returns participant details along with their recycling statistics
    /// Returns None if participant is not registered
    /// Retrieve a participant together with their recycling statistics.
    ///
    /// # Returns
    /// `Some(ParticipantInfo)` if registered, `None` if the address is unknown.
    pub fn get_participant_info(env: Env, address: Address) -> Option<ParticipantInfo> {
        let participant = Self::get_participant(env.clone(), address.clone())?;
        let stats =
            Self::get_stats(env, address.clone()).unwrap_or_else(|| RecyclingStats::new(address));

        Some(ParticipantInfo { participant, stats })
    }

    /// Get all registered participants with pagination
    /// Returns a paginated list of participant addresses
    ///
    /// # Arguments
    /// * `offset` - Starting index (0-based)
    /// * `limit` - Maximum number of results to return
    ///
    /// # Returns
    /// Vector of participant addresses, limited by the specified limit
    /// Returns empty vector if offset is beyond the list size
    pub fn get_all_participants(env: Env, offset: u32, limit: u32) -> Vec<Address> {
        let participant_index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        let total_count = participant_index.len();
        let mut result = Vec::new(&env);

        // Return empty if offset is beyond the list
        if offset >= total_count {
            return result;
        }

        // Calculate the end index
        let end = core::cmp::min(offset + limit, total_count);

        // Collect addresses from offset to end
        for i in offset..end {
            if let Some(addr) = participant_index.get(i) {
                result.push_back(addr);
            }
        }

        result
    }

    /// Grant certification to a participant (admin only)
    ///
    /// # Parameters
    /// - `address`: Participant's address
    /// - `level`: Certification level to grant
    ///
    /// # Errors
    /// - Panics if caller is not admin
    /// - Panics if participant not found
    pub fn grant_certification(env: Env, address: Address, level: CertificationLevel) {
        Self::require_admin(&env);

        let key = (address.clone(),);
        if let Some(mut participant) = env.storage().instance().get::<_, Participant>(&key) {
            let old_level = participant.certification;
            participant.certification = level;
            env.storage().instance().set(&key, &participant);

            // Emit event if level changed
            if level != old_level {
                events::emit_certification_granted(&env, &address, level);
            }
        } else {
            panic!("Participant not found");
        }
    }

    /// Get participants by certification level
    ///
    /// # Parameters
    /// - `level`: Certification level to filter by
    ///
    /// # Returns
    /// Vector of participant addresses with the specified certification level
    pub fn get_participants_by_certification(env: Env, level: CertificationLevel) -> Vec<Address> {
        let participant_index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        let mut result = Vec::new(&env);

        for addr in participant_index {
            if let Some(participant) = Self::get_participant(env.clone(), addr.clone()) {
                if participant.certification == level {
                    result.push_back(addr);
                }
            }
        }

        result
    }

    /// Create a new auction for waste material
    ///
    /// # Parameters
    /// - `waste_id`: ID of the waste to auction
    /// - `start_price`: Starting price in tokens
    /// - `duration`: Duration in seconds (1 hour to 7 days)
    ///
    /// # Returns
    /// The created auction ID
    ///
    /// # Errors
    /// - Panics if waste not found or not owned by caller
    /// - Panics if duration invalid
    pub fn create_auction(env: Env, waste_id: u128, start_price: u128, duration: u64) -> u64 {
        Self::require_not_paused(&env);

        // Validate duration
        if duration < MIN_AUCTION_DURATION || duration > MAX_AUCTION_DURATION {
            panic!("Invalid auction duration");
        }

        // Get waste and check ownership
        let waste = Self::get_waste_by_id(env.clone(), waste_id as u64)
            .expect("Waste not found");
        if waste.current_owner != env.invoker() {
            panic!("Not waste owner");
        }

        // Transfer waste to contract for auction
        let mut waste: types::Waste = env.storage().instance().get(&("waste_v2", waste_id)).expect("Waste not found");
        waste.current_owner = env.current_contract_address();
        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        // Get next auction ID
        let auction_id = env.storage().instance().get(&AUCTION_COUNT).unwrap_or(0u64) + 1;
        env.storage().instance().set(&AUCTION_COUNT, &auction_id);

        // Create auction
        let auction = Auction::new(&env, auction_id, waste_id, env.invoker(), start_price, duration);
        let key = ("auction", auction_id);
        env.storage().instance().set(&key, &auction);

        // Emit event
        events::emit_auction_created(&env, auction_id, waste_id, &env.invoker(), start_price, auction.end_time);

        auction_id
    }

    /// Place a bid on an active auction
    ///
    /// # Parameters
    /// - `auction_id`: ID of the auction
    /// - `amount`: Bid amount in tokens
    ///
    /// # Errors
    /// - Panics if auction not found or ended
    /// - Panics if bid too low
    pub fn place_bid(env: Env, auction_id: u64, amount: u128) {
        Self::require_not_paused(&env);
        env.invoker().require_auth();

        let key = ("auction", auction_id);
        let mut auction: Auction = env.storage().instance().get(&key).expect("Auction not found");

        if !auction.is_active || auction.is_ended(env.ledger().timestamp()) {
            panic!("Auction not active");
        }

        if !auction.is_valid_bid(amount) {
            panic!("Bid too low");
        }

        // Transfer tokens from bidder to contract (assuming token contract)
        // For simplicity, assume tokens are handled externally or use internal balance

        auction.place_bid(env.invoker(), amount);
        env.storage().instance().set(&key, &auction);

        // Emit event
        events::emit_bid_placed(&env, auction_id, &env.invoker(), amount);
    }

    /// End an auction and transfer waste to winner
    ///
    /// # Parameters
    /// - `auction_id`: ID of the auction
    ///
    /// # Errors
    /// - Panics if auction not found or not ended
    pub fn end_auction(env: Env, auction_id: u64) {
        Self::require_not_paused(&env);

        let key = ("auction", auction_id);
        let mut auction: Auction = env.storage().instance().get(&key).expect("Auction not found");

        if !auction.is_active || !auction.is_ended(env.ledger().timestamp()) {
            panic!("Auction not ended");
        }

        auction.end();
        env.storage().instance().set(&key, &auction);

        // Transfer waste to winner
        if let Some(winner) = &auction.bidder {
            let mut waste: types::Waste = env.storage().instance().get(&("waste_v2", auction.waste_id)).expect("Waste not found");
            waste.current_owner = winner.clone();
            env.storage().instance().set(&("waste_v2", auction.waste_id), &waste);
        }

        // Emit event
        events::emit_auction_ended(&env, auction_id, auction.bidder.as_ref(), auction.current_bid);
    }

    /// Cancel an auction (owner only)
    ///
    /// # Parameters
    /// - `auction_id`: ID of the auction
    ///
    /// # Errors
    /// - Panics if not owner or auction has bids
    pub fn cancel_auction(env: Env, auction_id: u64) {
        Self::require_not_paused(&env);
        env.invoker().require_auth();

        let key = ("auction", auction_id);
        let auction: Auction = env.storage().instance().get(&key).expect("Auction not found");

        if auction.creator != env.invoker() {
            panic!("Not auction creator");
        }

        if auction.bidder.is_some() {
            panic!("Cannot cancel auction with bids");
        }

        // Remove auction
        env.storage().instance().remove(&key);
    }

    /// Get waste by tracking code
    ///
    /// # Parameters
    /// - `code`: Tracking code to search for
    ///
    /// # Returns
    /// Waste item if found
    pub fn get_waste_by_tracking_code(env: Env, code: soroban_sdk::String) -> Option<types::Waste> {
        // This is inefficient, but for demo purposes
        // In production, would need a map from tracking_code to waste_id
        let participant_index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        for addr in participant_index {
            if let Some(waste_list) = env.storage().instance().get::<_, Vec<u128>>(&("participant_wastes", addr)) {
                for waste_id in waste_list {
                    if let Some(waste) = env.storage().instance().get::<_, types::Waste>(&("waste_v2", waste_id)) {
                        if waste.tracking_code == code {
                            return Some(waste);
                        }
                    }
                }
            }
        }
        None
    }

    /// Bulk import historical waste data (admin only)
    ///
    /// # Parameters
    /// - `wastes`: Vector of waste data to import
    ///
    /// # Errors
    /// - Panics if not admin
    /// - Panics if batch too large
    pub fn bulk_import_wastes(env: Env, wastes: soroban_sdk::Vec<types::Waste>) {
        Self::require_admin(&env);

        if wastes.len() > 100 {
            panic!("Batch too large");
        }

        // Validate all data
        for waste in wastes.iter() {
            if waste.weight == 0 {
                panic!("Invalid waste data");
            }
            // Add more validations as needed
        }

        // Import
        for waste in wastes.iter() {
            env.storage().instance().set(&("waste_v2", waste.waste_id), &waste);
            // Update participant wastes list if needed
        }

        // Emit event
        events::emit_bulk_import_completed(&env, "waste", wastes.len() as u32);
    }

    /// Bulk import historical participant data (admin only)
    ///
    /// # Parameters
    /// - `participants`: Vector of participant data to import
    ///
    /// # Errors
    /// - Panics if not admin
    /// - Panics if batch too large
    pub fn bulk_import_participants(env: Env, participants: soroban_sdk::Vec<Participant>) {
        Self::require_admin(&env);

        if participants.len() > 100 {
            panic!("Batch too large");
        }

        // Validate all data
        for participant in participants.iter() {
            if !participant.is_registered {
                panic!("Invalid participant data");
            }
            // Add more validations as needed
        }

        // Import
        for participant in participants.iter() {
            let key = (participant.address.clone(),);
            env.storage().instance().set(&key, &participant);
            // Add to index
            let mut participant_index: Vec<Address> = env
                .storage()
                .instance()
                .get(&PART_INDEX)
                .unwrap_or(Vec::new(&env));
            if !participant_index.contains(&participant.address) {
                participant_index.push_back(participant.address.clone());
                env.storage().instance().set(&PART_INDEX, &participant_index);
            }
        }

        // Emit event
        events::emit_bulk_import_completed(&env, "participant", participants.len() as u32);
    }

    /// Update participant role
    /// Preserves registration timestamp and other data
    /// Change the role of a registered participant.
    ///
    /// All other fields (name, location, stats, timestamps) are preserved.
    ///
    /// # Parameters
    /// - `address`: Participant's address. Must sign.
    /// - `new_role`: The [`ParticipantRole`] to assign.
    ///
    /// # Returns
    /// The updated [`Participant`].
    ///
    /// # Errors
    /// - Panics `"Participant not found"`.
    /// - Panics `"Participant is not registered"`.
    pub fn update_role(env: Env, address: Address, new_role: ParticipantRole) -> Participant {
        Self::require_not_paused(&env);
        address.require_auth();

        let mut participant: Participant =
            Self::get_participant(env.clone(), address.clone()).expect("Participant not found");

        // Validate participant is registered
        if !participant.is_registered {
            panic!("Participant is not registered");
        }

        participant.role = new_role;
        Self::set_participant(&env, &address, &participant);

        participant
    }

    /// Mark a participant as deregistered (`is_registered = false`).
    ///
    /// The record is retained in storage; the participant can no longer
    /// perform role-gated actions.
    ///
    /// # Parameters
    /// - `address`: Participant's address. Must sign.
    ///
    /// # Returns
    /// The updated [`Participant`] with `is_registered = false`.
    ///
    /// # Errors
    /// - Panics `"Participant not found"`.
    pub fn deregister_participant(env: Env, address: Address) -> Participant {
        Self::require_not_paused(&env);
        address.require_auth();

        let key = (address.clone(),);
        let mut participant: Participant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Participant not found");

        participant.is_registered = false;
        env.storage().instance().set(&key, &participant);

        // Remove from participant index
        let participant_index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        let mut new_index = Vec::new(&env);
        for addr in participant_index.iter() {
            if addr != address {
                new_index.push_back(addr);
            }
        }
        env.storage().instance().set(&PART_INDEX, &new_index);

        participant
    }

    /// Update participant location
    /// Update the location of a registered participant.
    /// Only the participant themselves can call this.
    /// Coordinates are scaled by 1e6 (e.g. 40_000_000 = 40.000000°).
    pub fn update_participant_location(
        env: Env,
        address: Address,
        latitude: i128,
        longitude: i128,
    ) -> Participant {
        Self::require_not_paused(&env);
        address.require_auth();

        validation::validate_coordinates(latitude, longitude);

        let key = (address.clone(),);
        let mut participant: Participant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Participant not found");

        if !participant.is_registered {
            panic!("Participant is not registered");
        }

        participant.latitude = latitude;
        participant.longitude = longitude;
        env.storage().instance().set(&key, &participant);

        events::emit_participant_location_updated(&env, &address, latitude, longitude);

        participant
    }

    /// Update participant location.
    ///
    /// # Deprecated
    /// Use [`update_participant_location`] instead. That function also validates
    /// coordinates and emits a `ParticipantLocationUpdated` event.
    pub fn update_location(
        env: Env,
        address: Address,
        latitude: i128,
        longitude: i128,
    ) -> Participant {
        Self::update_participant_location(env, address, latitude, longitude)
    }

    // ========== Waste Transfer History Functions ==========

    /// Get the full transfer history for a waste item (v1 storage).
    ///
    /// Checks v2 storage first (`transfer_history` key with `u128` waste ID),
    /// then falls back to v1 (`transfers` key with `u64` waste ID).
    ///
    /// # Returns
    /// Chronologically ordered `Vec<WasteTransfer>`. Empty if no transfers recorded.
    pub fn get_transfer_history(env: Env, waste_id: u64) -> Vec<WasteTransfer> {
        // Check v2 storage first (uses u128 waste_id)
        let v2_key = ("transfer_history", waste_id as u128);
        if let Some(history) = env
            .storage()
            .instance()
            .get::<_, Vec<WasteTransfer>>(&v2_key)
        {
            return history;
        }

        // Fall back to v1 storage
        let key = ("transfers", waste_id);
        env.storage().instance().get(&key).unwrap_or(Vec::new(&env))
    }

    /// Alias for [`get_transfer_history`] — returns the full transfer log for a waste item.
    pub fn get_waste_transfer_history(env: Env, waste_id: u64) -> Vec<WasteTransfer> {
        Self::get_transfer_history(env, waste_id)
    }

    /// Get the transfer history for a v2 waste item (uses `u128` waste ID).
    ///
    /// # Returns
    /// Chronologically ordered `Vec<WasteTransfer>`. Empty if no transfers recorded.
    pub fn get_waste_transfer_history_v2(env: Env, waste_id: u128) -> Vec<WasteTransfer> {
        let key = ("transfer_history", waste_id);
        env.storage().instance().get(&key).unwrap_or(Vec::new(&env))
    }

    /// Record a waste transfer
    /// Appends to immutable history
    fn record_transfer(env: &Env, waste_id: u64, from: Address, to: Address, _note: String) {
        let key = ("transfers", waste_id);
        let mut history: Vec<WasteTransfer> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(env));

        let transfer = WasteTransfer::new(
            waste_id as u128,
            from,
            to,
            env.ledger().timestamp(),
            0,
            0,
            soroban_sdk::symbol_short!("note"),
        );

        history.push_back(transfer);
        env.storage().instance().set(&key, &history);
    }

    /// Transfer waste ownership from one participants to another
    /// Transfer waste ownership (v1 — u64 waste IDs, String note).
    ///
    /// # Deprecated
    /// Use [`transfer_waste_v2`] instead. v2 uses u128 waste IDs, records
    /// GPS coordinates, validates transfer routes, and maintains the
    /// `participant_wastes` index. This function is kept for backward
    /// compatibility and will be removed in a future release.
    ///
    /// Migration: replace `transfer_waste(id, from, to, note)` with
    /// `transfer_waste_v2(id as u128, from, to, latitude, longitude)`.
    pub fn transfer_waste(
        env: Env,
        waste_id: u64,
        from: Address,
        to: Address,
        note: String,
    ) -> Material {
        from.require_auth();

        Self::require_not_paused(&env);
        Self::require_registered(&env, &from);
        Self::require_registered(&env, &to);

        let mut material: Material =
            Self::get_waste_internal(&env, waste_id).expect("Waste not found");

        if material.submitter != from {
            panic!("Only waste owner can transfer");
        }

        Self::require_addresses_different(&from, &to);

        // Align with v2: reject transfers on deactivated waste
        // Note: Material doesn't have is_active, assuming active for deprecated function
        // if !material.is_active {
        //     panic!("Cannot transfer deactivated waste");
        // }

        // Align with v2: enforce valid transfer routes
        if !Self::is_valid_transfer(&env, from.clone(), to.clone()) {
            panic!("Invalid transfer: role combination not allowed");
        }

        material.submitter = to.clone();
        Self::set_waste(&env, waste_id, &material);

        events::emit_waste_transferred(&env, waste_id, &from, &to);
        Self::record_transfer(&env, waste_id, from, to, note);

        material
    }

    /// Get all outbound transfers for a participant.
    ///
    /// > **Note:** Currently returns an empty list. A sender index is required
    /// > for efficient lookup and is not yet implemented.
    pub fn get_transfers_from(env: Env, _address: Address) -> Vec<(u64, Vec<WasteTransfer>)> {
        // Note: This is a simplified implementation
        // In production, you'd want to maintain an index for efficient queries
        // This would need to iterate through all wastes
        // For now, returning empty as this requires additional indexing
        Vec::new(&env)
    }

    /// Get all inbound transfers for a participant.
    ///
    /// > **Note:** Currently returns an empty list. A receiver index is required
    /// > for efficient lookup and is not yet implemented.
    pub fn get_transfers_to(env: Env, _address: Address) -> Vec<(u64, Vec<WasteTransfer>)> {
        // Note: This is a simplified implementation
        // In production, you'd want to maintain an index for efficient queries
        // This would need to iterate through all wastes
        // For now, returning empty as this requires additional indexing
        Vec::new(&env)
    }

    /// Check whether a participant is permitted to collect waste materials.
    ///
    /// # Returns
    /// `true` if the participant is registered and their role allows collection
    /// (`Recycler` or `Collector`).
    pub fn can_collect(env: Env, address: Address) -> bool {
        let key = (address,);
        if let Some(participant) = env.storage().instance().get::<_, Participant>(&key) {
            participant.is_registered && participant.role.can_collect_materials()
        } else {
            false
        }
    }

    /// Check whether a participant is permitted to manufacture products.
    ///
    /// # Returns
    /// `true` if the participant is registered and their role is `Manufacturer`.
    pub fn can_manufacture(env: Env, address: Address) -> bool {
        let key = (address,);
        if let Some(participant) = env.storage().instance().get::<_, Participant>(&key) {
            participant.is_registered && participant.role.can_manufacture()
        } else {
            false
        }
    }

    /// Submit a single waste material for recycling (v1 API).
    ///
    /// Creates a [`Material`] record, updates the submitter's recycling stats,
    /// and increments the global weight counter.
    ///
    /// # Parameters
    /// - `waste_type`: Category of the material (e.g. `Plastic`, `Metal`).
    /// - `weight`: Weight in grams (must be > 0 for meaningful rewards).
    /// - `submitter`: Registered participant submitting the material. Must sign.
    /// - `description`: Free-text description of the material.
    ///
    /// # Returns
    /// The newly created [`Material`] record with a unique `id`.
    ///
    /// # Errors
    /// - Panics `"Caller is not a registered participant"` if `submitter` is not registered.
    pub fn submit_material(
        env: Env,
        waste_type: WasteType,
        weight: u64,
        submitter: Address,
        description: String,
    ) -> Material {
        // Validate submitter is registered
        Self::require_not_paused(&env);
        Self::only_registered(&env, &submitter);

        if weight == 0 {
            panic!("Waste weight must be greater than zero");
        }

        if weight as u128 > MAX_WASTE_WEIGHT {
            panic!("Waste weight exceeds maximum allowed");
        }

        // Get next waste ID using the new storage system
        let waste_id = Self::next_waste_id(&env);

        // Create material
        let material = Material::new(
            waste_id,
            waste_type,
            weight,
            submitter.clone(),
            env.ledger().timestamp(),
            description,
        );

        // Store waste using the new storage systems
        Self::set_waste(&env, waste_id, &material);

        // Update stats
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", submitter.clone()))
            .unwrap_or_else(|| RecyclingStats::new(submitter.clone()));

        stats.record_submission(&material);
        env.storage()
            .instance()
            .set(&("stats", submitter.clone()), &stats);

        // Update participant stats
        Self::update_participant_stats(&env, &submitter, weight, 0);

        // Update global total weight
        Self::add_to_total_weight(&env, weight);

        material
    }

    /// Register a new waste item with GPS location data (v2 API).
    ///
    /// Stores the waste in the v2 storage layout and appends its ID to the
    /// recycler's waste list. Emits a `waste_registered` event.
    ///
    /// # Parameters
    /// - `waste_type`: Category of the waste.
    /// - `weight`: Weight in grams (must be > 0).
    /// - `recycler`: Registered participant creating the record. Must sign.
    /// - `latitude`: Collection latitude in microdegrees.
    /// - `longitude`: Collection longitude in microdegrees.
    ///
    /// # Returns
    /// The new waste ID (`u128`).
    ///
    /// # Errors
    /// - Panics `"Waste weight must be greater than zero"`.
    /// - Panics `"Caller is not a registered participant"`.
    pub fn recycle_waste(
        env: Env,
        waste_type: WasteType,
        weight: u128,
        recycler: Address,
        latitude: i128,
        longitude: i128,
    ) -> u128 {
        // Validate recycler is registered
        Self::require_not_paused(&env);
        Self::only_registered(&env, &recycler);

        if weight == 0 {
            panic!("Waste weight must be greater than zero");
        }

        if weight > MAX_WASTE_WEIGHT {
            panic!("Waste weight exceeds maximum allowed");
        }
        let waste_id = Self::next_waste_id(&env) as u128;
        let timestamp = env.ledger().timestamp();

        // Look up per-type TTL; 0 means no expiry
        let ttl: u64 = env
            .storage()
            .instance()
            .get(&("waste_ttl", waste_type))
            .unwrap_or(0u64);
        let expires_at = if ttl > 0 { timestamp + ttl } else { 0u64 };

        let waste = types::Waste::new(
            &env,
            waste_id,
            waste_type,
            weight,
            recycler.clone(),
            latitude,
            longitude,
            timestamp,
            true,
            false,
            recycler.clone(),
            expires_at,
        );

        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        let mut waste_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", recycler.clone()))
            .unwrap_or(Vec::new(&env));
        waste_list.push_back(waste_id);
        env.storage()
            .instance()
            .set(&("participant_wastes", recycler.clone()), &waste_list);

        // Track total weight in stats for recycling rate calculation
        let stats_key = ("stats", recycler.clone());
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&stats_key)
            .unwrap_or_else(|| RecyclingStats::new(recycler.clone()));
        stats.total_weight = stats.total_weight.saturating_add(weight as u64);
        stats.update_recycling_rate();
        env.storage().instance().set(&stats_key, &stats);

        // Emit waste registered event
        events::emit_waste_registered(
            &env, waste_id, &recycler, waste_type, weight, latitude, longitude,
        );

        waste_id
    }

    /// Retrieve a v2 waste record by its `u128` ID.
    ///
    /// # Returns
    /// `Some(Waste)` if found, `None` otherwise.
    pub fn get_waste_v2(env: Env, waste_id: u128) -> Option<types::Waste> {
        env.storage().instance().get(&("waste_v2", waste_id))
    }

    /// Get all v2 waste IDs currently owned by a participant.
    ///
    /// # Returns
    /// `Vec<u128>` of waste IDs. Empty if the participant owns none.
    pub fn get_participant_wastes_v2(env: Env, participant: Address) -> Vec<u128> {
        env.storage()
            .instance()
            .get(&("participant_wastes", participant))
            .unwrap_or(Vec::new(&env))
    }

    /// Update the processing status of a v2 waste item.
    ///
    /// Only the current owner may call this. Status must advance forward
    /// (Collected → Sorted → Processed → Recycled → Manufactured).
    ///
    /// # Panics
    /// - `"Waste not found"` if the waste ID does not exist.
    /// - `"Only current owner can update processing status"` if caller is not the owner.
    /// - `"Status must progress forward"` if new status is not strictly greater.
    pub fn update_processing_status(
        env: Env,
        waste_id: u128,
        caller: Address,
        new_status: ProcessingStatus,
    ) -> types::Waste {
        caller.require_auth();
        Self::require_not_paused(&env);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        assert!(waste.current_owner == caller, "Only current owner can update processing status");
        assert!(
            new_status.to_u32() > waste.processing_status.to_u32(),
            "Status must progress forward"
        );

        let timestamp = env.ledger().timestamp();
        waste.processing_status = new_status;
        waste.processing_history.push_back(ProcessingRecord {
            status: new_status,
            timestamp,
            updated_by: caller.clone(),
        });

        env.storage().instance().set(&("waste_v2", waste_id), &waste);
        events::emit_processing_status_changed(&env, waste_id, new_status.to_u32(), &caller, timestamp);

        // When waste reaches Recycled status, update stats and goals
        if new_status == ProcessingStatus::Recycled {
            let owner = waste.current_owner.clone();
            let stats_key = ("stats", owner.clone());
            let mut stats: RecyclingStats = env
                .storage()
                .instance()
                .get(&stats_key)
                .unwrap_or_else(|| RecyclingStats::new(owner.clone()));
            stats.recycled_weight = stats.recycled_weight.saturating_add(waste.weight);
            stats.update_recycling_rate();
            env.storage().instance().set(&stats_key, &stats);

            // Update goals progress
            let goals_key = ("goals", owner.clone());
            let mut goals: Vec<types::RecyclingGoal> = env
                .storage()
                .instance()
                .get(&goals_key)
                .unwrap_or(Vec::new(&env));
            let mut goals_updated = false;
            for i in 0..goals.len() {
                let mut goal = goals.get(i).unwrap();
                if goal.achieved {
                    continue;
                }
                let type_matches = goal.waste_type.map_or(true, |t| t == waste.waste_type);
                if type_matches {
                    goal.current_weight = goal.current_weight.saturating_add(waste.weight);
                    if goal.current_weight >= goal.target_weight {
                        goal.achieved = true;
                        events::emit_goal_achieved(&env, &owner, goal.target_weight);
                    }
                    goals.set(i, goal);
                    goals_updated = true;
                }
            }
            if goals_updated {
                env.storage().instance().set(&goals_key, &goals);
            }
        }

        waste
    }

    /// Return all v2 waste IDs whose current processing status matches `status`.
    ///
    /// Scans the global waste ID counter and checks each stored waste item.
    pub fn get_wastes_by_status(env: Env, status: ProcessingStatus) -> Vec<u128> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&("waste_count",))
            .unwrap_or(0);
        let mut result = Vec::new(&env);
        for id in 1u128..=(count as u128) {
            if let Some(w) = env
                .storage()
                .instance()
                .get::<_, types::Waste>(&("waste_v2", id))
            {
                if w.processing_status == status {
                    result.push_back(id);
                }
            }
        }
        result
    }

    /// Transfer a v2 waste item between participants with location tracking.
    ///
    /// Validates the role-based transfer route, updates ownership, adjusts both
    /// participants' waste lists, appends to transfer history, and emits a
    /// `transfer` event.
    ///
    /// # Parameters
    /// - `waste_id`: ID of the v2 waste to transfer.
    /// - `from`: Current owner. Must sign and own the waste.
    /// - `to`: New owner. Must be registered.
    /// - `latitude`: Transfer location latitude in microdegrees.
    /// - `longitude`: Transfer location longitude in microdegrees.
    ///
    /// # Returns
    /// The [`WasteTransfer`] record that was appended to history.
    ///
    /// # Errors
    /// - [`Error::WasteNotFound`] if no waste record exists for `waste_id`.
    /// - [`Error::WasteDeactivated`] if the waste item is deactivated.
    /// - [`Error::InvalidTransferRoute`] if the role transition is not permitted.
    /// - Panics `"Caller is not the owner of this waste item"`.
    pub fn transfer_waste_v2(
        env: Env,
        waste_id: u128,
        from: Address,
        to: Address,
        latitude: i128,
        longitude: i128,
    ) -> Result<WasteTransfer, Error> {
        from.require_auth();
        Self::require_not_paused(&env);
        Self::require_addresses_different(&from, &to);

        // Fetch waste first so we can return a typed error if not found
        let mut waste: types::Waste = match env.storage().instance().get(&("waste_v2", waste_id)) {
            Some(w) => w,
            None => return Err(Error::WasteNotFound),
        };

        // Verify caller owns the waste
        if waste.current_owner != from {
            panic!("Caller is not the owner of this waste item");
        }

        Self::require_registered(&env, &from);
        Self::require_registered(&env, &to);

        if !waste.is_active {
            return Err(Error::WasteDeactivated);
        }

        if waste.is_expired(env.ledger().timestamp()) {
            return Err(Error::WasteExpired);
        }

        // Route check after registration checks, before any storage mutation
        if !Self::is_valid_transfer(&env, from.clone(), to.clone()) {
            return Err(Error::InvalidTransferRoute);
        }

        // Reservation check: block transfer if waste is reserved by someone other than `to`
        let now = env.ledger().timestamp();
        if let (Some(reserver), Some(until)) = (waste.reserved_by.clone(), waste.reserved_until) {
            if until > now && reserver != to {
                return Err(Error::WasteReservedByOther);
            }
        }

        waste.transfer_to(to.clone());
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        let from_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", from.clone()))
            .unwrap_or(Vec::new(&env));
        let mut new_from_list = Vec::new(&env);
        for id in from_list.iter() {
            if id != waste_id {
                new_from_list.push_back(id);
            }
        }
        env.storage()
            .instance()
            .set(&("participant_wastes", from.clone()), &new_from_list);

        let mut to_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", to.clone()))
            .unwrap_or(Vec::new(&env));
        to_list.push_back(waste_id);
        env.storage()
            .instance()
            .set(&("participant_wastes", to.clone()), &to_list);

        let timestamp = env.ledger().timestamp();
        let transfer = WasteTransfer::new(
            waste_id,
            from.clone(),
            to.clone(),
            timestamp,
            latitude,
            longitude,
            soroban_sdk::symbol_short!("transfer"),
        );

        let mut history: Vec<WasteTransfer> = env
            .storage()
            .instance()
            .get(&("transfer_history", waste_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(transfer.clone());
        env.storage()
            .instance()
            .set(&("transfer_history", waste_id), &history);

        env.events().publish(
            (soroban_sdk::symbol_short!("transfer"), waste_id),
            (from.clone(), to.clone(), timestamp),
        );

        // Reputation: reward both parties for a successful transfer
        Self::apply_reputation(&env, &from, REP_TRANSFER);
        Self::apply_reputation(&env, &to, REP_TRANSFER);

        Ok(transfer)
    }

    /// Batch transfer multiple waste items to a single recipient
    /// All waste IDs are validated before any transfer is executed (atomic validation)
    /// Emits individual transfer events for each waste item
    pub fn batch_transfer_waste(
        env: Env,
        waste_ids: Vec<u128>,
        to: Address,
        latitude: i128,
        longitude: i128,
    ) -> Result<Vec<WasteTransfer>, Error> {
        // Validate recipient is registered
        Self::require_not_paused(&env);
        Self::require_registered(&env, &to);

        // Handle empty batch
        if waste_ids.is_empty() {
            return Ok(Vec::new(&env));
        }

        // Phase 1: Validate all waste IDs before executing any transfer
        let mut wastes_to_transfer: soroban_sdk::Vec<(u128, types::Waste, Address)> =
            soroban_sdk::Vec::new(&env);

        for waste_id in waste_ids.iter() {
            let waste: types::Waste = env
                .storage()
                .instance()
                .get(&("waste_v2", waste_id))
                .ok_or(Error::WasteNotFound)?;

            // Verify waste is active
            if !waste.is_active {
                return Err(Error::WasteDeactivated);
            }

            if waste.is_expired(env.ledger().timestamp()) {
                return Err(Error::WasteExpired);
            }
            let from = waste.current_owner.clone();

            Self::require_addresses_different(&from, &to);
            if from == to {
                return Err(Error::SameAddress);
            }
            // Verify caller owns the waste
            Self::only_waste_owner(&env, &from, waste_id);
            Self::require_registered(&env, &from);

            // Validate transfer route
            if !Self::is_valid_transfer(&env, from.clone(), to.clone()) {
                return Err(Error::InvalidTransferRoute);
            }

            wastes_to_transfer.push_back((waste_id, waste, from));
        }

        // Phase 2: Execute all transfers (all validations passed)
        let mut transfers: Vec<WasteTransfer> = Vec::new(&env);
        let timestamp = env.ledger().timestamp();

        for item in wastes_to_transfer.iter() {
            let (waste_id, mut waste, from) = item;

            // Update waste ownership
            waste.transfer_to(to.clone());
            env.storage()
                .instance()
                .set(&("waste_v2", waste_id), &waste);

            // Update from participant's waste list
            let from_list: Vec<u128> = env
                .storage()
                .instance()
                .get(&("participant_wastes", from.clone()))
                .unwrap_or(Vec::new(&env));
            let mut new_from_list = Vec::new(&env);
            for id in from_list.iter() {
                if id != waste_id {
                    new_from_list.push_back(id);
                }
            }
            env.storage()
                .instance()
                .set(&("participant_wastes", from.clone()), &new_from_list);

            // Update to participant's waste list
            let mut to_list: Vec<u128> = env
                .storage()
                .instance()
                .get(&("participant_wastes", to.clone()))
                .unwrap_or(Vec::new(&env));
            to_list.push_back(waste_id);
            env.storage()
                .instance()
                .set(&("participant_wastes", to.clone()), &to_list);

            // Create transfer record
            let transfer = WasteTransfer::new(
                waste_id,
                from.clone(),
                to.clone(),
                timestamp,
                latitude,
                longitude,
                soroban_sdk::symbol_short!("transfer"),
            );

            // Update transfer history
            let mut history: Vec<WasteTransfer> = env
                .storage()
                .instance()
                .get(&("transfer_history", waste_id))
                .unwrap_or(Vec::new(&env));
            history.push_back(transfer.clone());
            env.storage()
                .instance()
                .set(&("transfer_history", waste_id), &history);

            // Emit individual transfer event
            env.events().publish(
                (soroban_sdk::symbol_short!("transfer"), waste_id),
                (from, to.clone(), timestamp),
            );

            transfers.push_back(transfer);
        }

        Ok(transfers)
    }

    /// Transfer aggregated waste from collector to manufacturer
    /// Transfer aggregated waste from a collector directly to a manufacturer.
    ///
    /// Creates a new v2 waste record owned by `manufacturer`, records the
    /// transfer, and emits a `bulk_xfr` event. Useful for bulk handoffs where
    /// individual waste IDs are not tracked.
    ///
    /// # Parameters
    /// - `waste_type`: Type of the aggregated waste.
    /// - `collector`: Registered `Collector`. Must sign.
    /// - `manufacturer`: Registered `Manufacturer` receiving the waste.
    /// - `latitude` / `longitude`: Handoff location in microdegrees.
    /// - `notes`: Short symbol note attached to the transfer record.
    ///
    /// # Returns
    /// The new waste ID (`u128`).
    ///
    /// # Errors
    /// - Panics `"Only collectors can use this"`.
    /// - Panics `"Recipient must be manufacturer"`.
    pub fn transfer_collected_waste(
        env: Env,
        waste_type: WasteType,
        collector: Address,
        manufacturer: Address,
        latitude: i128,
        longitude: i128,
        notes: soroban_sdk::Symbol,
    ) -> u128 {
        collector.require_auth();
        Self::require_addresses_different(&collector, &manufacturer);
        Self::require_not_paused(&env);
        Self::require_registered(&env, &collector);
        Self::require_registered(&env, &manufacturer);

        let collector_key = (collector.clone(),);
        let collector_participant: Participant = env
            .storage()
            .instance()
            .get(&collector_key)
            .expect("Collector not registered");

        if collector_participant.role != ParticipantRole::Collector {
            panic!("Only collectors can use this");
        }

        let manufacturer_key = (manufacturer.clone(),);
        let manufacturer_participant: Participant = env
            .storage()
            .instance()
            .get(&manufacturer_key)
            .expect("Manufacturer not registered");

        if manufacturer_participant.role != ParticipantRole::Manufacturer {
            panic!("Recipient must be manufacturer");
        }

        let waste_id = Self::next_waste_id(&env) as u128;
        let timestamp = env.ledger().timestamp();

        let waste = types::Waste::new(
            &env,
            waste_id,
            waste_type,
            0,
            manufacturer.clone(),
            latitude,
            longitude,
            timestamp,
            true,
            false,
            manufacturer.clone(),
            0,
        );

        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        let mut manufacturer_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", manufacturer.clone()))
            .unwrap_or(Vec::new(&env));
        manufacturer_list.push_back(waste_id);
        env.storage().instance().set(
            &("participant_wastes", manufacturer.clone()),
            &manufacturer_list,
        );

        let transfer = WasteTransfer::new(
            waste_id,
            collector.clone(),
            manufacturer.clone(),
            timestamp,
            latitude,
            longitude,
            notes,
        );

        let mut history: Vec<WasteTransfer> = env
            .storage()
            .instance()
            .get(&("transfer_history", waste_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(transfer);
        env.storage()
            .instance()
            .set(&("transfer_history", waste_id), &history);

        env.events().publish(
            (soroban_sdk::symbol_short!("bulk_xfr"), waste_id),
            (collector, manufacturer, waste_type, timestamp),
        );

        waste_id
    }

    /// Confirm receipt of a v2 waste item.
    ///
    /// A participant other than the current owner calls this to acknowledge
    /// the waste. Emits a `waste_confirmed` event.
    ///
    /// # Parameters
    /// - `waste_id`: ID of the v2 waste to confirm.
    /// - `confirmer`: Registered participant confirming receipt. Must sign.
    ///   Must **not** be the current owner.
    ///
    /// # Returns
    /// The updated [`Waste`] with `is_confirmed = true`.
    ///
    /// # Errors
    /// - Panics `"Waste not found"`.
    /// - Panics `"Cannot confirm deactivated waste"`.
    /// - Panics `"Owner cannot confirm own waste"`.
    /// - Panics `"Waste already confirmed"`.
    pub fn confirm_waste_details(env: Env, waste_id: u128, confirmer: Address) -> types::Waste {
        Self::require_not_paused(&env);
        confirmer.require_auth();
        Self::require_registered(&env, &confirmer);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if !waste.is_active {
            panic!("Cannot confirm deactivated waste");
        }

        if waste.current_owner == confirmer {
            panic!("Owner cannot confirm own waste");
        }

        if waste.is_confirmed {
            panic!("Waste already confirmed");
        }

        waste.confirm(confirmer.clone());
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        events::emit_waste_confirmed(&env, waste_id, &confirmer);

        // Reputation: reward confirmer for timely confirmation
        Self::apply_reputation(&env, &confirmer, REP_CONFIRM);
        // Reputation: reward owner for having waste confirmed
        Self::apply_reputation(&env, &waste.current_owner, REP_CONFIRM);

        waste
    }

    /// Reset the confirmation status of a v2 waste item.
    ///
    /// Only the current owner may reset confirmation, allowing the item to be
    /// re-confirmed after a dispute or correction.
    ///
    /// # Parameters
    /// - `waste_id`: ID of the v2 waste.
    /// - `owner`: Current owner. Must sign.
    ///
    /// # Returns
    /// The updated [`Waste`] with `is_confirmed = false`.
    ///
    /// # Errors
    /// - Panics `"Waste item not found"`.
    /// - Panics `"Waste is not confirmed"`.
    /// - Panics `"Caller is not the owner of this waste item"`.
    pub fn reset_waste_confirmation(env: Env, waste_id: u128, owner: Address) -> types::Waste {
        Self::require_not_paused(&env);
        // Access control check - verify caller owns the waste
        Self::only_waste_owner(&env, &owner, waste_id);
        Self::require_registered(&env, &owner);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste item not found");

        if !waste.is_confirmed {
            panic!("Waste is not confirmed");
        }

        waste.reset_confirmation();
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        env.events().publish(
            (soroban_sdk::symbol_short!("reset"), waste_id),
            (owner, env.ledger().timestamp()),
        );

        waste
    }

    /// Permanently deactivate a v2 waste record (admin only).
    ///
    /// Deactivated waste cannot be transferred or confirmed. The operation is
    /// irreversible.
    ///
    /// # Parameters
    /// - `waste_id`: ID of the v2 waste to deactivate.
    /// - `admin`: Contract admin. Must sign.
    ///
    /// # Returns
    /// The updated [`Waste`] with `is_active = false`.
    ///
    /// # Errors
    /// - Panics `"Waste item not found"`.
    /// - Panics `"Waste already deactivated"`.
    pub fn deactivate_waste(env: Env, waste_id: u128, admin: Address) -> types::Waste {
        Self::only_admin(&env, &admin);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste item not found");

        if !waste.is_active {
            panic!("Waste already deactivated");
        }

        waste.deactivate();
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        events::emit_waste_deactivated(&env, waste_id, &admin);

        waste
    }

    /// Deactivate multiple v2 waste records in a single transaction (admin only).
    ///
    /// Skips IDs that do not exist or are already deactivated, and continues
    /// processing the remaining items. Emits a `WasteDeactivated` event for
    /// each successfully deactivated item.
    ///
    /// # Parameters
    /// - `waste_ids`: Vec of v2 waste IDs to deactivate.
    /// - `admin`: Contract admin. Must sign.
    ///
    /// # Returns
    /// Count of items that were successfully deactivated (`u32`).
    pub fn batch_deactivate_waste(env: Env, waste_ids: Vec<u128>, admin: Address) -> u32 {
        Self::only_admin(&env, &admin);

        let mut count: u32 = 0;
        for waste_id in waste_ids.iter() {
            let entry: Option<types::Waste> = env.storage().instance().get(&("waste_v2", waste_id));
            if let Some(mut waste) = entry {
                if waste.is_active {
                    waste.deactivate();
                    env.storage().instance().set(&("waste_v2", waste_id), &waste);
                    events::emit_waste_deactivated(&env, waste_id, &admin);
                    count += 1;
                }
            }
        }
        count
    }

    // ========== Waste TTL / Expiration Functions ==========

    /// Set the TTL (seconds) for a waste type. 0 disables expiry for that type.
    /// Admin only. Applied to newly registered waste; does not retroactively change existing items.
    pub fn set_waste_ttl(env: Env, admin: Address, waste_type: WasteType, ttl_seconds: u64) {
        Self::only_admin(&env, &admin);
        env.storage().instance().set(&("waste_ttl", waste_type), &ttl_seconds);
    }

    /// Get the configured TTL for a waste type (0 = no expiry).
    pub fn get_waste_ttl(env: Env, waste_type: WasteType) -> u64 {
        env.storage().instance().get(&("waste_ttl", waste_type)).unwrap_or(0)
    }

    /// Return IDs of all v2 waste items that have expired at the current ledger time.
    pub fn get_expired_wastes(env: Env) -> Vec<u128> {
        let now = env.ledger().timestamp();
        let total = Self::get_waste_count(&env);
        let mut result = Vec::new(&env);
        for id in 1..=total {
            let waste_id = id as u128;
            if let Some(waste) = env.storage().instance().get::<_, types::Waste>(&("waste_v2", waste_id)) {
                if waste.is_active && waste.is_expired(now) {
                    result.push_back(waste_id);
                }
            }
        }
        result
    }

    /// Deactivate all expired v2 waste items. Admin only.
    /// Emits WasteExpired for each item cleaned up.
    /// Returns the count of items deactivated.
    pub fn cleanup_expired_wastes(env: Env, admin: Address) -> u32 {
        Self::only_admin(&env, &admin);
        let now = env.ledger().timestamp();
        let total = Self::get_waste_count(&env);
        let mut count: u32 = 0;
        for id in 1..=total {
            let waste_id = id as u128;
            if let Some(mut waste) = env.storage().instance().get::<_, types::Waste>(&("waste_v2", waste_id)) {
                if waste.is_active && waste.is_expired(now) {
                    waste.deactivate();
                    env.storage().instance().set(&("waste_v2", waste_id), &waste);
                    events::emit_waste_expired(&env, waste_id);
                    count += 1;
                }
            }
        }
        count
    }
    ///
    /// More gas-efficient than repeated [`submit_material`] calls because stats
    /// and storage writes are batched. Emits no individual events per item.
    ///
    /// # Parameters
    /// - `materials`: Vec of `(WasteType, weight_grams, description)` tuples.
    /// - `submitter`: Registered participant. Must sign.
    ///
    /// # Returns
    /// `Vec<Material>` in the same order as the input.
    ///
    /// # Errors
    /// - Panics `"Caller is not a registered participant"`.
    /// - Panics `"Overflow in batch weight"` on u64 overflow.
    pub fn submit_materials_batch(
        env: Env,
        materials: soroban_sdk::Vec<(WasteType, u64, String)>,
        submitter: Address,
    ) -> soroban_sdk::Vec<Material> {
        // Validate submitter is registered
        Self::require_not_paused(&env);
        Self::only_registered(&env, &submitter);

        let mut results = soroban_sdk::Vec::new(&env);
        let timestamp = env.ledger().timestamp();

        // Get or create stats once
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", submitter.clone()))
            .unwrap_or_else(|| RecyclingStats::new(submitter.clone()));

        let mut total_weight: u64 = 0;

        // Process each material
        for item in materials.iter() {
            let (waste_type, weight, description) = item;
            let waste_id = Self::next_waste_id(&env);

            let material = Material::new(
                waste_id,
                waste_type,
                weight,
                submitter.clone(),
                timestamp,
                description,
            );

            Self::set_waste(&env, waste_id, &material);
            stats.record_submission(&material);
            results.push_back(material);

            // Accumulate weight with overflow check
            total_weight = total_weight
                .checked_add(weight)
                .expect("Overflow in batch weight");
        }

        // Update stats once at the end
        env.storage()
            .instance()
            .set(&("stats", submitter.clone()), &stats);

        // Update participant stats
        Self::update_participant_stats(&env, &submitter, total_weight, 0);

        results
    }

    /// Retrieve a v1 material record by ID. Alias for [`get_waste`].
    pub fn get_material(env: Env, material_id: u64) -> Option<Material> {
        Self::get_waste(env, material_id)
    }

    /// Get waste by ID (alias for backward compatibility)
    ///
    /// # Deprecated
    /// Use [`get_waste`] instead. This function is an exact alias and will be
    /// removed in a future release.
    pub fn get_waste_by_id(env: Env, waste_id: u64) -> Option<Material> {
        Self::get_waste(env, waste_id)
    }

    /// Retrieve a v1 waste/material record by ID.
    ///
    /// # Returns
    /// `Some(Material)` if found, `None` otherwise.
    pub fn get_waste(env: Env, waste_id: u64) -> Option<Material> {
        let key = ("waste", waste_id);
        env.storage().instance().get(&key)
    }

    /// Get all v1 waste IDs currently owned by a participant.
    ///
    /// Performs a linear scan over all waste records. For large datasets,
    /// prefer [`get_participant_wastes_v2`] which uses an indexed list.
    ///
    /// # Returns
    /// `Vec<u64>` of waste IDs where `material.submitter == participant`.
    pub fn get_participant_wastes(env: Env, participant: Address) -> Vec<u64> {
        let mut waste_ids = Vec::new(&env);
        let waste_count = env
            .storage()
            .instance()
            .get::<_, u64>(&("waste_count",))
            .unwrap_or(0);

        // Iterate through all wastes and collect IDs owned by participant
        for waste_id in 1..=waste_count {
            let key = ("waste", waste_id);
            if let Some(material) = env.storage().instance().get::<_, Material>(&key) {
                if material.submitter == participant {
                    waste_ids.push_back(waste_id);
                }
            }
        }

        waste_ids
    }

    /// Retrieve multiple v1 waste records in a single call.
    ///
    /// # Parameters
    /// - `waste_ids`: Vec of waste IDs to fetch.
    ///
    /// # Returns
    /// `Vec<Option<Material>>` in the same order as `waste_ids`.
    /// Each element is `None` if the corresponding ID does not exist.
    pub fn get_wastes_batch(
        env: Env,
        waste_ids: soroban_sdk::Vec<u64>,
    ) -> soroban_sdk::Vec<Option<Material>> {
        let mut results = soroban_sdk::Vec::new(&env);

        for waste_id in waste_ids.iter() {
            results.push_back(Self::get_waste_internal(&env, waste_id));
        }

        results
    }

    /// Verify a submitted material and distribute token rewards.
    ///
    /// Marks the material as verified, calculates reward points based on weight
    /// and waste type, then distributes shares to collectors, the current owner,
    /// and the recycler remainder via `_reward_tokens`.
    ///
    /// # Parameters
    /// - `material_id`: ID of the v1 material to verify.
    /// - `verifier`: Registered `Recycler`. Must sign.
    ///
    /// # Returns
    /// The updated [`Material`] with `verified = true`.
    ///
    /// # Errors
    /// - Panics `"Verifier not registered"` / `"Verifier is not registered"`.
    /// - Panics `"Only recyclers can verify materials"`.
    /// - Panics `"Material not found"`.
    pub fn verify_material(env: Env, material_id: u64, verifier: Address) -> Material {
        Self::require_not_paused(&env);
        verifier.require_auth();

        // Check if verifier is a recycler and is registered
        let verifier_key = (verifier.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&verifier_key)
            .expect("Verifier not registered");

        if !participant.is_registered {
            panic!("Verifier is not registered");
        }

        if !participant.role.can_process_recyclables() {
            panic!("Only recyclers can verify materials");
        }

        // Get and verify material using new storage system
        let mut material: Material =
            Self::get_waste_internal(&env, material_id).expect("Material not found");

        material.verify();
        Self::set_waste(&env, material_id, &material);

        // Calculate tokens earned, reduced by contamination if applicable
        let base_tokens = material.calculate_reward_points();
        let tokens_earned: u64 = {
            let v2: Option<types::Waste> = env
                .storage()
                .instance()
                .get(&("waste_v2", material_id as u128));
            match v2 {
                Some(w) if w.is_contaminated && w.contamination_level > 50 => 0,
                Some(w) if w.is_contaminated => {
                    base_tokens * (100 - w.contamination_level as u64) / 100
                }
                _ => base_tokens,
            }
        };

        // Update submitter stats
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", material.submitter.clone()))
            .unwrap_or_else(|| RecyclingStats::new(material.submitter.clone()));

        stats.record_verification(&material);
        env.storage()
            .instance()
            .set(&("stats", material.submitter.clone()), &stats);

        // Update global carbon credits counter
        let credits = types::calculate_carbon_credits(material.waste_type, material.weight as u128);
        let total_carbon: u128 = env.storage().instance().get(&TOTAL_CARBON).unwrap_or(0);
        env.storage().instance().set(&TOTAL_CARBON, &(total_carbon + credits));

        // Emit carbon credits earned event
        events::emit_carbon_credits_earned(&env, &material.submitter, material.waste_type, material.weight as u128, credits);

        // Distribute token rewards using the helper which also emits TOKENS_REWARDED events
        Self::_reward_tokens(&env, material_id, tokens_earned as u128);

        // Reputation: reward submitter for verified waste
        Self::apply_reputation(&env, &material.submitter, REP_VERIFY);

        material
    }

    /// Verify multiple materials in a single transaction.
    ///
    /// Skips any IDs that do not exist. For each verified material, reward
    /// tokens are distributed and a `tokens_rewarded` event is emitted.
    ///
    /// # Parameters
    /// - `material_ids`: Vec of v1 material IDs to verify.
    /// - `verifier`: Registered `Recycler`. Must sign.
    ///
    /// # Returns
    /// `Vec<Material>` containing only the materials that were found and verified.
    ///
    /// # Errors
    /// - Panics `"Verifier not registered"` / `"Only recyclers can verify materials"`.
    pub fn verify_materials_batch(
        env: Env,
        material_ids: soroban_sdk::Vec<u64>,
        verifier: Address,
    ) -> soroban_sdk::Vec<Material> {
        Self::require_not_paused(&env);
        verifier.require_auth();

        // Check if verifier is a recycler and is registered
        let verifier_key = (verifier.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&verifier_key)
            .expect("Verifier not registered");

        if !participant.is_registered {
            panic!("Verifier is not registered");
        }

        if !participant.role.can_process_recyclables() {
            panic!("Only recyclers can verify materials");
        }

        let mut results = soroban_sdk::Vec::new(&env);

        for material_id in material_ids.iter() {
            if let Some(mut material) = Self::get_waste_internal(&env, material_id) {
                material.verify();
                Self::set_waste(&env, material_id, &material);

                // Calculate tokens earned
                let tokens_earned = material.calculate_reward_points();

                // Update submitter stats
                let mut stats: RecyclingStats = env
                    .storage()
                    .instance()
                    .get(&("stats", material.submitter.clone()))
                    .unwrap_or_else(|| RecyclingStats::new(material.submitter.clone()));

                stats.record_verification(&material);
                env.storage()
                    .instance()
                    .set(&("stats", material.submitter.clone()), &stats);

                // Update global carbon credits counter
                let credits = types::calculate_carbon_credits(material.waste_type, material.weight as u128);
                let total_carbon: u128 = env.storage().instance().get(&TOTAL_CARBON).unwrap_or(0);
                env.storage().instance().set(&TOTAL_CARBON, &(total_carbon + credits));

                // Emit carbon credits earned event
                events::emit_carbon_credits_earned(&env, &material.submitter, material.waste_type, material.weight as u128, credits);

                // Distribute token rewards using the helper which also emits TOKENS_REWARDED events
                Self::_reward_tokens(&env, material_id, tokens_earned as u128);

                results.push_back(material);
            }
        }

        results
    }

    /// Get recycling statistics for a participant.
    ///
    /// # Returns
    /// `Some(RecyclingStats)` if the participant has submitted or verified
    /// at least one material, `None` otherwise.
    pub fn get_stats(env: Env, participant: Address) -> Option<RecyclingStats> {
        env.storage().instance().get(&("stats", participant))
    }

    /// Calculate carbon credits for a given waste type and weight in grams.
    /// Returns grams of CO2 equivalent.
    pub fn calculate_carbon_credits(waste_type: WasteType, weight_grams: u128) -> u128 {
        types::calculate_carbon_credits(waste_type, weight_grams)
    }

    /// Get total carbon credits earned across all participants.
    pub fn get_total_carbon_credits(env: Env) -> u128 {
        env.storage().instance().get(&TOTAL_CARBON).unwrap_or(0)
    }

    /// Get carbon credits earned by a specific participant.
    pub fn get_participant_carbon_credits(env: Env, participant: Address) -> u128 {
        let stats: Option<RecyclingStats> = env.storage().instance().get(&("stats", participant));
        stats.map(|s| s.carbon_credits_earned).unwrap_or(0)
    }

    /// Mark a v2 waste item as contaminated.
    ///
    /// Only a registered Recycler (verifier) may call this.
    /// Contamination level must be 0-100. Reason must not exceed 200 chars.
    /// Emits a `WasteContaminated` event.
    pub fn mark_contaminated(
        env: Env,
        waste_id: u128,
        verifier: Address,
        level: u32,
        reason: String,
    ) -> types::Waste {
        Self::require_not_paused(&env);
        verifier.require_auth();

        assert!(level <= 100, "Contamination level must be 0-100");
        assert!(reason.len() <= 200, "Reason exceeds 200 characters");

        let participant: Participant = env
            .storage()
            .instance()
            .get(&(verifier.clone(),))
            .expect("Verifier not registered");

        assert!(participant.is_registered, "Verifier is not registered");
        assert!(
            participant.role.can_process_recyclables(),
            "Only recyclers can mark contamination"
        );

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        waste.is_contaminated = true;
        waste.contamination_level = level;
        waste.contamination_reason = reason;

        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        // Append to contaminated list
        let mut list: Vec<u128> = env
            .storage()
            .instance()
            .get(&CONTAMINATED_LIST)
            .unwrap_or(Vec::new(&env));
        // Only add once
        if !list.contains(&waste_id) {
            list.push_back(waste_id);
            env.storage().instance().set(&CONTAMINATED_LIST, &list);
        }

        events::emit_waste_contaminated(&env, waste_id, &verifier, level);

        waste
    }

    /// Return all v2 waste IDs that have been marked as contaminated.
    pub fn get_contaminated_wastes(env: Env) -> Vec<u128> {
        env.storage()
            .instance()
            .get(&CONTAMINATED_LIST)
            .unwrap_or(Vec::new(&env))
    }

    /// Get global supply-chain statistics.
    ///
    /// # Returns
    /// A tuple `(total_waste_count, total_active_weight_grams, total_tokens_earned)`.
    ///
    /// > **Note:** `total_active_weight_grams` iterates all v2 waste records and
    /// > may be expensive for large datasets.
    pub fn get_supply_chain_stats(env: Env) -> (u64, u64, u128) {
        let total_wastes = Self::get_waste_count(&env);
        let total_weight = Self::get_total_active_waste_weight(&env);
        let total_tokens = Self::get_total_tokens(&env);

        (total_wastes, total_weight, total_tokens)
    }

    /// Get all incentives created by a specific rewarder/manufacturer
    /// Returns full Incentive structs for all incentives created by the rewarder
    /// Returns empty vector if rewarder has no incentives
    pub fn get_incentives_by_rewarder(env: Env, rewarder: Address) -> Vec<Incentive> {
        let key = ("rewarder_incentives", rewarder);
        let incentive_ids: Vec<u64> = env.storage().instance().get(&key).unwrap_or(Vec::new(&env));

        let mut incentives = Vec::new(&env);
        for incentive_id in incentive_ids.iter() {
            if let Some(incentive) = Self::get_incentive_internal(&env, incentive_id) {
                incentives.push_back(incentive);
            }
        }

        incentives
    }

    /// Get the best (highest reward) active incentive for a manufacturer and waste type.
    ///
    /// # Parameters
    /// - `manufacturer`: Address of the manufacturer whose incentives to search.
    /// - `waste_type`: The [`WasteType`] to match.
    ///
    /// # Returns
    /// `Some(Incentive)` with the highest `reward_points`, or `None` if no
    /// active matching incentive exists.
    pub fn get_active_mfr_incentive(
        env: Env,
        manufacturer: Address,
        waste_type: WasteType,
    ) -> Option<Incentive> {
        // Get all incentives for this manufacturer
        let key = ("rewarder_incentives", manufacturer.clone());
        let incentive_ids: Vec<u64> = env.storage().instance().get(&key).unwrap_or(Vec::new(&env));

        let mut best_incentive: Option<Incentive> = None;
        let mut highest_reward: u64 = 0;

        // Iterate through all incentives and find the best active one
        for incentive_id in incentive_ids.iter() {
            if let Some(incentive) = Self::get_incentive_internal(&env, incentive_id) {
                // Check if incentive matches criteria: active and correct waste type
                if incentive.active && incentive.waste_type == waste_type {
                    // Keep track of the incentive with highest reward
                    if incentive.reward_points > highest_reward {
                        highest_reward = incentive.reward_points;
                        best_incentive = Some(incentive);
                    }
                }
            }
        }

        best_incentive
    }

    /// Create a new manufacturer incentive program for a specific waste type.
    ///
    /// Only registered `Manufacturer` participants may create incentives.
    /// The incentive is indexed both by rewarder and by waste type for efficient lookup.
    ///
    /// # Parameters
    /// - `rewarder`: Manufacturer creating the incentive. Must sign.
    /// - `waste_type`: The [`WasteType`] this incentive applies to.
    /// - `reward_points`: Tokens awarded per kg of qualifying waste.
    /// - `total_budget`: Maximum total tokens this incentive can distribute.
    ///
    /// # Returns
    /// The newly created [`Incentive`] with `active = true`.
    ///
    /// # Errors
    /// - Panics `"Caller is not a manufacturer"`.
    ///
    /// # Example
    /// ```text
    /// create_incentive(manufacturer, WasteType::Plastic, 50, 10_000)
    /// // 50 tokens per kg, up to 10 000 tokens total
    /// ```
    pub fn create_incentive(
        env: Env,
        rewarder: Address,
        waste_type: WasteType,
        reward_points: u64,
        total_budget: u64,
    ) -> Incentive {
        // Access control check
        Self::require_not_paused(&env);
        Self::only_manufacturer(&env, &rewarder);

        // Get next incentive ID
        let incentive_id = Self::next_incentive_id(&env);

        // Create incentive
        let incentive = Incentive::new(
            incentive_id,
            rewarder.clone(),
            waste_type,
            reward_points,
            total_budget,
            env.ledger().timestamp(),
            &env,
        );

        // Store incentive
        Self::set_incentive(&env, incentive_id, &incentive);

        // Add to rewarder's incentive list
        let key = ("rewarder_incentives", rewarder.clone());
        let mut rewarder_incentives: Vec<u64> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(&env));
        rewarder_incentives.push_back(incentive_id);
        env.storage().instance().set(&key, &rewarder_incentives);

        // Add to general incentives list for this waste type
        let key = ("general_incentives", waste_type);
        let mut general_incentives: Vec<u64> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(&env));
        general_incentives.push_back(incentive_id);
        env.storage().instance().set(&key, &general_incentives);

        incentive
    }

    /// Claim reward tokens from an incentive for a verified material.
    ///
    /// Deducts the reward from `incentive.remaining_budget` and credits it to
    /// `claimer.total_tokens_earned`. Automatically deactivates the incentive
    /// if the budget reaches zero.
    ///
    /// # Parameters
    /// - `incentive_id`: ID of the incentive to claim from.
    /// - `material_id`: ID of the verified v1 material being claimed for.
    /// - `claimer`: Registered participant claiming the reward. Must sign.
    ///
    /// # Returns
    /// The reward amount as `i128`.
    ///
    /// # Errors
    /// - Panics `"Incentive not found"` / `"Incentive is not active"`.
    /// - Panics `"Material not found"` / `"Material not verified"`.
    /// - Panics `"Waste type mismatch"`.
    /// - Panics `"No reward available"` if calculated reward is zero.
    pub fn claim_incentive_reward(
        env: Env,
        incentive_id: u64,
        material_id: u64,
        claimer: Address,
    ) -> Result<i128, Error> {
        Self::require_not_paused(&env);
        Self::only_registered(&env, &claimer);

        let mut incentive =
            Self::get_incentive_internal(&env, incentive_id).expect("Incentive not found");
        if !incentive.active {
            panic!("Incentive is not active");
        }

        let material = Self::get_waste_internal(&env, material_id).expect("Material not found");
        if !material.verified {
            panic!("Material not verified");
        }
        if material.waste_type != incentive.waste_type {
            panic!("Waste type mismatch");
        }

        let reward = Self::calculate_incentive_reward(env.clone(), incentive_id, material.weight);
        if reward == 0 {
            panic!("No reward available");
        }
        if reward > incentive.remaining_budget {
            return Err(Error::InsufficientBudget);
        }

        incentive.remaining_budget = incentive.remaining_budget.saturating_sub(reward);
        if incentive.remaining_budget == 0 {
            incentive.active = false;
        }
        Self::set_incentive(&env, incentive_id, &incentive);

        Self::update_participant_stats(&env, &claimer, 0, reward);

        Ok(reward as i128)
    }

    /// Deactivate an incentive permanently.
    ///
    /// Only the original creator (`rewarder`) may deactivate their incentive.
    ///
    /// # Parameters
    /// - `incentive_id`: ID of the incentive to deactivate.
    /// - `rewarder`: Original creator. Must sign and be registered.
    ///
    /// # Returns
    /// The updated [`Incentive`] with `active = false`.
    ///
    /// # Errors
    /// - Panics `"Incentive not found"`.
    /// - Panics `"Only incentive creator can deactivate"`.
    pub fn deactivate_incentive(env: Env, incentive_id: u64, rewarder: Address) -> Incentive {
        Self::require_not_paused(&env);
        rewarder.require_auth();
        Self::require_registered(&env, &rewarder);

        let mut incentive =
            Self::get_incentive_internal(&env, incentive_id).expect("Incentive not found");

        // Verify caller is the creator
        if incentive.rewarder != rewarder {
            panic!("Only incentive creator can deactivate");
        }

        incentive.deactivate();
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }

    // ========== Waste Quality Grading ==========

    /// Assign a quality grade to a v2 waste item.
    /// Only registered `Collector` or `Manufacturer` participants may grade.
    pub fn set_waste_grade(
        env: Env,
        waste_id: u128,
        grade: WasteGrade,
        grader: Address,
    ) -> types::Waste {
        Self::require_not_paused(&env);
        grader.require_auth();
        Self::require_registered(&env, &grader);

        let grader_key = (grader.clone(),);
        let grader_participant: Participant = env
            .storage()
            .instance()
            .get(&grader_key)
            .expect("Caller is not a registered participant");

        if grader_participant.role == ParticipantRole::Recycler {
            panic!("Only collectors or manufacturers can grade waste");
        }

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if !waste.is_active {
            panic!("Waste is deactivated");
        }

        waste.grade = grade;
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        let history_key = ("grade_history", waste_id);
        let mut history: Vec<types::GradeRecord> = env
            .storage()
            .instance()
            .get(&history_key)
            .unwrap_or(Vec::new(&env));
        history.push_back(types::GradeRecord {
            waste_id,
            grade,
            grader: grader.clone(),
            graded_at: env.ledger().timestamp(),
        });
        env.storage().instance().set(&history_key, &history);

        let stats_key = ("stats", grader.clone());
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&stats_key)
            .unwrap_or_else(|| RecyclingStats::new(grader.clone()));
        stats.record_grade(grade);
        env.storage().instance().set(&stats_key, &stats);

        events::emit_waste_graded(&env, waste_id, grade, &grader);

        waste
    }

    /// Get the full grade history for a waste item.
    pub fn get_grade_history(env: Env, waste_id: u128) -> Vec<types::GradeRecord> {
        env.storage()
            .instance()
            .get(&("grade_history", waste_id))
            .unwrap_or(Vec::new(&env))
    }

    /// Get all active v2 waste IDs with a specific grade.
    pub fn get_wastes_by_grade(env: Env, grade: WasteGrade) -> Vec<u128> {
        let mut result = Vec::new(&env);
        let count = Self::get_waste_count(&env);
        for waste_id in 1..=count {
            if let Some(waste) = env
                .storage()
                .instance()
                .get::<_, types::Waste>(&("waste_v2", waste_id as u128))
            {
                if waste.is_active && waste.grade == grade {
                    result.push_back(waste_id as u128);
                }
            }
        }
        result
    }

    /// Apply the grade multiplier to a base reward: `base * grade.multiplier_pct() / 100`.
    pub fn apply_grade_multiplier(base_reward: u64, grade: WasteGrade) -> u64 {
        base_reward * grade.multiplier_pct() / 100
    }

    // ========== Waste Category Tags ==========

    const MAX_TAGS: u32 = 10;
    const MAX_TAG_LEN: u32 = 20;

    /// Normalise a tag: validate length, lowercase ASCII bytes, return new String.
    fn normalise_tag(env: &Env, tag: &String) -> String {
        let len = tag.len();
        if len == 0 {
            panic!("Tag cannot be empty");
        }
        if len > Self::MAX_TAG_LEN {
            panic!("Tag exceeds maximum length of 20 characters");
        }
        let mut buf = [0u8; 20];
        tag.copy_into_slice(&mut buf[..len as usize]);
        for b in buf[..len as usize].iter_mut() {
            if *b >= b'A' && *b <= b'Z' {
                *b += 32;
            }
        }
        String::from_bytes(env, &buf[..len as usize])
    }

    /// Add a tag to a v2 waste item. Tags are normalised to lowercase. Duplicates ignored.
    /// Max 10 tags; max 20 chars each. Only the current owner may add tags.
    pub fn add_waste_tag(env: Env, waste_id: u128, tag: String, caller: Address) -> types::Waste {
        Self::require_not_paused(&env);
        caller.require_auth();
        Self::require_registered(&env, &caller);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if !waste.is_active {
            panic!("Waste is deactivated");
        }
        if waste.current_owner != caller {
            panic!("Only the waste owner can add tags");
        }
        if waste.tags.len() >= Self::MAX_TAGS {
            panic!("Tag limit reached");
        }

        let normalised = Self::normalise_tag(&env, &tag);
        for existing in waste.tags.iter() {
            if existing == normalised {
                return waste;
            }
        }

        waste.tags.push_back(normalised);
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);
        waste
    }

    /// Remove a tag from a v2 waste item. Case-insensitive. No-op if absent.
    /// Only the current owner may remove tags.
    pub fn remove_waste_tag(
        env: Env,
        waste_id: u128,
        tag: String,
        caller: Address,
    ) -> types::Waste {
        Self::require_not_paused(&env);
        caller.require_auth();
        Self::require_registered(&env, &caller);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if waste.current_owner != caller {
            panic!("Only the waste owner can remove tags");
        }

        let normalised = Self::normalise_tag(&env, &tag);
        let mut new_tags: Vec<String> = Vec::new(&env);
        for existing in waste.tags.iter() {
            if existing != normalised {
                new_tags.push_back(existing);
            }
        }
        waste.tags = new_tags;
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);
        waste
    }

    /// Get all active v2 waste IDs that have a specific tag (case-insensitive).
    pub fn get_wastes_by_tag(env: Env, tag: String) -> Vec<u128> {
        let normalised = Self::normalise_tag(&env, &tag);
        let mut result = Vec::new(&env);
        let count = Self::get_waste_count(&env);
        for waste_id in 1..=count {
            if let Some(waste) = env
                .storage()
                .instance()
                .get::<_, types::Waste>(&("waste_v2", waste_id as u128))
            {
                if waste.is_active {
                    for t in waste.tags.iter() {
                        if t == normalised {
                            result.push_back(waste_id as u128);
                            break;
                        }
                    }
                }
            }
        }
        result
    }

    // ========== Waste Image / Document Hashes ==========

    const MAX_DOCUMENT_HASHES: u32 = 5;

    /// Validate that `hash` is a plausible IPFS CID (starts with "Qm" or "bafy").
    fn validate_ipfs_hash(env: &Env, hash: &String) {
        let len = hash.len() as usize;
        if !(4..=128).contains(&len) {
            panic!("Invalid IPFS hash");
        }
        let mut buf = [0u8; 128];
        hash.copy_into_slice(&mut buf[..len]);
        let starts_qm = buf[0] == b'Q' && buf[1] == b'm';
        let starts_bafy = buf[0] == b'b' && buf[1] == b'a' && buf[2] == b'f' && buf[3] == b'y';
        if !starts_qm && !starts_bafy {
            panic!("Invalid IPFS hash");
        }
        let _ = env;
    }

    /// Set (or replace) the primary image hash for a v2 waste item. Owner only.
    pub fn set_waste_image(
        env: Env,
        waste_id: u128,
        hash: String,
        caller: Address,
    ) -> types::Waste {
        Self::require_not_paused(&env);
        caller.require_auth();
        Self::require_registered(&env, &caller);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if !waste.is_active {
            panic!("Waste is deactivated");
        }
        if waste.current_owner != caller {
            panic!("Only the waste owner can set image hash");
        }

        Self::validate_ipfs_hash(&env, &hash);
        waste.image_hash = Some(hash);
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);
        waste
    }

    /// Add a document hash to a v2 waste item (max 5). Owner only.
    pub fn add_waste_document(
        env: Env,
        waste_id: u128,
        hash: String,
        caller: Address,
    ) -> types::Waste {
        Self::require_not_paused(&env);
        caller.require_auth();
        Self::require_registered(&env, &caller);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if !waste.is_active {
            panic!("Waste is deactivated");
        }
        if waste.current_owner != caller {
            panic!("Only the waste owner can add document hashes");
        }
        if waste.document_hashes.len() >= Self::MAX_DOCUMENT_HASHES {
            panic!("Document hash limit reached");
        }

        Self::validate_ipfs_hash(&env, &hash);
        waste.document_hashes.push_back(hash);
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);
        waste
    }

    // ========== Multi-Signature Admin Operations ==========

    /// Set the number of admin approvals required to execute a proposal (1 ≤ threshold ≤ admin count).
    pub fn set_multisig_threshold(env: Env, admin: Address, threshold: u32) {
        Self::require_admin(&env, &admin);
        let admins: Vec<Address> = env
            .storage()
            .instance()
            .get(&ADMINS)
            .expect("Admin not set");
        if threshold == 0 || threshold > admins.len() {
            panic!("Threshold must be between 1 and the number of admins");
        }
        env.storage()
            .instance()
            .set(&MULTISIG_THRESHOLD, &threshold);
    }

    /// Get the current multi-sig threshold (defaults to 1).
    pub fn get_multisig_threshold(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&MULTISIG_THRESHOLD)
            .unwrap_or(1)
    }

    fn next_proposal_id(env: &Env) -> u64 {
        let id: u64 = env.storage().instance().get(&PROPOSAL_COUNT).unwrap_or(0) + 1;
        env.storage().instance().set(&PROPOSAL_COUNT, &id);
        id
    }

    fn get_proposal(env: &Env, proposal_id: u64) -> AdminProposal {
        env.storage()
            .instance()
            .get(&("proposal", proposal_id))
            .expect("Proposal not found")
    }

    fn save_proposal(env: &Env, proposal: &AdminProposal) {
        env.storage()
            .instance()
            .set(&("proposal", proposal.id), proposal);
    }

    /// Propose a new admin action. The proposer's approval is counted automatically.
    pub fn propose_admin_action(env: Env, proposer: Address, action: AdminAction) -> AdminProposal {
        Self::require_admin(&env, &proposer);
        Self::require_not_paused(&env);

        let id = Self::next_proposal_id(&env);
        let mut approvers = Vec::new(&env);
        approvers.push_back(proposer.clone());

        let proposal = AdminProposal {
            id,
            action,
            proposer: proposer.clone(),
            approvers,
            executed: false,
            created_at: env.ledger().timestamp(),
        };

        Self::save_proposal(&env, &proposal);
        events::emit_proposal_created(&env, id, &proposer);
        proposal
    }

    /// Approve an existing proposal. Each admin may approve at most once.
    pub fn approve_admin_proposal(env: Env, approver: Address, proposal_id: u64) -> AdminProposal {
        Self::require_admin(&env, &approver);
        Self::require_not_paused(&env);

        let mut proposal = Self::get_proposal(&env, proposal_id);

        if proposal.executed {
            panic!("Proposal already executed");
        }
        if env.ledger().timestamp() > proposal.created_at + PROPOSAL_TTL_SECS {
            panic!("Proposal expired");
        }
        if proposal.approvers.contains(&approver) {
            panic!("Already approved");
        }

        proposal.approvers.push_back(approver.clone());
        Self::save_proposal(&env, &proposal);
        events::emit_proposal_approved(&env, proposal_id, &approver);
        proposal
    }

    /// Execute a proposal once the approval threshold is met.
    pub fn execute_admin_proposal(env: Env, executor: Address, proposal_id: u64) -> AdminProposal {
        Self::require_admin(&env, &executor);
        Self::require_not_paused(&env);

        let mut proposal = Self::get_proposal(&env, proposal_id);

        if proposal.executed {
            panic!("Proposal already executed");
        }
        if env.ledger().timestamp() > proposal.created_at + PROPOSAL_TTL_SECS {
            panic!("Proposal expired");
        }

        let threshold = Self::get_multisig_threshold(env.clone());
        if proposal.approvers.len() < threshold {
            panic!("Insufficient approvals");
        }

        match proposal.action.clone() {
            AdminAction::TransferAdmin(new_admins) => {
                if new_admins.is_empty() {
                    panic!("Admin list cannot be empty");
                }
                env.storage().instance().set(&ADMINS, &new_admins);
            }
            AdminAction::SetPercentages(collector_pct, owner_pct) => {
                if collector_pct + owner_pct > 100 {
                    panic!("Total percentages cannot exceed 100");
                }
                env.storage().instance().set(
                    &REWARD_CFG,
                    &RewardConfig {
                        collector_percentage: collector_pct,
                        owner_percentage: owner_pct,
                    },
                );
            }
            AdminAction::DeactivateWaste(waste_id) => {
                let mut waste: types::Waste = env
                    .storage()
                    .instance()
                    .get(&("waste_v2", waste_id))
                    .expect("Waste item not found");
                if !waste.is_active {
                    panic!("Waste already deactivated");
                }
                waste.deactivate();
                env.storage()
                    .instance()
                    .set(&("waste_v2", waste_id), &waste);
            }
        }

        proposal.executed = true;
        Self::save_proposal(&env, &proposal);
        events::emit_proposal_executed(&env, proposal_id, &executor);
        proposal
    }

    /// Get a proposal by ID.
    pub fn get_admin_proposal(env: Env, proposal_id: u64) -> AdminProposal {
        Self::get_proposal(&env, proposal_id)
    }

    // ========== Global Metrics ==========

    /// Get global contract metrics (total waste count and total tokens earned)
    pub fn get_metrics(env: Env) -> types::GlobalMetrics {
        let total_wastes_count: u64 = env
            .storage()
            .instance()
            .get(&symbol_short!("MAT_CNT"))
            .unwrap_or(0);
        let total_tokens_earned: u128 = env.storage().instance().get(&TOTAL_TOKENS).unwrap_or(0);
        let total_carbon_credits: u128 = env.storage().instance().get(&TOTAL_CARBON).unwrap_or(0);
        types::GlobalMetrics {
            total_wastes_count,
            total_tokens_earned,
            total_carbon_credits,
        }
    }

    // ========== Admin Transfer ==========

    /// Pause the contract (admin only) — blocks all state-changing functions
    pub fn pause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        assert!(
            !env.storage()
                .instance()
                .get::<_, bool>(&PAUSED)
                .unwrap_or(false),
            "Contract is already paused"
        );
        env.storage().instance().set(&PAUSED, &true);
        events::emit_contract_paused(&env, &admin);
    }

    /// Unpause the contract (admin only)
    pub fn unpause(env: Env, admin: Address) {
        Self::require_admin(&env, &admin);
        assert!(
            env.storage()
                .instance()
                .get::<_, bool>(&PAUSED)
                .unwrap_or(false),
            "Contract is not paused"
        );
        env.storage().instance().set(&PAUSED, &false);
        events::emit_contract_unpaused(&env, &admin);
    }

    /// Get current pause state
    pub fn is_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get::<_, bool>(&PAUSED)
            .unwrap_or(false)
    }

    fn require_not_paused(env: &Env) {
        assert!(
            !env.storage()
                .instance()
                .get::<_, bool>(&PAUSED)
                .unwrap_or(false),
            "Contract is paused"
        );
    }

    // ========== Incentive-Based Reward Distribution ==========

    /// Distribute rewards through the supply chain for a confirmed material using an incentive.
    /// Rewards collectors in the transfer history, the original submitter, and the current owner.
    /// Returns the total reward distributed.
    pub fn distribute_rewards(
        env: Env,
        waste_id: u64,
        incentive_id: u64,
        manufacturer: Address,
    ) -> i128 {
        manufacturer.require_auth();

        let material = Self::get_waste_internal(&env, waste_id).expect("Material not found");
        assert!(material.verified, "Material must be confirmed");

        Self::require_not_paused(&env);
        let mut incentive =
            Self::get_incentive_internal(&env, incentive_id).expect("Incentive not found");
        assert!(
            incentive.rewarder == manufacturer,
            "Only incentive creator can distribute rewards"
        );
        assert!(
            incentive.waste_type == material.waste_type,
            "Waste type mismatch"
        );
        assert!(incentive.active, "Incentive not active");

        let weight_kg = material.weight / 1000;
        let base_reward = (incentive.reward_points as i128) * (weight_kg as i128);
        let multiplier = Self::get_current_multiplier(env.clone()) as i128;
        let mut total_reward = (base_reward * multiplier) / 100;

        // Apply contamination reduction if a v2 waste record exists for this ID
        let v2_waste: Option<types::Waste> = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id as u128));
        if let Some(w) = v2_waste {
            if w.is_contaminated {
                if w.contamination_level > 50 {
                    total_reward = 0;
                } else {
                    total_reward = total_reward * (100 - w.contamination_level as i128) / 100;
                }
            }
        }
        assert!(
            (total_reward as u64) <= incentive.remaining_budget,
            "Insufficient incentive budget"
        );

        let transfers = Self::get_transfer_history(env.clone(), waste_id);
        let cfg = Self::get_reward_config(&env);
        let collector_pct = cfg.collector_percentage;
        let owner_pct = cfg.owner_percentage;

        let token_address: Address = env
            .storage()
            .instance()
            .get(&TOKEN_ADDR)
            .expect("Token address not set");
        let token_client = token::Client::new(&env, &token_address);

        let collector_share = (total_reward * (collector_pct as i128)) / 100;
        let owner_share = (total_reward * (owner_pct as i128)) / 100;
        let mut total_distributed: i128 = 0;

        for transfer in transfers.iter() {
            let key = (transfer.to.clone(),);
            if let Some(p) = env.storage().instance().get::<_, Participant>(&key) {
                if p.role.can_collect_materials() && !p.role.can_manufacture() {
                    token_client.transfer(&manufacturer, &transfer.to, &collector_share);
                    Self::update_participant_stats(&env, &transfer.to, 0, collector_share as u64);
                    events::emit_tokens_rewarded(
                        &env,
                        &transfer.to,
                        collector_share as u128,
                        waste_id,
                    );
                    total_distributed += collector_share;
                }
            }
        }

        token_client.transfer(&manufacturer, &material.submitter, &owner_share);
        Self::update_participant_stats(&env, &material.submitter, 0, owner_share as u64);
        events::emit_tokens_rewarded(&env, &material.submitter, owner_share as u128, waste_id);
        total_distributed += owner_share;

        let recycler_amount = total_reward - total_distributed;
        if recycler_amount > 0 {
            token_client.transfer(&manufacturer, &material.submitter, &recycler_amount);
            Self::update_participant_stats(&env, &material.submitter, 0, recycler_amount as u64);
            events::emit_tokens_rewarded(
                &env,
                &material.submitter,
                recycler_amount as u128,
                waste_id,
            );
        }

        incentive.remaining_budget = incentive
            .remaining_budget
            .saturating_sub(total_reward as u64);
        if incentive.remaining_budget == 0 {
            incentive.active = false;
        }
        Self::set_incentive(&env, incentive_id, &incentive);
        Self::add_to_total_tokens(&env, total_reward as u128);

        total_reward
    }

    /// Split a v2 waste item into multiple smaller items.
    ///
    /// The owner provides a list of weights that must sum to the original weight.
    /// The original waste is deactivated and new child waste items are created,
    /// inheriting the type, location, and owner. Transfer history is preserved
    /// by copying the parent's history to each child. A `WasteSplit` event is emitted.
    ///
    /// # Parameters
    /// - `waste_id`: ID of the v2 waste to split.
    /// - `owner`: Current owner. Must sign and own the waste.
    /// - `weights`: Vec of weights (in grams) for each child. Must sum to original weight.
    ///
    /// # Returns
    /// `Vec<u128>` of the newly created child waste IDs.
    ///
    /// # Errors
    /// - [`Error::WasteNotFound`] if no waste record exists for `waste_id`.
    /// - [`Error::NotWasteOwner`] if `owner` does not own the waste.
    /// - [`Error::WasteDeactivated`] if the waste is already deactivated.
    /// - [`Error::TooFewSplits`] if fewer than 2 weights are provided.
    /// - [`Error::TooManySplits`] if more than 10 weights are provided.
    /// - [`Error::WeightMismatch`] if the weights do not sum to the original weight.
    pub fn split_waste(
        env: Env,
        waste_id: u128,
        owner: Address,
        weights: Vec<u128>,
    ) -> Result<Vec<u128>, Error> {
        owner.require_auth();
        Self::require_not_paused(&env);

        // Load and validate the parent waste
        let mut parent: types::Waste = match env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
        {
            Some(w) => w,
            None => return Err(Error::WasteNotFound),
        };

        if parent.current_owner != owner {
            return Err(Error::NotWasteOwner);
        }

        if !parent.is_active {
            return Err(Error::WasteDeactivated);
        }

        let n = weights.len();
        if n < 2 {
            return Err(Error::TooFewSplits);
        }
        if n > 10 {
            return Err(Error::TooManySplits);
        }

        // Validate weights sum equals parent weight
        let mut total: u128 = 0;
        for w in weights.iter() {
            total = total.checked_add(w).ok_or(Error::Overflow)?;
        }
        if total != parent.weight {
            return Err(Error::WeightMismatch);
        }

        // Load parent transfer history once
        let parent_history: Vec<WasteTransfer> = env
            .storage()
            .instance()
            .get(&("transfer_history", waste_id))
            .unwrap_or(Vec::new(&env));

        let timestamp = env.ledger().timestamp();
        let mut child_ids: Vec<u128> = Vec::new(&env);

        for w in weights.iter() {
            let child_id = Self::next_waste_id(&env) as u128;

            let child = types::Waste::new(
                &env,
                child_id,
                parent.waste_type,
                w,
                owner.clone(),
                parent.latitude,
                parent.longitude,
                timestamp,
                true,
                false,
                owner.clone(),
                0,
            );

            env.storage()
                .instance()
                .set(&("waste_v2", child_id), &child);

            // Copy parent transfer history to child
            env.storage()
                .instance()
                .set(&("transfer_history", child_id), &parent_history);

            // Add child to owner's waste list
            let mut owner_list: Vec<u128> = env
                .storage()
                .instance()
                .get(&("participant_wastes", owner.clone()))
                .unwrap_or(Vec::new(&env));
            owner_list.push_back(child_id);
            env.storage()
                .instance()
                .set(&("participant_wastes", owner.clone()), &owner_list);

            child_ids.push_back(child_id);
        }

        // Deactivate parent
        parent.deactivate();
        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &parent);

        // Remove parent from owner's waste list
        let owner_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", owner.clone()))
            .unwrap_or(Vec::new(&env));
        let mut new_owner_list = Vec::new(&env);
        for id in owner_list.iter() {
            if id != waste_id {
                new_owner_list.push_back(id);
            }
        }
        env.storage()
            .instance()
            .set(&("participant_wastes", owner.clone()), &new_owner_list);

        events::emit_waste_split(&env, waste_id, &owner, &child_ids);

        Ok(child_ids)
    }

    /// Merge multiple v2 waste items of the same type into a single new item.
    ///
    /// All source wastes must be owned by `owner`, active, share the same
    /// [`WasteType`], and be at the same location. They are deactivated and a
    /// new waste with the combined weight is created. Transfer histories are
    /// aggregated (deduplicated by `waste_id`) into the merged item. A
    /// `WastesMerged` event is emitted.
    ///
    /// # Parameters
    /// - `waste_ids`: IDs of the v2 wastes to merge (2–20).
    /// - `owner`: Current owner of all wastes. Must sign.
    ///
    /// # Returns
    /// The new merged waste ID (`u128`).
    ///
    /// # Errors
    /// - [`Error::TooFewWastes`] if fewer than 2 IDs are provided.
    /// - [`Error::TooManyWastes`] if more than 20 IDs are provided.
    /// - [`Error::WasteNotFound`] if any ID does not exist.
    /// - [`Error::NotWasteOwner`] if `owner` does not own every waste.
    /// - [`Error::WasteDeactivated`] if any waste is already deactivated.
    /// - [`Error::WasteTypeMismatchMerge`] if wastes have different types.
    /// - [`Error::LocationMismatch`] if wastes are at different locations.
    pub fn merge_wastes(
        env: Env,
        waste_ids: Vec<u128>,
        owner: Address,
    ) -> Result<u128, Error> {
        owner.require_auth();
        Self::require_not_paused(&env);

        let n = waste_ids.len();
        if n < 2 {
            return Err(Error::TooFewWastes);
        }
        if n > 20 {
            return Err(Error::TooManyWastes);
        }

        // Validate all wastes and accumulate combined weight
        let mut combined_weight: u128 = 0;
        let mut ref_type: Option<types::WasteType> = None;
        let mut ref_lat: Option<i128> = None;
        let mut ref_lon: Option<i128> = None;

        for waste_id in waste_ids.iter() {
            let waste: types::Waste = env
                .storage()
                .instance()
                .get(&("waste_v2", waste_id))
                .ok_or(Error::WasteNotFound)?;

            if waste.current_owner != owner {
                return Err(Error::NotWasteOwner);
            }
            if !waste.is_active {
                return Err(Error::WasteDeactivated);
            }

            match ref_type {
                None => ref_type = Some(waste.waste_type),
                Some(t) if t != waste.waste_type => return Err(Error::WasteTypeMismatchMerge),
                _ => {}
            }

            match (ref_lat, ref_lon) {
                (None, None) => {
                    ref_lat = Some(waste.latitude);
                    ref_lon = Some(waste.longitude);
                }
                (Some(lat), Some(lon)) if lat != waste.latitude || lon != waste.longitude => {
                    return Err(Error::LocationMismatch);
                }
                _ => {}
            }

            combined_weight = combined_weight
                .checked_add(waste.weight)
                .ok_or(Error::Overflow)?;
        }

        let waste_type = ref_type.unwrap();
        let latitude = ref_lat.unwrap();
        let longitude = ref_lon.unwrap();
        let timestamp = env.ledger().timestamp();

        // Create merged waste
        let merged_id = Self::next_waste_id(&env) as u128;
        let merged = types::Waste::new(
            &env,
            merged_id,
            waste_type,
            combined_weight,
            owner.clone(),
            latitude,
            longitude,
            timestamp,
            true,
            false,
            owner.clone(),
            0,
        );
        env.storage()
            .instance()
            .set(&("waste_v2", merged_id), &merged);

        // Aggregate transfer histories (append all source histories)
        let mut merged_history: Vec<WasteTransfer> = Vec::new(&env);
        for waste_id in waste_ids.iter() {
            let history: Vec<WasteTransfer> = env
                .storage()
                .instance()
                .get(&("transfer_history", waste_id))
                .unwrap_or(Vec::new(&env));
            for record in history.iter() {
                merged_history.push_back(record);
            }
        }
        env.storage()
            .instance()
            .set(&("transfer_history", merged_id), &merged_history);

        // Update owner waste list: remove sources, add merged
        let owner_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", owner.clone()))
            .unwrap_or(Vec::new(&env));
        let mut new_owner_list = Vec::new(&env);
        for id in owner_list.iter() {
            if !waste_ids.contains(&id) {
                new_owner_list.push_back(id);
            }
        }
        new_owner_list.push_back(merged_id);
        env.storage()
            .instance()
            .set(&("participant_wastes", owner.clone()), &new_owner_list);

        // Deactivate all source wastes
        for waste_id in waste_ids.iter() {
            let mut waste: types::Waste = env
                .storage()
                .instance()
                .get(&("waste_v2", waste_id))
                .unwrap();
            waste.deactivate();
            env.storage()
                .instance()
                .set(&("waste_v2", waste_id), &waste);
        }

        events::emit_wastes_merged(&env, merged_id, &owner, &waste_ids);

        Ok(merged_id)
    }

    /// Reserve a v2 waste item for a limited time.
    ///
    /// Only registered participants may reserve. The waste must be active and
    /// not already reserved (or the previous reservation must have expired).
    /// Emits a `WasteReserved` event.
    ///
    /// # Parameters
    /// - `waste_id`: ID of the v2 waste to reserve.
    /// - `reserver`: Registered participant making the reservation. Must sign.
    /// - `duration`: Reservation duration in seconds.
    ///
    /// # Errors
    /// - [`Error::WasteNotFound`] if the waste does not exist.
    /// - [`Error::WasteDeactivated`] if the waste is inactive.
    /// - [`Error::WasteAlreadyReserved`] if an active reservation exists.
    pub fn reserve_waste(
        env: Env,
        waste_id: u128,
        reserver: Address,
        duration: u64,
    ) -> Result<types::Waste, Error> {
        reserver.require_auth();
        Self::require_not_paused(&env);
        Self::require_registered(&env, &reserver);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .ok_or(Error::WasteNotFound)?;

        if !waste.is_active {
            return Err(Error::WasteDeactivated);
        }

        let now = env.ledger().timestamp();

        // Block if an active (non-expired) reservation exists
        if let (Some(_), Some(until)) = (waste.reserved_by.clone(), waste.reserved_until) {
            if until > now {
                return Err(Error::WasteAlreadyReserved);
            }
        }

        let reserved_until = now.checked_add(duration).ok_or(Error::Overflow)?;
        waste.reserved_by = Some(reserver.clone());
        waste.reserved_until = Some(reserved_until);

        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        events::emit_waste_reserved(&env, waste_id, &reserver, reserved_until);

        Ok(waste)
    }

    /// Cancel a reservation on a v2 waste item.
    ///
    /// The reserver themselves or the waste owner may cancel. Emits a
    /// `ReservationCancelled` event.
    ///
    /// # Parameters
    /// - `waste_id`: ID of the v2 waste.
    /// - `caller`: Reserver or current owner. Must sign.
    ///
    /// # Errors
    /// - [`Error::WasteNotFound`] if the waste does not exist.
    /// - [`Error::WasteNotReserved`] if no reservation exists.
    /// - [`Error::NotReserver`] if caller is neither the reserver nor the owner.
    pub fn cancel_reservation(
        env: Env,
        waste_id: u128,
        caller: Address,
    ) -> Result<types::Waste, Error> {
        caller.require_auth();
        Self::require_not_paused(&env);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .ok_or(Error::WasteNotFound)?;

        if waste.reserved_by.is_none() {
            return Err(Error::WasteNotReserved);
        }

        let reserver = waste.reserved_by.clone().unwrap();
        if caller != reserver && caller != waste.current_owner {
            return Err(Error::NotReserver);
        }

        waste.reserved_by = None;
        waste.reserved_until = None;

        env.storage()
            .instance()
            .set(&("waste_v2", waste_id), &waste);

        events::emit_reservation_cancelled(&env, waste_id, &caller);

        Ok(waste)
    }

    // ========== Challenge Functions ==========

    /// Create a new time-limited challenge (admin only). Max 10 active challenges.
    pub fn create_challenge(
        env: Env,
        admin: Address,
        title: soroban_sdk::Symbol,
        target_weight: u128,
        waste_type: WasteType,
        start_time: u64,
        end_time: u64,
        reward: u128,
    ) -> Challenge {
        Self::only_admin(&env, &admin);
        Self::require_not_paused(&env);

        if target_weight == 0 {
            panic!("Target weight must be > 0");
        }
        if start_time >= end_time {
            panic!("start_time must be before end_time");
        }
        if reward == 0 {
            panic!("Reward must be > 0");
        }

        // Enforce max 10 active challenges
        let active_count = Self::count_active_challenges(&env);
        if active_count >= MAX_ACTIVE_CHALLENGES {
            panic!("Max active challenges reached");
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&CHALLENGE_COUNT)
            .unwrap_or(0u64)
            + 1;
        env.storage().instance().set(&CHALLENGE_COUNT, &id);

        let challenge = Challenge {
            id,
            title: title.clone(),
            target_weight,
            waste_type,
            start_time,
            end_time,
            reward,
            status: ChallengeStatus::Active,
            creator: admin.clone(),
        };

        env.storage()
            .instance()
            .set(&("challenge", id), &challenge);

        env.events().publish(
            (symbol_short!("chal_new"), id),
            (title, target_weight, waste_type, start_time, end_time, reward),
        );

        challenge
    }

    fn count_active_challenges(env: &Env) -> u32 {
        let total: u64 = env
            .storage()
            .instance()
            .get(&CHALLENGE_COUNT)
            .unwrap_or(0);
        let now = env.ledger().timestamp();
        let mut count = 0u32;
        for id in 1..=total {
            if let Some(c) = env
                .storage()
                .instance()
                .get::<_, Challenge>(&("challenge", id))
            {
                if c.status == ChallengeStatus::Active && now < c.end_time {
                    count += 1;
                }
            }
        }
        count
    }

    /// Join a challenge as a registered participant.
    pub fn join_challenge(env: Env, challenge_id: u64, participant: Address) {
        Self::require_not_paused(&env);
        Self::only_registered(&env, &participant);

        let challenge: Challenge = env
            .storage()
            .instance()
            .get(&("challenge", challenge_id))
            .expect("Challenge not found");

        let now = env.ledger().timestamp();
        if challenge.status != ChallengeStatus::Active || now >= challenge.end_time {
            panic!("Challenge is not active");
        }
        if now < challenge.start_time {
            panic!("Challenge has not started yet");
        }

        let prog_key = ("chal_prog", challenge_id, participant.clone());
        if env
            .storage()
            .instance()
            .has(&prog_key)
        {
            panic!("Already joined this challenge");
        }

        let progress = ChallengeProgress {
            challenge_id,
            participant: participant.clone(),
            weight_processed: 0,
            completed: false,
            joined_at: now,
        };
        env.storage().instance().set(&prog_key, &progress);

        env.events().publish(
            (symbol_short!("chal_join"), challenge_id),
            participant,
        );
    }

    /// Get a participant's progress in a challenge.
    pub fn get_challenge_progress(
        env: Env,
        challenge_id: u64,
        participant: Address,
    ) -> ChallengeProgress {
        env.storage()
            .instance()
            .get(&("chal_prog", challenge_id, participant))
            .expect("Not joined this challenge")
    }

    /// Record waste weight toward a challenge and auto-complete if target met.
    /// Called internally after waste is processed; also callable directly.
    pub fn update_challenge_progress(
        env: Env,
        challenge_id: u64,
        participant: Address,
        weight_grams: u128,
    ) {
        Self::require_not_paused(&env);
        participant.require_auth();

        let challenge: Challenge = env
            .storage()
            .instance()
            .get(&("challenge", challenge_id))
            .expect("Challenge not found");

        let now = env.ledger().timestamp();
        if challenge.status != ChallengeStatus::Active || now >= challenge.end_time {
            panic!("Challenge is not active");
        }

        let prog_key = ("chal_prog", challenge_id, participant.clone());
        let mut progress: ChallengeProgress = env
            .storage()
            .instance()
            .get(&prog_key)
            .expect("Not joined this challenge");

        if progress.completed {
            return;
        }

        progress.weight_processed = progress
            .weight_processed
            .checked_add(weight_grams)
            .expect("Overflow");

        if progress.weight_processed >= challenge.target_weight {
            progress.completed = true;
            env.storage().instance().set(&prog_key, &progress);
            Self::complete_challenge_internal(&env, challenge_id, &participant, &challenge);
        } else {
            env.storage().instance().set(&prog_key, &progress);
        }
    }

    fn complete_challenge_internal(
        env: &Env,
        challenge_id: u64,
        participant: &Address,
        challenge: &Challenge,
    ) {
        // Award bonus tokens to participant
        let key = (participant.clone(),);
        if let Some(mut p) = env.storage().instance().get::<_, Participant>(&key) {
            p.total_tokens_earned = p
                .total_tokens_earned
                .checked_add(challenge.reward)
                .expect("Overflow");
            env.storage().instance().set(&key, &p);
        }

        env.events().publish(
            (symbol_short!("chal_done"), challenge_id),
            (participant, challenge.reward),
        );
    }

    /// Manually complete a challenge for a participant (checks target met).
    pub fn complete_challenge(env: Env, challenge_id: u64, participant: Address) {
        Self::require_not_paused(&env);
        participant.require_auth();

        let challenge: Challenge = env
            .storage()
            .instance()
            .get(&("challenge", challenge_id))
            .expect("Challenge not found");

        let prog_key = ("chal_prog", challenge_id, participant.clone());
        let mut progress: ChallengeProgress = env
            .storage()
            .instance()
            .get(&prog_key)
            .expect("Not joined this challenge");

        if progress.completed {
            panic!("Challenge already completed");
        }
        if progress.weight_processed < challenge.target_weight {
            panic!("Target not yet reached");
        }

        progress.completed = true;
        env.storage().instance().set(&prog_key, &progress);
        Self::complete_challenge_internal(&env, challenge_id, &participant, &challenge);
    }

    /// Get all currently active challenges.
    pub fn get_active_challenges(env: Env) -> Vec<Challenge> {
        let total: u64 = env
            .storage()
            .instance()
            .get(&CHALLENGE_COUNT)
            .unwrap_or(0);
        let now = env.ledger().timestamp();
        let mut result = Vec::new(&env);
        for id in 1..=total {
            if let Some(c) = env
                .storage()
                .instance()
                .get::<_, Challenge>(&("challenge", id))
            {
                if c.status == ChallengeStatus::Active && now < c.end_time && now >= c.start_time {
                    result.push_back(c);
                }
            }
        }
        result
    }

    /// Get a challenge by ID.
    pub fn get_challenge(env: Env, challenge_id: u64) -> Option<Challenge> {
        env.storage()
            .instance()
            .get(&("challenge", challenge_id))
    }

    // ========== Transfer Approval Workflow Functions ==========

    /// Initiate a transfer that requires recipient approval.
    pub fn initiate_transfer(
        env: Env,
        waste_id: u128,
        from: Address,
        to: Address,
        latitude: i128,
        longitude: i128,
    ) -> PendingTransfer {
        Self::require_not_paused(&env);
        Self::only_registered(&env, &from);
        Self::require_registered(&env, &to);
        Self::require_addresses_different(&from, &to);

        let waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if waste.current_owner != from {
            panic!("Caller is not the owner of this waste item");
        }
        if !waste.is_active {
            panic!("Waste is deactivated");
        }

        let id: u64 = env
            .storage()
            .instance()
            .get(&PENDING_XFR_CNT)
            .unwrap_or(0u64)
            + 1;
        env.storage().instance().set(&PENDING_XFR_CNT, &id);

        let now = env.ledger().timestamp();
        let pending = PendingTransfer {
            id,
            waste_id,
            from: from.clone(),
            to: to.clone(),
            initiated_at: now,
            expires_at: now + TRANSFER_EXPIRY_SECS,
            status: PendingTransferStatus::Pending,
            latitude,
            longitude,
        };

        env.storage()
            .instance()
            .set(&("pending_xfr", id), &pending);

        env.events().publish(
            (symbol_short!("xfr_init"), id),
            (waste_id, from, to),
        );

        pending
    }

    /// Approve a pending transfer (recipient only).
    pub fn approve_transfer(env: Env, transfer_id: u64, recipient: Address) -> PendingTransfer {
        Self::require_not_paused(&env);
        recipient.require_auth();

        let mut pending: PendingTransfer = env
            .storage()
            .instance()
            .get(&("pending_xfr", transfer_id))
            .expect("Pending transfer not found");

        if pending.to != recipient {
            panic!("Only the recipient can approve");
        }
        if pending.status != PendingTransferStatus::Pending {
            panic!("Transfer is not pending");
        }

        let now = env.ledger().timestamp();
        if now >= pending.expires_at {
            pending.status = PendingTransferStatus::Expired;
            env.storage()
                .instance()
                .set(&("pending_xfr", transfer_id), &pending);
            panic!("Transfer has expired");
        }

        // Execute the actual waste transfer
        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", pending.waste_id))
            .expect("Waste not found");

        if !waste.is_active {
            panic!("Waste is deactivated");
        }

        // Update waste lists
        let from_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", pending.from.clone()))
            .unwrap_or(Vec::new(&env));
        let mut new_from_list = Vec::new(&env);
        for id in from_list.iter() {
            if id != pending.waste_id {
                new_from_list.push_back(id);
            }
        }
        env.storage()
            .instance()
            .set(&("participant_wastes", pending.from.clone()), &new_from_list);

        let mut to_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", pending.to.clone()))
            .unwrap_or(Vec::new(&env));
        to_list.push_back(pending.waste_id);
        env.storage()
            .instance()
            .set(&("participant_wastes", pending.to.clone()), &to_list);

        waste.transfer_to(pending.to.clone());
        env.storage()
            .instance()
            .set(&("waste_v2", pending.waste_id), &waste);

        // Record transfer history
        let transfer_record = WasteTransfer::new(
            pending.waste_id,
            pending.from.clone(),
            pending.to.clone(),
            now,
            pending.latitude,
            pending.longitude,
            symbol_short!("approved"),
        );
        let mut history: Vec<WasteTransfer> = env
            .storage()
            .instance()
            .get(&("transfer_history", pending.waste_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(transfer_record);
        env.storage()
            .instance()
            .set(&("transfer_history", pending.waste_id), &history);

        pending.status = PendingTransferStatus::Approved;
        env.storage()
            .instance()
            .set(&("pending_xfr", transfer_id), &pending);

        env.events().publish(
            (symbol_short!("xfr_appr"), transfer_id),
            (pending.waste_id, pending.from.clone(), pending.to.clone()),
        );

        pending
    }

    /// Reject a pending transfer (recipient only).
    pub fn reject_transfer(env: Env, transfer_id: u64, recipient: Address) -> PendingTransfer {
        Self::require_not_paused(&env);
        recipient.require_auth();

        let mut pending: PendingTransfer = env
            .storage()
            .instance()
            .get(&("pending_xfr", transfer_id))
            .expect("Pending transfer not found");

        if pending.to != recipient {
            panic!("Only the recipient can reject");
        }
        if pending.status != PendingTransferStatus::Pending {
            panic!("Transfer is not pending");
        }

        let now = env.ledger().timestamp();
        if now >= pending.expires_at {
            pending.status = PendingTransferStatus::Expired;
        } else {
            pending.status = PendingTransferStatus::Rejected;
        }

        env.storage()
            .instance()
            .set(&("pending_xfr", transfer_id), &pending);

        env.events().publish(
            (symbol_short!("xfr_rej"), transfer_id),
            (pending.waste_id, pending.from.clone(), pending.to.clone()),
        );

        pending
    }

    /// Get a pending transfer by ID.
    pub fn get_pending_transfer(env: Env, transfer_id: u64) -> Option<PendingTransfer> {
        env.storage()
            .instance()
            .get(&("pending_xfr", transfer_id))
    }

    /// Expire a pending transfer if past its deadline (callable by anyone).
    pub fn expire_transfer(env: Env, transfer_id: u64) -> PendingTransfer {
        let mut pending: PendingTransfer = env
            .storage()
            .instance()
            .get(&("pending_xfr", transfer_id))
            .expect("Pending transfer not found");

        if pending.status != PendingTransferStatus::Pending {
            panic!("Transfer is not pending");
        }

        let now = env.ledger().timestamp();
        if now < pending.expires_at {
            panic!("Transfer has not expired yet");
        }

        pending.status = PendingTransferStatus::Expired;
        env.storage()
            .instance()
            .set(&("pending_xfr", transfer_id), &pending);

        env.events().publish(
            (symbol_short!("xfr_exp"), transfer_id),
            pending.waste_id,
        );

        pending
    }

    // ========== Milestone Functions ==========

    /// Get the predefined milestones list.
    pub fn get_milestones(env: Env) -> Vec<Milestone> {
        let titles = [
            symbol_short!("100kg"),
            symbol_short!("500kg"),
            symbol_short!("1000kg"),
            symbol_short!("5000kg"),
            symbol_short!("10000kg"),
            symbol_short!("50000kg"),
            symbol_short!("100000kg"),
        ];
        let mut result = Vec::new(&env);
        for (i, &threshold) in MILESTONE_THRESHOLDS.iter().enumerate() {
            result.push_back(Milestone {
                threshold,
                title: titles[i].clone(),
                bonus_pct: 10,
            });
        }
        result
    }

    /// Get the milestone indices already achieved by a participant.
    pub fn get_participant_milestones(env: Env, participant: Address) -> Vec<u32> {
        env.storage()
            .instance()
            .get(&("milestones", participant))
            .unwrap_or(Vec::new(&env))
    }

    /// Check and award any newly reached milestones for a participant.
    /// Called internally after waste is processed.
    fn check_and_award_milestones(env: &Env, participant: &Address) {
        let key = (participant.clone(),);
        let p: Participant = match env.storage().instance().get::<_, Participant>(&key) {
            Some(p) => p,
            None => return,
        };

        let achieved_key = ("milestones", participant.clone());
        let mut achieved: Vec<u32> = env
            .storage()
            .instance()
            .get(&achieved_key)
            .unwrap_or(Vec::new(env));

        let titles = [
            symbol_short!("100kg"),
            symbol_short!("500kg"),
            symbol_short!("1000kg"),
            symbol_short!("5000kg"),
            symbol_short!("10000kg"),
            symbol_short!("50000kg"),
            symbol_short!("100000kg"),
        ];

        let mut updated = false;
        for (i, &threshold) in MILESTONE_THRESHOLDS.iter().enumerate() {
            let idx = i as u32;
            if achieved.contains(&idx) {
                continue;
            }
            if p.total_waste_processed >= threshold {
                achieved.push_back(idx);
                updated = true;

                // Award 10% bonus of total_tokens_earned
                let bonus = p.total_tokens_earned / 10;
                if bonus > 0 {
                    let mut p2: Participant = env
                        .storage()
                        .instance()
                        .get::<_, Participant>(&key)
                        .unwrap();
                    p2.total_tokens_earned = p2
                        .total_tokens_earned
                        .checked_add(bonus)
                        .expect("Overflow");
                    env.storage().instance().set(&key, &p2);

                    let total: u128 =
                        env.storage().instance().get(&TOTAL_TOKENS).unwrap_or(0);
                    env.storage()
                        .instance()
                        .set(&TOTAL_TOKENS, &(total + bonus));
                }

                env.events().publish(
                    (symbol_short!("milestone"), idx),
                    (participant, threshold, titles[i].clone()),
                );
            }
        }

        if updated {
            env.storage().instance().set(&achieved_key, &achieved);
        }
    }

    // ========== Leaderboard Functions ==========

    /// Get top N participants by total waste processed (recyclers).
    pub fn get_top_recyclers(env: Env, limit: u32) -> Vec<LeaderboardEntry> {
        let limit = limit.min(100) as usize;
        let index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        let mut entries: Vec<(u128, Address)> = Vec::new(&env);
        for addr in index.iter() {
            let key = (addr.clone(),);
            if let Some(p) = env.storage().instance().get::<_, Participant>(&key) {
                if p.is_registered && p.role == ParticipantRole::Recycler {
                    entries.push_back((p.total_waste_processed, addr));
                }
            }
        }

        Self::sort_and_rank(&env, entries, limit)
    }

    /// Get top N participants by number of waste items collected (collectors).
    pub fn get_top_collectors(env: Env, limit: u32) -> Vec<LeaderboardEntry> {
        let limit = limit.min(100) as usize;
        let index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        let mut entries: Vec<(u128, Address)> = Vec::new(&env);
        for addr in index.iter() {
            let key = (addr.clone(),);
            if let Some(p) = env.storage().instance().get::<_, Participant>(&key) {
                if p.is_registered && p.role == ParticipantRole::Collector {
                    entries.push_back((p.total_waste_processed, addr));
                }
            }
        }

        Self::sort_and_rank(&env, entries, limit)
    }

    /// Get top N participants by verified submissions (verifiers = recyclers with most verifications).
    pub fn get_top_verifiers(env: Env, limit: u32) -> Vec<LeaderboardEntry> {
        let limit = limit.min(100) as usize;
        let index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        let mut entries: Vec<(u128, Address)> = Vec::new(&env);
        for addr in index.iter() {
            let stats_key = ("stats", addr.clone());
            if let Some(stats) = env
                .storage()
                .instance()
                .get::<_, RecyclingStats>(&stats_key)
            {
                entries.push_back((stats.verified_submissions as u128, addr));
            }
        }

        Self::sort_and_rank(&env, entries, limit)
    }

    /// Get top N participants by total tokens earned.
    pub fn get_top_earners(env: Env, limit: u32) -> Vec<LeaderboardEntry> {
        let limit = limit.min(100) as usize;
        let index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        let mut entries: Vec<(u128, Address)> = Vec::new(&env);
        for addr in index.iter() {
            let key = (addr.clone(),);
            if let Some(p) = env.storage().instance().get::<_, Participant>(&key) {
                if p.is_registered {
                    entries.push_back((p.total_tokens_earned, addr));
                }
            }
        }

        Self::sort_and_rank(&env, entries, limit)
    }

    /// Get a participant's rank by a given metric: "weight", "tokens", "verified".
    pub fn get_participant_rank(env: Env, participant: Address, metric: soroban_sdk::Symbol) -> u32 {
        let index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));

        let weight_sym = symbol_short!("weight");
        let tokens_sym = symbol_short!("tokens");
        let verified_sym = symbol_short!("verified");

        let get_score = |addr: &Address| -> u128 {
            if metric == weight_sym {
                let key = (addr.clone(),);
                env.storage()
                    .instance()
                    .get::<_, Participant>(&key)
                    .map(|p| p.total_waste_processed)
                    .unwrap_or(0)
            } else if metric == tokens_sym {
                let key = (addr.clone(),);
                env.storage()
                    .instance()
                    .get::<_, Participant>(&key)
                    .map(|p| p.total_tokens_earned)
                    .unwrap_or(0)
            } else if metric == verified_sym {
                let stats_key = ("stats", addr.clone());
                env.storage()
                    .instance()
                    .get::<_, RecyclingStats>(&stats_key)
                    .map(|s| s.verified_submissions as u128)
                    .unwrap_or(0)
            } else {
                0
            }
        };

        let my_score = get_score(&participant);
        let mut rank = 1u32;
        for addr in index.iter() {
            if addr == participant {
                continue;
            }
            if get_score(&addr) > my_score {
                rank += 1;
            }
        }
        rank
    }

    /// Sort entries descending by score and return top `limit` as LeaderboardEntry.
    fn sort_and_rank(
        env: &Env,
        mut entries: Vec<(u128, Address)>,
        limit: usize,
    ) -> Vec<LeaderboardEntry> {
        // Simple insertion sort (descending by score)
        let n = entries.len() as usize;
        for i in 1..n {
            let mut j = i;
            while j > 0 {
                let a = entries.get(j as u32).unwrap();
                let b = entries.get((j - 1) as u32).unwrap();
                if a.0 > b.0 {
                    entries.set(j as u32, b);
                    entries.set((j - 1) as u32, a);
                    j -= 1;
                } else {
                    break;
                }
            }
        }

        let mut result = Vec::new(env);
        let take = n.min(limit);
        for i in 0..take {
            let (score, addr) = entries.get(i as u32).unwrap();
            result.push_back(LeaderboardEntry {
                participant: addr,
                score,
                rank: (i + 1) as u32,
            });
        }
        result
    }

    /// Returns true if the incentive is active and within its scheduled time window.
    fn incentive_in_window(incentive: &Incentive, now: u64) -> bool {
        if !incentive.active {
            return false;
        }
        if let Some(starts) = incentive.starts_at {
            if now < starts {
                return false;
            }
        }
        if let Some(ends) = incentive.ends_at {
            if now >= ends {
                return false;
            }
        }
        true
    }

    /// Set or update the activation schedule for an existing incentive.
    ///
    /// Only the original `rewarder` may schedule their incentive.
    /// `starts_at` must be strictly less than `ends_at` when both are provided.
    ///
    /// # Parameters
    /// - `incentive_id`: ID of the incentive to schedule.
    /// - `rewarder`: Original creator. Must sign.
    /// - `starts_at`: Optional UTC timestamp when the incentive becomes active.
    /// - `ends_at`: Optional UTC timestamp when the incentive expires.
    ///
    /// # Errors
    /// - Panics `"Incentive not found"` if the ID does not exist.
    /// - [`Error::NotCreator`] if `rewarder` is not the original creator.
    /// - [`Error::InvalidSchedule`] if `starts_at >= ends_at` (when both set),
    ///   or if `ends_at` is already in the past.
    pub fn schedule_incentive(
        env: Env,
        incentive_id: u64,
        rewarder: Address,
        starts_at: Option<u64>,
        ends_at: Option<u64>,
    ) -> Result<Incentive, Error> {
        rewarder.require_auth();
        Self::require_not_paused(&env);

        let mut incentive: Incentive =
            Self::get_incentive(&env, incentive_id).expect("Incentive not found");

        if incentive.rewarder != rewarder {
            return Err(Error::NotCreator);
        }

        let now = env.ledger().timestamp();

        // ends_at must be in the future
        if let Some(ends) = ends_at {
            if ends <= now {
                return Err(Error::InvalidSchedule);
            }
        }

        // starts_at must be before ends_at when both provided
        if let (Some(starts), Some(ends)) = (starts_at, ends_at) {
            if starts >= ends {
                return Err(Error::InvalidSchedule);
            }
        }

        incentive.starts_at = starts_at;
        incentive.ends_at = ends_at;

        Self::set_incentive(&env, incentive_id, &incentive);

        events::emit_incentive_scheduled(&env, incentive_id, &rewarder, starts_at, ends_at);

        Ok(incentive)
    }

    // ========== Feature: Processing Costs Tracking ==========

    /// Set the processing cost for a v2 waste item.
    ///
    /// Only the current owner may call this.
    ///
    /// # Errors
    /// - [`Error::WasteNotFound`] if the waste does not exist.
    /// - [`Error::NotWasteOwner`] if caller is not the current owner.
    pub fn set_processing_cost(
        env: Env,
        waste_id: u128,
        owner: Address,
        cost: u128,
    ) -> Result<types::Waste, Error> {
        owner.require_auth();
        Self::require_not_paused(&env);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .ok_or(Error::WasteNotFound)?;

        if waste.current_owner != owner {
            return Err(Error::NotWasteOwner);
        }

        waste.processing_cost = cost;
        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        // Update participant stats
        let stats_key = ("stats", owner.clone());
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&stats_key)
            .unwrap_or_else(|| RecyclingStats::new(owner.clone()));
        stats.total_processing_costs = stats.total_processing_costs.saturating_add(cost);
        env.storage().instance().set(&stats_key, &stats);

        Ok(waste)
    }

    /// Get the total processing costs across all v2 waste items.
    pub fn get_total_processing_costs(env: Env) -> u128 {
        let count: u64 = env
            .storage()
            .instance()
            .get(&("waste_count",))
            .unwrap_or(0);
        let mut total: u128 = 0;
        for id in 1u128..=(count as u128) {
            if let Some(w) = env
                .storage()
                .instance()
                .get::<_, types::Waste>(&("waste_v2", id))
            {
                total = total.saturating_add(w.processing_cost);
            }
        }
        total
    }

    /// Get the total processing costs for a specific participant.
    pub fn get_participant_processing_costs(env: Env, participant: Address) -> u128 {
        let stats: Option<RecyclingStats> = env.storage().instance().get(&("stats", participant));
        stats.map(|s| s.total_processing_costs).unwrap_or(0)
    }

    // ========== Feature: Material Composition Tracking ==========

    /// Set the material composition for a v2 waste item.
    ///
    /// Only a registered Collector or Manufacturer (verifier) may call this.
    /// Percentages must sum to exactly 100. Maximum 10 composition entries.
    ///
    /// # Errors
    /// - [`Error::WasteNotFound`] if the waste does not exist.
    /// - [`Error::NotRegistered`] if caller is not registered.
    /// - [`Error::InvalidAmount`] if percentages don't sum to 100 or > 10 entries.
    pub fn set_waste_composition(
        env: Env,
        waste_id: u128,
        verifier: Address,
        composition: Vec<types::MaterialComposition>,
    ) -> Result<types::Waste, Error> {
        verifier.require_auth();
        Self::require_not_paused(&env);

        // Verifier must be a registered Collector or Manufacturer
        let participant = Self::get_participant(env.clone(), verifier.clone())
            .ok_or(Error::NotRegistered)?;
        if participant.role == ParticipantRole::Recycler {
            return Err(Error::Unauthorized);
        }

        if composition.len() > 10 {
            return Err(Error::InvalidAmount);
        }

        let mut total: u32 = 0;
        for entry in composition.iter() {
            total = total.saturating_add(entry.percentage);
        }
        if total != 100 {
            return Err(Error::InvalidAmount);
        }

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .ok_or(Error::WasteNotFound)?;

        waste.composition = composition;
        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        Ok(waste)
    }

    /// Get all v2 waste IDs that contain at least `min_percentage` of `material_type`.
    pub fn get_wastes_by_composition(
        env: Env,
        material_type: WasteType,
        min_percentage: u32,
    ) -> Vec<u128> {
        let count: u64 = env
            .storage()
            .instance()
            .get(&("waste_count",))
            .unwrap_or(0);
        let mut result = Vec::new(&env);
        for id in 1u128..=(count as u128) {
            if let Some(w) = env
                .storage()
                .instance()
                .get::<_, types::Waste>(&("waste_v2", id))
            {
                for entry in w.composition.iter() {
                    if entry.material_type == material_type && entry.percentage >= min_percentage {
                        result.push_back(id);
                        break;
                    }
                }
            }
        }
        result
    }

    // ========== Feature: Recycling Goals ==========

    /// Set a recycling goal for the caller.
    ///
    /// Maximum 5 active goals per participant.
    /// Goal duration must be between 1 and 12 months from now (approx 30–365 days).
    ///
    /// # Errors
    /// - [`Error::NotRegistered`] if caller is not registered.
    /// - [`Error::InvalidAmount`] if target_weight is 0, too many goals, or invalid duration.
    pub fn set_recycling_goal(
        env: Env,
        participant: Address,
        target_weight: u128,
        target_date: u64,
        waste_type: Option<WasteType>,
    ) -> Result<(), Error> {
        participant.require_auth();
        Self::require_not_paused(&env);

        if Self::get_participant(env.clone(), participant.clone()).is_none() {
            return Err(Error::NotRegistered);
        }

        if target_weight == 0 {
            return Err(Error::InvalidAmount);
        }

        let now = env.ledger().timestamp();
        let one_month_secs: u64 = 30 * 24 * 60 * 60;
        let twelve_months_secs: u64 = 365 * 24 * 60 * 60;

        if target_date < now + one_month_secs || target_date > now + twelve_months_secs {
            return Err(Error::InvalidAmount);
        }

        let goals_key = ("goals", participant.clone());
        let mut goals: Vec<types::RecyclingGoal> = env
            .storage()
            .instance()
            .get(&goals_key)
            .unwrap_or(Vec::new(&env));

        // Count active (non-achieved) goals
        let mut active_count: u32 = 0;
        for goal in goals.iter() {
            if !goal.achieved {
                active_count += 1;
            }
        }
        if active_count >= 5 {
            return Err(Error::InvalidAmount);
        }

        let goal = types::RecyclingGoal {
            target_weight,
            target_date,
            waste_type,
            current_weight: 0,
            achieved: false,
            created_at: now,
        };

        goals.push_back(goal);
        env.storage().instance().set(&goals_key, &goals);

        env.events().publish(
            (symbol_short!("goal_set"), participant.clone()),
            (target_weight, target_date),
        );

        Ok(())
    }

    /// Get the recycling goals and progress for a participant.
    pub fn get_goal_progress(env: Env, participant: Address) -> Vec<types::RecyclingGoal> {
        env.storage()
            .instance()
            .get(&("goals", participant))
            .unwrap_or(Vec::new(&env))
    }

    /// Get all participants who have at least one achieved goal.
    pub fn get_participants_meeting_goals(env: Env) -> Vec<Address> {
        let index: Vec<Address> = env
            .storage()
            .instance()
            .get(&PART_INDEX)
            .unwrap_or(Vec::new(&env));
        let mut result = Vec::new(&env);
        for addr in index.iter() {
            let goals: Vec<types::RecyclingGoal> = env
                .storage()
                .instance()
                .get(&("goals", addr.clone()))
                .unwrap_or(Vec::new(&env));
            let mut has_achieved = false;
            for goal in goals.iter() {
                if goal.achieved {
                    has_achieved = true;
                    break;
                }
            }
            if has_achieved {
                result.push_back(addr);
            }
        }
        result
    }

    // ========== Feature: Recycling Rate Calculations ==========

    /// Get the global recycling rate in basis points (e.g. 9500 = 95.00%).
    /// Calculated as (total recycled weight / total weight) * 10000.
    pub fn get_global_recycling_rate(env: Env) -> u32 {
        let count: u64 = env
            .storage()
            .instance()
            .get(&("waste_count",))
            .unwrap_or(0);
        let mut total_weight: u128 = 0;
        let mut recycled_weight: u128 = 0;
        for id in 1u128..=(count as u128) {
            if let Some(w) = env
                .storage()
                .instance()
                .get::<_, types::Waste>(&("waste_v2", id))
            {
                total_weight = total_weight.saturating_add(w.weight);
                if w.recycled_timestamp > 0 {
                    recycled_weight = recycled_weight.saturating_add(w.weight);
                }
            }
        }
        if total_weight == 0 {
            return 0;
        }
        ((recycled_weight * 10_000) / total_weight) as u32
    }

    /// Get the recycling rate for a specific participant in basis points.
    pub fn get_participant_recycling_rate(env: Env, participant: Address) -> u32 {
        let stats: Option<RecyclingStats> = env.storage().instance().get(&("stats", participant));
        stats.map(|s| s.recycling_rate).unwrap_or(0)
    }
}
