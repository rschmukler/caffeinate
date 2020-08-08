extern crate serde;

use serde::{Deserialize, Serialize};

use std::io::{self, Write, LineWriter};
use std::os::unix::net::UnixStream;
use std::path::Path;

#[repr(u8)]
pub enum Command {
    Disable = 0,
    Enable = 1,
    #[allow(dead_code)]
    TriggerNow = 2,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum Filter {
    All,
}

impl Default for Filter {
    fn default() -> Self {
        Self::All
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum Action {
    Disable,
    Enable,
    Trigger,
}

#[derive(Debug, Deserialize, Serialize)]
struct Control {
    #[serde(default)]
    pub timer: Filter,
    pub action: Action,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum Message {
    Control(Control),
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
        let packet = Message::Control(Control {
            timer: Filter::All,
            action: match cmd {
                Command::Disable => Action::Disable,
                Command::Enable => Action::Enable,
                Command::TriggerNow => Action::Trigger,
            },
        });

        let mut writer = LineWriter::new(&self.stream);

        serde_json::to_writer(&mut writer, &packet)?;
        writer.write_all(&[b'\n'])?;
        writer.flush()?;

        // xidlehook's new socket api sends a response back on successful recv.
        // this function could check the response ("Empty") before returning Ok.

        Ok(())
    }
}
