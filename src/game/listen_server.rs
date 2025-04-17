use crate::game::constants as CONST;
use crate::game::game_server;
use tokio::net::UdpSocket;
use std::sync::Arc;

pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting UDP listen server on {}:{}", CONST::SERVER_IP, CONST::SERVER_PORT);
    
    // For UDP, we don't need a separate listen server since game_server already handles
    // the UDP socket creation and binding. We'll just start the game server directly.
    game_server::run().await
} 