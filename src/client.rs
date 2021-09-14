use crate::server;
use crate::{Error, Result};

use bincode::serialize;

use std::io::prelude::*;
use std::io::{self, Write};
use std::net::Shutdown;
use std::os::unix::net::UnixStream;

fn send_one_request(request: &server::Request) -> Result<()> {
    let mut socket = UnixStream::connect(server::SOCKET_PATH)
        .map_err(|err| Error::new("connecting to server", &err))?;
    send_request(request, &mut socket)?;
    socket
        .shutdown(Shutdown::Write)
        .map_err(|err| Error::new("shutting down connection", &err))
}

fn send_request(request: &server::Request, socket: &mut UnixStream) -> Result<()> {
    let mut serialized_request =
        serialize(&request).map_err(|err| Error::new("serializing request", &err))?;
    serialized_request.push(server::END_OF_REQUEST_SEPARATOR);
    socket
        .write_all(&serialized_request)
        .map_err(|err| Error::new("sending request", &err))?;
    socket
        .flush()
        .map_err(|err| Error::new("flushing socket", &err))?;
    Ok(())
}

pub fn show_stream() -> Result<()> {
    let stdin = io::stdin();
    let mut socket = UnixStream::connect(server::SOCKET_PATH)
        .map_err(|err| Error::new("connecting to server", &err))?;
    for line in stdin.lock().lines() {
        let line = line.map_err(|err| Error::new("reading io", &err))?;
        let tokens = line.split(' ').collect::<Vec<&str>>();
        let (profile, value_str) = match tokens.as_slice() {
            [profile, value_str] => (profile.to_string(), value_str),
            _ => {
                eprintln!("Expected each line to be in format `profile value`");
                continue;
            }
        };
        let value = match value_str.parse::<u8>() {
            Ok(value) => value,
            Err(_) => {
                eprintln!("Expected `value` to be a u8");
                continue;
            }
        };
        if value > 100 {
            eprintln!("Expected `value` to be in [0, 100]");
            continue;
        }
        if let Err(err) = send_request(&server::Request::Show { profile, value }, &mut socket) {
            eprintln!("Failed to send request {:?}", err);
        }
    }
    socket
        .shutdown(Shutdown::Write)
        .map_err(|err| Error::new("shutting down connection", &err))?;
    Ok(())
}

pub fn show(profile: String, value: u8) -> Result<()> {
    if value > 100 {
        return Err(Error::from_description(
            "processing request",
            "Expected `value` in [0, 100].",
        ));
    }
    send_one_request(&server::Request::Show { profile, value })
}

pub fn hide() -> Result<()> {
    send_one_request(&server::Request::Hide)
}

pub fn stop() -> Result<()> {
    send_one_request(&server::Request::Stop)
}
