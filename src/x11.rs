use std::io::{BufRead, BufReader, Error, ErrorKind, Write};
use std::os::unix::process::CommandExt; // For uid().
use std::process::{ChildStdin, Command, Stdio};

// We prefer not to run other processes as root. However, a constraint for X11
// with authorization--used by default on modern systems--is access to the
// user's .Xauthority file (try `echo "$XAUTHORITY"`); the UID below must have
// access for the application to function in X11 mode.
const X11_UID: u32 = 1000;

pub fn title_iter() -> Result<impl Iterator<Item = String>, Error> {
    let stdout = Command::new("xtitle")
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
        let stdin = Command::new("xdotool")
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

    pub fn run(&mut self, cmd: &[u8]) {
        self.input.write(cmd).expect("failed to write");
    }
}
