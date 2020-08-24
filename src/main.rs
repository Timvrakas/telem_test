use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use std::collections::HashMap;
use std::collections::HashSet;

mod serial_listener;
mod msg_decoder;

fn main() {
    let (msg_tx, msg_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let mut active_ports: HashMap<String, (Sender<Option<String>>, thread::JoinHandle<()>)> =
        HashMap::new();

    msg_decoder::init(msg_rx);

    loop {
        let new_ports: HashSet<String> = serialport::available_ports()
            .expect("Could not read serial ports!")
            .into_iter()
            .filter(|p| !matches!(p.port_type, serialport::SerialPortType::Unknown))
            .map(|x| x.port_name)
            .collect();

        for port in &new_ports {
            if !active_ports.contains_key(port) {
                let (cmd_tx, handle) = serial_listener::init(port.clone(), &msg_tx);
                active_ports.insert(port.clone(), (cmd_tx, handle));
            }
        }

        active_ports = active_ports
            .drain()
            .filter_map(|(port, (cmd_tx, thread))| {
                if new_ports.contains(&port) {
                    Some((port, (cmd_tx, thread)))
                } else {
                    drop(cmd_tx);
                    thread.join().expect("Thread failed");
                    None
                }
            })
            .collect();

        
    }
}
