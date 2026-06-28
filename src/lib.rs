use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::RwLock;
use std::any::Any;

pub struct DataRegistry {
    stores: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,
}

impl DataRegistry {
    pub fn new() -> Self {
        Self {
            stores: RwLock::new(HashMap::new()),
        }
    }

    pub fn register_store<T: Send + Sync + 'static>(&self, name: &str, store: T) {
        let mut stores = self.stores.write().unwrap();
        stores.insert(name.to_string(), Box::new(store));
    }
}

impl Default for DataRegistry {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static! {
    pub static ref REGISTRY: DataRegistry = DataRegistry::new();
}

pub fn dispatch<F, T>(store_name: &str, action: F)
where
    F: FnOnce(&mut T),
    T: 'static,
{
    let mut stores = REGISTRY.stores.write().unwrap();
    if let Some(store) = stores.get_mut(store_name) {
        if let Some(typed_store) = store.downcast_mut::<T>() {
            action(typed_store);
        }
    }
}

pub fn select<F, T, R>(store_name: &str, selector: F) -> Option<R>
where
    F: FnOnce(&T) -> R,
    T: 'static,
{
    let stores = REGISTRY.stores.read().unwrap();
    if let Some(store) = stores.get(store_name) {
        if let Some(typed_store) = store.downcast_ref::<T>() {
            return Some(selector(typed_store));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MyStore {
        count: i32,
    }

    #[test]
    fn test_registry() {
        REGISTRY.register_store("my_store", MyStore { count: 0 });

        let count = select("my_store", |store: &MyStore| store.count);
        assert_eq!(count, Some(0));

        dispatch("my_store", |store: &mut MyStore| {
            store.count += 1;
        });

        let count = select("my_store", |store: &MyStore| store.count);
        assert_eq!(count, Some(1));
    }
}
