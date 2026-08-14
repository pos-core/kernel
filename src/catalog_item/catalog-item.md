# Catalog Item

A catalog item is the catalog definition of something that can be configured for ordering.

It owns its stable item ID and label, one or more variants, shared modifier definitions, and the policy used to price those modifiers. The catalog item itself does not own a price; each variant supplies the invariant price used during configuration.

Configuring a catalog item means choosing an existing variant and hydrating its applicable modifier selections.
