use std::io::{ErrorKind, Read, Write};
//use std::net::UdpSocket;
use std::net::TcpListener;
use std::sync::mpsc; // Multi-producer, single-consumer FIFO queue
use std::thread; // Standard threading

const LOCAL: &str = "127.0.0.1:6000";
const BUF_SIZE: usize = 32;

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("Failed to initiate non-blocking");

    // Server can host multiple clients
    let mut clients = vec![];

    // Initialize the communication channel (one for each client)
    let (tx, rx) = mpsc::channel::<String>();

    loop {
        // Accept client connection on a socket. Accept is used with connection-based socket
        // types. It extracts the first connection request on the queue of pending
        // connections for listening socket.
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connected", addr);

            // Clone new trasnmitter for each connected client
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("Failed to clone client socket"));

            // Spawn a thread
            thread::spawn(move || loop {
                let mut buf = vec![0; BUF_SIZE];

                // Read a message into buffer
                match socket.read_exact(&mut buf) {
                    Ok(_) => {
                        // Covert msg into iterator and remove whitespace
                        let msg = buf.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid UTF-8 message");

                        // Print the received message and client address
                        println!("{}: {:?}", addr, msg);
                        tx.send(msg).expect("Failed to send msg to rx");
                    }

                    // In case Error would block continue receiveing by returning `unit type`.
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),

                    // Other errors will close the connection
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }

                thread::sleep(::std::time::Duration::from_millis(100));
            });

            // Try receive messages from each channel
            if let Ok(msg) = rx.try_recv() {
                clients = clients
                    .into_iter()
                    .filter_map(|mut client| {
                        let mut buf = msg.clone().into_bytes();
                        buf.resize(BUF_SIZE, 0);
                        clients.write_all(&buf).map(|_| client).ok()
                    })
                    .collect::<Vec<_>>();
            }
        }
    }
}
