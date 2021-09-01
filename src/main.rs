use evdev::{Device, InputEvent, InputEventKind::*};

use std::io::Error;
use std::sync::mpsc::channel;
use std::thread;

mod context;
mod x11;

use context::Context;

const DEV: &str = "/dev/input/by-id/YOUR_DEV";

fn main() -> Result<(), Error> {
    let _title_thread = thread::spawn(|| {
        for title in x11::title_iter().unwrap() {
            let mut new_ctx = (*Context::current()).clone();
            new_ctx.title = title.to_string();
            new_ctx.make_current();
        }
    });

    let (tx, rx) = channel::<InputEvent>();
    let _mapper_thread = thread::spawn(|| {
        let mut exec = x11::Executor::new();
        for ev in rx {
            match ev.kind() {
                Key(_key) => {
                    println!("{} -- {:?}", Context::current().title, ev);
                    // In the context of Key, 1 is keypress & 2 is keyhold.
                    if ev.value() == 1 || ev.value() == 2 {
                        exec.run(b"key a\n");
                    }
                }
                _ => { /* Ignore for now. */ }
            }
        }
    });

    let mut d = Device::open(DEV)?;
    d.grab()?;
    loop {
        for ev in d.fetch_events().unwrap() {
            tx.send(ev).expect("Unable to send on channel");
        }
    }
}
