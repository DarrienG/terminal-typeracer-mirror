use crossbeam_channel::Sender;
use std::{io::stdin, thread};
use termion::{event::Key, input::TermRead};

pub fn capture(sender: Sender<Key>) {
    thread::spawn(|| capture_internal(sender));
}

fn capture_internal(sender: Sender<Key>) {
    let mut keys = stdin().keys();
    loop {
        match sender.send(keys.find_map(Result::ok).unwrap()) {
            Ok(_) => (), // continue just fine
            Err(_) => return, // sender thread has died, either we have quit the game or the thread has died
                              // either way we need to exit
        };
    }
}
