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
        Self {
            inner: std::sync::Arc::new(std::sync::Mutex::new(value)),
        }
    }

    /// Get a cloned copy of the inner value (requires `T: Clone`).
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.lock().unwrap().clone()
    }

    /// Set the inner value.
    pub fn set(&self, value: T) {
        *self.inner.lock().unwrap() = value;
    }

    /// Mutate the inner value via a closure.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut b = self.inner.lock().unwrap();
        f(&mut *b);
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

// Allow converting `State<T>` into an `Object` when the inner `T` can be converted.
impl<T> From<State<T>> for Object
where
    T: Clone + Into<Object>,
{
    fn from(s: State<T>) -> Self {
        s.get().into()
    }
}
