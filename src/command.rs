//! Representation of commands to perform on behalf of the user.

#[derive(Debug)]
pub enum Command {
    Key(Key),
}

#[derive(Debug)]
pub struct Key {
    key: evdev::Key,
    mods: u8,
}


impl Command {
    pub fn from_key(key: evdev::Key) -> Self {
        Command::Key(Key::new(key))
    }
}

impl Key {
    pub fn new(key: evdev::Key) -> Self {
        Self { key, mods: 0 }
    }

    pub fn key(&self) -> &evdev::Key {
        &self.key
    }

    pub fn shift(&self) -> bool {
        (self.mods & SHIFT) != 0
    }
    pub fn control(&self) -> bool {
        (self.mods & CONTROL) != 0
    }
    pub fn alt(&self) -> bool {
        (self.mods & ALT) != 0
    }
    pub fn win(&self) -> bool {
        (self.mods & WIN) != 0
    }

    pub fn toggle_shift(mut self) -> Self {
        self.mods ^= SHIFT;
        self
    }
    pub fn toggle_control(mut self) -> Self {
        self.mods ^= CONTROL;
        self
    }
    pub fn toggle_alt(mut self) -> Self {
        self.mods ^= ALT;
        self
    }
    pub fn toggle_win(mut self) -> Self {
        self.mods ^= WIN;
        self
    }
}


const SHIFT: u8 = 1;
const CONTROL: u8 = 2;
const ALT: u8 = 4;
const WIN: u8 = 8;
