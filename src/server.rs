use crate::config;
use crate::display;
use crate::{Error, Result};
use bincode::{deserialize, serialize};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub const MAX_REQUEST_SIZE: usize = 32;
pub const SOCKET_PATH: &str = "/tmp/robar";

#[derive(Serialize, Deserialize)]
pub enum Request {
    Show { profile: String, value: f64 },
    Hide,
    Stop,
    Empty,
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
    display: &display::Display,
    global_config: &config::GlobalConfig,
    color_configs: &HashMap<String, config::ColorConfig>,
) -> Result<()> {
    if Path::new(SOCKET_PATH).exists() {
        fs::remove_file(SOCKET_PATH).map_err(|err| Error::new("removing existing socket", &err))?;
    }

    let color_configs_clone = color_configs.clone();
    let socket =
        UnixListener::bind(SOCKET_PATH).map_err(|err| Error::new("binding socket", &err))?;

    let pair1 = Arc::new((Mutex::new(Request::Empty), Condvar::new()));
    let pair2 = pair1.clone();

    thread::spawn(move || {
        for stream in socket.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(err) => {
                    println!("Error with incoming connection: {}.", err);
                    continue;
                }
            };

            let &(ref lock, ref cvar) = &*pair2;
            let mut request = lock.lock().unwrap();
            *request = Request::Empty;

            let mut buffer = Vec::with_capacity(MAX_REQUEST_SIZE);
            let result = stream
                .read_to_end(&mut buffer)
                .map_err(|err| Error::new("reading request", &err));

            if result.is_ok() {
                match validate_request(&color_configs_clone, &buffer) {
                    Ok(new_request) => {
                        *request = new_request;
                        stream
                            .write_all(&serialize(&Ok::<(), Error>(())).unwrap())
                            .unwrap();
                    }
                    Err(err) => stream
                        .write_all(&serialize(&Err::<(), Error>(err)).unwrap())
                        .unwrap(),
                }
            }

            cvar.notify_one();
        }
    });

    let &(ref lock, ref cvar) = &*pair1;
    let mut request = lock.lock().unwrap();

    loop {
        if global_config.timeout != 0 {
            let result = cvar
                .wait_timeout(request, Duration::from_millis(global_config.timeout))
                .unwrap();
            request = result.0;

            if result.1.timed_out() {
                display.hide();
                continue;
            }
        } else {
            let result = cvar.wait(request).unwrap();
            request = result;
        }

        match *request {
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
