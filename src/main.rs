//! Start a server that serves `web/index.html` and another server that listens for incoming web socket connections.

use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        ConnectInfo, WebSocketUpgrade,
    },
    response::{Html, IntoResponse},
    routing::get,
    serve, Router,
};
use axum_extra::{headers, TypedHeader};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let frontend = async {
        let app = Router::new().route("/", get(serve_index));
        let listener = TcpListener::bind("0.0.0.0:3000")
            .await
            .expect("can bind to :3000");
        info!("Serving client on localhost:3000");
        serve(listener, app).await.unwrap();
    };

    let backend = async {
        let listener = TcpListener::bind("0.0.0.0:4000")
            .await
            .expect("can bind to :4000");
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .into_make_service_with_connect_info::<SocketAddr>();
        info!("Serving backend on localhost:4000");
        serve(listener, app).await.unwrap();
    };
    tokio::join!(frontend, backend);
}

async fn serve_index() -> impl IntoResponse {
    let content = tokio::fs::read_to_string("./web/index.html")
        .await
        .expect("file is there");
    Html(content)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, who: SocketAddr) {
    //send a ping (unsupported by some browsers) just to kick things off and get a response
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        println!("Pinged {who}...");
    } else {
        println!("Could not send ping {who}!");
        // no Error here since the only thing we can do is to close the connection.
        return;
    }

    if let Some(msg) = socket.recv().await {
        if msg.is_ok() {
            info!("Got message from client!");
        } else {
            println!("client {who} abruptly disconnected");
            return;
        }
    }

    for i in 1..5 {
        if socket
            .send(Message::Text(format!("Hi {i} times!")))
            .await
            .is_err()
        {
            println!("client {who} abruptly disconnected");
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    info!("Closing connection with {who}...");
}
