use std::env;
mod client;
mod server;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "client" {
        client::run();
    } else {
        server::run();
    }
}
