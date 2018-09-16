use bincode::serialize;
use server;
use std::io::Write;
use std::os::unix::net::UnixStream;
use super::Result;

pub fn show(profile: String, value: f64) -> Result<()> {
    let mut socket = UnixStream::connect(server::SOCKET_PATH)?;
    let serialized_request = serialize(&server::Request::Show { profile, value })?;
    socket.write_all(&serialized_request)?;
    Ok(())
}

pub fn hide() -> Result<()> {
    let mut socket = UnixStream::connect(server::SOCKET_PATH)?;
    let serialized_request = serialize(&server::Request::Hide)?;
    socket.write_all(&serialized_request)?;
    Ok(())
}

pub fn stop() -> Result<()> {
    let mut socket = UnixStream::connect(server::SOCKET_PATH)?;
    let serialized_request = serialize(&server::Request::Stop)?;
    socket.write_all(&serialized_request)?;
    Ok(())
}
