extern crate clap;
extern crate dbus;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate xcb;

mod config;
mod display;
mod server;

use clap::{Arg, App, SubCommand};
use std::path::{Path, PathBuf};

fn main() {
    let matches = App::new("x11-overlay-bar-rs")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Jeffrey Xiao <jeffrey.xiao1998@gmail.com>")
        .about("A simple, but flexible system overlay bar for the X Window System (X11).")
        .subcommand(SubCommand::with_name("start")
            .about("Starts daemon that listens to requests.")
            .arg(
                Arg::with_name("config")
                    .help("Path to configuration file.")
                    .takes_value(true)
                    .short("c")
                    .long("config")
            )
        )
        .subcommand(SubCommand::with_name("show")
            .about("Shows bar with a specific value and in a specific color profile.")
            .arg(
                Arg::with_name("value")
                    .help("The value of the bar.")
                    .index(2)
                    .required(true)
            )
            .arg(
                Arg::with_name("profile")
                    .help("The color profile to use.")
                    .index(1)
                    .required(true)
            )
        )
        .subcommand(SubCommand::with_name("hide").about("Hides the bar."))
        .subcommand(SubCommand::with_name("stop").about("Stops daemon."))
        .get_matches();

    match matches.subcommand() {
        ("start", Some(matches)) => {
            let config_path = match matches.value_of("config") {
                Some(config) => PathBuf::from(config),
                None => {
                    let config_home_dir = option_env!("XDG_CONFIG_HOME").unwrap_or("$HOME/.config");
                    Path::new(config_home_dir).join("rob").join("rob.toml")
                }
            };
            let (global_config, color_configs) = config::parse_config(config_path);
            let display = display::Display::new().unwrap();
            server::start_server(display, global_config, color_configs);
        },
        ("show", Some(matches)) => {

        },
        ("hide", Some(matches)) => {

        },
        ("stop", Some(matches)) => {

        },
        _ => {

        },
    }
}
