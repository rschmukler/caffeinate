use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub enum Command {
    Disable,
    Enable,
    #[allow(dead_code)]
    TriggerNow,
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
        use Command::*;
        let byte = match cmd {
            Disable => 0x00_u8,
            Enable => 0x01_u8,
            TriggerNow => 0x02_u8,
        };

        self.stream.write(&[byte]).map(|_| ())
    }
}
