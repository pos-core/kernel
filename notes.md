# Overview

Publishing a catalog version creates a new catalog version and copies all the data from the previous version.

Deleting never deletes, it just marks is_deleted as true.

Pruning can be done to remove deleted data.

The nice thing about publishing is that we have a single cache generation point instead of clearing cache on any edit.

Clients always request the latest catalog version, which will be an extremeley fast query to serve the single catalog.

Clients with old catalog versions will use the catalog version they have.

Clients with old catalog versions will be notified of the new catalog version and will need to update their catalog.

# Catalog Tables

Every catalog_ table will have these fields:

* id (int) always unique and primary key.
* entity_id (int) 
* entity_version (int) 
* brand_id (int) fk
* catalog_version_id (int) fk, drafts are version 0
* is_deleted (boolean)

## Hours (catalog_hours)
- Hours json format is {"MON" : [[0 : 60]], "XMAS": null, "02/14" null} (in minutes),

* name (string)
* hours (json) 

## Media (catalog_media)

* url (string)
* alt_text (string)
* title (string)
* pixel_width (int)
* pixel_height (int)
* hash (string)

## Tax Class (catalog_tax_class)
- Copy on edit, updates catalog entries.
- IDS should be readable slugs.
- id plus version unique and primary key.
- rate is in cents.

* name (string)
* description (string)
* rate (float)

## Surface (catalog_surface)
Surfaces are points of purchase. IDS should be readable slugs.

For example:
    - hub
    - pos
    - web
    - app
    - kiosk

* name (string)
* description (string)

## Fullfillment (catalog_fullfillment)
Fullfillments define how an item is fulfilled. 
This will include delivery specific settings, pickup specific settings, etc.
IDS should be readable slugs.

For example:
    - delivery (delivery to address)
    - courier_delivery (delivery to address via courier)
    - courier_pickup (pickup at address via courier)
    - take_out (take out)
    - dine_in (dine in)
    - table_dine_in (dine in at table)
    - external_pickup (external pickup)
    - external_delivery (external delivery, like grubhub)

TODO fields


## Catalog (catalog_component)

- The external_id field is not for mapping, it is for SKU type identifiers.
- External ids for aggregation are store in a mapping table
- Kind and slug are unique (kind + slug unique)
- External id and kind are unique (external_id + kind unique)
- Price is in cents.
- Tax class is nullable.
- Media ids is an array of media ids.
- Tax class and price can be overriden in the catalog tree.
- Metadata is a json object that can be used to store additional information.
- Metadata will have different schemas for different kinds.
- Stock quantity and 86ing is stored outside of the catalog entity since it is live data.
- Translations is a json object that can be used to store translations for the catalog entity.
- Still deciding on schema for translations. (Unsure if we want a single translation table for many tables)
- Short name and tiny name are nullable and are cascaded from display_name -> short_name -> tiny_name.
- Short name is a shorter name normally used for Point of Sale display.
- Tiny name is an even shorter name normally used for kitchen tickets or staff labels.
- Internal name is only shown when editing the catalog entity and is not used for end user display. 
- extra_data is a user defined json object that can be used to store additional information. { "key": { "value": "value", "type": "string" } }

* external_id
* kind (enum: catalog, category, item_list, item, modifier_list, modifier)
* slug (string)
* internal_name (string)
* display_name (string)
* short_name (string?)
* tiny_name (string?)
* description (string)
* media_ids (array: int) fk
* tax_class_id (int?) fk
* price (int)
* meta_data (json)
* extra_data (json)
* stock_tracking (boolean)
* translations (json)

## Catalog Tree Table (catalog_tree)
- Defines the catalog tree and allows for price and tax class overrides
- Catalogs store a tree of catalog entities, hours and price overrides
- null fullfillment_id means all fullfillments
- parent_id + child_id + fullfillment_id + order unique
- these point to the entity_id of the parent and child

* parent_id (int) fk
* child_id (int) fk
* fullfillment_id (int?) fk
* order (int)
* price (int?)
* tax_class_id (int?)



# Brand tables

Brands represent a collection of merchants that share a catalog.

## Brand (brand)

* id (int) primary key
* name (string)
* description (string)
* created_at (datetime)
* created_by (int) fk
* updated_at (datetime)
* updated_by (int) fk



# Merchant tables

Merchants represent a single merchant location. This can be a physical location or a virtual location.

Merchants in the same brand share a catalog, but can have different or shared catalog trees as long as the catalog tree is part of the brand.

## Merchant (merchant)

description will be in markdown.

* id (int) primary key
* slug (string)
* internal_name (string)
* display_name (string)
* description (string)
* brand_id (int) fk
* timezone (string)
* hours (json)
* meta_data (json)
* extra_data (json)


## Merchant Catalog (merchant_catalog)

Links a merchant to a catalog.

* id (int) primary key
* merchant_id (int) fk
* catalog_id (int) fk



# Audit Log (audit_log)

Audit log is used to track changes to the database.
client_data is information about the client (ip address, user agent, etc).
change_data is the data that was changed.

* id (int) primary key
* created_at (datetime)
* merchant_id (int) fk
* user_id (int) fk
* action (enum: create, update, delete, publish, prune)
* table (string)
* client_data (json)
* change_data (json)