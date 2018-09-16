extern crate bincode;
extern crate clap;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate xcb;

mod client;
mod config;
mod display;
mod server;

use clap::{Arg, App, SubCommand};
use std::error;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::result;

#[derive(Debug)]
pub struct ParseError(String);
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid first item to double")
    }
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        self.0.as_str()
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    SerdeError(bincode::Error),
    ParseError(ParseError),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Error {
        Error::SerdeError(err)
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::ParseError(err)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match self {
            Error::IOError(ref error) => error.description(),
            Error::SerdeError(ref error) => error.description(),
            Error::ParseError(ref error) => error.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match self {
            Error::IOError(ref error) => error.cause(),
            Error::SerdeError(ref error) => error.cause(),
            Error::ParseError(ref error) => error.cause(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(ref error) => write!(f, "{}", error),
            Error::SerdeError(ref error) => write!(f, "{}", error),
            Error::ParseError(ref error) => write!(f, "{}", error),
        }
    }
}

pub type Result<T> = result::Result<T, Error>;

fn run() -> Result<()> {
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
                Arg::with_name("profile")
                    .help("The color profile to use.")
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
            let config_path = match matches.value_of("config") {
                Some(config) => PathBuf::from(config),
                None => {
                    let config_home_dir = option_env!("XDG_CONFIG_HOME").unwrap_or("$HOME/.config");
                    Path::new(config_home_dir).join("rob").join("rob.toml")
                }
            };
            let (global_config, color_configs) = config::parse_config(config_path)?;
            let display = display::Display::new().unwrap();
            server::start_server(display, global_config, color_configs);
        },
        ("show", Some(matches)) => {
            client::show(
                matches.value_of("profile").expect("Expected `profile` to exist.").to_owned(),
                matches.value_of("value").expect("Expected `value` to exist.").parse().unwrap(),
            )
        },
        ("hide", Some(matches)) => {
            client::hide()
        },
        ("stop", Some(matches)) => {
            client::stop()
        },
        _ => {},
    };
}

fn main() {

}
