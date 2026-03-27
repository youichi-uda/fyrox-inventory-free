use crate::item::{ItemDefinition, ItemId};
use fyrox::core::{
    reflect::prelude::*, type_traits::prelude::*, uuid_provider, visitor::prelude::*,
};
use fyrox::resource::{
    io::ResourceIo,
    loader::{BoxedLoaderFuture, LoaderPayload, ResourceLoader},
    state::LoadError,
    ResourceData,
};
use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

/// A database of item definitions. Stored as a Fyrox resource (.itemdb files).
#[derive(Clone, Debug, Default, Visit, Reflect, ComponentProvider)]
pub struct ItemDatabase {
    /// All item definitions in this database.
    #[reflect(display_name = "Items")]
    pub items: Vec<ItemDefinition>,
}

uuid_provider!(ItemDatabase = "e5f6a7b8-c9d0-1234-efab-345678901234");

impl ItemDatabase {
    /// Looks up an item definition by its ID.
    pub fn get(&self, id: ItemId) -> Option<&ItemDefinition> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Looks up an item definition by its name.
    pub fn get_by_name(&self, name: &str) -> Option<&ItemDefinition> {
        self.items.iter().find(|item| item.name == name)
    }

    /// Adds an item definition. Returns false if an item with the same ID already exists.
    pub fn add(&mut self, item: ItemDefinition) -> bool {
        if self.items.iter().any(|i| i.id == item.id) {
            return false;
        }
        self.items.push(item);
        true
    }

    /// Removes an item definition by ID. Returns the removed item, if found.
    pub fn remove(&mut self, id: ItemId) -> Option<ItemDefinition> {
        let pos = self.items.iter().position(|i| i.id == id)?;
        Some(self.items.remove(pos))
    }

    /// Returns the next available item ID.
    pub fn next_id(&self) -> ItemId {
        self.items.iter().map(|i| i.id).max().unwrap_or(0) + 1
    }

    /// Returns an iterator over all item definitions.
    pub fn iter(&self) -> impl Iterator<Item = &ItemDefinition> {
        self.items.iter()
    }

    /// Returns the number of item definitions.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns true if the database is empty.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl ResourceData for ItemDatabase {
    fn type_uuid(&self) -> uuid::Uuid {
        <Self as TypeUuidProvider>::type_uuid()
    }

    fn save(&mut self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut visitor = Visitor::new();
        self.visit("ItemDatabase", &mut visitor)?;
        visitor.save_ascii_to_file(path)?;
        Ok(())
    }

    fn can_be_saved(&self) -> bool {
        true
    }

    fn try_clone_box(&self) -> Option<Box<dyn ResourceData>> {
        Some(Box::new(self.clone()))
    }
}

/// Error type for item database loading.
#[derive(Debug)]
pub enum ItemDatabaseLoadError {
    Visit(VisitError),
    Io(String),
}

impl fmt::Display for ItemDatabaseLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Visit(e) => write!(f, "Visit error: {:?}", e),
            Self::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl Error for ItemDatabaseLoadError {}

/// Resource loader for `.itemdb` files.
pub struct ItemDatabaseLoader;

impl ResourceLoader for ItemDatabaseLoader {
    fn extensions(&self) -> &[&str] {
        &["itemdb"]
    }

    fn data_type_uuid(&self) -> uuid::Uuid {
        <ItemDatabase as fyrox_core::TypeUuidProvider>::type_uuid()
    }

    fn load(&self, path: PathBuf, io: Arc<dyn ResourceIo>) -> BoxedLoaderFuture {
        Box::pin(async move {
            let data = io
                .load_file(&path)
                .await
                .map_err(|e| LoadError::new(ItemDatabaseLoadError::Io(e.to_string())))?;

            let mut visitor = Visitor::load_from_memory(&data)
                .map_err(|e| LoadError::new(ItemDatabaseLoadError::Visit(e)))?;

            let mut database = ItemDatabase::default();
            database
                .visit("ItemDatabase", &mut visitor)
                .map_err(|e| LoadError::new(ItemDatabaseLoadError::Visit(e)))?;

            Ok(LoaderPayload::new(database))
        })
    }
}
