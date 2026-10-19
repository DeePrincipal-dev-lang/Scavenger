//! Participant management domain module (issue #925).
//!
//! Re-exports participant-related types for domain-scoped imports.
//!
//! State-changing operations on `ScavengerContract` in `lib.rs`:
//! - `register_participant`, `get_participant`, `get_participant_info`
//! - `update_role`, `deregister_participant`, `update_participant_location`
//! - `get_all_participants`, `is_participant_registered`
//! - `get_participant_wastes`, `get_participant_wastes_v2`
//! - `get_stats`, `get_participant_earnings`, `get_participant_carbon_credits`
//! - `grant_certification`, `get_participants_by_cert`
//! - `get_top_recyclers`, `get_top_collectors`, `get_top_earners`, `get_top_verifiers`
//! - `get_participant_rank`
//! - `create_challenge`, `join_challenge`, `complete_challenge`
//! - `set_recycling_goal`, `get_goal_progress`
//! - `get_milestones`, `get_participant_milestones`

pub use crate::{Participant, ParticipantInfo};
pub use crate::types::{
    CertificationLevel, Challenge, ChallengeProgress, ChallengeStatus,
    LeaderboardEntry, Milestone, ParticipantRole, RecyclingGoal, RecyclingStats,
    ReputationBadge,
};
