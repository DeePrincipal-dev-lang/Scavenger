//! Contract analytics module (issue #759)
//!
//! Provides on-chain aggregation helpers used by the stats and metrics
//! functions.  Extracted from `lib.rs` to keep analytics logic cohesive and
//! independently testable.
//!
//! # Responsibilities
//! - Supply-chain statistics aggregation
//! - Global metrics updates
//! - Per-participant stat tracking helpers
//! - Carbon-credit and impact-score accumulation

use crate::types::{GlobalMetrics, RecyclingStats, WasteGrade, WasteType};

// ── GlobalMetrics helpers ─────────────────────────────────────────────────────

impl GlobalMetrics {
    /// Increments the counter for the given `grade`.
    pub fn record_grade(&mut self, grade: WasteGrade) {
        match grade {
            WasteGrade::A => self.grade_a_count += 1,
            WasteGrade::B => self.grade_b_count += 1,
            WasteGrade::C => self.grade_c_count += 1,
            WasteGrade::D => self.grade_d_count += 1,
        }
    }

    /// Records a new active waste item.
    pub fn record_waste_registered(&mut self) {
        self.total_wastes_count += 1;
        self.active_waste_count += 1;
    }

    /// Records a waste item being deactivated.
    pub fn record_waste_deactivated(&mut self) {
        if self.active_waste_count > 0 {
            self.active_waste_count -= 1;
        }
    }

    /// Records a waste item as expired.
    pub fn record_waste_expired(&mut self) {
        self.expired_waste_count += 1;
        if self.active_waste_count > 0 {
            self.active_waste_count -= 1;
        }
    }

    /// Accumulates token rewards into global totals.
    pub fn record_tokens_earned(&mut self, tokens: u128) {
        self.total_tokens_earned = self.total_tokens_earned.saturating_add(tokens);
    }

    /// Accumulates carbon credits into global totals.
    pub fn record_carbon_credits(&mut self, credits: u128) {
        self.total_carbon_credits = self.total_carbon_credits.saturating_add(credits);
    }
}

// ── RecyclingStats helpers ────────────────────────────────────────────────────

/// Accumulates a waste-type submission into `stats`.
pub fn record_waste_type_submission(stats: &mut RecyclingStats, waste_type: WasteType, weight_grams: u64) {
    stats.total_submissions += 1;
    stats.total_weight = stats.total_weight.saturating_add(weight_grams);
    match waste_type {
        WasteType::Paper => stats.paper_count += 1,
        WasteType::PetPlastic => stats.pet_plastic_count += 1,
        WasteType::Plastic => stats.plastic_count += 1,
        WasteType::Metal => stats.metal_count += 1,
        WasteType::Glass => stats.glass_count += 1,
        WasteType::Organic => stats.organic_count += 1,
        WasteType::Electronic => stats.electronic_count += 1,
    }
}

/// Records token and carbon-credit earnings for a participant.
pub fn record_earnings(stats: &mut RecyclingStats, tokens: u64, carbon_credits: u128) {
    stats.total_points = stats.total_points.saturating_add(tokens);
    stats.carbon_credits_earned = stats.carbon_credits_earned.saturating_add(carbon_credits);
}

/// Recalculates the recycling rate and updates `stats.recycling_rate`.
///
/// Rate is stored as basis points: 10 000 = 100 %.
pub fn refresh_recycling_rate(stats: &mut RecyclingStats) {
    stats.recycling_rate = if stats.total_weight == 0 {
        0
    } else {
        ((stats.recycled_weight * 10_000) / stats.total_weight as u128) as u32
    };
}
