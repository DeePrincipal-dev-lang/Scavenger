use chrono::{DateTime, Utc};
/// #805 - Automated Contract Upgrade Service
/// Upgrade automation, validation, migration, rollback, and testing for Soroban contracts.
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

// ── Errors ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error, Clone)]
pub enum UpgradeError {
    #[error("Upgrade not found: {0}")]
    NotFound(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Migration error: {0}")]
    MigrationError(String),
    #[error("Rollback error: {0}")]
    RollbackError(String),
    #[error("Invalid state transition from {from} to {to}")]
    InvalidTransition { from: String, to: String },
    #[error("Upgrade already in progress: {0}")]
    AlreadyInProgress(String),
}

// ── Status ────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpgradeStatus {
    Pending,
    Validating,
    MigrationReady,
    Deploying,
    Completed,
    Failed,
    RolledBack,
}

impl UpgradeStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::RolledBack)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Validating => "validating",
            Self::MigrationReady => "migration_ready",
            Self::Deploying => "deploying",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::RolledBack => "rolled_back",
        }
    }
}

// ── Validation ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub all_passed: bool,
    pub checks: Vec<ValidationCheck>,
    pub validated_at: DateTime<Utc>,
}

// ── Migration ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    pub step_id: String,
    pub description: String,
    /// Soroban storage key to migrate
    pub storage_key: String,
    /// Transformation type
    pub transform: MigrationTransform,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationTransform {
    /// Copy value unchanged
    Copy,
    /// Rename key (new key name provided)
    Rename(String),
    /// Delete key
    Delete,
    /// Custom logic identifier (resolved at runtime)
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub from_version: u32,
    pub to_version: u32,
    pub steps: Vec<MigrationStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub steps_completed: usize,
    pub steps_failed: usize,
    pub errors: Vec<String>,
}

// ── Upgrade plan ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradePlan {
    pub id: String,
    pub name: String,
    pub description: String,
    pub from_version: u32,
    pub to_version: u32,
    /// WASM hash of the new contract binary
    pub wasm_hash: String,
    pub migration_plan: Option<MigrationPlan>,
    pub status: UpgradeStatus,
    pub validation_report: Option<ValidationReport>,
    pub migration_result: Option<MigrationResult>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub events: Vec<UpgradeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeEvent {
    pub timestamp: DateTime<Utc>,
    pub event: String,
}

// ── Rollback snapshot ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackSnapshot {
    pub plan_id: String,
    pub from_version: u32,
    pub wasm_hash: String,
    /// Serialised contract state before migration
    pub state_snapshot: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
}

// ── Validator trait ───────────────────────────────────────────────────────────

/// Pluggable upgrade validator. Implement to add custom pre-flight checks.
#[async_trait::async_trait]
pub trait UpgradeValidator: Send + Sync {
    async fn validate(&self, plan: &UpgradePlan) -> Vec<ValidationCheck>;
}

pub struct DefaultValidator;

#[async_trait::async_trait]
impl UpgradeValidator for DefaultValidator {
    async fn validate(&self, plan: &UpgradePlan) -> Vec<ValidationCheck> {
        vec![
            ValidationCheck {
                name: "version_increment".to_string(),
                passed: plan.to_version > plan.from_version,
                message: if plan.to_version > plan.from_version {
                    "Version is correctly incremented".to_string()
                } else {
                    format!(
                        "to_version {} must be > from_version {}",
                        plan.to_version, plan.from_version
                    )
                },
            },
            ValidationCheck {
                name: "wasm_hash_present".to_string(),
                passed: !plan.wasm_hash.is_empty(),
                message: if plan.wasm_hash.is_empty() {
                    "WASM hash is required".to_string()
                } else {
                    "WASM hash present".to_string()
                },
            },
            ValidationCheck {
                name: "migration_version_match".to_string(),
                passed: plan.migration_plan.as_ref().map_or(true, |m| {
                    m.from_version == plan.from_version && m.to_version == plan.to_version
                }),
                message: "Migration plan versions match upgrade plan".to_string(),
            },
        ]
    }
}

// ── Upgrade service ───────────────────────────────────────────────────────────

