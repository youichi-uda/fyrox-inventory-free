use crate::database::ItemDatabase;
use crate::inventory::Inventory;
use crate::item::{ItemCategory, ItemId, ItemStack};
use fyrox::core::{reflect::prelude::*, type_traits::prelude::*, uuid_provider, visitor::prelude::*};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, EnumString, VariantNames};

/// Equipment slot type. Each slot accepts specific item categories.
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Hash,
    Visit,
    Reflect,
    Serialize,
    Deserialize,
    AsRefStr,
    EnumString,
    VariantNames,
)]
pub enum EquipmentSlotType {
    #[default]
    MainHand,
    OffHand,
    Head,
    Body,
    Legs,
    Feet,
    Accessory1,
    Accessory2,
}

impl EquipmentSlotType {
    /// Returns the item categories accepted by this slot type.
    pub fn accepted_categories(&self) -> &[ItemCategory] {
        match self {
            Self::MainHand => &[ItemCategory::Weapon],
            Self::OffHand => &[ItemCategory::Shield, ItemCategory::Weapon],
            Self::Head => &[ItemCategory::Helmet],
            Self::Body => &[ItemCategory::Armor],
            Self::Legs => &[ItemCategory::Armor],
            Self::Feet => &[ItemCategory::Boots],
            Self::Accessory1 | Self::Accessory2 => &[ItemCategory::Accessory],
        }
    }

    /// Returns all standard equipment slot types.
    pub fn all() -> &'static [EquipmentSlotType] {
        &[
            Self::MainHand,
            Self::OffHand,
            Self::Head,
            Self::Body,
            Self::Legs,
            Self::Feet,
            Self::Accessory1,
            Self::Accessory2,
        ]
    }
}

/// A single equipment slot holding an optional item.
#[derive(Clone, Debug, Default, Visit, Reflect, Serialize, Deserialize, ComponentProvider)]
pub struct EquipmentSlot {
    /// The type of this slot (determines accepted categories).
    #[reflect(display_name = "Slot Type")]
    pub slot_type: EquipmentSlotType,

    /// The equipped item, if any (always quantity=1).
    #[reflect(display_name = "Equipped Item")]
    pub item: Option<ItemStack>,
}

uuid_provider!(EquipmentSlot = "c3d4e5f6-a7b8-9012-cdef-123456789012");

/// Equipment system with named slots.
#[derive(Clone, Debug, Default, Visit, Reflect, Serialize, Deserialize, ComponentProvider)]
pub struct Equipment {
    #[reflect(display_name = "Slots")]
    pub slots: Vec<EquipmentSlot>,
}

uuid_provider!(Equipment = "d4e5f6a7-b8c9-0123-defa-234567890123");

impl Equipment {
    /// Creates equipment with the standard set of slots.
    pub fn standard() -> Self {
        Self {
            slots: EquipmentSlotType::all()
                .iter()
                .map(|&slot_type| EquipmentSlot {
                    slot_type,
                    item: None,
                })
                .collect(),
        }
    }

    /// Finds a slot by type. Returns the first matching slot.
    pub fn find_slot(&self, slot_type: EquipmentSlotType) -> Option<&EquipmentSlot> {
        self.slots.iter().find(|s| s.slot_type == slot_type)
    }

    /// Finds a mutable slot by type.
    pub fn find_slot_mut(&mut self, slot_type: EquipmentSlotType) -> Option<&mut EquipmentSlot> {
        self.slots.iter_mut().find(|s| s.slot_type == slot_type)
    }

    /// Checks if an item can be equipped in the given slot.
    pub fn can_equip(
        &self,
        slot_type: EquipmentSlotType,
        item_id: ItemId,
        db: &ItemDatabase,
    ) -> bool {
        let def = match db.get(item_id) {
            Some(d) => d,
            None => return false,
        };
        slot_type
            .accepted_categories()
            .contains(&def.category)
    }

