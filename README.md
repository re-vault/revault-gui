# revault-gui

Revault GUI is an user graphical interface written in rust for the 
[Revault daemon](https:://github.com/revault/revaultd).

## Usage

`cargo run --release -- --conf revault_gui.toml` or
`cargo run --release -- --datadir revault`

If no argument is provided, the GUI checks for the configuration file
in the default revaultd `datadir` (`~/.revault` for linux).

If the provided `datadir` is empty or does not exist, the GUI starts with
the installer mode.

After start up, The GUI will connect to the running revaultd.
A command starting revaultd is launched if no connection is made.

## Get started

See [doc/DEMO.md](doc/DEMO.md) for instructions on how to start the GUI
