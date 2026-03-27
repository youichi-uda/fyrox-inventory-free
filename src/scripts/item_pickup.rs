use crate::item::ItemId;
use fyrox::core::{reflect::prelude::*, type_traits::prelude::*, visitor::prelude::*};
use fyrox::{
    plugin::error::GameResult,
    script::{ScriptContext, ScriptDeinitContext, ScriptTrait},
};

/// Script component for an item that can be picked up in the world.
/// Attach this to a 3D/2D node representing a collectible item.
///
/// The pickup logic should be triggered by your game code (e.g., collision detection).
/// Call `try_pickup()` when the player interacts with this node.
#[derive(Visit, Reflect, Default, Debug, Clone, TypeUuidProvider, ComponentProvider)]
#[type_uuid(id = "1a2b3c4d-5e6f-7890-abcd-ef1234500002")]
#[visit(optional)]
pub struct ItemPickup {
    /// The item definition ID to give when picked up.
    #[reflect(display_name = "Item ID")]
    pub item_id: ItemId,

    /// Quantity to give on pickup.
    #[reflect(display_name = "Quantity", min_value = "1", max_value = "9999")]
    pub quantity: u32,

    /// If true, the node will be destroyed after pickup.
    #[reflect(display_name = "Destroy On Pickup")]
    pub destroy_on_pickup: bool,

    /// If true, this pickup has already been collected and should be ignored.
    #[reflect(display_name = "Collected")]
    pub collected: bool,
}

impl ItemPickup {
    /// Marks this pickup as collected. Call this after successfully adding the item
    /// to the player's inventory. If `destroy_on_pickup` is true, you should also
    /// remove the node from the scene graph.
    pub fn collect(&mut self) {
        self.collected = true;
    }

    /// Returns true if this pickup can still be collected.
    pub fn is_available(&self) -> bool {
        !self.collected && self.quantity > 0
    }
}

impl ScriptTrait for ItemPickup {
    fn on_init(&mut self, _ctx: &mut ScriptContext) -> GameResult {
        if self.quantity == 0 {
            self.quantity = 1;
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
