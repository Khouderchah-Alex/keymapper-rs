use evdev::{Device, InputEvent, InputEventKind};

use std::fs::File;
use std::io::{BufReader, Error};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

mod command;
mod config;
mod context;
mod keycode;
mod x11;

use command::{Command, Key};
use config::DeviceConfig;
use context::Context;


fn main() -> Result<(), Error> {
    let conf_file = File::open("/etc/keymapper.d/test.json")?;
    let conf: DeviceConfig = serde_json::from_reader(BufReader::new(conf_file))?;

    watch_title_changes();

    // TODO(func) Support multiple devices either by multiplexing channel or by
    // creating separate channel per device.
    let (tx, rx) = channel::<InputEvent>();
    watch_device(&conf.device_path, tx)?;

    map_keys(rx, conf);
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

fn map_keys(rx: Receiver<InputEvent>, conf: DeviceConfig) {
    let mut exec = x11::Executor::new();

    for event in rx {
        match event.kind() {
            InputEventKind::Key(key) => {
                // In the context of Key, 1 is keypress & 2 is keyhold.
                if event.value() != 1 && event.value() != 2 {
                    continue;
                }

                let i = match conf.hw_keys.iter().position(|e| *e as u16 == key.code()) {
                    Some(i) => i,
                    None => {
                        passthrough(&mut exec, key);
                        continue;
                    }
                };

                let title = &Context::current().title;
                let cmd_map = match conf
                    .title_map
                    .iter()
                    .find(|map| map.title_regex.is_match(title))
                {
                    Some(title_map) => &title_map.commands,
                    None => &conf.default_map,
                };

                if i < cmd_map.len() {
                    exec.run(&cmd_map[i]);
                } else {
                    passthrough(&mut exec, key);
                }
            }
            _ => { /* Ignore for now. */ }
        }
    }
}

// Reproduce key without modification. Since we grab the device, ignoring a
// key means no one sees it.
fn passthrough(exec: &mut x11::Executor, key: evdev::Key) {
    exec.run(&Command::Key(Key::new(key.code().into())));
}
