use crate::database::ItemDatabase;
use crate::item::{ItemId, ItemStack};
use fyrox::core::{reflect::prelude::*, type_traits::prelude::*, uuid_provider, visitor::prelude::*};

/// Result of an inventory operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InventoryResult {
    /// Operation succeeded fully.
    Success,
    /// Partially added - returns the leftover quantity that didn't fit.
    Partial { leftover: u32 },
    /// Item not found in the database.
    ItemNotFound,
    /// Slot index out of range.
    SlotOutOfRange,
    /// Not enough items to remove.
    InsufficientQuantity,
    /// Slot is already occupied (for non-stackable operations).
    SlotOccupied,
}

/// A fixed-size inventory grid. Each slot can hold one `ItemStack` or be empty.
#[derive(Clone, Debug, Default, Visit, Reflect, ComponentProvider)]
pub struct Inventory {
    /// The slots in this inventory. `None` means the slot is empty.
    #[reflect(display_name = "Slots")]
    pub slots: Vec<Option<ItemStack>>,

    /// Number of columns for UI layout.
    #[reflect(display_name = "Columns", min_value = "1", max_value = "20")]
    pub columns: u32,
}

uuid_provider!(Inventory = "b2c3d4e5-f6a7-8901-bcde-f12345678901");

impl Inventory {
    /// Creates a new inventory with the given number of slots.
    pub fn new(slot_count: usize, columns: u32) -> Self {
        Self {
            slots: vec![None; slot_count],
            columns: columns.max(1),
        }
    }

    /// Returns the number of slots.
    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    /// Returns the number of rows based on columns.
    pub fn rows(&self) -> usize {
        let cols = self.columns.max(1) as usize;
        (self.slots.len() + cols - 1) / cols
    }

    /// Adds items to the inventory, filling existing stacks first, then empty slots.
    /// Returns the number of items that could NOT be added (0 = all added).
    pub fn add_item(&mut self, item_id: ItemId, mut quantity: u32, db: &ItemDatabase) -> u32 {
        let max_stack = match db.get(item_id) {
            Some(def) => def.max_stack_size.max(1),
            None => return quantity,
        };

        // First pass: fill existing stacks of the same item.
        for slot in self.slots.iter_mut() {
            if quantity == 0 {
                break;
            }
            if let Some(stack) = slot {
                if stack.item_id == item_id {
                    let can_add = stack.remaining_capacity(max_stack).min(quantity);
                    stack.quantity += can_add;
                    quantity -= can_add;
                }
            }
        }

        // Second pass: place in empty slots.
        for slot in self.slots.iter_mut() {
            if quantity == 0 {
                break;
            }
            if slot.is_none() {
                let stack_qty = quantity.min(max_stack);
                *slot = Some(ItemStack::new(item_id, stack_qty));
                quantity -= stack_qty;
            }
        }

        quantity
    }

    /// Removes a quantity of the given item from the inventory.
    /// Returns `InventoryResult::Success` if all removed, `InsufficientQuantity` if not enough.
    pub fn remove_item(&mut self, item_id: ItemId, mut quantity: u32) -> InventoryResult {
        let total = self.count_item(item_id);
        if total < quantity {
            return InventoryResult::InsufficientQuantity;
        }

        for slot in self.slots.iter_mut() {
            if quantity == 0 {
                break;
            }
            if let Some(stack) = slot {
                if stack.item_id == item_id {
                    let remove = stack.quantity.min(quantity);
                    stack.quantity -= remove;
                    quantity -= remove;
                    if stack.quantity == 0 {
                        *slot = None;
                    }
                }
            }
        }

        InventoryResult::Success
    }

    /// Removes all items from a specific slot, returning what was there.
    pub fn take_slot(&mut self, index: usize) -> Option<ItemStack> {
        self.slots.get_mut(index)?.take()
    }

    /// Sets a slot to a given item stack (replacing whatever was there).
    pub fn set_slot(&mut self, index: usize, stack: Option<ItemStack>) -> InventoryResult {
        if index >= self.slots.len() {
            return InventoryResult::SlotOutOfRange;
        }
        self.slots[index] = stack;
        InventoryResult::Success
    }

    /// Swaps two slots.
    pub fn swap_slots(&mut self, a: usize, b: usize) -> InventoryResult {
        if a >= self.slots.len() || b >= self.slots.len() {
            return InventoryResult::SlotOutOfRange;
        }
        self.slots.swap(a, b);
        InventoryResult::Success
    }

