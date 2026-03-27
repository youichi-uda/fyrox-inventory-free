//! # fyrox-inventory-free
//!
//! Core inventory system for the [Fyrox](https://fyrox.rs) game engine.
//!
//! ## Features
//!
//! - **Item Definitions**: Items with name, description, icon, rarity, category, weight, value, tags
//! - **Item Database Resource**: Load/save item databases as `.itemdb` files
//! - **Inventory Grid**: Fixed-size inventory with stacking, splitting, merging, sorting
//! - **Equipment System**: Named equipment slots with category validation
//! - **Scripts**: `InventoryHolder` and `ItemPickup` scripts for scene nodes
//! - **Save/Load**: All types derive `Visit` for serialization
//!
//! ## Pro Version
//!
//! The Pro version (`fyrox-inventory-pro`) adds:
//! - Drag & drop UI widgets (`ItemSlot`, `InventoryPanel`)
//! - Item tooltip builder
//! - Rarity-colored borders
//!
//! ## Quick Start
//!
//! ```ignore
//! use fyrox_inventory_free::InventorySystemPlugin;
//!
//! impl Plugin for Game {
//!     fn register(&self, context: PluginRegistrationContext) -> GameResult {
//!         InventorySystemPlugin::register_all(&context);
//!         Ok(())
//!     }
//! }
//! ```

pub mod database;
pub mod equipment;
pub mod inventory;
pub mod item;
pub mod scripts;

use database::{ItemDatabase, ItemDatabaseLoader};
use fyrox::core::SafeLock;
use fyrox::plugin::PluginRegistrationContext;
use scripts::inventory_holder::InventoryHolder;
use scripts::item_pickup::ItemPickup;

/// Convenience struct for registering all inventory system types with the engine.
pub struct InventorySystemPlugin;

impl InventorySystemPlugin {
    /// Registers all scripts and resource types with the engine.
    pub fn register_all(context: &PluginRegistrationContext) {
        context
            .serialization_context
            .script_constructors
            .add::<InventoryHolder>("Inventory Holder");
        context
            .serialization_context
            .script_constructors
            .add::<ItemPickup>("Item Pickup");

        let state = context.resource_manager.state();
        state.constructors_container.add::<ItemDatabase>();
        state.loaders.safe_lock().set(ItemDatabaseLoader);
    }
}
