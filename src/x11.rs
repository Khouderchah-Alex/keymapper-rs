use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::os::unix::process::CommandExt; // For uid().
use std::process::{self, ChildStdin, Stdio};

use crate::command::Command;
use crate::keycode::KeyCode;


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

    pub fn run(&mut self, cmd: &Command) {
        println!("{:?}", cmd);
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

                let s = format!("key {}{}\n", mods, code_to_string(key.key()));
                s.into_boxed_str().into_boxed_bytes()
            }
        };

        self.input.write(&cmd_bytes).expect("failed to write");
    }
}


fn code_to_string(code: &KeyCode) -> String {
    // TODO(func) Provide support for special characters.
    match code {
        KeyCode::UP => "Up",
        KeyCode::LEFT => "Left",
        KeyCode::RIGHT => "Right",
        KeyCode::DOWN => "Down",
        _ => return code.to_string(),
    }
    .to_string()
}
