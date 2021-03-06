use std::collections::HashMap;

use crate::emulator::io::event::{Event, EventHandler, Key};
use crate::emulator::memory::{Reader, Writer};
use crate::emulator::state::{ControllerState, SaveState};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Button {
    Start,
    Select,
    A,
    B,
    Up,
    Down,
    Left,
    Right,
}

pub type KeyMap = HashMap<Key, Button>;

pub type KeyState = HashMap<Button, bool>;

pub struct Controller {
    keymap: KeyMap,
    keystate: KeyState,
    strobe_ix: u8,
    register: u8,
}

impl Controller {
    const STROBE_ORDER: [Button; 8] = [
        Button::A,
        Button::B,
        Button::Select,
        Button::Start,
        Button::Up,
        Button::Down,
        Button::Left,
        Button::Right,
    ];

    pub fn new(keymap: KeyMap) -> Controller {
        Controller {
            keymap,
            keystate: HashMap::new(),
            strobe_ix: 0,
            register: 0,
        }
    }
}

impl EventHandler for Controller {
    fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyDown(key) => {
                if let Some(button) = self.keymap.get(&key) {
                    self.keystate.insert(*button, true);
                }
            }
            Event::KeyUp(key) => {
                if let Some(button) = self.keymap.get(&key) {
                    self.keystate.insert(*button, false);
                }
            }
        }
    }
}

impl Reader for Controller {
    fn read(&mut self, _address: u16) -> u8 {
        // If strobe bit is 1, constantly reset state.
        if self.register & 1 != 0 {
            self.strobe_ix = 0;
        }
        let button = Controller::STROBE_ORDER[self.strobe_ix as usize];
        let is_pressed = *self.keystate.get(&button).unwrap_or(&false);
        let byte = if is_pressed { 1 } else { 0 };
        self.strobe_ix += 1;
        self.strobe_ix %= 8;
        byte
    }
}

impl Writer for Controller {
    fn write(&mut self, _address: u16, byte: u8) {
        // Controller is only responsible for the bit 0.
        self.register = byte & 1;
    }
}

impl<'de> SaveState<'de, ControllerState> for Controller {
    fn freeze(&mut self) -> ControllerState {
        ControllerState {
            strobe_ix: self.strobe_ix,
            register: self.register,
        }
    }

    fn hydrate(&mut self, state: ControllerState) {
        self.strobe_ix = state.strobe_ix;
        self.register = state.register;
    }
}
