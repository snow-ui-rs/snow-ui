use crate::object::Object;

// ============================================================================
// State<T> - simple reactive-esque container for component state
// Stored as `Rc<RefCell<T>>` so we can cheaply clone and share between
// component instances and background tasks/handlers.
// ============================================================================

#[derive(Debug, Clone)]
pub struct State<T> {
    inner: std::sync::Arc<std::sync::Mutex<T>>,
}

impl<T> State<T> {
    /// Create a new state wrapping the given value.
    pub fn new(value: T) -> Self {
        eprintln!("[snow-ui::state] State::<{}>::new()", std::any::type_name::<T>());
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(value)),
        }
    }

    /// Get a cloned copy of the inner value (requires `T: Clone`).
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        let value = self.inner.lock().unwrap().clone();
        eprintln!("[snow-ui::state] State::<{}>::get() -> value read", std::any::type_name::<T>());
        value
    }

    /// Set the inner value.
    pub fn set(&self, value: T) {
        eprintln!("[snow-ui::state] State::<{}>::set()", std::any::type_name::<T>());
        *self.inner.lock().unwrap() = value;
    }

    /// Mutate the inner value via a closure.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        eprintln!("[snow-ui::state] State::<{}>::update() begin", std::any::type_name::<T>());
        let mut b = self.inner.lock().unwrap();
        f(&mut *b);
        eprintln!("[snow-ui::state] State::<{}>::update() end", std::any::type_name::<T>());
    }

    /// Borrow the inner value immutably (returns a guard).
    pub fn borrow(&self) -> std::sync::MutexGuard<'_, T> {
        self.inner.lock().unwrap()
    }

    /// Borrow the inner value mutably (returns a guard).
    pub fn borrow_mut(&self) -> std::sync::MutexGuard<'_, T> {
        self.inner.lock().unwrap()
    }
}

impl<T: Default> Default for State<T> {
    fn default() -> Self {
        State::new(T::default())
    }
}

// Keep a live state handle in the object tree instead of converting to a one-time snapshot.
impl<T> From<State<T>> for Object
where
    T: Clone + std::fmt::Display + Send + Sync + 'static,
{
    fn from(s: State<T>) -> Self {
        let live = s.clone();
        Object::DynamicText {
            value: std::sync::Arc::new(move || live.get().to_string())
                as std::sync::Arc<dyn Fn() -> String + Send + Sync + 'static>,
        }
    }
}
