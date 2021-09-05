use evdev::{Device, InputEvent, InputEventKind};

use std::fs::File;
use std::io::{BufReader, Error};
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::thread;

mod command;
mod config;
mod context;
mod keycode;
mod x11;

use command::{Command, Key};
use context::Context;


fn main() -> Result<(), Error> {
    let file = File::open("/etc/keymapper.d/test.json")?;
    let reader = BufReader::new(file);
    let conf: config::DeviceConfig = serde_json::from_reader(reader)?;

    watch_title_changes();

    // TODO(func) Support multiple devices either by multiplexing channel or by
    // creating separate channel per device.
    let (tx, rx) = channel::<InputEvent>();
    watch_device(&conf.device_path, tx)?;

    let mut exec = x11::Executor::new();
    for ev in rx {
        match ev.kind() {
            InputEventKind::Key(key) => {
                // In the context of Key, 1 is keypress & 2 is keyhold.
                if ev.value() != 1 && ev.value() != 2 {
                    continue;
                }

                let i = conf
                    .hw_keys
                    .iter()
                    .position(|e| *e as u16 == key.code())
                    .unwrap();
                let mut matched = false;
                let title = &Context::current().title;
                for maps in &conf.title_map {
                    if maps.title_regex.is_match(title) {
                        if i < maps.commands.len() {
                            exec.run(&maps.commands[i]);
                        } else {
                            exec.run(&Command::Key(Key::new(key.code().into())));
                        }
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    if i < conf.default_map.len() {
                        exec.run(&conf.default_map[i]);
                    } else {
                        exec.run(&Command::Key(Key::new(key.code().into())));
                    }
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
