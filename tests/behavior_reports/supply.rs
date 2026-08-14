use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "supply",
        title: "Supply",
        description: "Described behavior tests for generic fulfillability, provider resolution, bucketed supply, and reversible supply claims.",
        definitions: vec![
            DefinitionLink::new("Supply view", "../src/supply/supply-view.md"),
            DefinitionLink::new("Supply bucket", "../src/supply/supply-bucket.md"),
            DefinitionLink::new("Supply ledger", "../src/supply/supply-ledger.md"),
        ],
        cases: vec![
            SUPPLY_VIEW_RESOLVES_AVAILABLE_UNAVAILABLE_AND_UNRESOLVED_REQUESTS.report_case(),
            SUPPLY_BUCKETS_DISTINGUISH_CALCULATED_SUPPLY.report_case(),
            SUPPLY_SHAPES_REJECT_INVALID_KEYS_QUANTITIES_AND_DUPLICATES.report_case(),
            SUPPLY_RESERVE_AND_UNRESERVE_ARE_REVERSIBLE.report_case(),
            SUPPLY_CONSUME_AND_UNCONSUME_ARE_REVERSIBLE.report_case(),
            SUPPLY_CONSUME_CAN_COMMIT_A_MATCHING_RESERVATION.report_case(),
            SUPPLY_LEDGER_REJECTS_INVALID_TRANSITIONS_AND_MISMATCHED_CONSUMES.report_case(),
        ],
    }
}

pub const SUPPLY_VIEW_RESOLVES_AVAILABLE_UNAVAILABLE_AND_UNRESOLVED_REQUESTS: DescribedBehavior =
    DescribedBehavior::new(
        "supply view resolves available unavailable and unresolved requests",
        "A provider-backed supply view answers exact target and bucket requests with Available, Unavailable, or Unresolved.",
        supply_view_resolves_available_unavailable_and_unresolved_requests,
    );

#[test]
fn supply_view_resolves_available_unavailable_and_unresolved_requests() {
    let target = SupplyTarget::choice(component_id("01PEPPER"));
    let view = SupplyView::new(vec![AvailableSupply::new(
        target.clone(),
        SupplyBucket::empty(),
        3,
    )])
    .unwrap();

    assert_eq!(
        view.resolve_supply(&supply_request(target.clone(), 2)),
        SupplyResolution::Available {
            requested: 2,
            available: 3
        }
    );
    assert_eq!(
        view.resolve_supply(&supply_request(target.clone(), 5)),
        SupplyResolution::Unavailable {
            requested: 5,
            available: 3,
            reason: SupplyUnavailableReason::InsufficientSupply
        }
    );
    assert_eq!(
        view.resolve_supply(&supply_request(
            SupplyTarget::catalog_item(catalog_item_id("01PIZZA")),
            1,
        )),
        SupplyResolution::Unresolved {
            reason: SupplyUnresolvedReason::MissingAvailableSupply
        }
    );
}

pub const SUPPLY_BUCKETS_DISTINGUISH_CALCULATED_SUPPLY: DescribedBehavior = DescribedBehavior::new(
    "supply buckets distinguish calculated supply",
    "Supply buckets let one target represent separate calculated resources such as time windows or capacity classes.",
    supply_buckets_distinguish_calculated_supply,
);

#[test]
fn supply_buckets_distinguish_calculated_supply() {
    let target = SupplyTarget::custom("delivery-slot").unwrap();
    let six_pm = supply_bucket("time-window", "18:00-18:30");
    let seven_pm = supply_bucket("time-window", "19:00-19:30");
    let view = SupplyView::new(vec![AvailableSupply::new(
        target.clone(),
        six_pm.clone(),
        2,
    )])
    .unwrap();

    assert!(
        view.resolve_supply(&supply_request(target.clone(), 1).with_bucket(six_pm))
            .is_available()
    );
    assert!(
        view.resolve_supply(&supply_request(target, 1).with_bucket(seven_pm))
            .is_unresolved()
    );
}

