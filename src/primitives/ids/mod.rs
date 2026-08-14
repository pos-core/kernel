mod act_actor_id;
mod alc_allocation_id;
mod atr_consumer_attribute_id;
mod bag_bag_id;
mod brd_brand_id;
mod cat_catalog_id;
mod chk_check_id;
mod clm_supply_claim_id;
mod cmd_command_id;
mod cmp_component_id;
mod cus_customer_id;
mod cvn_catalog_version_id;
mod cvr_component_version_id;
mod dvc_device_id;
mod edg_edge_id;
mod ent_entry_id;
mod evt_event_id;
mod ful_fulfillment_mode_id;
mod itm_catalog_item_id;
mod lbl_label_id;
mod mch_merchant_id;
mod med_media_id;
mod oit_order_item_id;
mod opr_client_operation_id;
mod ord_order_id;
mod pay_payment_id;
mod per_permission_id;
mod pfx_prefixed_id;
mod rol_role_id;
mod sur_surface_id;
mod txn_transaction_id;
mod usr_user_id;
mod var_variant_id;

pub use act_actor_id::ActorId;
pub use alc_allocation_id::AllocationId;
pub use atr_consumer_attribute_id::ConsumerAttributeId;
pub use bag_bag_id::BagId;
pub use brd_brand_id::BrandId;
pub use cat_catalog_id::CatalogId;
pub use chk_check_id::CheckId;
pub use clm_supply_claim_id::SupplyClaimId;
pub use cmd_command_id::CommandId;
pub use cmp_component_id::ComponentId;
pub use cus_customer_id::CustomerId;
pub use cvn_catalog_version_id::CatalogVersionId;
pub use cvr_component_version_id::ComponentVersionId;
pub use dvc_device_id::DeviceId;
pub use edg_edge_id::EdgeId;
pub use ent_entry_id::EntryId;
pub use evt_event_id::EventId;
pub use ful_fulfillment_mode_id::FulfillmentModeId;
pub use itm_catalog_item_id::CatalogItemId;
pub use lbl_label_id::LabelId;
pub use mch_merchant_id::MerchantId;
pub use med_media_id::MediaId;
pub use oit_order_item_id::OrderItemId;
pub use opr_client_operation_id::ClientOperationId;
pub use ord_order_id::OrderId;
pub use pay_payment_id::PaymentId;
pub use per_permission_id::PermissionId;
pub use pfx_prefixed_id::{IdParseError, PrefixedId};
pub use rol_role_id::RoleId;
pub use sur_surface_id::SurfaceId;
pub use txn_transaction_id::TransactionId;
pub use usr_user_id::UserId;
pub use var_variant_id::VariantId;

macro_rules! define_prefixed_id {
    ($name:ident, $prefix:literal) => {
        #[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
        pub struct $name(super::PrefixedId);

        impl $name {
            pub const PREFIX: &'static str = $prefix;

            pub fn parse(value: impl AsRef<str>) -> Result<Self, super::IdParseError> {
                super::PrefixedId::parse(Self::PREFIX, value).map(Self)
            }

            pub fn from_suffix(suffix: impl AsRef<str>) -> Result<Self, super::IdParseError> {
                super::PrefixedId::from_suffix(Self::PREFIX, suffix).map(Self)
            }

            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }

            pub fn suffix(&self) -> &str {
                self.0.suffix()
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_tuple(stringify!($name))
                    .field(&self.as_str())
                    .finish()
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl std::str::FromStr for $name {
            type Err = super::IdParseError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = super::IdParseError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::parse(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }
    };
}

pub(crate) use define_prefixed_id;

#[cfg(test)]
mod tests {
    use super::{EntryId, OrderId};

    #[test]
    fn typed_ids_validate_prefixes() {
        let order_id = OrderId::parse("ORD-01HX7Y9M8N6ZQ4K3V2B1C0D9EA").unwrap();

        assert_eq!(order_id.as_str(), "ORD-01HX7Y9M8N6ZQ4K3V2B1C0D9EA");
        assert!(EntryId::parse(order_id.as_str()).is_err());
    }
}
