mod client;
mod config;
mod display;
mod server;

use clap::{App, AppSettings, Arg, SubCommand};
use serde_derive::{Deserialize, Serialize};
use std::error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process;
use std::result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Error {
    context: String,
    description: String,
}

impl Error {
    pub fn new<T, U>(context: T, error: &U) -> Self
    where
        T: Into<String>,
        U: error::Error,
    {
        Error {
            context: context.into(),
            description: error.to_string(),
        }
    }

    pub fn from_description<T, U>(context: T, description: U) -> Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        Error {
            context: context.into(),
            description: description.into(),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in {} - {}", self.context, self.description)
    }
}

pub type Result<T> = result::Result<T, Error>;

fn run() -> Result<()> {
    let matches = App::new("robar")
        .setting(AppSettings::ArgRequiredElseHelp)
        .version(env!("CARGO_PKG_VERSION"))
        .author("Jeffrey Xiao <jeffrey.xiao1998@gmail.com>")
        .about("A simple, but flexible system overlay bar for the X Window System (X11).")
        .subcommand(
            SubCommand::with_name("start")
                .about("Starts daemon that listens to requests.")
                .arg(
                    Arg::with_name("config")
                        .help("Path to configuration file.")
                        .takes_value(true)
                        .short("c")
                        .long("config"),
                ),
        )
        .subcommand(
            SubCommand::with_name("show")
                .about("Shows bar with a specific value and in a specific color profile.")
                .arg(
                    Arg::with_name("profile")
                        .help("The color profile to use.")
                        .index(1)
                        .required(true),
                )
                .arg(
                    Arg::with_name("value")
                        .help("The value of the bar.")
                        .index(2)
                        .required(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("show-stream")
                .about("Shows bar using lines from standard input in the form of `profile value`"),
        )
        .subcommand(SubCommand::with_name("hide").about("Hides the bar."))
        .subcommand(SubCommand::with_name("stop").about("Stops daemon."))
        .get_matches();

    match matches.subcommand() {
        ("start", Some(matches)) => {
            let config_path = match matches.value_of("config") {
                Some(config) => PathBuf::from(config),
                None => {
                    let xdg_config_home = option_env!("XDG_CONFIG_HOME");
                    let config_home_dir = format!("{}/{}", env!("HOME"), ".config");
                    Path::new(xdg_config_home.unwrap_or(&config_home_dir))
                        .join(env!("CARGO_PKG_NAME"))
                        .join(format!("{}.toml", env!("CARGO_PKG_NAME")))
                }
            };
            let (global_config, color_configs) = config::parse_config(config_path)?;
            let mut display = display::Display::new().unwrap();
            server::start_server(&mut display, &global_config, &color_configs)
        }
        ("show", Some(matches)) => client::show(
            matches
                .value_of("profile")
                .expect("Expected `profile` to exist.")
                .to_owned(),
            matches
                .value_of("value")
                .expect("Expected `value` to exist.")
                .parse()
                .map_err(|err| Error::new("parsing `value`", &err))?,
        ),
        ("show-stream", Some(_)) => client::show_stream(),
        ("hide", Some(_)) => client::hide(),
        ("stop", Some(_)) => client::stop(),
        _ => Ok(()),
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        process::exit(1);
    }
}