pub const SUPPLY_SHAPES_REJECT_INVALID_KEYS_QUANTITIES_AND_DUPLICATES: DescribedBehavior =
    DescribedBehavior::new(
        "supply shapes reject invalid keys quantities and duplicates",
        "Supply keys, bucket dimensions, request quantities, and available supply rows validate their deterministic shape.",
        supply_shapes_reject_invalid_keys_quantities_and_duplicates,
    );

#[test]
fn supply_shapes_reject_invalid_keys_quantities_and_duplicates() {
    let target = SupplyTarget::custom("daily-cap").unwrap();
    let bucket = SupplyBucket::empty();

    assert_eq!(SupplyKey::new(" "), Err(SupplyError::EmptySupplyKey));
    assert_eq!(
        SupplyKey::new("bad key"),
        Err(SupplyError::InvalidSupplyKey("bad key".to_owned()))
    );
    assert_eq!(
        SupplyBucket::empty().with_dimension("", "x"),
        Err(SupplyError::EmptyBucketDimensionKey)
    );
    assert_eq!(
        SupplyBucket::empty().with_dimension("window", ""),
        Err(SupplyError::EmptyBucketDimensionValue {
            key: "window".to_owned()
        })
    );
    assert_eq!(
        SupplyBucket::empty()
            .with_dimension("window", "am")
            .unwrap()
            .with_dimension("window", "pm"),
        Err(SupplyError::DuplicateBucketDimension("window".to_owned()))
    );
    assert_eq!(
        SupplyRequest::new(target.clone(), 0),
        Err(SupplyError::ZeroSupplyQuantity)
    );
    assert_eq!(
        SupplyView::new(vec![
            AvailableSupply::new(target.clone(), bucket.clone(), 1),
            AvailableSupply::new(target.clone(), bucket.clone(), 2),
        ]),
        Err(SupplyError::DuplicateAvailableSupply { target, bucket })
    );
}

pub const SUPPLY_RESERVE_AND_UNRESERVE_ARE_REVERSIBLE: DescribedBehavior = DescribedBehavior::new(
    "supply reserve and unreserve are reversible",
    "Reserve creates a provisional claim and Unreserve reverses that exact claim without consuming supply.",
    supply_reserve_and_unreserve_are_reversible,
);

#[test]
fn supply_reserve_and_unreserve_are_reversible() {
    let claim_id = supply_claim_id("01CLAIM");
    let target = SupplyTarget::choice(component_id("01PEPPER"));
    let request = supply_request(target.clone(), 2);
    let mut ledger = SupplyLedger::new();

    ledger
        .apply(SupplyOperation::Reserve(SupplyReserve::new(
            claim_id.clone(),
            request.clone(),
        )))
        .unwrap();

    assert_eq!(
        ledger.claim(&claim_id),
        Some(&SupplyClaimState::Reserved {
            request: request.clone()
        })
    );
    assert_eq!(ledger.reserved_quantity(&target, &SupplyBucket::empty()), 2);

    ledger
        .apply(SupplyOperation::Unreserve(SupplyUnreserve::new(
            claim_id.clone(),
        )))
        .unwrap();

    assert_eq!(
        ledger.claim(&claim_id),
        Some(&SupplyClaimState::Unreserved { request })
    );
    assert_eq!(ledger.reserved_quantity(&target, &SupplyBucket::empty()), 0);
}

pub const SUPPLY_CONSUME_AND_UNCONSUME_ARE_REVERSIBLE: DescribedBehavior = DescribedBehavior::new(
    "supply consume and unconsume are reversible",
    "Consume records final use and Unconsume reverses that exact consumed claim.",
    supply_consume_and_unconsume_are_reversible,
);

