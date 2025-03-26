use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures::{StreamExt, SinkExt};
use zmq::{Context, Socket, DONTWAIT};
use tokio::sync::{mpsc, RwLock};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone)]
struct Client {
    sender: mpsc::UnboundedSender<String>,
}

struct SharedState {
    clients: RwLock<HashMap<Uuid, Client>>,
    pub_socket: Arc<Mutex<zmq::Socket>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ZMQ Context
    let ctx = Context::new();
    
    // PUB Socket (Rust -> Python)
    let pub_socket = Arc::new(Mutex::new(
        ctx.socket(zmq::PUB).expect("Failed to create PUB socket")
    ));
    pub_socket.lock().unwrap()
        .bind("tcp://*:5555")
        .expect("PUB connection error");

    // SUB Socket (Python -> Rust)
    let sub_socket = Arc::new(Mutex::new(
        ctx.socket(zmq::SUB).expect("Failed to create SUB socket")
    ));
    
    sub_socket.lock().unwrap()
        .connect("tcp://localhost:5556")
        .expect("SUB connection error");
    sub_socket.lock().unwrap()
        .set_subscribe(b"")
        .expect("Failed to set subscribe");

    println!("✅ ZMQ PUB:5555 and SUB:5556 active");

    // Create shared state
    let state = Arc::new(SharedState {
        clients: RwLock::new(HashMap::new()),
        pub_socket: pub_socket.clone(),
    });

    // Clone state for SUB listener
    let sub_state = state.clone();

    // SUB socket listener thread
    tokio::task::spawn_blocking(move || {
        let sub_socket = sub_socket.clone();
        loop {
            let msg = match sub_socket.lock().unwrap().recv_msg(0) {
                Ok(msg) => msg,
                Err(e) if e == zmq::Error::EAGAIN => continue,
                Err(e) => {
                    eprintln!("Critical SUB error: {}", e);
                    break;
                }
            };
            
            if !msg.is_empty() {
                if let Ok(msg_str) = String::from_utf8(msg.to_vec()) {
                    println!("Message from Python: {}", msg_str);
                    
                    // Clone the parts we need before moving into async task
                    if let Some((client_id_str, message_content)) = msg_str.split_once('|') {
                        if let Ok(client_uuid) = Uuid::parse_str(client_id_str) {
                            let message_content = message_content.to_string();
                            let state_clone = sub_state.clone();
                            
                            tokio::spawn(async move {
                                let clients = state_clone.clients.read().await;
                                if let Some(client) = clients.get(&client_uuid) {
                                    if client.sender.send(message_content).is_err() {
                                        eprintln!("Failed to send to client {}", client_uuid);
                                    }
                                } else {
                                    eprintln!("Client {} not found", client_uuid);
                                }
                            });
                        }
                    }
                }
            }
        }
    });

    // WebSocket server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    println!("🚀 WebSocket server started on port 3000");

    while let Ok((stream, addr)) = listener.accept().await {
        let state = state.clone();
        
        tokio::spawn(async move {
            let state = state.clone(); // Clone again for this task
            
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let (mut ws_write, mut ws_read) = ws_stream.split();
                    println!("🔄 New WS connection: {}", addr);

                    // Generate unique ID for this client
                    let client_id = Uuid::new_v4();
                    
                    // Create channel for this client
                    let (tx, mut rx) = mpsc::unbounded_channel();
                    
                    // Add client to shared state
                    {
                        let mut clients = state.clients.write().await;
                        clients.insert(client_id.clone(), Client { sender: tx });
                        println!("🌐 New client connected: {} (Total: {})", client_id, clients.len());
                    }

                    // Task to send messages to client
                    let send_task = tokio::spawn(async move {
                        while let Some(msg) = rx.recv().await {
                            if let Err(e) = ws_write.send(
                                tokio_tungstenite::tungstenite::Message::Text(msg)
                            ).await {
                                eprintln!("Error sending to client {}: {}", client_id, e);
                                break;
                            }
                        }
                    });

                    // Task to receive messages from client
                    let recv_task = {
                        let state = state.clone();
                        tokio::spawn(async move {
                            while let Some(msg) = ws_read.next().await {
                                match msg {
                                    Ok(data) if data.is_text() => {
                                        let text = data.to_string();
                                        println!("Received from client {}: {}", client_id, text);
                                        
                                        // Forward to Python via ZMQ with client ID prefix
                                        let message_with_id = format!("{}|{}", client_id, text);
                                        state.pub_socket.lock().unwrap()
                                            .send(message_with_id.as_bytes(), DONTWAIT)
                                            .unwrap_or_else(|e| {
                                                eprintln!("PUB send error: {}", e)
                                            });
                                    },
                                    Ok(_) => println!("Non-text message received from {}", client_id),
                                    Err(e) => {
                                        eprintln!("Error reading from client {}: {}", client_id, e);
                                        break;
                                    }
                                }
                            }
                        })
                    };

                    // Wait for either task to complete
                    tokio::select! {
                        _ = send_task => {},
                        _ = recv_task => {},
                    }

                    // Remove client from shared state
                    {
                        let mut clients = state.clients.write().await;
                        clients.remove(&client_id);
                        println!("🌐 Client disconnected: {} (Remaining: {})", client_id, clients.len());
                    }

                    println!("🔌 Connection closed: {}", addr);
                },
                Err(e) => eprintln!("WS handshake error: {}", e),
            }
        });
    }
    Ok(())
}