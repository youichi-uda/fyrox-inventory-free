use fyrox::core::{
    reflect::prelude::*, type_traits::prelude::*, uuid_provider, visitor::prelude::*,
};
use fyrox::resource::texture::TextureResource;
use std::fmt;
use strum_macros::{AsRefStr, EnumString, VariantNames};

/// Rarity level of an item, affects display color and sorting.
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
pub enum ItemRarity {
    #[default]
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

/// Category of an item, used for filtering and slot validation.
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
pub enum ItemCategory {
    #[default]
    Misc,
    Weapon,
    Armor,
    Helmet,
    Shield,
    Boots,
    Accessory,
    Consumable,
    Material,
    QuestItem,
}

/// Unique identifier for an item definition, used as a lightweight reference.
pub type ItemId = u64;

/// Static definition of an item type. This is the "blueprint" - instances reference this by id.
#[derive(Clone, Debug, Default, Visit, Reflect, ComponentProvider)]
pub struct ItemDefinition {
    /// Unique numeric identifier for this item definition.
    #[reflect(display_name = "Item ID")]
    pub id: ItemId,

    /// Human-readable name of the item.
    #[reflect(display_name = "Name")]
    pub name: String,

    /// Description shown in tooltips.
    #[reflect(display_name = "Description")]
    pub description: String,

    /// Icon texture displayed in inventory slots.
    #[reflect(display_name = "Icon")]
    pub icon: Option<TextureResource>,

    /// Rarity affects display color.
    #[reflect(display_name = "Rarity")]
    pub rarity: ItemRarity,

    /// Category determines which equipment slots accept this item.
    #[reflect(display_name = "Category")]
    pub category: ItemCategory,

    /// Maximum number of items in a single stack. 1 = non-stackable.
    #[reflect(display_name = "Max Stack Size", min_value = "1", max_value = "9999")]
    pub max_stack_size: u32,

    /// Weight of a single item unit.
    #[reflect(display_name = "Weight", min_value = "0.0")]
    pub weight: f32,

    /// Base value (for shop/trade systems).
    #[reflect(display_name = "Base Value", min_value = "0")]
    pub base_value: u32,

    /// Whether the item can be dropped/discarded by the player.
    #[reflect(display_name = "Can Drop")]
    pub can_drop: bool,

    /// Custom string tags for game-specific logic (e.g., "fire_element", "two_handed").
    #[reflect(display_name = "Tags")]
    pub tags: Vec<String>,
}

uuid_provider!(ItemDefinition = "f47ac10b-58cc-4372-a567-0e02b2c3d479");

impl fmt::Display for ItemDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.id, self.name)
    }
}

/// A runtime instance of an item in an inventory slot.
#[derive(Clone, Debug, Default, Visit, Reflect, ComponentProvider)]
pub struct ItemStack {
    /// References the item definition by its ID.
    #[reflect(display_name = "Item ID")]
    pub item_id: ItemId,

    /// Current quantity in this stack.
    #[reflect(display_name = "Quantity", min_value = "0", max_value = "9999")]
    pub quantity: u32,
}

uuid_provider!(ItemStack = "a1b2c3d4-e5f6-7890-abcd-ef1234567890");

impl ItemStack {
    /// Creates a new item stack.
    pub fn new(item_id: ItemId, quantity: u32) -> Self {
        Self { item_id, quantity }
    }

    /// Returns true if this stack is empty.
    pub fn is_empty(&self) -> bool {
        self.quantity == 0
    }

    /// Returns the number of items that can still be added to this stack given the max.
    pub fn remaining_capacity(&self, max_stack_size: u32) -> u32 {
        max_stack_size.saturating_sub(self.quantity)
    }
}
