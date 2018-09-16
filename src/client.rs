use bincode::serialize;
use server;
use std::io::Write;
use std::os::unix::net::UnixStream;

const TIMEOUT: i32 = 1000;

pub fn show(profile: String, value: f64) {
    let mut socket = match UnixStream::connect("/tmp/rob") {
        Ok(sock) => sock,
        Err(e) => {
            println!("Couldn't connect: {:?}", e);
            return
        }
    };

    socket.write(&serialize(&server::Request::Show { profile, value }).unwrap()).unwrap();
}
