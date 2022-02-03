use evdev::{Device, InputEvent, InputEventKind};

use std::fs::File;
use std::io::{BufReader, Error, ErrorKind};
use std::os::unix::fs::MetadataExt; // For uid, gid, mode.
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::time::Duration;

mod command;
mod config;
mod context;
mod keycode;
mod x11;

use command::{Command, Key};
use config::DeviceConfig;
use context::Context;


fn main() -> Result<(), Error> {
    watch_title_changes();
    // TODO(func) Support multiple devices either by multiplexing channel or by
    // creating separate channel per device.
    let (tx, rx) = channel::<InputEvent>();

    let conf = load_config("/etc/keymapper.d/test.json")?;
    watch_device(conf.device_path.clone(), tx)?;

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

fn load_config<P: AsRef<Path>>(path: P) -> Result<DeviceConfig, Error> {
    let conf_file = File::open(path.as_ref())?;
    // While an unprivileged attacker can't control the stream of input events,
    // allowing unprivileged modification of device configs opens up too many
    // undesirable scenarios. Ensure config modification is a privileged
    // operation.
    let metadata = conf_file.metadata().unwrap();
    if metadata.uid() != 0 || metadata.gid() != 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Config not owned by root",
        ));
    } else if metadata.mode() & 2 != 0 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Config writable by non-root",
        ));
    }

    Ok(serde_json::from_reader(BufReader::new(conf_file))?)
}

fn watch_device(dev_path: PathBuf, tx: Sender<InputEvent>) -> Result<(), Error> {
    thread::spawn(move || {
        let fetch_events = |dev_path| -> Result<(), Error> {
            let mut d = Device::open(dev_path)?;
            d.grab()?;
            loop {
                for ev in d.fetch_events().unwrap() {
                    tx.send(ev).expect("Unable to send on channel");
                }
            }
        };

        while let Err(_err) = fetch_events(&dev_path) {
            thread::sleep(Duration::from_secs(2));
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
