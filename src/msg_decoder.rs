use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub fn init(msg_rx : Receiver<String>){
    thread::spawn(move || {
        print_msg(msg_rx);
    });
}

fn print_msg(msg_rx : Receiver<String>){
    loop {
        let s = msg_rx.recv();
        if s.is_ok() {
            println!("{}", s.unwrap());
        }
    }     
}