    /// Equips an item. Returns the previously equipped item (if any).
    pub fn equip(
        &mut self,
        slot_type: EquipmentSlotType,
        item_id: ItemId,
        db: &ItemDatabase,
    ) -> Result<Option<ItemStack>, EquipError> {
        if !self.can_equip(slot_type, item_id, db) {
            return Err(EquipError::IncompatibleSlot);
        }

        let slot = self
            .find_slot_mut(slot_type)
            .ok_or(EquipError::SlotNotFound)?;

        let previous = slot.item.take();
        slot.item = Some(ItemStack::new(item_id, 1));
        Ok(previous)
    }

    /// Equips one unit of `item_id` taken from `inventory`, automatically returning
    /// any previously equipped item back into the inventory (swap semantics).
    ///
    /// On success the item is removed from the inventory and the previously equipped
    /// item (if any) is added back. Returns an error without mutating anything if the
    /// item is not present in the inventory or is incompatible with the slot.
    pub fn equip_from_inventory(
        &mut self,
        slot_type: EquipmentSlotType,
        item_id: ItemId,
        inventory: &mut Inventory,
        db: &ItemDatabase,
    ) -> Result<(), EquipError> {
        if !self.can_equip(slot_type, item_id, db) {
            return Err(EquipError::IncompatibleSlot);
        }
        if !inventory.has_item(item_id, 1) {
            return Err(EquipError::NotInInventory);
        }
        // Ensure the slot exists before mutating the inventory.
        if self.find_slot(slot_type).is_none() {
            return Err(EquipError::SlotNotFound);
        }

        inventory.remove_item(item_id, 1);
        let previous = self.equip(slot_type, item_id, db)?;

        // Return the previously equipped item to the inventory.
        if let Some(prev) = previous {
            inventory.add_item(prev.item_id, prev.quantity, db);
        }
        Ok(())
    }

    /// Unequips an item from the given slot. Returns the item that was there.
    pub fn unequip(&mut self, slot_type: EquipmentSlotType) -> Option<ItemStack> {
        self.find_slot_mut(slot_type)?.item.take()
    }

    /// Returns the item equipped in the given slot.
    pub fn get_equipped(&self, slot_type: EquipmentSlotType) -> Option<&ItemStack> {
        self.find_slot(slot_type)?.item.as_ref()
    }

    /// Sum of weights of every currently equipped item, looked up via `db`.
    ///
    /// Items missing from the database contribute zero weight.
    pub fn total_weight(&self, db: &ItemDatabase) -> f32 {
        self.slots
            .iter()
            .filter_map(|s| s.item.as_ref())
            .map(|stack| {
                db.get(stack.item_id)
                    .map(|def| def.weight * stack.quantity as f32)
                    .unwrap_or(0.0)
            })
            .sum()
    }

    /// Equips `item_id` into the first compatible slot (in [`EquipmentSlotType::all`]
    /// iteration order). Returns the displaced item id if a slot was occupied,
    /// or `None` if the slot was empty.
    ///
    /// Returns [`EquipError::IncompatibleSlot`] if no slot accepts the item's
    /// category, and [`EquipError::SlotNotFound`] if the matching slot type is
    /// not present in this equipment set.
    pub fn equip_first_compatible(
        &mut self,
        item_id: ItemId,
        db: &ItemDatabase,
    ) -> Result<Option<ItemId>, EquipError> {
        for &slot_type in EquipmentSlotType::all() {
            if self.can_equip(slot_type, item_id, db) {
                let previous = self.equip(slot_type, item_id, db)?;
                return Ok(previous.map(|stack| stack.item_id));
            }
        }
        Err(EquipError::IncompatibleSlot)
    }
}

/// Errors that can occur during equip operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipError {
    /// No slot of the requested type exists in this equipment set.
    SlotNotFound,
    /// The item's category is not accepted by the target slot.
    IncompatibleSlot,
    /// The item to equip was not present in the source inventory.
    NotInInventory,
}

impl std::fmt::Display for EquipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SlotNotFound => write!(f, "no equipment slot of the requested type exists"),
            Self::IncompatibleSlot => {
                write!(f, "item category is not accepted by the target slot")
            }
            Self::NotInInventory => write!(f, "item is not present in the source inventory"),
        }
    }
}

impl std::error::Error for EquipError {}
