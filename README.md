# alacritty-scheme-switcher

A single-purpose script for switching [alacritty](https://github.com/alacritty/alacritty) color schemes with a shortcut.

## Quickstart

1. Install using one of the following options:
  - Get the statically linked binary for x86_64 from the [releases section](https://github.com/magnickolas/alacritty-scheme-switcher/releases/latest) and put it in some directory from `PATH`
  - Install from source with `cargo install --path .`
2. Add a script trigger by any binding that you like under the `key_bindings` section inside `alacritty.yml`:
  ```yaml
    - { key: F, mods: Control, command: {program: "alacritty_scheme_switcher"} }
  ```
