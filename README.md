# fyrox-inventory-free

Core inventory system for the [Fyrox](https://fyrox.rs) game engine.

## Features

- Item definitions with rarity, category, weight, stacking, tags
- Item database resource (`.itemdb` files, RON format)
- Fixed-size inventory grid with stacking, splitting, merging, and multi-key sorting
  (id / name / rarity / category / value), plus category & tag queries
- Equipment system with slot type validation and auto-swap back to inventory
- `InventoryHolder` and `ItemPickup` scripts (interaction radius + auto-stack)
- Full `Visit` / `Reflect` / `serde` (`Serialize`/`Deserialize`) support

## Pro Version

[fyrox-inventory-pro](https://y1uda.itch.io/fyrox-inventory-pro) adds drag & drop UI widgets, inventory panels, item tooltips, an ItemDatabase editor GUI, and fyrox-savegame integration examples.

## License

MIT
