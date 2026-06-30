//! Integration tests for the fyrox-inventory-free core API.

use fyrox_inventory_free::database::ItemDatabase;
use fyrox_inventory_free::equipment::{Equipment, EquipError, EquipmentSlotType};
use fyrox_inventory_free::inventory::{Inventory, InventoryResult, SortKey};
use fyrox_inventory_free::item::{ItemCategory, ItemDefinition, ItemId, ItemRarity, ItemStack};

/// Builds a small test database with a few well-known items.
fn test_db() -> ItemDatabase {
    let mut db = ItemDatabase::default();
    db.add(ItemDefinition {
        id: 1,
        name: "Iron Sword".into(),
        category: ItemCategory::Weapon,
        rarity: ItemRarity::Uncommon,
        max_stack_size: 1,
        weight: 3.5,
        base_value: 150,
        tags: vec!["melee".into(), "metal".into()],
        ..Default::default()
    });
    db.add(ItemDefinition {
        id: 2,
        name: "Health Potion".into(),
        category: ItemCategory::Consumable,
        rarity: ItemRarity::Common,
        max_stack_size: 10,
        weight: 0.5,
        base_value: 25,
        tags: vec!["healing".into()],
        ..Default::default()
    });
    db.add(ItemDefinition {
        id: 3,
        name: "Dragon Shield".into(),
        category: ItemCategory::Shield,
        rarity: ItemRarity::Legendary,
        max_stack_size: 1,
        weight: 8.0,
        base_value: 5000,
        ..Default::default()
    });
    db.add(ItemDefinition {
        id: 4,
        name: "Iron Helmet".into(),
        category: ItemCategory::Helmet,
        rarity: ItemRarity::Common,
        max_stack_size: 1,
        weight: 2.0,
        base_value: 90,
        ..Default::default()
    });
    db
}

// ─── ItemDatabase ────────────────────────────────────────────────────────────

#[test]
fn database_add_get() {
    let db = test_db();
    assert_eq!(db.len(), 4);
    assert!(!db.is_empty());
    assert_eq!(db.get(1).unwrap().name, "Iron Sword");
    assert_eq!(db.get_by_name("Health Potion").unwrap().id, 2);
    assert!(db.get(999).is_none());
}

#[test]
fn database_add_duplicate_rejected() {
    let mut db = test_db();
    let dup = ItemDefinition {
        id: 1,
        name: "Copy".into(),
        ..Default::default()
    };
    assert!(!db.add(dup));
    assert_eq!(db.len(), 4);
}

#[test]
fn database_remove_and_next_id() {
    let mut db = test_db();
    assert_eq!(db.next_id(), 5);
    let removed = db.remove(2).unwrap();
    assert_eq!(removed.name, "Health Potion");
    assert!(db.get(2).is_none());
    assert_eq!(db.len(), 3);
}

// ─── Inventory: add / remove / stacking ──────────────────────────────────────

#[test]
fn inventory_add_into_empty() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    let leftover = inv.add_item(1, 1, &db);
    assert_eq!(leftover, 0);
    assert_eq!(inv.count_item(1), 1);
}

#[test]
fn inventory_add_unknown_item_returns_all() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    assert_eq!(inv.add_item(999, 3, &db), 3);
    assert_eq!(inv.count_item(999), 0);
}

#[test]
fn inventory_stacking_fills_existing_first() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    // max_stack of potion is 10.
    inv.add_item(2, 6, &db);
    inv.add_item(2, 3, &db);
    assert_eq!(inv.count_item(2), 9);
    // Should occupy exactly one slot.
    let occupied = inv.slots.iter().filter(|s| s.is_some()).count();
    assert_eq!(occupied, 1);
}

#[test]
fn inventory_stack_capacity_overflow_spills_to_new_slot() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    // 25 potions, max stack 10 → 10 + 10 + 5 across three slots.
    let leftover = inv.add_item(2, 25, &db);
    assert_eq!(leftover, 0);
    assert_eq!(inv.count_item(2), 25);
    let occupied = inv.slots.iter().filter(|s| s.is_some()).count();
    assert_eq!(occupied, 3);
}

#[test]
fn inventory_full_returns_leftover() {
    let db = test_db();
    let mut inv = Inventory::new(2, 2);
    // Two non-stackable swords use both slots; third has nowhere to go.
    assert_eq!(inv.add_item(1, 1, &db), 0);
    assert_eq!(inv.add_item(1, 1, &db), 0);
    assert!(inv.is_full());
    assert_eq!(inv.add_item(1, 1, &db), 1);
}

