use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

use std::collections::HashMap;
use std::collections::HashSet;
//use sqlx::postgres::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;

mod handler;
mod msg_decoder;
mod serial_listener;
mod server;
mod ws;

fn main() {
    //This pipe caries data from the serial port threads to msg_decoder thread
    let (msg_tx, msg_rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    //All active serial ports are registered here
    let mut active_ports: HashMap<String, (Sender<Option<String>>, thread::JoinHandle<()>)> =
        HashMap::new();

    //All web clients are registered here
    let clients: crate::server::Clients = Arc::new(RwLock::new(HashMap::new()));

    let (data_tx, mut data_rx1): (
        tokio::sync::broadcast::Sender<String>,
        tokio::sync::broadcast::Receiver<String>,
    ) = tokio::sync::broadcast::channel(100);

    //Spawns a new thread that runs tokio async tasks
    std::thread::spawn(move || {
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        
        //print data from MPMC broadcast channel
        let print_fut = async move {
            println!("Kek");
            loop {
                let rx = data_rx1.recv().await.expect("Error");
                println!("{}", rx);
            }
        };

        //start the websocket server
        let serve_fut = server::server(clients.clone());

        //join all tasks
        rt.block_on(async move {
            tokio::join!(print_fut, serve_fut);
        });
    });

    //The msg_decoder turns lines from connected serial ports into KV data structures
    msg_decoder::init(msg_rx, data_tx);

    //greedy USB serial connection loop
    loop {
        let new_ports: HashSet<String> = serialport::available_ports()
            .expect("Could not read serial ports!")
            .into_iter()
            .filter(|p| !matches!(p.port_type, serialport::SerialPortType::Unknown)) //Why is this not USB?
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
