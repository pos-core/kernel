# Event Envelope

An event envelope pairs a domain event payload with metadata about the event itself.

The envelope carries an event ID, actor, occurrence time, and optional command and idempotency information. The generic payload remains the domain-specific fact that is applied during replay.

Replay consumes envelopes in their supplied order; the envelope does not reorder events.
