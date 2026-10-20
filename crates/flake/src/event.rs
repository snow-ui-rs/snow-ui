use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};

/// Stable identity shared between the native backend and Snow UI controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Create an identity from the Snow UI control ID or another stable key.
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Return the underlying numeric identity.
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
}

/// Platform-neutral input and lifecycle events emitted by the native backend.
#[derive(Debug, Clone, PartialEq)]
pub enum NativeEvent {
    PointerMoved {
        pointer_id: u64,
        x: f64,
        y: f64,
        target: Option<WidgetId>,
    },
    PointerButton {
        pointer_id: u64,
        button: PointerButton,
        pressed: bool,
        x: f64,
        y: f64,
        target: Option<WidgetId>,
    },
    Key {
        key: String,
        pressed: bool,
        repeat: bool,
    },
    TextInput(String),
    FocusChanged(bool),
    Resized {
        width: u32,
        height: u32,
        scale_factor: f64,
    },
    CloseRequested,
}

/// Receives native events and adapts them to the application's event system.
pub trait NativeEventSink: Send + Sync {
    fn send(&self, event: NativeEvent) -> Result<(), SendError<NativeEvent>>;
}

#[derive(Clone)]
pub struct NativeEventSender(Sender<NativeEvent>);

pub struct NativeEventReceiver(Receiver<NativeEvent>);

impl NativeEventSender {
    pub fn send(&self, event: NativeEvent) -> Result<(), SendError<NativeEvent>> {
        self.0.send(event)
    }
}

impl NativeEventSink for NativeEventSender {
    fn send(&self, event: NativeEvent) -> Result<(), SendError<NativeEvent>> {
        self.0.send(event)
    }
}

impl NativeEventReceiver {
    pub fn recv(&self) -> Result<NativeEvent, mpsc::RecvError> {
        self.0.recv()
    }

    pub fn try_recv(&self) -> Result<NativeEvent, TryRecvError> {
        self.0.try_recv()
    }
}

/// Create a thread-safe channel for forwarding native events to an EventBus adapter.
pub fn native_event_channel() -> (NativeEventSender, NativeEventReceiver) {
    let (sender, receiver) = mpsc::channel();
    (NativeEventSender(sender), NativeEventReceiver(receiver))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_events_cross_the_channel() {
        let (sender, receiver) = native_event_channel();
        let event = NativeEvent::PointerButton {
            pointer_id: 1,
            button: PointerButton::Primary,
            pressed: true,
            x: 12.0,
            y: 24.0,
            target: Some(WidgetId::from_raw(7)),
        };

        sender.send(event.clone()).unwrap();

        assert_eq!(receiver.try_recv(), Ok(event));
    }
}