#[test]
fn inventory_remove() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    inv.add_item(2, 8, &db);
    assert_eq!(inv.remove_item(2, 5), InventoryResult::Success);
    assert_eq!(inv.count_item(2), 3);
    assert_eq!(
        inv.remove_item(2, 99),
        InventoryResult::InsufficientQuantity
    );
    assert_eq!(inv.remove_item(2, 3), InventoryResult::Success);
    assert_eq!(inv.count_item(2), 0);
    assert!(inv.first_empty_slot().is_some());
}

#[test]
fn inventory_has_item() {
    let db = test_db();
    let mut inv = Inventory::new(5, 5);
    inv.add_item(2, 4, &db);
    assert!(inv.has_item(2, 4));
    assert!(!inv.has_item(2, 5));
}

// ─── Inventory: split / merge / swap ─────────────────────────────────────────

#[test]
fn inventory_split_stack() {
    let mut inv = Inventory::new(5, 5);
    inv.set_slot(0, Some(ItemStack::new(2, 10)));
    assert_eq!(inv.split_stack(0, 1, 4), InventoryResult::Success);
    assert_eq!(inv.slots[0].as_ref().unwrap().quantity, 6);
    assert_eq!(inv.slots[1].as_ref().unwrap().quantity, 4);
}

#[test]
fn inventory_split_into_occupied_fails() {
    let mut inv = Inventory::new(5, 5);
    inv.set_slot(0, Some(ItemStack::new(2, 10)));
    inv.set_slot(1, Some(ItemStack::new(2, 1)));
    assert_eq!(inv.split_stack(0, 1, 4), InventoryResult::SlotOccupied);
}

#[test]
fn inventory_split_too_many_fails() {
    let mut inv = Inventory::new(5, 5);
    inv.set_slot(0, Some(ItemStack::new(2, 3)));
    assert_eq!(
        inv.split_stack(0, 1, 5),
        InventoryResult::InsufficientQuantity
    );
}

#[test]
fn inventory_merge_same_item_respects_capacity() {
    let db = test_db();
    let mut inv = Inventory::new(5, 5);
    inv.set_slot(0, Some(ItemStack::new(2, 7))); // src
    inv.set_slot(1, Some(ItemStack::new(2, 8))); // dst, capacity 10
    assert_eq!(inv.merge_stacks(0, 1, &db), InventoryResult::Success);
    // dst fills to 10, src keeps 5.
    assert_eq!(inv.slots[1].as_ref().unwrap().quantity, 10);
    assert_eq!(inv.slots[0].as_ref().unwrap().quantity, 5);
}

#[test]
fn inventory_merge_full_clears_source() {
    let db = test_db();
    let mut inv = Inventory::new(5, 5);
    inv.set_slot(0, Some(ItemStack::new(2, 3)));
    inv.set_slot(1, Some(ItemStack::new(2, 4)));
    assert_eq!(inv.merge_stacks(0, 1, &db), InventoryResult::Success);
    assert_eq!(inv.slots[1].as_ref().unwrap().quantity, 7);
    assert!(inv.slots[0].is_none());
}

#[test]
fn inventory_merge_different_items_swaps() {
    let db = test_db();
    let mut inv = Inventory::new(5, 5);
    inv.set_slot(0, Some(ItemStack::new(1, 1)));
    inv.set_slot(1, Some(ItemStack::new(2, 2)));
    assert_eq!(inv.merge_stacks(0, 1, &db), InventoryResult::Success);
    assert_eq!(inv.slots[0].as_ref().unwrap().item_id, 2);
    assert_eq!(inv.slots[1].as_ref().unwrap().item_id, 1);
}

#[test]
fn inventory_swap_out_of_range() {
    let mut inv = Inventory::new(3, 3);
    assert_eq!(inv.swap_slots(0, 99), InventoryResult::SlotOutOfRange);
}

// ─── Inventory: sorting ──────────────────────────────────────────────────────

