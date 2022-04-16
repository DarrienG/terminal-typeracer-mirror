use std::{
    io::stdin,
    sync::mpsc::{Receiver, Sender},
    thread,
};
use termion::{event::Key, input::TermRead};

pub fn capture(sender: Sender<Key>, receiver: Receiver<bool>) {
    thread::spawn(|| capture_internal(sender, receiver));
}

fn capture_internal(sender: Sender<Key>, receiver: Receiver<bool>) {
    let stdin = &mut stdin();
    loop {
        match sender.send(stdin.keys().find_map(Result::ok).unwrap()) {
            Ok(_) => (), // continue just fine
            Err(_) => return, // sender thread has died, either we have quit the game or the thread has died
                              // either way we need to exit
        };
    }
}
