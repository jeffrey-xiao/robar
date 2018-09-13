extern crate clap;
extern crate dbus;
extern crate xcb;

mod display;
mod server;

use clap::{Arg, App, SubCommand};
use std::sync::Arc;

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
            .about("Shows bar with a specific value and mode.")
            .arg(
                Arg::with_name("mode")
                    .help("The mode of the bar.")
                    .index(1)
                    .required(true)
            )
            .arg(
                Arg::with_name("value")
                    .help("The value of the bar.")
                    .index(2)
                    .required(true)
            )
        )
        .subcommand(SubCommand::with_name("hide").about("Hides the bar."))
        .subcommand(SubCommand::with_name("stop").about("Stops daemon."))
        .get_matches();

    match matches.subcommand() {
        ("start", Some(matches)) => {
            let display = display::Display::new().unwrap();
            display.show();
            server::start_server(display);
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
