# keymapper-rs
A Linux program to remap keys given the title of the currently-focused window.

### Current features:
* X11 support through `xtitle` and `xdotool`
* Mapping keys to keys w/modifiers
* Mapping keys to strings
* Mapping keys to a sequence of commands

### Planned features:
* Mapping keys to executables
* Multiple-device support
* Wayland support (xtitle-like support will likely be on a per-compositor basis)
