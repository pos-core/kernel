#[path = "support/order_item_cases.rs"]
mod order_item_cases;

use order_item_cases::{
    order_item_expands_to_base_and_modifier_entries,
    order_item_rejects_zero_quantity_and_wrong_modifier_entry_id_count,
    order_item_snapshots_configured_catalog_item,
    unconnected_order_item_supports_none_ids_down_to_modifiers,
};

#[test]
fn snapshots_configured_catalog_item() {
    order_item_snapshots_configured_catalog_item();
}

#[test]
fn expands_to_base_and_modifier_entries() {
    order_item_expands_to_base_and_modifier_entries();
}

#[test]
fn rejects_zero_quantity_and_wrong_modifier_entry_id_count() {
    order_item_rejects_zero_quantity_and_wrong_modifier_entry_id_count();
}

#[test]
fn supports_none_ids_down_to_modifiers() {
    unconnected_order_item_supports_none_ids_down_to_modifiers();
}
