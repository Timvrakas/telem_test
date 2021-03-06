use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection};

use crate::handler;
use crate::ws;

pub type Result<T> = std::result::Result<T, Rejection>;
pub type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

pub async fn server(clients: Clients, mut data_to_send: tokio::sync::broadcast::Receiver<String>) {
    let clients_new = clients.clone();
    tokio::spawn(async move {
        while let Ok(result) = data_to_send.recv().await {
            for x in clients_new.read().await.values() {
                if let Some(sender) = &x.sender {
                    let _ = sender.send(Ok(Message::text(result.clone())));
                }
            }
        }
    });

    let health_route = warp::path!("health").and_then(handler::health_handler);

    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handler::unregister_handler));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    let sub_route = warp::path("sub")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::sub_handler);

    let static_route = warp::path("html")
        .and(warp::fs::dir("C:\\Users\\timv\\Git\\telem_test\\html"));

    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .or(static_route)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
