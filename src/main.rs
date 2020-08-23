use std::str;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use std::io::{self, Write};
use std::time::Duration;

use clap::{App, AppSettings, Arg};
use serialport::prelude::*;

use std::collections::HashMap;
use std::collections::HashSet;

use std::io::BufReader;
use std::io::BufRead;

fn main() {
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let mut active_ports: HashMap<String, thread::JoinHandle<()>> = HashMap::new();

    loop {
        let new_ports: HashSet<String> = serialport::available_ports()
            .expect("Could not read serial ports!")
            .into_iter()
            .filter(|p| !matches!(p.port_type, serialport::SerialPortType::Unknown))
            .map(|x| x.port_name)
            .collect();

        for port in &new_ports {
            if !active_ports.contains_key(port) {
                let tx = tx.clone();
                let copyport = port.clone();
                let thread = thread::spawn(move || {
                    connect_port(copyport, tx);
                });
                active_ports.insert(port.clone(), thread);
            }
        }

        active_ports = active_ports
            .drain()
            .filter_map(|(port, thread)| {
                if new_ports.contains(&port) {
                    Some((port, thread))
                } else {
                    thread.join().expect("Thread failed");
                    None
                }
            })
            .collect();

        loop{
            let s = rx.try_recv();
            if s.is_ok(){
                println!("{}", s.unwrap());
            }else{
                break;
            }
        }
    }
}

fn connect_port(port_name: String, pipe: Sender<String>) {
    println!("Connected: {}", port_name);

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);

    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            port.write_data_terminal_ready(true)
                .expect("Error Setting DTR");

            let mut b = BufReader::new(port);

            loop {
                let mut s = String::new();
                match b.read_line(&mut s){
                    Ok(t) => {
                        s = s.trim().to_owned();
                        pipe.send(s);
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => {
                        println!("Disconnected: {}", port_name);
                        break;
                    }
                    
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
        }
    }
}
