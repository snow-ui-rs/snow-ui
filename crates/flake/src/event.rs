use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SendError, Sender, TryRecvError};

static NEXT_WIDGET_ID: AtomicU64 = AtomicU64::new(1);

/// Stable identity shared between the native backend and Snow UI controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct WidgetId(u64);

impl WidgetId {
    /// Allocate a new widget identity.
    pub fn new() -> Self {
        Self(NEXT_WIDGET_ID.fetch_add(1, Ordering::Relaxed))
    }

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

/// Converts winit window events into the platform-neutral native event stream.
#[derive(Debug, Clone)]
pub struct WinitEventAdapter {
    scale_factor: f64,
    physical_size: [u32; 2],
    cursor_position: [f64; 2],
}

impl WinitEventAdapter {
    pub fn new(scale_factor: f64, physical_size: [u32; 2]) -> Self {
        Self {
            scale_factor: scale_factor.max(f64::MIN_POSITIVE),
            physical_size,
            cursor_position: [0.0, 0.0],
        }
    }

    pub fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    /// Translate supported winit events. Unhandled events return an empty vector.
    pub fn translate(&mut self, event: &winit::event::WindowEvent) -> Vec<NativeEvent> {
        use winit::event::{ElementState, Ime, MouseButton, TouchPhase, WindowEvent};

        match event {
            WindowEvent::Resized(size) => {
                self.physical_size = [size.width, size.height];
                vec![self.resize_event()]
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                self.scale_factor = scale_factor.max(f64::MIN_POSITIVE);
                vec![self.resize_event()]
            }
            WindowEvent::Focused(focused) => vec![NativeEvent::FocusChanged(*focused)],
            WindowEvent::CloseRequested => vec![NativeEvent::CloseRequested],
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = [
                    position.x / self.scale_factor,
                    position.y / self.scale_factor,
                ];
                vec![NativeEvent::PointerMoved {
                    pointer_id: 0,
                    x: self.cursor_position[0],
                    y: self.cursor_position[1],
                    target: None,
                }]
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match button {
                    MouseButton::Left => PointerButton::Primary,
                    MouseButton::Right => PointerButton::Secondary,
                    MouseButton::Middle => PointerButton::Middle,
                    MouseButton::Back => PointerButton::Other(4),
                    MouseButton::Forward => PointerButton::Other(5),
                    MouseButton::Other(value) => PointerButton::Other(*value),
                };
                vec![NativeEvent::PointerButton {
                    pointer_id: 0,
                    button,
                    pressed: *state == ElementState::Pressed,
                    x: self.cursor_position[0],
                    y: self.cursor_position[1],
                    target: None,
                }]
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let mut translated = vec![NativeEvent::Key {
                    key: format!("{:?}", event.logical_key),
                    pressed: event.state == ElementState::Pressed,
                    repeat: event.repeat,
                }];
                if event.state == ElementState::Pressed
                    && let Some(text) = &event.text
                {
                    translated.push(NativeEvent::TextInput(text.to_string()));
                }
                translated
            }
            WindowEvent::Ime(Ime::Commit(text)) => vec![NativeEvent::TextInput(text.clone())],
            WindowEvent::Touch(touch) => {
                let x = touch.location.x / self.scale_factor;
                let y = touch.location.y / self.scale_factor;
                match touch.phase {
                    TouchPhase::Moved => vec![NativeEvent::PointerMoved {
                        pointer_id: touch.id,
                        x,
                        y,
                        target: None,
                    }],
                    TouchPhase::Started | TouchPhase::Ended | TouchPhase::Cancelled => {
                        vec![NativeEvent::PointerButton {
                            pointer_id: touch.id,
                            button: PointerButton::Primary,
                            pressed: touch.phase == TouchPhase::Started,
                            x,
                            y,
                            target: None,
                        }]
                    }
                }
            }
            _ => Vec::new(),
        }
    }

    /// Translate and send a winit event through an application-provided sink.
    ///
    /// This lets Snow UI adapt native input to its EventBus without making this
    /// crate depend on the core crate or its message types.
    pub fn dispatch<S: NativeEventSink + ?Sized>(
        &mut self,
        event: &winit::event::WindowEvent,
        sink: &S,
    ) -> Result<usize, SendError<NativeEvent>> {
        let translated = self.translate(event);
        let count = translated.len();
        for event in translated {
            sink.send(event)?;
        }
        Ok(count)
    }

    fn resize_event(&self) -> NativeEvent {
        NativeEvent::Resized {
            width: self.physical_size[0],
            height: self.physical_size[1],
            scale_factor: self.scale_factor,
        }
    }
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

    #[test]
    fn winit_resize_events_preserve_physical_size_and_scale_factor() {
        let mut adapter = WinitEventAdapter::new(2.0, [100, 80]);
        let translated = adapter.translate(&winit::event::WindowEvent::Resized(
            winit::dpi::PhysicalSize::new(640, 480),
        ));

        assert_eq!(
            translated,
            vec![NativeEvent::Resized {
                width: 640,
                height: 480,
                scale_factor: 2.0,
            }]
        );
    }

    #[test]
    fn winit_cursor_coordinates_are_converted_to_logical_pixels() {
        let mut adapter = WinitEventAdapter::new(2.0, [100, 80]);
        let translated = adapter.translate(&winit::event::WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(30.0, 50.0),
        });

        assert_eq!(
            translated,
            vec![NativeEvent::PointerMoved {
                pointer_id: 0,
                x: 15.0,
                y: 25.0,
                target: None,
            }]
        );
    }

    #[test]
    fn winit_dispatch_forwards_translated_events_to_the_sink() {
        let (sender, receiver) = native_event_channel();
        let mut adapter = WinitEventAdapter::new(1.0, [100, 80]);
        let count = adapter
            .dispatch(&winit::event::WindowEvent::Focused(true), &sender)
            .unwrap();

        assert_eq!(count, 1);
        assert_eq!(receiver.try_recv(), Ok(NativeEvent::FocusChanged(true)));
    }
}
