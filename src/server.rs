use config;
use display;
use bincode::deserialize;
use std::collections::HashMap;
use std::io::Read;
use std::os::unix::net::UnixListener;
use std::fs;
use std::thread;
use std::time::Duration;
use std::sync::{Arc, Mutex, Condvar};
use super::Result;

pub const MAX_REQUEST_SIZE: usize = 32;
pub const SOCKET_PATH: &str = "/tmp/rob";

#[derive(Serialize, Deserialize)]
pub enum Request {
    Show {
        profile: String,
        value: f64,
    },
    Hide,
    Stop,
}

pub fn start_server(
    display: &display::Display,
    global_config: &config::GlobalConfig,
    color_configs: &HashMap<String, config::ColorConfig>,
) -> Result<()> {
    let display = Arc::new(display);
    let socket = UnixListener::bind(SOCKET_PATH)?;

    let pair1 = Arc::new((Mutex::new(Vec::new()), Condvar::new()));
    let pair2 = pair1.clone();

    thread::spawn(move || {
        for stream in socket.incoming() {
            let mut stream = match stream {
                Ok(stream) => stream,
                Err(err) => {
                    println!("Error with incoming connection: {}.", err);
                    continue;
                },
            };

            let &(ref lock, ref cvar) = &*pair2;
            let mut buffer = lock.lock().unwrap();
            *buffer = Vec::with_capacity(MAX_REQUEST_SIZE);

            if let Err(err) = stream.read_to_end(&mut buffer) {
                println!("Error with reading request: {}.", err);
            }

            cvar.notify_one();
        }
    });

    let &(ref lock, ref cvar) = &*pair1;
    let mut buffer = lock.lock().unwrap();

    loop {
        let result = cvar.wait_timeout(buffer, Duration::from_millis(global_config.timeout)).unwrap();
        buffer = result.0;

        if result.1.timed_out() {
            display.hide();
            continue;
        }

        match deserialize(&buffer).unwrap() {
            Request::Show { profile, value } => display.show(value, &global_config, &color_configs[&profile]),
            Request::Hide => display.hide(),
            Request::Stop => break,
        }
    }

    fs::remove_file(SOCKET_PATH)?;

    Ok(())
}
