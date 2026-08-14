use crate::actor::ActorContext;
use crate::primitives::ids::{CommandId, EventId};
use crate::primitives::time::UtcTime;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EventEnvelope<E> {
    event_id: EventId,
    command_id: Option<CommandId>,
    idempotency_key: Option<String>,
    actor: ActorContext,
    occurred_at: UtcTime,
    payload: E,
}

impl<E> EventEnvelope<E> {
    pub fn new(event_id: EventId, actor: ActorContext, occurred_at: UtcTime, payload: E) -> Self {
        Self {
            event_id,
            command_id: None,
            idempotency_key: None,
            actor,
            occurred_at,
            payload,
        }
    }

    pub fn with_command_id(mut self, command_id: CommandId) -> Self {
        self.command_id = Some(command_id);
        self
    }

    pub fn with_idempotency_key(mut self, idempotency_key: impl Into<String>) -> Self {
        self.idempotency_key = Some(idempotency_key.into());
        self
    }

    pub fn event_id(&self) -> &EventId {
        &self.event_id
    }

    pub fn actor(&self) -> &ActorContext {
        &self.actor
    }

    pub fn occurred_at(&self) -> UtcTime {
        self.occurred_at
    }

    pub fn payload(&self) -> &E {
        &self.payload
    }

    pub fn into_payload(self) -> E {
        self.payload
    }
}
