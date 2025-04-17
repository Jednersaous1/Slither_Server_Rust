use slither_io_server::models::{bait, snake};
use slither_io_server::game::listen_server;
use std::env;

#[tokio::main]
async fn main() {
    // Set up logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    
    println!("Slither.io Server Rust Implementation");
    println!("Starting UDP game server...");
    
    // Start the UDP server
    match listen_server::run().await {
        Ok(_) => println!("Server stopped normally"),
        Err(e) => eprintln!("Server error: {}", e),
    }
} 