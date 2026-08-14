use crate::support::behavior::*;
use crate::support::md_report::ModuleReport;
use crate::support::order_item_cases::{
    order_item_expands_to_base_and_modifier_entries,
    order_item_rejects_zero_quantity_and_wrong_modifier_entry_id_count,
    order_item_snapshots_configured_catalog_item,
    unconnected_order_item_supports_none_ids_down_to_modifiers,
};

pub fn report() -> ModuleReport {
    ModuleReport {
        slug: "order-item",
        title: "Order Item",
        description: "Described behavior tests for catalog-backed order item snapshots and entry expansion.",
        cases: vec![
            case(
                "order item snapshots configured catalog item",
                "An order item preserves catalog item labels, variant labels, effects, modifier snapshot, unit prices, and total price.",
                order_item_snapshots_configured_catalog_item,
            ),
            case(
                "order item expands to base and modifier entries",
                "A catalog-backed order item expands into one base item entry and one entry for each priced modifier contribution.",
                order_item_expands_to_base_and_modifier_entries,
            ),
            case(
                "order item rejects zero quantity and wrong modifier entry id count",
                "Order item construction rejects zero quantity and entry expansion requires one modifier entry ID per priced contribution.",
                order_item_rejects_zero_quantity_and_wrong_modifier_entry_id_count,
            ),
            case(
                "unconnected order item supports none ids down to modifiers",
                "Manual order items can preserve labels, prompts, choices, and modifier price contributions without catalog IDs.",
                unconnected_order_item_supports_none_ids_down_to_modifiers,
            ),
        ],
    }
}
