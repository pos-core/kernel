# Order

An order is the current state derived from order events for one order ID.

It owns its status and entries. Commands produce events, events mutate the state through `apply`, and an ordered sequence of event envelopes can rebuild the same state through replay.

Totals are derived from the order's entries rather than stored as independently mutable order state.
