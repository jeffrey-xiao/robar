use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc::{channel, RecvError, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use bincode::deserialize;
use serde_derive::{Deserialize, Serialize};

use crate::config;
use crate::display;
use crate::{Error, Result};

pub const MAX_REQUEST_SIZE: usize = 32;
pub const SOCKET_PATH: &str = "/tmp/robar";
pub const END_OF_REQUEST_SEPARATOR: u8 = 13;

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    Show { profile: String, value: u8 },
    Hide,
    Stop,
    Empty,
}

pub struct RequestBuffer {
    pending: Option<(usize, usize, Vec<u8>)>,
    next: Vec<u8>,
}

impl RequestBuffer {
    pub fn new() -> Self {
        Self {
            pending: None,
            next: vec![0; MAX_REQUEST_SIZE],
        }
    }

    pub fn read_request(&mut self, stream: &mut UnixStream) -> Result<Option<Vec<u8>>> {
        let next_end = stream
            .read(self.next.as_mut_slice())
            .map_err(|err| Error::new("reading from socket", &err))?;
        if next_end == 0 {
            return Ok(None);
        }
        let end_index = self
            .next
            .iter()
            .position(|b| *b == END_OF_REQUEST_SEPARATOR)
            .ok_or_else(|| {
                Error::from_description(
                    "reading from socket",
                    "request body exceeded max request size",
                )
            })?;
        match &mut self.pending {
            Some((pending_start, pending_end, pending)) => {
                let mut curr = vec![0; *pending_end - *pending_start + end_index];
                let (left, right) = curr
                    .as_mut_slice()
                    .split_at_mut(*pending_end - *pending_start);
                left.clone_from_slice(&pending[*pending_start..*pending_end]);
                right.clone_from_slice(&self.next[0..end_index]);
                std::mem::swap(pending, &mut self.next);
                *pending_start = end_index + 1;
                *pending_end = next_end;
                Ok(Some(curr))
            }
            None => {
                let mut curr = vec![0; end_index];
                curr.clone_from_slice(&self.next[0..end_index]);
                let new_buf = vec![0; MAX_REQUEST_SIZE];
                let pending_buf = std::mem::replace(&mut self.next, new_buf);
                self.pending = Some((end_index + 1, next_end, pending_buf));
                Ok(Some(curr))
            }
        }
    }
}

fn validate_request(
    color_configs: &HashMap<String, config::ColorConfig>,
    buffer: &[u8],
) -> Result<Request> {
    let request = deserialize(&buffer).map_err(|err| Error::new("deserializing request", &err))?;

    if let Request::Show { ref profile, .. } = request {
        if !color_configs.contains_key(profile) {
            return Err(Error::from_description(
                "processing request",
                format!("Color profile `{}` not found.", profile),
            ));
        }
    }

    Ok(request)
}

pub fn start_server(
    display: &mut display::Display,
    global_config: &config::GlobalConfig,
    color_configs: &HashMap<String, config::ColorConfig>,
) -> Result<()> {
    if Path::new(SOCKET_PATH).exists() {
        fs::remove_file(SOCKET_PATH).map_err(|err| Error::new("removing existing socket", &err))?;
    }

    let socket =
        UnixListener::bind(SOCKET_PATH).map_err(|err| Error::new("binding socket", &err))?;
    let (tx, rx) = channel();

    let color_configs_clone = color_configs.clone();
    let tx_clone = tx;
    thread::spawn(move || {
        for stream in socket.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(err) => {
                    eprintln!("Error with incoming connection: {}.", err);
                    continue;
                }
            };

            let color_configs_clone = color_configs_clone.clone();
            let tx_clone = tx_clone.clone();
            thread::spawn(move || {
                let mut buffer = RequestBuffer::new();
                while let Ok(Some(buffer)) = buffer.read_request(&mut stream) {
                    match validate_request(&color_configs_clone, &buffer) {
                        Ok(new_request) => tx_clone.send(new_request).unwrap(),
                        Err(err) => eprintln!("Error with request: {}", err),
                    }
                }
            });
        }
    });

    loop {
        let request = if global_config.timeout != 0 {
            let request = rx.recv_timeout(Duration::from_millis(global_config.timeout));

            match request {
                Ok(request) => request,
                Err(RecvTimeoutError::Timeout) => {
                    display.hide();
                    continue;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            match rx.recv() {
                Ok(request) => request,
                Err(RecvError) => break,
            }
        };

        match request {
            Request::Show {
                ref profile,
                ref value,
            } => display.show(*value, &global_config, &color_configs[profile]),
            Request::Hide => display.hide(),
            Request::Stop => break,
            Request::Empty => {}
        }
    }

    Ok(())
}