#[test]
fn inventory_sort_by_id_merges() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    inv.set_slot(3, Some(ItemStack::new(2, 5)));
    inv.set_slot(7, Some(ItemStack::new(1, 1)));
    inv.set_slot(1, Some(ItemStack::new(2, 4)));
    inv.sort(&db);
    // After sort by id: slot 0 = item 1, slots after = merged item 2 (9 total).
    assert_eq!(inv.slots[0].as_ref().unwrap().item_id, 1);
    assert_eq!(inv.count_item(2), 9);
    let occupied_for_2 = inv
        .slots
        .iter()
        .filter(|s| s.as_ref().map(|st| st.item_id == 2).unwrap_or(false))
        .count();
    assert_eq!(occupied_for_2, 1);
}

#[test]
fn inventory_sort_by_rarity_legendary_first() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    inv.set_slot(0, Some(ItemStack::new(2, 1))); // Common
    inv.set_slot(1, Some(ItemStack::new(3, 1))); // Legendary
    inv.set_slot(2, Some(ItemStack::new(1, 1))); // Uncommon
    inv.sort_by_rarity(&db);
    assert_eq!(inv.slots[0].as_ref().unwrap().item_id, 3); // Legendary first
    assert_eq!(inv.slots[1].as_ref().unwrap().item_id, 1); // Uncommon
    assert_eq!(inv.slots[2].as_ref().unwrap().item_id, 2); // Common
}

#[test]
fn inventory_sort_by_value_and_name() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    inv.set_slot(0, Some(ItemStack::new(2, 1))); // value 25
    inv.set_slot(1, Some(ItemStack::new(3, 1))); // value 5000
    inv.set_slot(2, Some(ItemStack::new(1, 1))); // value 150
    inv.sort_by(SortKey::Value, &db);
    assert_eq!(inv.slots[0].as_ref().unwrap().item_id, 3);
    assert_eq!(inv.slots[1].as_ref().unwrap().item_id, 1);
    assert_eq!(inv.slots[2].as_ref().unwrap().item_id, 2);

    inv.sort_by(SortKey::Name, &db);
    // Alphabetical: Dragon Shield, Health Potion, Iron Sword.
    assert_eq!(inv.slots[0].as_ref().unwrap().item_id, 3);
    assert_eq!(inv.slots[1].as_ref().unwrap().item_id, 2);
    assert_eq!(inv.slots[2].as_ref().unwrap().item_id, 1);
}

// ─── Inventory: weight & queries ─────────────────────────────────────────────

#[test]
fn inventory_total_weight() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    inv.add_item(1, 2, &db); // 2 * 3.5 = 7.0
    inv.add_item(2, 4, &db); // 4 * 0.5 = 2.0
    let w = inv.total_weight(&db);
    assert!((w - 9.0).abs() < 1e-4, "weight was {w}");
}

#[test]
fn inventory_query_by_category_and_tag() {
    let db = test_db();
    let mut inv = Inventory::new(10, 5);
    inv.add_item(1, 1, &db); // Weapon, tags melee/metal
    inv.add_item(2, 3, &db); // Consumable, tag healing
    inv.add_item(4, 1, &db); // Helmet

    let weapons = inv.slots_in_category(ItemCategory::Weapon, &db);
    assert_eq!(weapons.len(), 1);
    assert_eq!(inv.count_in_category(ItemCategory::Consumable, &db), 3);

    let healing = inv.slots_with_tag("healing", &db);
    assert_eq!(healing.len(), 1);
    assert!(inv.slots_with_tag("nonexistent", &db).is_empty());
}

// ─── Equipment ───────────────────────────────────────────────────────────────

#[test]
fn equipment_standard_has_eight_slots() {
    let eq = Equipment::standard();
    assert_eq!(eq.slots.len(), 8);
    assert!(eq.find_slot(EquipmentSlotType::MainHand).is_some());
}

#[test]
fn equipment_can_equip_validates_category() {
    let db = test_db();
    let eq = Equipment::standard();
    assert!(eq.can_equip(EquipmentSlotType::MainHand, 1, &db)); // sword → main hand
    assert!(!eq.can_equip(EquipmentSlotType::Head, 1, &db)); // sword → head: no
    assert!(eq.can_equip(EquipmentSlotType::Head, 4, &db)); // helmet → head
    assert!(eq.can_equip(EquipmentSlotType::OffHand, 3, &db)); // shield → off hand
}

#[test]
fn equipment_equip_returns_previous() {
    let db = test_db();
    let mut eq = Equipment::standard();
    assert_eq!(eq.equip(EquipmentSlotType::Head, 4, &db), Ok(None));
    // Equip another helmet-category item is not in db; reuse same id to test swap.
    let prev = eq.equip(EquipmentSlotType::Head, 4, &db).unwrap();
    assert_eq!(prev.unwrap().item_id, 4);
    assert_eq!(
        eq.get_equipped(EquipmentSlotType::Head).unwrap().item_id,
        4
    );
}

