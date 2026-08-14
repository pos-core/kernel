# Supply View

A supply view is a provider-backed lookup of currently available quantities.

Rows are keyed by the exact pair of supply target and supply bucket. Resolving a request returns Available when quantity is sufficient, Unavailable when a row exists but is insufficient, and Unresolved when the view has no matching row.

Unresolved is distinct from unavailable: it means the provider cannot answer from the information it has.
