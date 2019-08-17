use crossbeam_channel::{Receiver, bounded, after};
use std::thread;
use std::path::Path;
use std::time::{Instant,Duration};

pub fn ctrl_c() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(0);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}


pub fn pid(pid: u64) -> Option<Receiver<()>> {
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

pub fn timer(secs: u64) -> Receiver<Instant> {
    after(Duration::from_secs(secs))
}
