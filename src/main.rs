use clap::{value_t_or_exit, App, Arg};

mod client;

use client::{Command, XIdleHookClient};
use crossbeam_channel::{after, bounded, never, select, Receiver};
use std::time::Duration;
use std::path::Path;
use std::thread;
use std::str::FromStr;
use std::io;

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(0);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn pid_channel(pid: u64) -> Option<Receiver<()>> {
    let (sender, receiver) = bounded(0);
    let path_str: String = format!("/proc/{:?}", pid);
    let path = Path::new(&path_str);
    if !path.exists() {
        return None
    }
    thread::spawn(move ||{
        let path = Path::new(&path_str);
        loop {
            if !path.exists() {
                sender.send(()).unwrap();
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
    Some(receiver)
}

enum Error {
    InvalidArgument(String)
}

#[derive(PartialEq)]
enum QuitAction {
    Nothing,
    Suspend,
    Shutdown,
    Restart
}

impl QuitAction {
    fn perform(self) -> Result<(), io::Error> {
        if self == Self::Nothing {
            return Ok(())
        }

        let arg =
            match self {
                Self::Suspend => "suspend",
                Self::Restart => "reboot",
                Self::Shutdown => "poweroff",
                Self::Nothing => unreachable!()
            };

        std::process::Command::new("systemctl")
            .arg(arg)
            .spawn()
            .map(|_| ())
    }
}

impl FromStr for QuitAction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nothing" => Ok(Self::Nothing),
            "suspend" => Ok(Self::Suspend),
            "shutdown" => Ok(Self::Shutdown),
            "restart" => Ok(Self::Restart),
            _ => Err(Error::InvalidArgument(s.to_owned()))
        }
    }
}

fn main() {
    let matches = App::new("caffeinate")
        .version("1.0")
        .about("Keeping xidlehook woke")
        .after_help("If multiple triggers are specified, caffeinate will exit after the first one is fired")
        .arg(
            Arg::with_name("socket")
                .short("s")
                .long("socket")
                .value_name("FILE")
                .help("Path to the xidlehook socket")
                .default_value("/tmp/xidlehook.sock"),
        )
        .arg(
            Arg::with_name("timer")
                .short("t")
                .long("timer")
                .value_name("SECONDS")
                .help("Wait a specified number of seconds")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pid")
                .short("p")
                .long("pid")
                .value_name("PROCESS_ID")
                .help("Wait for a process to quit")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("screen-off")
                .short("o")
                .long("screen-off")
                .takes_value(false)
                .help("Turns the screen off on start"),
        )
        .arg(
            Arg::with_name("quit")
                .short("q")
                .long("quit")
                .value_name("SHUTDOWN")
                .default_value("nothing")
                .help("action to perform upon quit [shutdown|suspend|restart|nothing]"),
        )
        .get_matches();

    let socket = matches.value_of("socket").expect("no socket path provided");
    let mut client = XIdleHookClient::new(socket).expect("Error connecting to xidlehook socket");

    let timer_event = if matches.is_present("timer") {
        let secs = value_t_or_exit!(matches, "timer", u64);
        let duration = Duration::new(secs, 0);
        after(duration)
    } else {
        never()
    };

    let ctrl_c_event = ctrl_channel().expect("Error wiring up ctrl-c listener");
    let (pid_event, pid) = if matches.is_present("pid") {
        let pid = value_t_or_exit!(matches, "pid", u64);
        let pid_event = pid_channel(pid).expect(&format!("Process with pid {:?} does not exist", pid));
        (pid_event, Some(pid))
    } else {
        (never(), None)
    };

    let quit_action = value_t_or_exit!(matches, "quit", QuitAction);

    client
        .send(Command::Disable)
        .expect("error communicating with xidlehook");

    if matches.is_present("screen-off") {
        std::process::Command::new("xset")
            .args(&["dpms","force","off"])
            .spawn().map(|_| ()).expect("Error powering off display");
    }

    select! {
        recv(timer_event) -> _ => {
            println!("Times up! Goodbye");
        }
        recv(pid_event) -> _ => {
            println!("Process {:?} exited...", pid);
        }
        recv(ctrl_c_event) -> _ => {
            println!("Shutting down...");
        }
    }

    client
        .send(Command::Enable)
        .expect("error communicating with xidlehook");

    quit_action.perform().expect("Error performing quit action");
}
