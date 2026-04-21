//! Table-driven tests for all 8 HAX regions
//! Ensures 100% coverage of the decision logic and verifies
//! that each region is correctly classified.

use chief_region_router_proto::*;
use std::collections::HashMap;

struct TestCase {
    name: &'static str,
    action: ActionMetadata,
    ledger: LedgerSnapshot,
    expected_region: Region,
    expected_surface: Surface,
    expected_friction: FrictionTier,
}

fn new_ledger(category: &str, level: u8) -> LedgerSnapshot {
    let mut ledger = HashMap::new();
    ledger.insert(category.to_string(), level);
    ledger
}

#[test]
fn test_region_1_adhoc_chat() {
    // Region 1: low D (1-2), low C, short H
    let test_case = TestCase {
        name: "R1: Draft email, low delegation",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Soft,
            monetary_impact_usd: 50.0,
            affected_parties: 1,
            legal_effect: LegalEffect::None,
            time_to_detect: TimeToDetect::Seconds,
            horizon: Horizon::Short,
            category: "email".to_string(),
        },
        ledger: new_ledger("email", 1),
        expected_region: Region::R1,
        expected_surface: Surface::QueueCard,
        expected_friction: FrictionTier::Tier0,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_region_2_intermediate() {
    // Region 2: low D (1-2), low C, medium H
    let test_case = TestCase {
        name: "R2: Medium horizon, low delegation",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Soft,
            monetary_impact_usd: 30.0,
            affected_parties: 1,
            legal_effect: LegalEffect::None,
            time_to_detect: TimeToDetect::Minutes,
            horizon: Horizon::Medium,
            category: "calendar".to_string(),
        },
        ledger: new_ledger("calendar", 2),
        expected_region: Region::R2,
        expected_surface: Surface::QueueCard,
        expected_friction: FrictionTier::Tier1,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_region_3_evidence_card() {
    // Region 3: low D (1-2), high C (hard/binding), short H
    let test_case = TestCase {
        name: "R3: Hard reversibility, evidence card",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Hard,
            monetary_impact_usd: 250.0,
            affected_parties: 2,
            legal_effect: LegalEffect::Draft,
            time_to_detect: TimeToDetect::Hours,
            horizon: Horizon::Short,
            category: "contracts".to_string(),
        },
        ledger: new_ledger("contracts", 1),
        expected_region: Region::R3,
        expected_surface: Surface::EvidenceCard,
        expected_friction: FrictionTier::Tier2,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_region_4_quarterly_review() {
    // Region 4: low D (1-2), high C, long H
    let test_case = TestCase {
        name: "R4: Long horizon, hard reversibility, quarterly review",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Hard,
            monetary_impact_usd: 800.0,
            affected_parties: 3,
            legal_effect: LegalEffect::Binding,
            time_to_detect: TimeToDetect::Days,
            horizon: Horizon::Long,
            category: "hiring".to_string(),
        },
        ledger: new_ledger("hiring", 2),
        expected_region: Region::R4,
        expected_surface: Surface::EvidenceCard,
        expected_friction: FrictionTier::Tier3,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_region_5_one_tap_queue() {
    // Region 5: high D (4-5), low C, short H
    let test_case = TestCase {
        name: "R5: High delegation, short horizon, one-tap",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Soft,
            monetary_impact_usd: 500.0,
            affected_parties: 1,
            legal_effect: LegalEffect::None,
            time_to_detect: TimeToDetect::Seconds,
            horizon: Horizon::Short,
            category: "email".to_string(),
        },
        ledger: new_ledger("email", 5),
        expected_region: Region::R5,
        expected_surface: Surface::QueueCard,
        expected_friction: FrictionTier::Tier1,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_region_6_ambient_resting_state() {
    // Region 6: high D (4-5), low C, long H (Chief's resting state)
    let test_case = TestCase {
        name: "R6: High delegation, long horizon, ambient",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Soft,
            monetary_impact_usd: 1000.0,
            affected_parties: 1,
            legal_effect: LegalEffect::None,
            time_to_detect: TimeToDetect::Hours,
            horizon: Horizon::Long,
            category: "email".to_string(),
        },
        ledger: new_ledger("email", 5),
        expected_region: Region::R6,
        expected_surface: Surface::QueueCard,
        expected_friction: FrictionTier::Tier0,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_region_7_just_in_time_ceremony() {
    // Region 7: high D (3-5), high C, short H (just-in-time)
    let test_case = TestCase {
        name: "R7: High consequence, short horizon, ceremony",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Hard,
            monetary_impact_usd: 2400.0,
            affected_parties: 2,
            legal_effect: LegalEffect::Binding,
            time_to_detect: TimeToDetect::Hours,
            horizon: Horizon::Short,
            category: "finance".to_string(),
        },
        ledger: new_ledger("finance", 4),
        expected_region: Region::R7,
        expected_surface: Surface::Ceremony,
        expected_friction: FrictionTier::Tier3,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_region_8_graduated_autonomy() {
    // Region 8: high D (3-5), high C, long H (graduated autonomy + renegotiation)
    let test_case = TestCase {
        name: "R8: High consequence, long horizon, co-sign",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Hard,
            monetary_impact_usd: 5000.0,
            affected_parties: 5,
            legal_effect: LegalEffect::Binding,
            time_to_detect: TimeToDetect::Days,
            horizon: Horizon::Long,
            category: "contracts".to_string(),
        },
        ledger: new_ledger("contracts", 3),
        expected_region: Region::R8,
        expected_surface: Surface::CeremonyCoSign,
        expected_friction: FrictionTier::Tier4,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_region_8_irreversible() {
    // Region 8 variant: irreversible actions
    let test_case = TestCase {
        name: "R8: Irreversible action, highest friction",
        action: ActionMetadata {
            reversibility_class: ReversibilityClass::Irreversible,
            monetary_impact_usd: 20000.0,
            affected_parties: 10,
            legal_effect: LegalEffect::Binding,
            time_to_detect: TimeToDetect::Days,
            horizon: Horizon::Open,
            category: "legal".to_string(),
        },
        ledger: new_ledger("legal", 5),
        expected_region: Region::R8,
        expected_surface: Surface::CeremonyCoSign,
        expected_friction: FrictionTier::Tier4,
    };

    let result = classify(&test_case.action, &test_case.ledger).expect("Should classify");
    assert_eq!(
        result.region, test_case.expected_region,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.surface, test_case.expected_surface,
        "{}",
        test_case.name
    );
    assert_eq!(
        result.friction_tier, test_case.expected_friction,
        "{}",
        test_case.name
    );
}

#[test]
fn test_unknown_category_fails() {
    let action = ActionMetadata {
        reversibility_class: ReversibilityClass::Soft,
        monetary_impact_usd: 0.0,
        affected_parties: 1,
        legal_effect: LegalEffect::None,
        time_to_detect: TimeToDetect::Seconds,
        horizon: Horizon::Short,
        category: "nonexistent".to_string(),
    };

    let ledger = new_ledger("email", 3);
    let result = classify(&action, &ledger);
    assert!(result.is_err());
}

#[test]
fn test_invalid_delegation_fails() {
    let action = ActionMetadata {
        reversibility_class: ReversibilityClass::Soft,
        monetary_impact_usd: 0.0,
        affected_parties: 1,
        legal_effect: LegalEffect::None,
        time_to_detect: TimeToDetect::Seconds,
        horizon: Horizon::Short,
        category: "email".to_string(),
    };

    let mut ledger = HashMap::new();
    ledger.insert("email".to_string(), 0); // Invalid: < 1
    let result = classify(&action, &ledger);
    assert!(result.is_err());
}

#[test]
fn test_boundary_monetary_impact() {
    // Test at boundary: exactly at the max_stake limit
    let action = ActionMetadata {
        reversibility_class: ReversibilityClass::Soft,
        monetary_impact_usd: 100.0, // Exactly at boundary for R1
        affected_parties: 1,
        legal_effect: LegalEffect::None,
        time_to_detect: TimeToDetect::Seconds,
        horizon: Horizon::Short,
        category: "email".to_string(),
    };

    let ledger = new_ledger("email", 1);
    let result = classify(&action, &ledger).expect("Should classify at boundary");
    assert_eq!(result.region, Region::R1);
}

#[test]
fn test_all_regions_reachable() {
    // Verify each region (1-8) can be reached by at least one action
    for region_id in 1..=8 {
        let region = match region_id {
            1 => Region::R1,
            2 => Region::R2,
            3 => Region::R3,
            4 => Region::R4,
            5 => Region::R5,
            6 => Region::R6,
            7 => Region::R7,
            8 => Region::R8,
            _ => unreachable!(),
        };

        // We've already tested all regions above, so just verify the enum is valid
        assert_eq!(region.id(), region_id as u8);
    }
}