#[test]
fn supply_consume_and_unconsume_are_reversible() {
    let claim_id = supply_claim_id("01CLAIM");
    let target = SupplyTarget::custom("brunch-special").unwrap();
    let request = supply_request(target.clone(), 1);
    let mut ledger = SupplyLedger::new();

    ledger
        .apply(SupplyOperation::Consume(SupplyConsume::new(
            claim_id.clone(),
            request.clone(),
        )))
        .unwrap();

    assert_eq!(ledger.consumed_quantity(&target, &SupplyBucket::empty()), 1);

    ledger
        .apply(SupplyOperation::Unconsume(SupplyUnconsume::new(
            claim_id.clone(),
        )))
        .unwrap();

    assert_eq!(
        ledger.claim(&claim_id),
        Some(&SupplyClaimState::Unconsumed { request })
    );
    assert_eq!(ledger.consumed_quantity(&target, &SupplyBucket::empty()), 0);
}

pub const SUPPLY_CONSUME_CAN_COMMIT_A_MATCHING_RESERVATION: DescribedBehavior =
    DescribedBehavior::new(
        "supply consume can commit a matching reservation",
        "A reserved claim can be committed by consuming the same target, bucket, and quantity.",
        supply_consume_can_commit_a_matching_reservation,
    );

#[test]
fn supply_consume_can_commit_a_matching_reservation() {
    let claim_id = supply_claim_id("01CLAIM");
    let request = supply_request(SupplyTarget::choice(component_id("01PEPPER")), 1);
    let mut ledger = SupplyLedger::new();

    ledger
        .apply(SupplyOperation::Reserve(SupplyReserve::new(
            claim_id.clone(),
            request.clone(),
        )))
        .unwrap();
    ledger
        .apply(SupplyOperation::Consume(SupplyConsume::new(
            claim_id.clone(),
            request.clone(),
        )))
        .unwrap();

    assert_eq!(
        ledger.claim(&claim_id),
        Some(&SupplyClaimState::Consumed { request })
    );
}

pub const SUPPLY_LEDGER_REJECTS_INVALID_TRANSITIONS_AND_MISMATCHED_CONSUMES: DescribedBehavior =
    DescribedBehavior::new(
        "supply ledger rejects invalid transitions and mismatched consumes",
        "The supply ledger rejects unknown claims, duplicate claims, mismatched consume requests, and impossible reversals.",
        supply_ledger_rejects_invalid_transitions_and_mismatched_consumes,
    );

#[test]
fn supply_ledger_rejects_invalid_transitions_and_mismatched_consumes() {
    let claim_id = supply_claim_id("01CLAIM");
    let pepperoni_request = supply_request(SupplyTarget::choice(component_id("01PEPPER")), 1);
    let mismatch = supply_request(SupplyTarget::choice(component_id("01BACON")), 1);
    let mut ledger = SupplyLedger::new();

    assert_eq!(
        ledger.apply(SupplyOperation::Unreserve(SupplyUnreserve::new(
            claim_id.clone()
        ))),
        Err(SupplyError::UnknownSupplyClaim(claim_id.clone()))
    );

    ledger
        .apply(SupplyOperation::Reserve(SupplyReserve::new(
            claim_id.clone(),
            pepperoni_request.clone(),
        )))
        .unwrap();

    assert_eq!(
        ledger.apply(SupplyOperation::Reserve(SupplyReserve::new(
            claim_id.clone(),
            pepperoni_request.clone(),
        ))),
        Err(SupplyError::DuplicateSupplyClaim(claim_id.clone()))
    );

    assert_eq!(
        ledger.apply(SupplyOperation::Consume(SupplyConsume::new(
            claim_id.clone(),
            mismatch.clone(),
        ))),
        Err(SupplyError::MismatchedSupplyClaim {
            claim_id: claim_id.clone(),
            expected: pepperoni_request,
            actual: mismatch
        })
    );

    ledger
        .apply(SupplyOperation::Unreserve(SupplyUnreserve::new(
            claim_id.clone(),
        )))
        .unwrap();

    assert!(matches!(
        ledger.apply(SupplyOperation::Unconsume(SupplyUnconsume::new(
            claim_id.clone()
        ))),
        Err(SupplyError::InvalidSupplyTransition {
            operation: SupplyOperationKind::Unconsume,
            ..
        })
    ));
}
