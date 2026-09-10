use std::sync::{Arc, atomic::AtomicBool};

use crate::object::Object;

#[derive(Debug, Clone)]
pub struct IntervalTimer<E> {
    pub interval: std::time::Duration,
    started: Arc<AtomicBool>,
    _marker: std::marker::PhantomData<E>,
}

impl<E> IntervalTimer<E> {
    pub fn from_interval(interval: std::time::Duration) -> Self {
        Self {
            interval,
            started: Arc::new(AtomicBool::new(false)),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<E> Default for IntervalTimer<E> {
    fn default() -> Self {
        Self::from_interval(std::time::Duration::ZERO)
    }
}

impl<E> IntervalTimer<E>
where
    E: crate::traits::Message + Default + Send + Sync + 'static,
{
    pub fn start(&self) {
        if self.interval.is_zero() || self.started.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let interval = self.interval;
        crate::runtime::interval(interval, || crate::event_bus().send(E::default()));
    }
}

impl<E> crate::traits::IntoObject for IntervalTimer<E> {
    fn into_object(self) -> Object {
        Object::from(crate::elements::Element::from(
            crate::widgets::Text::default(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::IntervalTimer;

    #[test]
    fn interval_timer_starts_only_once() {
        #[derive(Default)]
        struct Tick;
        impl crate::traits::Message for Tick {}

        let timer = IntervalTimer::<Tick>::from_interval(std::time::Duration::from_millis(10));
        timer.start();
        timer.start();
        assert!(timer.started.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn zero_interval_timer_does_not_start() {
        #[derive(Default)]
        struct Tick;
        impl crate::traits::Message for Tick {}

        let timer = IntervalTimer::<Tick>::default();
        timer.start();
        assert!(!timer.started.load(std::sync::atomic::Ordering::SeqCst));
    }
}
