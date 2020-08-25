use serde_json::{Result, Value};

use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub fn init(msg_rx: Receiver<String>) {
    thread::spawn(move || {
        print_msg(msg_rx);
    });
}

fn flatten(prefix: String, msg: &Value) -> Vec<(String, &Value)> {
    match msg {
        Value::Null => vec![(prefix, msg)],
        Value::Bool(_) => vec![(prefix, msg)],
        Value::Number(_) => vec![(prefix, msg)],
        Value::String(_) => vec![(prefix, msg)],

        Value::Array(_) => {
            let arr = &msg.as_array().unwrap();
            let prefix = if prefix.len() > 0 {prefix + "."} else {prefix};
            arr.iter().enumerate()
                .flat_map(|(x,y)| flatten(prefix.clone() + &x.to_string(), y))
                .collect()
        }
        Value::Object(_) => {
            let obj = &msg.as_object().unwrap();
            let prefix = if prefix.len() > 0 {prefix + "."} else {prefix};
            obj.iter()
                .flat_map(|(x, y)| flatten(prefix.clone() + x, y))
                .collect()
        }
    }
}

fn print_msg(msg_rx: Receiver<String>) {
    loop {
        let s = msg_rx.recv().unwrap();
        let x = serde_json::from_str(&s);
        if x.is_ok() {
            let x: Value = x.unwrap();
            println!("{}", x);
            let flat = flatten(String::from(""), &x);
            println!("{:?} len: {}", flat, flat.len());
        } else {
            println!("Could not parse str of len {} : {}", s.len(), s);
        }
    }
}
