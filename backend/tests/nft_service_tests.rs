//! Comprehensive integration tests for the NFT service module
//! Tests for NFT certificate minting, verification, and state management

use scavenger_backend::services::nft::{NFTCertificate, NFTMintRequest, NFTManager};

// ── Helpers for test setup ──────────────────────────────────────────────────

fn valid_mint_request() -> NFTMintRequest {
    NFTMintRequest {
        participant_id: "recycler_001".to_string(),
        waste_type: "plastic".to_string(),
        weight: 500,
    }
}

fn valid_certificate() -> NFTCertificate {
    NFTCertificate {
        token_id: "nft_recycler_001".to_string(),
        participant_id: "recycler_001".to_string(),
        waste_type: "plastic".to_string(),
        weight: 500,
        timestamp: 1_700_000_000,
        metadata_uri: "ipfs://metadata/1700000000".to_string(),
    }
}

// ── Mint Certificate Tests ──────────────────────────────────────────────────

#[test]
fn test_mint_certificate_generates_valid_token_id() {
    let request = valid_mint_request();
    let cert = NFTManager::mint_certificate(request);

    assert!(!cert.token_id.is_empty());
    assert!(cert.token_id.starts_with("nft_"));
}

#[test]
fn test_mint_certificate_preserves_participant_id() {
    let request = NFTMintRequest {
        participant_id: "user_xyz".to_string(),
        waste_type: "glass".to_string(),
        weight: 1000,
    };
    let cert = NFTManager::mint_certificate(request);

    assert_eq!(cert.participant_id, "user_xyz");
}

#[test]
fn test_mint_certificate_preserves_waste_type() {
    let request = NFTMintRequest {
        participant_id: "recycler".to_string(),
        waste_type: "metal".to_string(),
        weight: 250,
    };
    let cert = NFTManager::mint_certificate(request);

    assert_eq!(cert.waste_type, "metal");
}

#[test]
fn test_mint_certificate_preserves_weight() {
    let request = NFTMintRequest {
        participant_id: "recycler".to_string(),
        waste_type: "plastic".to_string(),
        weight: 7500,
    };
    let cert = NFTManager::mint_certificate(request);

    assert_eq!(cert.weight, 7500);
}

#[test]
fn test_mint_certificate_sets_timestamp() {
    let request = valid_mint_request();
    let cert = NFTManager::mint_certificate(request);

    assert!(cert.timestamp > 0);
    // Timestamp should be recent (within last minute)
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert!(cert.timestamp <= now);
    assert!(now - cert.timestamp < 60);
}

#[test]
fn test_mint_certificate_generates_ipfs_metadata_uri() {
    let request = valid_mint_request();
    let cert = NFTManager::mint_certificate(request);

    assert!(cert.metadata_uri.starts_with("ipfs://metadata/"));
}

#[test]
fn test_mint_certificate_metadata_uri_contains_timestamp() {
    let request = valid_mint_request();
    let cert = NFTManager::mint_certificate(request);

    let timestamp_str = cert.timestamp.to_string();
    assert!(cert.metadata_uri.contains(&timestamp_str));
}

#[test]
fn test_mint_certificate_with_empty_participant_id() {
    let request = NFTMintRequest {
        participant_id: String::new(),
        waste_type: "plastic".to_string(),
        weight: 500,
    };
    let cert = NFTManager::mint_certificate(request);

    assert!(cert.token_id.starts_with("nft_"));
    assert_eq!(cert.participant_id, "");
}

#[test]
fn test_mint_certificate_with_empty_waste_type() {
    let request = NFTMintRequest {
        participant_id: "recycler".to_string(),
        waste_type: String::new(),
        weight: 500,
    };
    let cert = NFTManager::mint_certificate(request);

    assert_eq!(cert.waste_type, "");
}

#[test]
fn test_mint_certificate_with_zero_weight() {
    let request = NFTMintRequest {
        participant_id: "recycler".to_string(),
        waste_type: "plastic".to_string(),
        weight: 0,
    };
    let cert = NFTManager::mint_certificate(request);

    assert_eq!(cert.weight, 0);
}

#[test]
fn test_mint_certificate_with_max_u128_weight() {
    let request = NFTMintRequest {
        participant_id: "recycler".to_string(),
        waste_type: "plastic".to_string(),
        weight: u128::MAX,
    };
    let cert = NFTManager::mint_certificate(request);

    assert_eq!(cert.weight, u128::MAX);
}

