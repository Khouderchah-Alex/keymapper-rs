use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::os::unix::process::CommandExt; // For uid().
use std::process::{self, ChildStdin, Stdio};

use crate::command::Command;


// We prefer not to run other processes as root. However, a constraint for X11
// with authorization--used by default on modern systems--is access to the
// user's .Xauthority file (try `echo "$XAUTHORITY"`); the UID below must have
// access for the application to function in X11 mode.
const X11_UID: u32 = 1000;


pub fn title_iter() -> Result<impl Iterator<Item = String>, Error> {
    let stdout = process::Command::new("xtitle")
        .arg("-s")
        .stdout(Stdio::piped())
        .uid(X11_UID)
        .spawn()?
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::Other, "Could not capture standard output."))?;

    let reader = BufReader::new(stdout);
    Ok(reader.lines().filter_map(|line| line.ok()))
}

pub struct Executor {
    input: ChildStdin,
}

impl Executor {
    pub fn new() -> Self {
        // Use xdotool's script mode rather than a wide-open bash process.
        let stdin = process::Command::new("xdotool")
            .arg("-")
            .stdin(Stdio::piped())
            .uid(X11_UID)
            .spawn()
            .unwrap()
            .stdin
            .take()
            .expect("failed to get stdin");
        Self { input: stdin }
    }

    pub fn run(&mut self, cmd: Command) {
        let cmd_bytes = match cmd {
            Command::Key(key) => {
                let mut mods = String::default();
                if key.shift() {
                    mods.push_str("shift+");
                }
                if key.control() {
                    mods.push_str("ctrl+");
                }
                if key.alt() {
                    mods.push_str("alt+");
                }
                if key.win() {
                    mods.push_str("super+");
                }

                let s = format!("key {}{}\n", mods, key_str(key.key()));
                s.into_boxed_str().into_boxed_bytes()
            }
        };

        self.input.write(&cmd_bytes).expect("failed to write");
    }
}


fn key_str(key: &evdev::Key) -> &'static str {
    match key {
        &evdev::Key::KEY_A => "a",
        &evdev::Key::KEY_B => "b",
        &evdev::Key::KEY_C => "c",
        &evdev::Key::KEY_D => "d",
        &evdev::Key::KEY_E => "e",
        &evdev::Key::KEY_F => "f",
        &evdev::Key::KEY_G => "g",
        &evdev::Key::KEY_H => "h",
        &evdev::Key::KEY_I => "i",
        &evdev::Key::KEY_J => "j",
        &evdev::Key::KEY_K => "k",
        &evdev::Key::KEY_L => "l",
        &evdev::Key::KEY_M => "m",
        &evdev::Key::KEY_N => "n",
        &evdev::Key::KEY_O => "o",
        &evdev::Key::KEY_P => "p",
        &evdev::Key::KEY_Q => "q",
        &evdev::Key::KEY_R => "r",
        &evdev::Key::KEY_S => "s",
        &evdev::Key::KEY_T => "t",
        &evdev::Key::KEY_U => "u",
        &evdev::Key::KEY_V => "v",
        &evdev::Key::KEY_W => "w",
        &evdev::Key::KEY_X => "x",
        &evdev::Key::KEY_Y => "y",
        &evdev::Key::KEY_Z => "z",

        &evdev::Key::KEY_0 => "0",
        &evdev::Key::KEY_1 => "1",
        &evdev::Key::KEY_2 => "2",
        &evdev::Key::KEY_3 => "3",
        &evdev::Key::KEY_4 => "4",
        &evdev::Key::KEY_5 => "5",
        &evdev::Key::KEY_6 => "6",
        &evdev::Key::KEY_7 => "7",
        &evdev::Key::KEY_8 => "8",
        &evdev::Key::KEY_9 => "9",

        _ => panic!(),
    }
}
