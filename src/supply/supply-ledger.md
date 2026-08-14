# Supply Ledger

A supply ledger tracks the lifecycle of supply claims by claim ID.

A claim can be reserved, unreserved, consumed, or unconsumed. Reserve records provisional use; consume records final use and can commit an exactly matching reservation. Reversal operations retain the original request so the transition remains explainable.

The ledger validates claim uniqueness, request equality, and allowed state transitions. It does not itself resolve whether new supply is available.