pub struct ContractUpgradeService {
    plans: Mutex<HashMap<String, UpgradePlan>>,
    snapshots: Mutex<HashMap<String, RollbackSnapshot>>,
    validator: Box<dyn UpgradeValidator>,
}

impl ContractUpgradeService {
    pub fn new(validator: Box<dyn UpgradeValidator>) -> Self {
        Self {
            plans: Mutex::new(HashMap::new()),
            snapshots: Mutex::new(HashMap::new()),
            validator,
        }
    }

    // ── CRUD ───────────────────────────────────────────────────────────────

    pub fn create_plan(
        &self,
        name: impl Into<String>,
        description: impl Into<String>,
        from_version: u32,
        to_version: u32,
        wasm_hash: impl Into<String>,
        migration_plan: Option<MigrationPlan>,
    ) -> UpgradePlan {
        let plan = UpgradePlan {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: description.into(),
            from_version,
            to_version,
            wasm_hash: wasm_hash.into(),
            migration_plan,
            status: UpgradeStatus::Pending,
            validation_report: None,
            migration_result: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            events: vec![UpgradeEvent {
                timestamp: Utc::now(),
                event: "Plan created".to_string(),
            }],
        };
        self.plans.lock().unwrap().insert(plan.id.clone(), plan.clone());
        plan
    }

    pub fn get_plan(&self, id: &str) -> Option<UpgradePlan> {
        self.plans.lock().unwrap().get(id).cloned()
    }

    pub fn list_plans(&self) -> Vec<UpgradePlan> {
        self.plans.lock().unwrap().values().cloned().collect()
    }

    fn save_plan(&self, plan: UpgradePlan) {
        self.plans.lock().unwrap().insert(plan.id.clone(), plan);
    }

    fn transition(&self, plan: &mut UpgradePlan, new_status: UpgradeStatus, event: &str) -> Result<(), UpgradeError> {
        if plan.status.is_terminal() {
            return Err(UpgradeError::InvalidTransition {
                from: plan.status.as_str().to_string(),
                to: new_status.as_str().to_string(),
            });
        }
        plan.status = new_status;
        plan.updated_at = Utc::now();
        plan.events.push(UpgradeEvent {
            timestamp: Utc::now(),
            event: event.to_string(),
        });
        Ok(())
    }

    // ── Validation ─────────────────────────────────────────────────────────

    pub async fn validate(&self, plan_id: &str) -> Result<ValidationReport, UpgradeError> {
        let plan = self
            .get_plan(plan_id)
            .ok_or_else(|| UpgradeError::NotFound(plan_id.to_string()))?;

        let checks = self.validator.validate(&plan).await;
        let all_passed = checks.iter().all(|c| c.passed);

        let report = ValidationReport {
            all_passed,
            checks,
            validated_at: Utc::now(),
        };

        let mut plan = plan;
        plan.validation_report = Some(report.clone());

        if all_passed {
            self.transition(&mut plan, UpgradeStatus::MigrationReady, "Validation passed")?;
        } else {
            self.transition(&mut plan, UpgradeStatus::Failed, "Validation failed")?;
        }
        self.save_plan(plan);

        if !all_passed {
            return Err(UpgradeError::ValidationFailed(
                "One or more validation checks failed".to_string(),
            ));
        }
        Ok(report)
    }

    // ── Migration ──────────────────────────────────────────────────────────

