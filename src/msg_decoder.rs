use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use serde_json::{Result, Value};

pub fn init(msg_rx : Receiver<String>){
    thread::spawn(move || {
        print_msg(msg_rx);
    });
}

fn print_msg(msg_rx : Receiver<String>){
    loop {
        let s = msg_rx.recv().unwrap();
        let x = serde_json::from_str(&s);
        if x.is_ok(){
            let x:Value = x.unwrap();
            //println!("{}", x["tick"]);
        }else{
            println!("Could not parse str of len {} : {}", s.len(), s);
        }
        
    }     
}
