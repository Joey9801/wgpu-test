use std::collections::{HashMap, VecDeque};

use winit::event::{Event, DeviceEvent, KeyboardInput, ElementState};
use scancode::Scancode;

pub enum LogicalKey {
    MoveForward,
    MoveBackward,
    StrafeLeft,
    StrafeRight,
    MoveUp,
    MoveDown,
}

impl LogicalKey {
    // Effectively hardcode the key bindings for now
    // TODO: Configurable key bindings
    fn from_scancode(scancode: u32) -> Option<Self> {
        let scancode = match Scancode::new(scancode as u8) {
            Some(scancode) => scancode,
            None => return None,
        };

        Some(match scancode {
            Scancode::W => LogicalKey::MoveForward,
            Scancode::A => LogicalKey::StrafeLeft,
            Scancode::S => LogicalKey::MoveBackward,
            Scancode::D => LogicalKey::StrafeRight,
            Scancode::Space => LogicalKey::MoveUp,
            Scancode::LeftControl => LogicalKey::MoveDown,
            _ => return None,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Up,
    Down,
}


pub enum LogicalEvent {
    Key {
        new_state: KeyState,
        logical_key: LogicalKey,
    },
    /// Represents a relative movement of the mouse in pixels, where X is right and Y is down.
    MouseMovement {
        x: f32,
        y: f32,
    },
}

pub struct InputManager {
    // Maps hardware scancode to current pressed state
    key_states: HashMap<u32, KeyState>,
    logical_events: VecDeque<LogicalEvent>,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            key_states: HashMap::new(),
            logical_events: VecDeque::new(),
        }
    }

    fn handle_keyboard_input(&mut self, ki: &KeyboardInput)  {
        let tracked_state = self.key_states
            .entry(ki.scancode)
            .or_insert(KeyState::Up);

        let new_state = match ki.state {
            ElementState::Pressed => KeyState::Down,
            ElementState::Released => KeyState::Up,
        };

        if *tracked_state == new_state {
            return;
        }
        
        *tracked_state = new_state;
        if let Some(logical_key) = LogicalKey::from_scancode(ki.scancode) {
            self.logical_events.push_back(LogicalEvent::Key {
                new_state,
                logical_key,
            });
        }
    }

    fn handle_device_event(&mut self, event: &DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.logical_events.push_back(LogicalEvent::MouseMovement {
                    x: delta.0 as f32,
                    y: delta.1 as f32,
                });
            }
            DeviceEvent::Key(ki) => self.handle_keyboard_input(ki),
            _ => (),
        }
    }

    /// Update the internal state of this InputManager, potentially queuing more logical events
    pub fn update(&mut self, event: &Event<()>) {
        match event {
            Event::DeviceEvent { event, .. } => self.handle_device_event(event),
            _ => (),
        }
    }

    /// Returns the next logical event, if one is on the queue
    pub fn poll_logical_event(&mut self) -> Option<LogicalEvent> {
        self.logical_events.pop_front()
    }
}