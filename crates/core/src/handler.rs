// ============================================================================
// Inventory-based handler registration system
// ============================================================================

/// A handler registry entry collected at compile time via `inventory`.
/// Each entry knows how to register its handler for a specific (Element, Message) pair.
pub struct HandlerRegistryEntry {
    /// TypeId of the element type this handler is for
    pub element_type_id: fn() -> std::any::TypeId,
    /// Registers the handler onto the given `Rc<RefCell<dyn Any>>` element instance.
    /// Returns `true` if the element was the expected type and registration succeeded.
    pub register_fn: fn(&std::rc::Rc<std::cell::RefCell<dyn std::any::Any>>),
}

inventory::collect!(HandlerRegistryEntry);

/// Register all handlers for a given element instance using the inventory.
/// This is called from the generated `into_object()` method.
pub fn register_handlers_for_instance<T: 'static>(instance: &std::rc::Rc<std::cell::RefCell<T>>) {
    let target_type_id = std::any::TypeId::of::<T>();
    // Create an Rc<RefCell<dyn Any>> from the instance for type-erased registration
    let any_rc: std::rc::Rc<std::cell::RefCell<dyn std::any::Any>> = instance.clone();

    for entry in inventory::iter::<HandlerRegistryEntry> {
        if (entry.element_type_id)() == target_type_id {
            (entry.register_fn)(&any_rc);
        }
    }
}

/// Check if there are any registered handlers for a given element type.
pub fn has_registered_handlers<T: 'static>() -> bool {
    let target_type_id = std::any::TypeId::of::<T>();
    for entry in inventory::iter::<HandlerRegistryEntry> {
        if (entry.element_type_id)() == target_type_id {
            return true;
        }
    }
    false
}
