use crate::object::Object;

// ============================================================================
// State<T> - simple reactive-esque container for component state
// Stored as `Rc<RefCell<T>>` so we can cheaply clone and share between
// component instances and background tasks/handlers.
// ============================================================================

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct State<T> {
    inner: std::rc::Rc<std::cell::RefCell<T>>,
}

impl<T> State<T> {
    /// Create a new state wrapping the given value.
    pub fn new(value: T) -> Self {
        Self {
            inner: std::rc::Rc::new(std::cell::RefCell::new(value)),
        }
    }

    /// Get a cloned copy of the inner value (requires `T: Clone`).
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.borrow().clone()
    }

    /// Set the inner value.
    pub fn set(&self, value: T) {
        *self.inner.borrow_mut() = value;
    }

    /// Mutate the inner value via a closure.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut b = self.inner.borrow_mut();
        f(&mut *b);
    }

    /// Borrow the inner value immutably (returns a `Ref<T>`).
    pub fn borrow(&self) -> std::cell::Ref<'_, T> {
        self.inner.borrow()
    }

    /// Borrow the inner value mutably (returns a `RefMut<T>`).
    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, T> {
        self.inner.borrow_mut()
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