#[test]
fn test_mint_certificate_idempotent_for_same_request() {
    let request = valid_mint_request();
    let cert1 = NFTManager::mint_certificate(request.clone());
    let cert2 = NFTManager::mint_certificate(request);

    // Token IDs should be identical for same participant_id
    assert_eq!(cert1.token_id, cert2.token_id);
    // Timestamps will differ slightly, but participant/waste/weight same
    assert_eq!(cert1.participant_id, cert2.participant_id);
    assert_eq!(cert1.waste_type, cert2.waste_type);
    assert_eq!(cert1.weight, cert2.weight);
}

#[test]
fn test_mint_certificate_unique_timestamps_across_calls() {
    let request = valid_mint_request();
    let cert1 = NFTManager::mint_certificate(request.clone());
    std::thread::sleep(std::time::Duration::from_millis(10));
    let cert2 = NFTManager::mint_certificate(request);

    assert!(cert2.timestamp >= cert1.timestamp);
}

// ── Verify Certificate Tests ────────────────────────────────────────────────

#[test]
fn test_verify_certificate_valid() {
    let cert = valid_certificate();
    assert!(NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_fails_empty_token_id() {
    let mut cert = valid_certificate();
    cert.token_id = String::new();

    assert!(!NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_fails_empty_participant_id() {
    let mut cert = valid_certificate();
    cert.participant_id = String::new();

    assert!(!NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_fails_zero_weight() {
    let mut cert = valid_certificate();
    cert.weight = 0;

    assert!(!NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_edge_case_weight_one() {
    let mut cert = valid_certificate();
    cert.weight = 1;

    assert!(NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_allows_empty_waste_type() {
    let mut cert = valid_certificate();
    cert.waste_type = String::new();

    assert!(NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_allows_empty_metadata_uri() {
    let mut cert = valid_certificate();
    cert.metadata_uri = String::new();

    assert!(NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_with_zero_timestamp() {
    let mut cert = valid_certificate();
    cert.timestamp = 0;

    assert!(NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_multiple_validations() {
    let cert = valid_certificate();

    // Multiple calls should be consistent
    assert!(NFTManager::verify_certificate(&cert));
    assert!(NFTManager::verify_certificate(&cert));
    assert!(NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_with_special_characters_in_participant_id() {
    let mut cert = valid_certificate();
    cert.participant_id = "recycler_@#$%_001".to_string();

    assert!(NFTManager::verify_certificate(&cert));
}

#[test]
fn test_verify_certificate_with_long_waste_type() {
    let mut cert = valid_certificate();
    cert.waste_type = "a".repeat(1000);

    assert!(NFTManager::verify_certificate(&cert));
}

// ── Round-trip Tests (Mint then Verify) ─────────────────────────────────────

#[test]
fn test_mint_and_verify_roundtrip() {
    let request = valid_mint_request();
    let minted = NFTManager::mint_certificate(request);

    assert!(NFTManager::verify_certificate(&minted));
}

#[test]
fn test_mint_and_verify_with_extreme_weight() {
    let request = NFTMintRequest {
        participant_id: "recycler".to_string(),
        waste_type: "plastic".to_string(),
        weight: u128::MAX / 2,
    };
    let minted = NFTManager::mint_certificate(request);

    assert!(NFTManager::verify_certificate(&minted));
}

#[test]
fn test_mint_and_verify_with_unicode_participant_id() {
    let request = NFTMintRequest {
        participant_id: "recycler_🌍_001".to_string(),
        waste_type: "plastic".to_string(),
        weight: 500,
    };
    let minted = NFTManager::mint_certificate(request);

    assert!(NFTManager::verify_certificate(&minted));
}

// ── Serialization Tests ─────────────────────────────────────────────────────

#[test]
fn test_certificate_serialization() {
    let cert = valid_certificate();
    let json = serde_json::to_string(&cert).expect("serialization should work");
    let deserialized: NFTCertificate = serde_json::from_str(&json)
        .expect("deserialization should work");

    assert_eq!(cert.token_id, deserialized.token_id);
    assert_eq!(cert.participant_id, deserialized.participant_id);
    assert_eq!(cert.waste_type, deserialized.waste_type);
    assert_eq!(cert.weight, deserialized.weight);
}

#[test]
fn test_mint_request_serialization() {
    let request = valid_mint_request();
    let json = serde_json::to_string(&request).expect("serialization should work");
    let deserialized: NFTMintRequest = serde_json::from_str(&json)
        .expect("deserialization should work");

    assert_eq!(request.participant_id, deserialized.participant_id);
    assert_eq!(request.waste_type, deserialized.waste_type);
    assert_eq!(request.weight, deserialized.weight);
}
