#![feature(async_await)]

use clap::{value_t_or_exit, App, Arg};

mod client;

use client::{Command, XIdleHookClient};
use crossbeam_channel::{after, bounded, never, select, Receiver};
use std::time::Duration;

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main() {
    let matches = App::new("caffeinate")
        .version("1.0")
        .about("Keeping xidlehook woke since 2019")
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
        .get_matches();

    let socket = matches.value_of("socket").expect("no socket path provided");
    let mut client = XIdleHookClient::new(socket).expect("Error connecting to xidlehook socket");

    let quit_timer = if matches.is_present("timer") {
        let secs = value_t_or_exit!(matches, "timer", u64);
        let duration = Duration::new(secs, 0);
        after(duration)
    } else {
        never()
    };

    let ctrl_c_event = ctrl_channel().expect("Error wiring up ctrl-c listener");
    client
        .send(Command::Disable)
        .expect("error communicating with xidlehook");

    select! {
        recv(quit_timer) -> _ => {
            println!("Times up! Goodbye");
        }
        recv(ctrl_c_event) -> _ => {
            println!("Shutting down...");
        }
    }

    client
        .send(Command::Enable)
        .expect("error communicating with xidlehook");
}