    /// Apply migration steps from the plan. Takes a mutable state map to
    /// simulate contract storage. In production, this would interact with
    /// Soroban's storage via XDR.
    pub async fn run_migration(
        &self,
        plan_id: &str,
        state: &mut HashMap<String, String>,
    ) -> Result<MigrationResult, UpgradeError> {
        let plan = self
            .get_plan(plan_id)
            .ok_or_else(|| UpgradeError::NotFound(plan_id.to_string()))?;

        if plan.status != UpgradeStatus::MigrationReady {
            return Err(UpgradeError::MigrationError(format!(
                "Plan must be in migration_ready state, is {:?}",
                plan.status
            )));
        }

        let mut completed = 0usize;
        let mut failed = 0usize;
        let mut errors = Vec::new();

        if let Some(mplan) = &plan.migration_plan {
            for step in &mplan.steps {
                match &step.transform {
                    MigrationTransform::Copy => {
                        // No-op: key already exists
                        completed += 1;
                    }
                    MigrationTransform::Rename(new_key) => {
                        if let Some(val) = state.remove(&step.storage_key) {
                            state.insert(new_key.clone(), val);
                            completed += 1;
                        } else {
                            errors.push(format!("Key not found: {}", step.storage_key));
                            failed += 1;
                        }
                    }
                    MigrationTransform::Delete => {
                        state.remove(&step.storage_key);
                        completed += 1;
                    }
                    MigrationTransform::Custom(handler) => {
                        // Custom handlers would be dispatched here; log for now.
                        completed += 1;
                        let _ = handler;
                    }
                }
            }
        }

        let result = MigrationResult {
            steps_completed: completed,
            steps_failed: failed,
            errors,
        };

        let mut plan = plan;
        plan.migration_result = Some(result.clone());
        if result.steps_failed == 0 {
            self.transition(&mut plan, UpgradeStatus::Deploying, "Migration complete")?;
        } else {
            self.transition(&mut plan, UpgradeStatus::Failed, "Migration had failures")?;
        }
        self.save_plan(plan);

        Ok(result)
    }

    // ── Deploy / complete ──────────────────────────────────────────────────

    /// Mark an upgrade as completed (called after WASM is deployed on-chain).
    pub fn complete_upgrade(&self, plan_id: &str) -> Result<(), UpgradeError> {
        let mut plan = self
            .get_plan(plan_id)
            .ok_or_else(|| UpgradeError::NotFound(plan_id.to_string()))?;

        if plan.status != UpgradeStatus::Deploying {
            return Err(UpgradeError::InvalidTransition {
                from: plan.status.as_str().to_string(),
                to: "completed".to_string(),
            });
        }
        plan.status = UpgradeStatus::Completed;
        plan.completed_at = Some(Utc::now());
        plan.updated_at = Utc::now();
        plan.events.push(UpgradeEvent {
            timestamp: Utc::now(),
            event: "Upgrade completed".to_string(),
        });
        self.save_plan(plan);
        Ok(())
    }

    // ── Rollback ───────────────────────────────────────────────────────────

    /// Capture a pre-upgrade state snapshot to enable rollback.
    pub fn create_snapshot(
        &self,
        plan_id: &str,
        from_version: u32,
        wasm_hash: impl Into<String>,
        state_snapshot: HashMap<String, String>,
    ) -> RollbackSnapshot {
        let snap = RollbackSnapshot {
            plan_id: plan_id.to_string(),
            from_version,
            wasm_hash: wasm_hash.into(),
            state_snapshot,
            created_at: Utc::now(),
        };
        self.snapshots.lock().unwrap().insert(plan_id.to_string(), snap.clone());
        snap
    }

    pub fn get_snapshot(&self, plan_id: &str) -> Option<RollbackSnapshot> {
        self.snapshots.lock().unwrap().get(plan_id).cloned()
    }

    /// Rollback: restore state from snapshot and mark plan as rolled back.
    pub fn rollback(
        &self,
        plan_id: &str,
        state: &mut HashMap<String, String>,
    ) -> Result<RollbackSnapshot, UpgradeError> {
        let snap = self
            .get_snapshot(plan_id)
            .ok_or_else(|| UpgradeError::RollbackError("No snapshot found".to_string()))?;

        *state = snap.state_snapshot.clone();

        if let Some(mut plan) = self.get_plan(plan_id) {
            if !plan.status.is_terminal() {
                plan.status = UpgradeStatus::RolledBack;
                plan.updated_at = Utc::now();
                plan.events.push(UpgradeEvent {
                    timestamp: Utc::now(),
                    event: format!("Rolled back to version {}", snap.from_version),
                });
                self.save_plan(plan);
            }
        }

        Ok(snap)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_service() -> ContractUpgradeService {
        ContractUpgradeService::new(Box::new(DefaultValidator))
    }

    fn simple_plan(svc: &ContractUpgradeService) -> UpgradePlan {
        svc.create_plan("v2 upgrade", "Add new storage fields", 1, 2, "abc123wasmhash", None)
    }

    #[tokio::test]
    async fn test_create_and_validate_plan() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        assert_eq!(plan.status, UpgradeStatus::Pending);

        let report = svc.validate(&plan.id).await.unwrap();
        assert!(report.all_passed);
        assert_eq!(svc.get_plan(&plan.id).unwrap().status, UpgradeStatus::MigrationReady);
    }

