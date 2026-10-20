//! The interface each chain adapter implements, so the orchestrator in `multichain.rs` can work
//! with any chain without chain-specific branches.

/// A chain the orchestrator can talk to.
pub trait ChainAdapter {
    /// Stable identifier for the chain, for example `"stellar"`.
    fn chain_id(&self) -> &'static str;
}
