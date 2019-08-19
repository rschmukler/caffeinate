use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

#[repr(u8)]
pub enum Command {
    Disable = 0,
    Enable = 1,
    #[allow(dead_code)]
    TriggerNow = 2,
}

pub struct XIdleHookClient {
    stream: UnixStream,
}

impl XIdleHookClient {
    /// Initializes a new XIdleHookClient for a socket at the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let stream = UnixStream::connect(path)?;

        Ok(XIdleHookClient { stream })
    }

    /// Sends the specified command to the XIdleHook socket
    pub fn send(&mut self, cmd: Command) -> Result<(), io::Error> {
        self.stream.write(&[cmd as u8]).map(|_| ())
    }
}