    #[tokio::test]
    async fn test_validation_fails_on_bad_version() {
        let svc = make_service();
        let plan = svc.create_plan("bad", "desc", 5, 3, "hash", None); // to < from
        let err = svc.validate(&plan.id).await;
        assert!(matches!(err, Err(UpgradeError::ValidationFailed(_))));
        assert_eq!(svc.get_plan(&plan.id).unwrap().status, UpgradeStatus::Failed);
    }

    #[tokio::test]
    async fn test_migration_rename_step() {
        let svc = make_service();
        let migration = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep {
                step_id: "s1".to_string(),
                description: "Rename key".to_string(),
                storage_key: "old_key".to_string(),
                transform: MigrationTransform::Rename("new_key".to_string()),
            }],
        };
        let plan = svc.create_plan("rename", "rename key", 1, 2, "hash", Some(migration));

        svc.validate(&plan.id).await.unwrap();

        let mut state = HashMap::from([("old_key".to_string(), "value".to_string())]);
        let result = svc.run_migration(&plan.id, &mut state).await.unwrap();

        assert_eq!(result.steps_completed, 1);
        assert_eq!(result.steps_failed, 0);
        assert!(state.contains_key("new_key"));
        assert!(!state.contains_key("old_key"));
        assert_eq!(svc.get_plan(&plan.id).unwrap().status, UpgradeStatus::Deploying);
    }

    #[tokio::test]
    async fn test_migration_delete_step() {
        let svc = make_service();
        let migration = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep {
                step_id: "d1".to_string(),
                description: "Remove deprecated key".to_string(),
                storage_key: "deprecated".to_string(),
                transform: MigrationTransform::Delete,
            }],
        };
        let plan = svc.create_plan("del", "delete key", 1, 2, "hash", Some(migration));
        svc.validate(&plan.id).await.unwrap();

        let mut state = HashMap::from([("deprecated".to_string(), "old".to_string())]);
        svc.run_migration(&plan.id, &mut state).await.unwrap();
        assert!(!state.contains_key("deprecated"));
    }

    #[tokio::test]
    async fn test_complete_upgrade() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        svc.validate(&plan.id).await.unwrap();
        let mut state = HashMap::new();
        svc.run_migration(&plan.id, &mut state).await.unwrap();
        svc.complete_upgrade(&plan.id).unwrap();
        assert_eq!(svc.get_plan(&plan.id).unwrap().status, UpgradeStatus::Completed);
    }

    #[tokio::test]
    async fn test_rollback() {
        let svc = make_service();
        let plan = simple_plan(&svc);

        let original_state = HashMap::from([("key".to_string(), "original_val".to_string())]);
        svc.create_snapshot(&plan.id, 1, "old_hash", original_state.clone());

        svc.validate(&plan.id).await.unwrap();
        let mut state = HashMap::from([("key".to_string(), "migrated_val".to_string())]);
        svc.run_migration(&plan.id, &mut state).await.unwrap();

        // Rollback
        let snap = svc.rollback(&plan.id, &mut state).unwrap();
        assert_eq!(snap.from_version, 1);
        assert_eq!(state["key"], "original_val");
        assert_eq!(svc.get_plan(&plan.id).unwrap().status, UpgradeStatus::RolledBack);
    }

    #[tokio::test]
    async fn test_cannot_migrate_without_validation() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        let mut state = HashMap::new();
        let err = svc.run_migration(&plan.id, &mut state).await;
        assert!(matches!(err, Err(UpgradeError::MigrationError(_))));
    }

    #[test]
    fn test_rollback_no_snapshot() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        let mut state = HashMap::new();
        let err = svc.rollback(&plan.id, &mut state);
        assert!(matches!(err, Err(UpgradeError::RollbackError(_))));
    }

    #[test]
    fn test_list_plans() {
        let svc = make_service();
        simple_plan(&svc);
        simple_plan(&svc);
        assert_eq!(svc.list_plans().len(), 2);
    }

    #[test]
    fn test_upgrade_status_is_terminal() {
        assert!(UpgradeStatus::Completed.is_terminal());
        assert!(UpgradeStatus::Failed.is_terminal());
        assert!(UpgradeStatus::RolledBack.is_terminal());
        assert!(!UpgradeStatus::Pending.is_terminal());
        assert!(!UpgradeStatus::Validating.is_terminal());
        assert!(!UpgradeStatus::MigrationReady.is_terminal());
        assert!(!UpgradeStatus::Deploying.is_terminal());
    }

    #[test]
    fn test_upgrade_status_as_str() {
        assert_eq!(UpgradeStatus::Pending.as_str(), "pending");
        assert_eq!(UpgradeStatus::Validating.as_str(), "validating");
        assert_eq!(UpgradeStatus::MigrationReady.as_str(), "migration_ready");
        assert_eq!(UpgradeStatus::Deploying.as_str(), "deploying");
        assert_eq!(UpgradeStatus::Completed.as_str(), "completed");
        assert_eq!(UpgradeStatus::Failed.as_str(), "failed");
        assert_eq!(UpgradeStatus::RolledBack.as_str(), "rolled_back");
    }

    #[tokio::test]
    async fn test_validation_missing_wasm_hash() {
        let svc = make_service();
        let plan = svc.create_plan("no hash", "desc", 1, 2, "", None);
        let err = svc.validate(&plan.id).await;
        assert!(matches!(err, Err(UpgradeError::ValidationFailed(_))));
    }

    #[tokio::test]
    async fn test_validation_report_all_checks() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        let report = svc.validate(&plan.id).await.unwrap();
        assert!(report.all_passed);
        assert_eq!(report.checks.len(), 3);
    }

    #[tokio::test]
    async fn test_migration_copy_step() {
        let svc = make_service();
        let migration = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep {
                step_id: "c1".to_string(),
                description: "Copy key".to_string(),
                storage_key: "key".to_string(),
                transform: MigrationTransform::Copy,
            }],
        };
        let plan = svc.create_plan("copy", "copy key", 1, 2, "hash", Some(migration));
        svc.validate(&plan.id).await.unwrap();

        let mut state = HashMap::from([("key".to_string(), "value".to_string())]);
        let result = svc.run_migration(&plan.id, &mut state).await.unwrap();
        assert_eq!(result.steps_completed, 1);
        assert_eq!(result.steps_failed, 0);
        assert_eq!(state["key"], "value");
    }

    #[tokio::test]
    async fn test_migration_custom_handler() {
        let svc = make_service();
        let migration = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep {
                step_id: "custom1".to_string(),
                description: "Custom handler".to_string(),
                storage_key: "key".to_string(),
                transform: MigrationTransform::Custom("my_handler".to_string()),
            }],
        };
        let plan = svc.create_plan("custom", "custom transform", 1, 2, "hash", Some(migration));
        svc.validate(&plan.id).await.unwrap();

        let mut state = HashMap::new();
        let result = svc.run_migration(&plan.id, &mut state).await.unwrap();
        assert_eq!(result.steps_completed, 1);
    }

    #[tokio::test]
    async fn test_migration_rename_missing_key() {
        let svc = make_service();
        let migration = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep {
                step_id: "r1".to_string(),
                description: "Rename missing key".to_string(),
                storage_key: "missing".to_string(),
                transform: MigrationTransform::Rename("new".to_string()),
            }],
        };
        let plan = svc.create_plan("rename_miss", "missing key", 1, 2, "hash", Some(migration));
        svc.validate(&plan.id).await.unwrap();

        let mut state = HashMap::new();
        let result = svc.run_migration(&plan.id, &mut state).await.unwrap();
        assert_eq!(result.steps_completed, 0);
        assert_eq!(result.steps_failed, 1);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_migration_multiple_steps() {
        let svc = make_service();
        let migration = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![
                MigrationStep {
                    step_id: "s1".to_string(),
                    description: "Step 1".to_string(),
                    storage_key: "old1".to_string(),
                    transform: MigrationTransform::Rename("new1".to_string()),
                },
                MigrationStep {
                    step_id: "s2".to_string(),
                    description: "Step 2".to_string(),
                    storage_key: "old2".to_string(),
                    transform: MigrationTransform::Delete,
                },
            ],
        };
        let plan = svc.create_plan("multi", "multiple steps", 1, 2, "hash", Some(migration));
        svc.validate(&plan.id).await.unwrap();

        let mut state = HashMap::from([
            ("old1".to_string(), "val1".to_string()),
            ("old2".to_string(), "val2".to_string()),
        ]);
        let result = svc.run_migration(&plan.id, &mut state).await.unwrap();
        assert_eq!(result.steps_completed, 2);
        assert_eq!(result.steps_failed, 0);
    }

    #[test]
    fn test_cannot_complete_non_deploying() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        let err = svc.complete_upgrade(&plan.id);
        assert!(matches!(err, Err(UpgradeError::InvalidTransition { .. })));
    }

    #[tokio::test]
    async fn test_cannot_transition_from_terminal_state() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        svc.validate(&plan.id).await.unwrap();
        let mut state = HashMap::new();
        svc.run_migration(&plan.id, &mut state).await.unwrap();
        svc.complete_upgrade(&plan.id).unwrap();

        // Try to validate again
        let err = svc.validate(&plan.id).await;
        assert!(matches!(err, Err(UpgradeError::InvalidTransition { .. })));
    }

    #[test]
    fn test_get_nonexistent_plan() {
        let svc = make_service();
        assert!(svc.get_plan("nonexistent").is_none());
    }

    #[test]
    fn test_snapshot_creation() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        let state = HashMap::from([("key".to_string(), "value".to_string())]);
        let snap = svc.create_snapshot(&plan.id, 1, "old_hash", state.clone());
        assert_eq!(snap.from_version, 1);
        assert_eq!(snap.wasm_hash, "old_hash");
    }

    #[test]
    fn test_get_snapshot() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        let state = HashMap::from([("key".to_string(), "value".to_string())]);
        svc.create_snapshot(&plan.id, 1, "hash", state);
        let snap = svc.get_snapshot(&plan.id).unwrap();
        assert_eq!(snap.plan_id, plan.id);
    }

    #[test]
    fn test_snapshot_state_isolation() {
        let svc = make_service();
        let plan1 = simple_plan(&svc);
        let plan2 = simple_plan(&svc);

        let state1 = HashMap::from([("key".to_string(), "val1".to_string())]);
        let state2 = HashMap::from([("key".to_string(), "val2".to_string())]);

        svc.create_snapshot(&plan1.id, 1, "hash1", state1);
        svc.create_snapshot(&plan2.id, 1, "hash2", state2);

        let snap1 = svc.get_snapshot(&plan1.id).unwrap();
        let snap2 = svc.get_snapshot(&plan2.id).unwrap();

        assert_eq!(snap1.state_snapshot["key"], "val1");
        assert_eq!(snap2.state_snapshot["key"], "val2");
    }

    #[test]
    fn test_plan_events_creation() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        assert!(!plan.events.is_empty());
        assert_eq!(plan.events[0].event, "Plan created");
    }

    #[tokio::test]
    async fn test_validation_report_timestamp() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        let before = Utc::now();
        let report = svc.validate(&plan.id).await.unwrap();
        let after = Utc::now();
        assert!(before <= report.validated_at && report.validated_at <= after);
    }

    #[tokio::test]
    async fn test_plan_updated_at_changes() {
        let svc = make_service();
        let plan = simple_plan(&svc);
        let initial_updated = plan.updated_at;

        svc.validate(&plan.id).await.unwrap();
        let plan_after = svc.get_plan(&plan.id).unwrap();
        assert!(plan_after.updated_at > initial_updated);
    }

    #[test]
    fn test_migration_version_mismatch() {
        let svc = make_service();
        let migration = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![],
        };
        let plan = svc.create_plan("mismatch", "desc", 2, 3, "hash", Some(migration));
        assert!(plan.migration_plan.is_some());
    }

    #[tokio::test]
    async fn test_validation_with_mismatched_migration_versions() {
        let svc = make_service();
        let migration = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![],
        };
        let plan = svc.create_plan("mismatch", "desc", 2, 3, "hash", Some(migration));
        let report = svc.validate(&plan.id).await.unwrap();
        assert!(!report.all_passed);
    }
}
