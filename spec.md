# POS Spec

## Tech Stack
- **Lang**: Rust
- **OS**: Cross-platform (Windows, Linux, macOS)

## Core Architecture
- **Type**: Library Crate (Embeddable "Tight API")
- **Responsibility**: Pure business logic & state management.
- **Data**: Abstracted persistence.

## Naming Conventions
- **Rule**: NO reserved keywords as field names (e.g., use `kind` not `type`).
- **Style**: All field names must use `snake_case`.

## Catalog Model
- **Unified Structure**: Single `Item` struct for EVERYTHING.
    - **Rationale**: The "Composite Pattern" allows for infinite nesting (Categories in Categories) and generic traversal.
    - **Trade-off**: Relies on runtime checks (`kind` tag) rather than compile-time types, but offers superior flexibility for dynamic menus.
    - **Query Strategy**:
        - **Storage**: Single Table/Collection indexed by `kind`.
        - **API**: Returns **Strict Types** (e.g., `ProductDTO`) by transforming the raw `Item` on read.
        - **Performance**: `SELECT * FROM items WHERE kind = 'product'` is fast with proper indexing.
    - **Kinds**: `category`, `product`, `variant`, `modifier` are just `Item`s with a `kind` tag.
    - **Hierarchy**: Recursive. A `category` Item contains `product` Items. A `product` Item contains `modifier` Items.
- **Composition Rules** (Safety):
    - **Recursive**: `category` -> `category`, `product` -> `product` (variants).
    - **Order**: `category` -> `product` -> `modifier_group` -> `modifier`.
    - **Constraint**: Cannot put a `category` inside a `product`/`modifier`.
- **Identity**:
    - **Strategy**: `slug` (Stable, User-Defined, URL-friendly).
    - **Lookup**: `external_id` (Optional Barcode/PLU/ERP Code).
- **Content**:
    - **Fields**: `name`, `short_name`, `description` (Default/Fallback).
    - **Translations**: Map of `locale -> localized_content`.
        - `localized_content` contains optional overrides for `name`, `short_name`, `description`.
    - **Media**: List of `media_item` `{ url, alt_text, sort_order }`.
        - Supports multiple images per item (e.g., Carousel, Thumbnail).
- **Properties** (Divergent Data):
    - Flexible storage for logic specific to the Item's role.
    - **Standard Properties**:
        - `availability`:
            - `weekly`: Map `Day -> [[start_min, end_min]]` (e.g., `mon: [[540, 1020]]`).
            - `dates`: Map `Key -> { name, ranges }`.
                - `Key` can be `YYYY-MM-DD` (Specific Date).
                - `Key` can be `KnownHoliday` (e.g., "christmas", "thanksgiving") -> System resolves date dynamically.
        - `fulfillment_types`: List of allowed options (e.g., `[shipped, pickup]`).
        - `surfaces`: List of allowed channels (e.g., `[pos, web, kiosk]`).
        - `modifier_constraints`: `{ min_select, max_select }`.
        - `tax_code`: String.
        - `price`: Contextual Pricing Logic.
            - **Base**: `Money` (Default).
            - **Rules**: List of `{ condition: { surface, fulfillment }, adjustment: Money }`.
    - **Strategy**: **Cascading** (Parent properties inherit to children unless overridden).

## Modifier Logic
- **Constraints**: Defined on the Parent Item (e.g., "Cheese Group").
    - `min_select`: Minimum quantity (0 = optional, 1+ = required).
    - `max_select`: Maximum quantity (1 = single pick, >1 = multi pick).
    - `defaults`: List of pre-selected Item IDs.

## Stock & Availability
- **Strategy**: Defined in `properties` (Divergent).
- **Modes**:
    - **Infinite**: No tracking (Services, Digital, or high-volume Kitchen).
    - **Tracked**: Decrements count on order.
    - **Manual**: Simple "In Stock" / "Out of Stock" toggle (86'ing).
- **Locations**: Stock is tracked *per location* (separate from the Catalog Item definition).

## Labels (Metadata)
- **Entity**: `Label` (Distinct from Item).
    - **Fields**: `id`, `slug`, `name`, `description`, `color/icon`, `is_allergen` (bool).
- **Usage**: `Item` has `label_ids: [id]`.
- **Purpose**: Dietary (Vegan), Marketing (New), Warnings (Spicy).

## Order Model
- **Entity**: `Order` (The Transaction).
    - **Fields**: `id`, `customer_id` (optional), `table_id` (optional), `status`, `items` (List of LineItems), `totals`.
- **Line Item**:
    - **Concept**: A snapshot of an Item at the time of addition.
    - **Fields**: `item_id`, `name` (snapshot), `price` (snapshot), `quantity`, `modifiers` (List of LineItems).
    - **Snapshotting**: Critical. If the Catalog price changes, the Order line item price MUST NOT change.
- **Lifecycle**:
    - `Open` -> `AwaitingPayment` -> `Paid` -> `Fulfilled` -> `Closed`.
    - `Voided` (Cancelled).

## API Capabilities
- **Auth**: RBAC (Role-Based Access Control).
- **Catalog**: CRUD for Items/Variants/Modifiers.
- **Orders**: Cart management, Totals calculation, State transitions.

## Out of Scope
- Hardware Integrations (Printers, Scanners, Drawers)
- Payment Processing (CNP, Terminals)
