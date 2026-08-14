use pos_core_kernel::prelude::*;

use crate::support::behavior::*;
use crate::support::md_report::{DefinitionLink, ModuleReport};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "ids",
        title: "Typed IDs",
        description: "Described behavior tests for prefixed public domain identifiers.",
        definitions: vec![DefinitionLink::new(
            "Typed ID",
            "../src/primitives/ids/typed-id.md",
        )],
        cases: vec![TYPED_IDS_VALIDATE_PREFIXES.report_case()],
    }
}

pub const TYPED_IDS_VALIDATE_PREFIXES: DescribedBehavior = DescribedBehavior::new(
    "typed ids validate prefixes",
    "A typed public ID accepts its own standard prefix and rejects an ID belonging to another domain type.",
    typed_ids_validate_prefixes,
);

#[test]
fn typed_ids_validate_prefixes() {
    let order_id = OrderId::parse("ORD-01HX7Y9M8N6ZQ4K3V2B1C0D9EA").unwrap();

    assert_eq!(order_id.as_str(), "ORD-01HX7Y9M8N6ZQ4K3V2B1C0D9EA");
    assert!(EntryId::parse(order_id.as_str()).is_err());
}
