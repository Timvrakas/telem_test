use std::io::BufReader;
use std::io::BufRead;
use std::io::{self};
use std::time::Duration;
use serialport::prelude::*;
use std::sync::mpsc::{self,Sender,Receiver};
use std::thread;

/*
This module implements a threaded serial port listener.
It connects to a port and pipes all lines out of a channel,
TODO:
-Implement CMD send channel
-better error handling
*/

pub fn init(port_name: String, msg_tx: &Sender<String>) -> (Sender<Option<String>>, thread::JoinHandle<()>){
    let msg_tx = msg_tx.clone();
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let handle = thread::spawn(move || {
        connect_port(port_name, msg_tx, cmd_rx);
    });
    return (cmd_tx, handle)
}

fn connect_port(port_name: String, msg_tx: Sender<String>, cmd_rx: Receiver<Option<String>>) {
    println!("Connected: {}", port_name);

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10); //should this change?

    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            port.write_data_terminal_ready(true)
                .expect("Error Setting DTR");

            let mut b = BufReader::new(port);

            loop {
                
                match cmd_rx.try_recv(){
                    Ok(_) => (),
                    Err(mpsc::TryRecvError::Empty) => (),
                    Err(mpsc::TryRecvError::Disconnected) => break
                }

                let mut s = String::new();
                match b.read_line(&mut s){
                    Ok(_) => {
                        s = s.trim().to_owned();
                        if s.len() > 0{
                            msg_tx.send(s).expect("Failed to insert");
                        }
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(_) => {
                        break;
                    }
                    
                }
            }
            println!("Disconnected: {}", port_name);
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
        }
    }
}