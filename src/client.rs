use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;

const LOCAL: &str = "127.0.0.1:6000";
const BUF_SIZE: usize = 32;

pub fn run() {
    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop {
        let mut buf = vec![0; BUF_SIZE];
        match client.read_exact(&mut buf) {
            Ok(_) => {
                let msg = buf.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("Message received: {:?}", msg);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server was severed");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buf = msg.clone().into_bytes();
                buf.resize(BUF_SIZE, 0);
                client.write_all(&buf).expect("Writing socket failed");
                println!("Message sent {:?}, msg", msg);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break,
        }

        thread::sleep(::std::time::Duration::from_millis(100));
    });

    println!("Write a message: ");
    loop {
        let mut buf = String::new();
        io::stdin()
            .read_line(&mut buf)
            .expect("Reading from stdin failed");
        let msg = buf.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }
    println!("Bye!");
}
