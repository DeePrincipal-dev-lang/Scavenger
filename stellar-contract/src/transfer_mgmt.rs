//! Transfer workflow domain module (issue #925).
//!
//! Re-exports transfer-approval types for domain-scoped imports.
//!
//! State-changing operations on `ScavengerContract` in `lib.rs`:
//! - `initiate_transfer` (sender must be registered)
//! - `approve_transfer` (recipient must sign)
//! - `reject_transfer` (recipient must sign)
//! - `expire_transfer` (callable by anyone after deadline)
//! - `get_pending_transfer`
//! - `approve_high_value_transfer`
//! - `set_transfer_threshold`, `set_required_approvals`
//! - `validate_transfer_path`, `admin_override_transfer`

pub use crate::types::{PendingTransfer, PendingTransferStatus, TransferApproval, TransferApprovalStatus};
