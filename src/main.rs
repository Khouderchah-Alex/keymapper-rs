use evdev::{Device, InputEvent, InputEventKind};

use std::io::Error;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;

mod command;
mod context;
mod x11;

use command::{Command, Key};
use context::Context;

const DEV: &str = "/dev/input/by-id/YOUR_DEV";


fn main() -> Result<(), Error> {
    watch_title_changes();

    // TODO(func) Support multiple devices either by multiplexing channel or by
    // creating separate channel per device.
    let (tx, rx) = channel::<InputEvent>();
    watch_device(Path::new(DEV), tx)?;

    let mut exec = x11::Executor::new();
    for ev in rx {
        match ev.kind() {
            InputEventKind::Key(_key) => {
                println!("{} -- {:?}", Context::current().title, ev);
                // In the context of Key, 1 is keypress & 2 is keyhold.
                if ev.value() == 1 || ev.value() == 2 {
                    exec.run(Command::Key(Key::new(evdev::Key::KEY_A).toggle_shift()));
                }
            }
            _ => { /* Ignore for now. */ }
        }
    }
    Ok(())
}

fn watch_title_changes() {
    thread::spawn(|| {
        for title in x11::title_iter().unwrap() {
            let mut new_ctx = (*Context::current()).clone();
            new_ctx.title = title.to_string();
            new_ctx.make_current();
        }
    });
}

fn watch_device(dev_path: &Path, tx: Sender<InputEvent>) -> Result<(), Error> {
    let mut d = Device::open(dev_path)?;
    d.grab()?;
    thread::spawn(move || {
        loop {
            for ev in d.fetch_events().unwrap() {
                tx.send(ev).expect("Unable to send on channel");
            }
        }
    });
    Ok(())
}