#[test]
fn equipment_equip_incompatible_errors() {
    let db = test_db();
    let mut eq = Equipment::standard();
    assert_eq!(
        eq.equip(EquipmentSlotType::Head, 1, &db),
        Err(EquipError::IncompatibleSlot)
    );
}

#[test]
fn equipment_unequip() {
    let db = test_db();
    let mut eq = Equipment::standard();
    eq.equip(EquipmentSlotType::MainHand, 1, &db).unwrap();
    let removed = eq.unequip(EquipmentSlotType::MainHand);
    assert_eq!(removed.unwrap().item_id, 1);
    assert!(eq.get_equipped(EquipmentSlotType::MainHand).is_none());
}

#[test]
fn equipment_equip_from_inventory_swaps_back() {
    let db = test_db();
    let mut eq = Equipment::standard();
    let mut inv = Inventory::new(10, 5);
    inv.add_item(4, 1, &db); // helmet id 4

    // Equip helmet from inventory.
    eq.equip_from_inventory(EquipmentSlotType::Head, 4, &mut inv, &db)
        .unwrap();
    assert_eq!(eq.get_equipped(EquipmentSlotType::Head).unwrap().item_id, 4);
    assert_eq!(inv.count_item(4), 0);

    // Trying to equip an item not in the inventory fails.
    assert_eq!(
        eq.equip_from_inventory(EquipmentSlotType::MainHand, 1, &mut inv, &db),
        Err(EquipError::NotInInventory)
    );
}

// ─── Inventory: drop / clear ─────────────────────────────────────────────────

#[test]
fn inventory_drop_partial_keeps_remainder() {
    let db = test_db();
    let mut inv = Inventory::new(5, 5);
    inv.add_item(2, 8, &db); // 8 potions in slot 0
    let dropped = inv.drop_item(0, 3).expect("partial drop returns stack");
    assert_eq!(dropped.item_id, 2);
    assert_eq!(dropped.quantity, 3);
    assert_eq!(inv.count_item(2), 5);
    assert_eq!(inv.slots[0].as_ref().unwrap().quantity, 5);
}

#[test]
fn inventory_drop_all_and_clear() {
    let db = test_db();
    let mut inv = Inventory::new(5, 5);
    inv.add_item(2, 4, &db);
    inv.add_item(1, 1, &db);

    // qty >= stack quantity empties the slot.
    let dropped = inv.drop_item(0, 99).expect("drop returns full stack");
    assert_eq!(dropped.quantity, 4);
    assert!(inv.slots[0].is_none());

    // Out-of-range slot and qty=0 both return None without mutating.
    assert!(inv.drop_item(99, 1).is_none());
    assert!(inv.drop_item(1, 0).is_none());
    assert_eq!(inv.count_item(1), 1);

    // clear wipes everything.
    inv.clear();
    assert!(inv.slots.iter().all(|s| s.is_none()));
    assert_eq!(inv.count_item(1), 0);
}

// ─── serde round-trip ────────────────────────────────────────────────────────

#[test]
fn serde_inventory_roundtrip_ron() {
    let db = test_db();
    let mut inv = Inventory::new(6, 3);
    inv.add_item(2, 12, &db);
    let s = ron::to_string(&inv).unwrap();
    let back: Inventory = ron::from_str(&s).unwrap();
    assert_eq!(back.count_item(2), 12);
    assert_eq!(back.slot_count(), 6);
}

#[test]
fn serde_database_roundtrip_ron() {
    let db = test_db();
    let s = ron::to_string(&db).unwrap();
    let back: ItemDatabase = ron::from_str(&s).unwrap();
    assert_eq!(back.len(), 4);
    assert_eq!(back.get_by_name("Dragon Shield").unwrap().base_value, 5000);
}

// ─── ItemStack helpers ───────────────────────────────────────────────────────

#[test]
fn item_stack_capacity_helpers() {
    let stack = ItemStack::new(1, 3);
    assert!(!stack.is_empty());
    assert_eq!(stack.remaining_capacity(10), 7);
    let empty: ItemStack = ItemStack::new(1, 0);
    assert!(empty.is_empty());
    let _: ItemId = 5; // ItemId type is publicly usable.
}
