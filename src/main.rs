use slither_io_server::models::{bait, snake};
use slither_io_server::game::listen_server;
use std::env;
use std::future::IntoFuture;


fn main() {
    // Set up logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    
    println!("Slither.io Server Rust Implementation");
    println!("Starting UDP game server...");
    
    // Start the UDP server

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            // Initialize the game server
            let server = listen_server::run().await;
            match server {
                Ok(_) => println!("Server started successfully"),
                Err(e) => eprintln!("Failed to start server: {}", e),
            }
        })
} 