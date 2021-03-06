use serde_json::{Result, Value};

use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::collections::hash_map::HashMap;

pub fn init(msg_rx: Receiver<String>, data_tx: tokio::sync::broadcast::Sender<String>) {
    thread::spawn(move || {
        print_msg(msg_rx, data_tx);
    });
}

fn flatten(prefix: String, msg: Value) -> Vec<(String, Value)> {
    match msg {
        Value::Null => vec![(prefix, msg)],
        Value::Bool(_) => vec![(prefix, msg)],
        Value::Number(_) => vec![(prefix, msg)],
        Value::String(_) => vec![(prefix, msg)],

        Value::Array(_) => {
            let arr = &msg.as_array().unwrap();
            let prefix = if prefix.len() > 0 {prefix + "."} else {prefix};
            arr.iter().enumerate()
                .flat_map(|(x,y)| flatten(prefix.clone() + &x.to_string(), y.clone()))
                .collect()
        }
        Value::Object(_) => {
            let obj = &msg.as_object().unwrap();
            let prefix = if prefix.len() > 0 {prefix + "."} else {prefix};
            obj.iter()
                .flat_map(|(x, y)| flatten(prefix.clone() + x, y.clone()))
                .collect()
        }
    }
}

fn print_msg(msg_rx: Receiver<String>, data_tx: tokio::sync::broadcast::Sender<String>) {
    loop {
        let s = msg_rx.recv().unwrap();
        let x = serde_json::from_str(&s);
        if x.is_ok() {
            let x: Value = x.unwrap();
            //println!("{}", x.as_object().unwrap()["tick"]);
            let id = x.as_object().unwrap()["id"].as_str().unwrap();
            let flat = flatten(id.to_owned(), x);
            let map: HashMap<_, _> = flat.into_iter().collect();
            let s = serde_json::to_string(&map).unwrap();
            data_tx.send(s).unwrap();
        } else {
            println!("Could not parse str of len {} : {}", s.len(), s);
        }

        //let mut s = serde_json::to_string(&flat).unwrap();
        //let s : Vec<String>  = data.iter().filter(|(k,_)| k.find("tick") != None).map(|(k,v)| format!("{} : {}", k, v) ).collect();
        //s.push_str("\r");
        //push data into broadcast channel
        

    }
}
