//! Per-endpoint cache TTL configuration for the Scavenger backend.
//!
//! Added as part of issue #908 — Add caching layer for read-heavy endpoints.
//!
//! # Usage
//!
//! ```rust
//! use crate::cache::ttl::CacheTtl;
//!
//! // In a handler:
//! cache.set_with_ttl(key, data, CacheTtl::ContractStats.duration());
//! ```
//!
//! # Strategy
//!
//! | Endpoint | TTL | Rationale |
//! |---|---|---|
//! | `contract:stats` | 2 min | Aggregates; low churn |
//! | `contract:info` | 5 min | Nearly static |
//! | `contract:waste:{id}` | 30 s | May change on transfer/verify |
//! | `contract:wastes:*` (list) | 60 s | Paginated list; moderate churn |
//! | `contract:participant:{id}` | 60 s | Changes infrequently |
//! | `contract:participants:*` | 60 s | Paginated list |

use std::time::Duration;

/// Named TTL presets matched to specific endpoint categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheTtl {
    /// Individual waste item — 30 seconds.
    WasteItem,
    /// Waste list (paginated) — 60 seconds.
    WasteList,
    /// Individual participant — 60 seconds.
    ParticipantItem,
    /// Participant list (paginated) — 60 seconds.
    ParticipantList,
    /// Global contract statistics — 2 minutes.
    ContractStats,
    /// Contract metadata/info — 5 minutes.
    ContractInfo,
    /// Default fallback — 5 minutes.
    Default,
    /// Short-lived, e.g. for search suggestions — 10 seconds.
    Short,
}

impl CacheTtl {
    /// Returns the `Duration` corresponding to this TTL preset.
    pub fn duration(self) -> Duration {
        match self {
            CacheTtl::WasteItem => Duration::from_secs(30),
            CacheTtl::WasteList => Duration::from_secs(60),
            CacheTtl::ParticipantItem => Duration::from_secs(60),
            CacheTtl::ParticipantList => Duration::from_secs(60),
            CacheTtl::ContractStats => Duration::from_secs(120),
            CacheTtl::ContractInfo => Duration::from_secs(300),
            CacheTtl::Default => Duration::from_secs(300),
            CacheTtl::Short => Duration::from_secs(10),
        }
    }
}

/// Cache key prefixes — centralised to avoid typos.
pub mod keys {
    /// Individual waste record: `contract:waste:{id}`
    pub fn waste_item(id: &str) -> String {
        format!("contract:waste:{}", id)
    }

    /// Waste list page with query params: `contract:wastes:{qs}`
    pub fn waste_list(query_string: &str) -> String {
        format!("contract:wastes:{}", query_string)
    }

    /// Individual participant: `contract:participant:{id}`
    pub fn participant_item(id: &str) -> String {
        format!("contract:participant:{}", id)
    }

    /// Participant list page: `contract:participants:{qs}`
    pub fn participant_list(query_string: &str) -> String {
        format!("contract:participants:{}", query_string)
    }

    /// Contract global stats.
    pub const CONTRACT_STATS: &str = "contract:stats";

    /// Contract info/metadata.
    pub const CONTRACT_INFO: &str = "contract:info";

    /// Pattern prefix for all waste keys (used in invalidation).
    pub const WASTE_PATTERN: &str = "contract:waste";

    /// Pattern prefix for all participant keys.
    pub const PARTICIPANT_PATTERN: &str = "contract:participant";

    /// Pattern prefix covering all contract keys.
    pub const ALL_CONTRACT: &str = "contract:";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttl_ordering() {
        // WasteItem < WasteList < ContractStats < ContractInfo
        assert!(CacheTtl::WasteItem.duration() < CacheTtl::WasteList.duration());
        assert!(CacheTtl::WasteList.duration() < CacheTtl::ContractStats.duration());
        assert!(CacheTtl::ContractStats.duration() < CacheTtl::ContractInfo.duration());
    }

    #[test]
    fn test_key_helpers() {
        assert_eq!(keys::waste_item("w1"), "contract:waste:w1");
        assert_eq!(keys::participant_item("p1"), "contract:participant:p1");
        assert!(keys::waste_list("page=1").starts_with("contract:wastes:"));
    }

    #[test]
    fn test_all_ttl_variants_non_zero() {
        let variants = [
            CacheTtl::WasteItem,
            CacheTtl::WasteList,
            CacheTtl::ParticipantItem,
            CacheTtl::ParticipantList,
            CacheTtl::ContractStats,
            CacheTtl::ContractInfo,
            CacheTtl::Default,
            CacheTtl::Short,
        ];
        for ttl in variants {
            assert!(ttl.duration().as_secs() > 0, "{:?} has zero TTL", ttl);
        }
    }
}