    /// Splits a stack: takes `quantity` items from `src` and places them in `dst` (must be empty).
    pub fn split_stack(&mut self, src: usize, dst: usize, quantity: u32) -> InventoryResult {
        if src >= self.slots.len() || dst >= self.slots.len() {
            return InventoryResult::SlotOutOfRange;
        }
        if self.slots[dst].is_some() {
            return InventoryResult::SlotOccupied;
        }
        if let Some(ref mut stack) = self.slots[src] {
            if stack.quantity < quantity {
                return InventoryResult::InsufficientQuantity;
            }
            let item_id = stack.item_id;
            stack.quantity -= quantity;
            if stack.quantity == 0 {
                self.slots[src] = None;
            }
            self.slots[dst] = Some(ItemStack::new(item_id, quantity));
            InventoryResult::Success
        } else {
            InventoryResult::InsufficientQuantity
        }
    }

    /// Merges stack from `src` into `dst` (if same item). Returns leftover in src.
    pub fn merge_stacks(&mut self, src: usize, dst: usize, db: &ItemDatabase) -> InventoryResult {
        if src >= self.slots.len() || dst >= self.slots.len() {
            return InventoryResult::SlotOutOfRange;
        }

        let (src_id, src_qty) = match &self.slots[src] {
            Some(s) => (s.item_id, s.quantity),
            None => return InventoryResult::Success,
        };

        let max_stack = match db.get(src_id) {
            Some(def) => def.max_stack_size.max(1),
            None => return InventoryResult::ItemNotFound,
        };

        if let Some(ref mut dst_stack) = self.slots[dst] {
            if dst_stack.item_id != src_id {
                // Different items: swap instead.
                self.slots.swap(src, dst);
                return InventoryResult::Success;
            }
            let can_move = dst_stack.remaining_capacity(max_stack).min(src_qty);
            dst_stack.quantity += can_move;
            let remaining = src_qty - can_move;
            if remaining == 0 {
                self.slots[src] = None;
            } else if let Some(ref mut s) = self.slots[src] {
                s.quantity = remaining;
            }
        } else {
            // dst is empty: just move.
            self.slots[dst] = self.slots[src].take();
        }

        InventoryResult::Success
    }

    /// Counts total quantity of a given item across all slots.
    pub fn count_item(&self, item_id: ItemId) -> u32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .filter(|s| s.item_id == item_id)
            .map(|s| s.quantity)
            .sum()
    }

    /// Returns true if the inventory contains at least `quantity` of the given item.
    pub fn has_item(&self, item_id: ItemId, quantity: u32) -> bool {
        self.count_item(item_id) >= quantity
    }

    /// Returns the index of the first empty slot, or None if full.
    pub fn first_empty_slot(&self) -> Option<usize> {
        self.slots.iter().position(|s| s.is_none())
    }

    /// Returns true if the inventory is completely full.
    pub fn is_full(&self) -> bool {
        self.slots.iter().all(|s| s.is_some())
    }

    /// Sorts the inventory by item_id, grouping same items together and merging stacks.
    pub fn sort(&mut self, db: &ItemDatabase) {
        // Collect all items.
        let mut items: Vec<(ItemId, u32)> = Vec::new();
        for slot in self.slots.iter() {
            if let Some(stack) = slot {
                if let Some(entry) = items.iter_mut().find(|(id, _)| *id == stack.item_id) {
                    entry.1 += stack.quantity;
                } else {
                    items.push((stack.item_id, stack.quantity));
                }
            }
        }

        // Sort by item_id.
        items.sort_by_key(|(id, _)| *id);

        // Clear and refill.
        for slot in self.slots.iter_mut() {
            *slot = None;
        }

        let mut slot_idx = 0;
        for (item_id, mut total) in items {
            let max_stack = db
                .get(item_id)
                .map(|d| d.max_stack_size.max(1))
                .unwrap_or(1);
            while total > 0 && slot_idx < self.slots.len() {
                let qty = total.min(max_stack);
                self.slots[slot_idx] = Some(ItemStack::new(item_id, qty));
                total -= qty;
                slot_idx += 1;
            }
        }
    }

    /// Calculates total weight of all items in the inventory.
    pub fn total_weight(&self, db: &ItemDatabase) -> f32 {
        self.slots
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|stack| {
                db.get(stack.item_id)
                    .map(|def| def.weight * stack.quantity as f32)
                    .unwrap_or(0.0)
            })
            .sum()
    }
}
