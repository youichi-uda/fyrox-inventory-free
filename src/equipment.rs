use crate::database::ItemDatabase;
use crate::item::{ItemCategory, ItemId, ItemStack};
use fyrox::core::{reflect::prelude::*, type_traits::prelude::*, uuid_provider, visitor::prelude::*};
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
#[derive(Clone, Debug, Default, Visit, Reflect, ComponentProvider)]
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
#[derive(Clone, Debug, Default, Visit, Reflect, ComponentProvider)]
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

    /// Unequips an item from the given slot. Returns the item that was there.
    pub fn unequip(&mut self, slot_type: EquipmentSlotType) -> Option<ItemStack> {
        self.find_slot_mut(slot_type)?.item.take()
    }

    /// Returns the item equipped in the given slot.
    pub fn get_equipped(&self, slot_type: EquipmentSlotType) -> Option<&ItemStack> {
        self.find_slot(slot_type)?.item.as_ref()
    }
}

/// Errors that can occur during equip operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EquipError {
    SlotNotFound,
    IncompatibleSlot,
}
