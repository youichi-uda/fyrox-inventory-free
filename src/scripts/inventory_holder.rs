use crate::equipment::Equipment;
use crate::inventory::Inventory;
use fyrox::core::{reflect::prelude::*, type_traits::prelude::*, visitor::prelude::*};
use fyrox::{
    plugin::error::GameResult,
    script::{ScriptContext, ScriptDeinitContext, ScriptTrait},
};

/// Script component that gives a scene node an inventory and equipment.
/// Attach this to any character/chest/container node.
///
/// Access from other scripts:
/// ```ignore
/// let holder = ctx.scene.graph[some_handle]
///     .try_get_script::<InventoryHolder>()
///     .unwrap();
/// ```
#[derive(Visit, Reflect, Default, Debug, Clone, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "1a2b3c4d-5e6f-7890-abcd-ef1234500001")]
#[visit(optional)]
pub struct InventoryHolder {
    /// The inventory (item grid).
    #[reflect(display_name = "Inventory")]
    pub inventory: Inventory,

    /// Equipment slots (armor, weapon, etc.).
    #[reflect(display_name = "Equipment")]
    pub equipment: Equipment,

    /// Number of inventory slots. Only used during initialization.
    #[reflect(display_name = "Slot Count", min_value = "1", max_value = "200")]
    pub slot_count: u32,

    /// Number of columns for UI layout.
    #[reflect(display_name = "Columns", min_value = "1", max_value = "20")]
    pub columns: u32,

    /// Whether to create standard equipment slots on init.
    #[reflect(display_name = "Use Standard Equipment")]
    pub use_standard_equipment: bool,

    #[visit(skip)]
    #[reflect(hidden)]
    initialized: bool,
}

impl ScriptTrait for InventoryHolder {
    fn on_init(&mut self, _ctx: &mut ScriptContext) -> GameResult {
        if !self.initialized {
            let slot_count = if self.slot_count == 0 { 20 } else { self.slot_count as usize };
            let columns = if self.columns == 0 { 5 } else { self.columns };

            if self.inventory.slots.is_empty() {
                self.inventory = Inventory::new(slot_count, columns);
            }

            if self.use_standard_equipment && self.equipment.slots.is_empty() {
                self.equipment = Equipment::standard();
            }

            self.initialized = true;
        }
        Ok(())
    }

    fn on_deinit(&mut self, _ctx: &mut ScriptDeinitContext) -> GameResult {
        Ok(())
    }

    fn on_update(&mut self, _ctx: &mut ScriptContext) -> GameResult {
        Ok(())
    }
}
