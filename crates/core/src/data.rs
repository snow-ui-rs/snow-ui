use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::state::State;

type SharedValue = Arc<dyn Any + Send + Sync>;

/// A cloneable collection of globally shared, typed state values.
#[derive(Clone, Default)]
pub struct Data {
    values: Arc<Mutex<HashMap<String, SharedValue>>>,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Data")
            .field("len", &self.values.lock().unwrap().len())
            .finish()
    }
}

impl Data {
    /// Insert or replace a value and return its shared state handle.
    pub fn insert<T>(&self, key: impl Into<String>, value: T) -> State<T>
    where
        T: Send + Sync + 'static,
    {
        let state = State::new(value);
        self.values
            .lock()
            .unwrap()
            .insert(key.into(), Arc::new(state.clone()));
        state
    }

    /// Get a typed shared state value. Returns `None` if the key is absent or has another type.
    pub fn get<T>(&self, key: &str) -> Option<State<T>>
    where
        T: Send + Sync + 'static,
    {
        let value = self.values.lock().unwrap().get(key)?.clone();
        Arc::downcast::<State<T>>(value)
            .ok()
            .map(|state| (*state).clone())
    }

    /// Get or initialize a typed value, creating `T::default()` when the key is absent.
    pub fn get_or_insert_default<T>(
        &self,
        key: impl Into<String>,
    ) -> Result<State<T>, DataTypeMismatch>
    where
        T: Default + Send + Sync + 'static,
    {
        self.get_or_insert_with(key, T::default)
    }

    /// Get or initialize a typed value using a custom default factory.
    pub fn get_or_insert_with<T, F>(
        &self,
        key: impl Into<String>,
        default: F,
    ) -> Result<State<T>, DataTypeMismatch>
    where
        T: Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        let key = key.into();
        let mut values = self.values.lock().unwrap();
        if let Some(value) = values.get(&key) {
            return Arc::downcast::<State<T>>(value.clone())
                .map(|state| (*state).clone())
                .map_err(|_| DataTypeMismatch {
                    key,
                    expected: std::any::type_name::<T>(),
                });
        }

        let state = State::new(default());
        values.insert(key, Arc::new(state.clone()));
        Ok(state)
    }

    /// Remove a key. Handles already returned for that key remain valid.
    pub fn remove(&self, key: &str) -> bool {
        self.values.lock().unwrap().remove(key).is_some()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.lock().unwrap().contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.values.lock().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.lock().unwrap().is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataTypeMismatch {
    pub key: String,
    pub expected: &'static str,
}

impl std::fmt::Display for DataTypeMismatch {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "global data key {:?} does not contain type {}",
            self.key, self.expected
        )
    }
}

impl std::error::Error for DataTypeMismatch {}

#[cfg(test)]
mod tests {
    use super::Data;

    #[test]
    fn values_are_shared_between_clones_and_default_on_demand() {
        let data = Data::default();
        let clone = data.clone();
        let count = data.get_or_insert_default::<u32>("count").unwrap();
        count.set(4);

        assert_eq!(clone.get::<u32>("count").unwrap().get(), 4);
        assert_eq!(
            clone.get_or_insert_default::<u32>("count").unwrap().get(),
            4
        );
    }

    #[test]
    fn custom_defaults_and_type_mismatches_are_explicit() {
        let data = Data::default();
        let name = data
            .get_or_insert_with("name", || String::from("guest"))
            .unwrap();
        assert_eq!(name.get(), "guest");
        assert!(data.get_or_insert_default::<u32>("name").is_err());
    }
}
