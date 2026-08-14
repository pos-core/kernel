# Configured Catalog Item

A configured catalog item is the validated result of selecting one concrete variant match and hydrating its modifier selections.

It contains the selected variant IDs in dimension order, each selected variant's separate authored and resolved label, an optional exact match label, the concrete match's effects and explicit invariant price, hydrated modifier configuration, modifier pricing, and combined total price. Its convenience variant title uses the exact match label when present and otherwise joins the resolved component labels in dimension order. An empty unlabeled match has no variant title, so a simple item renders only its catalog-item label.

Consumer profile and evaluation time are applied while producing this value when the caller supplies them. Descriptions and media remain catalog presentation metadata and are not copied here.

This is configuration output. An order item copies the facts it needs into order-owned snapshot types.
