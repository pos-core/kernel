use crate::primitives::ids::{ActorId, BrandId, DeviceId, MerchantId, UserId};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ActorContext {
    actor_id: ActorId,
    actor_kind: ActorKind,
    user_id: Option<UserId>,
    brand_id: Option<BrandId>,
    merchant_id: Option<MerchantId>,
    device_id: Option<DeviceId>,
}

impl ActorContext {
    pub fn new(actor_id: ActorId, actor_kind: ActorKind) -> Self {
        Self {
            actor_id,
            actor_kind,
            user_id: None,
            brand_id: None,
            merchant_id: None,
            device_id: None,
        }
    }

    pub fn with_user_id(mut self, user_id: UserId) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_brand_id(mut self, brand_id: BrandId) -> Self {
        self.brand_id = Some(brand_id);
        self
    }

    pub fn with_merchant_id(mut self, merchant_id: MerchantId) -> Self {
        self.merchant_id = Some(merchant_id);
        self
    }

    pub fn with_device_id(mut self, device_id: DeviceId) -> Self {
        self.device_id = Some(device_id);
        self
    }

    pub fn actor_id(&self) -> &ActorId {
        &self.actor_id
    }

    pub fn actor_kind(&self) -> ActorKind {
        self.actor_kind
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ActorKind {
    StaffUser,
    CustomerUser,
    ServiceAccount,
    Integration,
    Device,
    System,
}
