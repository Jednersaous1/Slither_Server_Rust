use crate::game::constants as CONST;
use crate::models::{player, bait, snake};
use crate::game::collision::{Rect, rect_intersect};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::{self, Duration};
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use uuid::Uuid;
use rand::prelude::*;
use std::time::SystemTime;

// UDP packet structure
pub struct UdpPacket {
    pub addr: SocketAddr,
    pub data: Vec<u8>,
}

// Game server state
struct GameServer {
    socket: Arc<UdpSocket>,
    running: Arc<AtomicBool>,
}

// A Channel to send messages to clients
type UdpSender = mpsc::Sender<UdpPacket>;

// Generate random bait
fn generate_bait(low: f64, high: f64) -> bait::Bait {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(low..high);
    let y = rng.gen_range(low..high);
    
    let color = rng.gen_range(0..CONST::MAX_BAIT_COLOR_RANGE).to_string();
    let size = rng.gen_range(0.0..CONST::MAX_BAIT_SIZE);
    
    bait::create(x, y, color, size)
}

// Generate specific bait at a location
fn generate_specific_bait(x: f64, y: f64, color: i32, size: f64) -> bait::Bait {
    bait::create(x, y, color.to_string(), size)
}

// Generate mass baits based on a dead snake
fn generate_mass_bait(snake: &snake::Snake) -> Vec<bait::Bait> {
    let mut new_bait_arr = Vec::new();
    let mut rng = rand::thread_rng();
    let color = rng.gen_range(0..CONST::MAX_BAIT_COLOR_RANGE).to_string();
    
    for i in (0..snake.nodes.len()).step_by(2) {
        if i >= snake.nodes.len() - 1 {
            break;
        }
        
        let offset_x = rng.gen_range(-5.0..5.0);
        let offset_y = rng.gen_range(-5.0..5.0);
        
        let new_bait = bait::create(
            snake.nodes[i].x + offset_x,
            snake.nodes[i].y + offset_y,
            color.clone(),
            CONST::MAX_BAITS_SIZE_ON_DEAD as f64
        );
        
        new_bait_arr.push(new_bait);
    }
    
    new_bait_arr
}

// The main game loop
async fn game_loop(socket: Arc<UdpSocket>, tx: UdpSender) {
    println!("Game loop started");
    
    let mut interval = time::interval(Duration::from_millis(CONST::GAME_LOOP_DELAY as u64));
    
    loop {
        interval.tick().await;
        
        let mut new_bait_arr = Vec::new();
        let mut msg_new_bait_arr = String::new();
        let mut dead_players = Vec::new();
        
        // Create new bait if needed
        if bait::length() < CONST::MAX_BAITS as usize {
            new_bait_arr.push(generate_bait(
                CONST::OFFSET_X + 10.0, 
                CONST::TRUE_MAP_WIDTH - 10.0
            ));
        }
        
        // Update all player positions
        let player_keys = player::keys();
        
        for &i in &player_keys {
            if let Some(mut player_i) = player::read(i) {
                // Handle snake acceleration and shortening
                if player_i.snake.accelerate {
                    if player_i.snake.nodes.len() > CONST::SNAKE_INITIAL_LENGTH {
                        if player_i.snake.accelerate_time < CONST::SNAKE_IT_IS_TIME_TO_SHORTER as f64 {
                            player_i.snake.accelerate_time += 1.0;
                        } else {
                            player_i.snake.accelerate_time = 0.0;
                            
                            let last_node = &player_i.snake.nodes[player_i.snake.nodes.len() - 1];
                            let mut rng = rand::thread_rng();
                            let color = rng.gen_range(0..CONST::MAX_BAIT_COLOR_RANGE);
                            
                            new_bait_arr.push(generate_specific_bait(
                                last_node.x,
                                last_node.y,
                                color,
                                5.0
                            ));
                            
                            // Remove last node
                            snake::shorter(&mut player_i.snake);
                        }
                    }
                }
                
                // Move the snake
                snake::move_snake(
                    &mut player_i.snake,
                    player_i.move_x,
                    player_i.move_y,
                    player_i.window_w,
                    player_i.window_h
                );
                
                // Update the player in the collection
                player::update_player_snake(i, player_i.snake.clone());
            }
        }
        
        // Check if a player hits another player
        for &i in &player_keys {
            if let Some(player_i) = player::read(i) {
                // If player is already dead, skip
                if dead_players.contains(&i) {
                    continue;
                }
                
                // Check against all other players
                for &j in &player_keys {
                    if i == j {
                        continue; // A player cannot hit itself
                    }
                    
                    if let Some(player_j) = player::read(j) {
                        let player_j_head = Rect {
                            top: player_j.snake.nodes[0].y - CONST::SNAKE_INITIAL_SIZE / 3.0,
                            left: player_j.snake.nodes[0].x - CONST::SNAKE_INITIAL_SIZE / 3.0,
                            right: player_j.snake.nodes[0].x + CONST::SNAKE_INITIAL_SIZE / 3.0,
                            bottom: player_j.snake.nodes[0].y + CONST::SNAKE_INITIAL_SIZE / 3.0,
                        };
                        
                        // Check collision with each node of player i
                        let mut hit = false;
                        for k in 0..player_i.snake.nodes.len() {
                            let player_i_node = Rect {
                                top: player_i.snake.nodes[k].y - CONST::SNAKE_INITIAL_SIZE / 3.0,
                                left: player_i.snake.nodes[k].x - CONST::SNAKE_INITIAL_SIZE / 3.0,
                                right: player_i.snake.nodes[k].x + CONST::SNAKE_INITIAL_SIZE / 3.0,
                                bottom: player_i.snake.nodes[k].y + CONST::SNAKE_INITIAL_SIZE / 3.0,
                            };
                            
                            if rect_intersect(&player_i_node, &player_j_head) {
                                hit = true;
                                
                                // Generate baits from dead snake
                                let new_bait_on_dead = generate_mass_bait(&player_j.snake);
                                for bait in &new_bait_on_dead {
                                    msg_new_bait_arr.push_str(&format!(
                                        "{}3,{},{},{}",
                                        CONST::COMM_START_NEW_MESS,
                                        bait.x,
                                        bait.y,
                                        bait.size
                                    ));
                                }
                                
                                dead_players.push(j);
                                
                                // Notify player about death
                                let death_msg = format!("{}8", CONST::COMM_START_NEW_MESS);
                                let _ = tx.send(UdpPacket {
                                    addr: player_j.addr,
                                    data: death_msg.into_bytes(),
                                }).await;
                                
                                break;
                            }
                        }
                        
                        if hit {
                            break;
                        }
                    }
                }
            }
        }
        
        // Inform all remaining players about dead players
        let mut msg_dead_players = String::new();
        for &dead_id in &dead_players {
            msg_dead_players.push_str(&format!(
                "{}7,{}",
                CONST::COMM_START_NEW_MESS,
                dead_id
            ));
        }
        
        // Send death notifications to all players
        if !msg_dead_players.is_empty() {
            for &i in &player_keys {
                if let Some(player_i) = player::read(i) {
                    let _ = tx.send(UdpPacket {
                        addr: player_i.addr,
                        data: msg_dead_players.clone().into_bytes(),
                    }).await;
                }
            }
        }
        
        // Send new baits to all players
        if !msg_new_bait_arr.is_empty() {
            for &i in &player_keys {
                if let Some(player_i) = player::read(i) {
                    let _ = tx.send(UdpPacket {
                        addr: player_i.addr,
                        data: msg_new_bait_arr.clone().into_bytes(),
                    }).await;
                }
            }
        }
        
        // Check if a player eats a bait
        let bait_keys = bait::keys();
        let mut deleted_baits = Vec::new();
        let mut grown_players = Vec::new();
        let mut msg_grown_players = String::new();
        
        for &i in &player_keys {
            if let Some(player_i) = player::read(i) {
                let player_i_head = Rect {
                    top: player_i.snake.nodes[0].y - CONST::SNAKE_INITIAL_SIZE / 2.0,
                    left: player_i.snake.nodes[0].x - CONST::SNAKE_INITIAL_SIZE / 2.0,
                    right: player_i.snake.nodes[0].x + CONST::SNAKE_INITIAL_SIZE / 2.0,
                    bottom: player_i.snake.nodes[0].y + CONST::SNAKE_INITIAL_SIZE / 2.0,
                };
                
                for &j in &bait_keys {
                    if let Some(bait_temp) = bait::read(j) {
                        let bait_rect = Rect {
                            top: bait_temp.y - bait_temp.size / 2.0,
                            left: bait_temp.x - bait_temp.size / 2.0,
                            right: bait_temp.x + bait_temp.size / 2.0,
                            bottom: bait_temp.y + bait_temp.size / 2.0,
                        };
                        
                        if rect_intersect(&player_i_head, &bait_rect) {
                            // Grow the snake
                            player::grow_player_snake(i);
                            
                            // New update method notification
                            if CONST::SERVER_CURRENT_SENDING_PLAYER_METHOD == 21 {
                                if let Some(player) = player::read(i) {
                                    let _ = tx.send(UdpPacket {
                                        addr: player.addr,
                                        data: format!("{}22", CONST::COMM_START_NEW_MESS).into_bytes(),
                                    }).await;
                                }
                            }
                            
                            grown_players.push(i);
                            msg_grown_players.push_str(&format!(
                                "{}62,{}",
                                CONST::COMM_START_NEW_MESS,
                                i
                            ));
                            
                            bait::destroy(j);
                            deleted_baits.push(bait_temp);
                        }
                    }
                }
            }
        }
        
        // Inform players about deleted baits
        let mut msg_deleted_baits = String::new();
        for bait in &deleted_baits {
            msg_deleted_baits.push_str(&format!(
                "{}4,{},{}",
                CONST::COMM_START_NEW_MESS,
                bait.x,
                bait.y
            ));
        }
        
        // Send bait deletion and growth notifications
        for &i in &player_keys {
            if let Some(player_i) = player::read(i) {
                if !msg_deleted_baits.is_empty() {
                    let _ = tx.send(UdpPacket {
                        addr: player_i.addr,
                        data: msg_deleted_baits.clone().into_bytes(),
                    }).await;
                }
                
                if !msg_grown_players.is_empty() {
                    let _ = tx.send(UdpPacket {
                        addr: player_i.addr,
                        data: msg_grown_players.clone().into_bytes(),
                    }).await;
                }
            }
        }
        
        // Send each snake back to its player based on the current update method
        if CONST::SERVER_CURRENT_SENDING_PLAYER_METHOD == 2 {
            // Old method: send all nodes
            for &i in &player_keys {
                if let Some(player_i) = player::read(i) {
                    let mut msg_update_player = format!("{}2,", CONST::COMM_START_NEW_MESS);
                    
                    for (j, node) in player_i.snake.nodes.iter().enumerate() {
                        msg_update_player.push_str(&format!("{:.4},{:.4}", node.x, node.y));
                        if j < player_i.snake.nodes.len() - 1 {
                            msg_update_player.push(',');
                        }
                    }
                    
                    let _ = tx.send(UdpPacket {
                        addr: player_i.addr,
                        data: msg_update_player.into_bytes(),
                    }).await;
                }
            }
        } else if CONST::SERVER_CURRENT_SENDING_PLAYER_METHOD == 21 {
            // New method: send only the head
            for &i in &player_keys {
                if let Some(player_i) = player::read(i) {
                    let head = &player_i.snake.nodes[0];
                    let msg_update_player_head = format!(
                        "{}21,{},{}",
                        CONST::COMM_START_NEW_MESS,
                        head.x,
                        head.y
                    );
                    
                    let _ = tx.send(UdpPacket {
                        addr: player_i.addr,
                        data: msg_update_player_head.into_bytes(),
                    });
                }
            }
        }
        
        // Send all snakes to each player based on the update method
        if CONST::SERVER_UPDATE_ENEMY_METHOD == 61 {
            // New method: head only
            for &i in &player_keys {
                if let Some(player_i) = player::read(i) {
                    let mut msg_update_enemies_position = String::new();
                    
                    for &j in &player_keys {
                        if i == j {
                            continue;
                        }
                        
                        if let Some(player_j) = player::read(j) {
                            let head = &player_j.snake.nodes[0];
                            msg_update_enemies_position.push_str(&format!(
                                "{}61,{},{},{}",
                                CONST::COMM_START_NEW_MESS,
                                j,
                                head.x,
                                head.y
                            ));
                        }
                    }
                    
                    if !msg_update_enemies_position.is_empty() {
                        let _ = tx.send(UdpPacket {
                            addr: player_i.addr,
                            data: msg_update_enemies_position.into_bytes(),
                        }).await;
                    }
                }
            }
        } else if CONST::SERVER_UPDATE_ENEMY_METHOD == 6 {
            // Old method: all nodes
            for &i in &player_keys {
                if let Some(player_i) = player::read(i) {
                    let mut msg_update_enemies_position = String::new();
                    
                    for &j in &player_keys {
                        if i == j {
                            continue;
                        }
                        
                        if let Some(player_j) = player::read(j) {
                            msg_update_enemies_position.push_str(&format!(
                                "{}6,{},",
                                CONST::COMM_START_NEW_MESS,
                                j
                            ));
                            
                            for (k, node) in player_j.snake.nodes.iter().enumerate() {
                                msg_update_enemies_position.push_str(&format!("{:.4},{:.4}", node.x, node.y));
                                if k < player_j.snake.nodes.len() - 1 {
                                    msg_update_enemies_position.push(',');
                                }
                            }
                        }
                    }
                    
                    if !msg_update_enemies_position.is_empty() {
                        let _ = tx.send(UdpPacket {
                            addr: player_i.addr,
                            data: msg_update_enemies_position.into_bytes(),
                        }).await;
                    }
                }
            }
        }
        
        // Send all new randomly generated baits
        msg_new_bait_arr.clear();
        for bait in &new_bait_arr {
            msg_new_bait_arr.push_str(&format!(
                "{}3,{},{},{}",
                CONST::COMM_START_NEW_MESS,
                bait.x,
                bait.y,
                bait.size
            ));
        }
        
        if !msg_new_bait_arr.is_empty() {
            for &i in &player_keys {
                if let Some(player_i) = player::read(i) {
                    let _ = tx.send(UdpPacket {
                        addr: player_i.addr,
                        data: msg_new_bait_arr.clone().into_bytes(),
                    }).await;
                }
            }
        }
        
        // Clean up inactive players (UDP connection management)
        let inactive_players = player::clean_inactive_players(30); // 30 seconds timeout
        for id in inactive_players {
            println!("Player {} disconnected due to inactivity", id);
            println!("Total player(s): {}", player::length());
            let msg = format!("{}7,{}", CONST::COMM_START_NEW_MESS, id);
            
            // Notify remaining players
            for &i in &player_keys {
                if i != id {
                    if let Some(player_i) = player::read(i) {
                        let _ = tx.send(UdpPacket {
                            addr: player_i.addr,
                            data: msg.clone().into_bytes(),
                        }).await;
                    }
                }
            }
        }
    }
}

// Process a received packet from a client
pub async fn process_packet(data: &[u8], addr: SocketAddr, tx: &mpsc::Sender<UdpPacket>) {
    let message = String::from_utf8_lossy(data);
    let splitted: Vec<&str> = message.split(',').collect();
    
    if splitted.is_empty() {
        return;
    }
    
    println!("{}", message);

    // Try to find the player by address
    let player_id_opt = player::find_id_by_addr(&addr);

    // println!("{}", player_id_opt.unwrap_or(0));
    
    match splitted[0] {
        "0" => {
            // New connection/player request
            create_player(addr, tx.clone()).await;
        }
        "2" => {
            // Update player's mouse position
            if let Some(player_id) = player_id_opt {
                if splitted.len() >= 5 {
                    player::update_player_xy(
                        player_id,
                        splitted[1].parse().unwrap_or(0.0),
                        splitted[2].parse().unwrap_or(0.0),
                        splitted[3].parse().unwrap_or(0.0),
                        splitted[4].parse().unwrap_or(0.0)
                    );
                }
            }
        }
        "9" => {
            // Player sends their name to all other players
            if let Some(player_id) = player_id_opt {
                if splitted.len() >= 2 {
                    let name = splitted[1].to_string();
                    
                    // Update the player's name
                    player::update_player_name(player_id, name.clone());
                    
                    // Notify all other players
                    let msg_enemy_name = format!(
                        "{}{}{}",
                        CONST::COMM_START_NEW_MESS,
                        CONST::COMM_ENEMY_NAME,
                        player_id
                    );
                    
                    let player_keys = player::keys();
                    for &i in &player_keys {
                        if i != player_id {
                            if let Some(other_player) = player::read(i) {
                                let _ = tx.send(UdpPacket {
                                    addr: other_player.addr,
                                    data: msg_enemy_name.clone().into_bytes(),
                                }).await;
                            }
                        }
                    }
                }
            }
        }
        "10" => {
            // Player is accelerating
            if let Some(player_id) = player_id_opt {
                player::update_player_acceleration(player_id, true);
            }
        }
        "11" => {
            // Player stops accelerating
            if let Some(player_id) = player_id_opt {
                player::update_player_acceleration(player_id, false);
            }
        }
        _ => {}
    }
}

// Create a new player
async fn create_player(addr: SocketAddr, tx: mpsc::Sender<UdpPacket>) -> String {
    let player_id = Uuid::new_v4().to_string();
    println!("New player created: {}", player_id);
    
    // Create a new snake
    let player_snake = snake::create(
        CONST::SNAKE_INITIAL_LENGTH as f64,
        rand::random_range(0..CONST::SNAKE_SKIN_COLOR_RANGE),
        CONST::SNAKE_SPEED
    );
    
    // Create the player
    let new_player = player::create(
        player_id.clone(),
        String::new(),
        0,
        player_id.clone(),
        player_snake.clone(),
        addr
    );
    
    // Send first snake back to the client
    let mut msg = format!("{}1,", CONST::COMM_START_NEW_MESS);
    for (i, node) in player_snake.nodes.iter().enumerate() {
        msg.push_str(&format!("{:.4},{:.4}", node.x, node.y));
        if i < player_snake.nodes.len() - 1 {
            msg.push(',');
        }
    }

    println!("{} {}",addr.ip(), addr.port());
    println!("{}", msg);
        
    let _ = tx.send(UdpPacket {
        addr,
        data: msg.into_bytes(),
    }).await;
    
    // Prepare new enemy message for other players
    let new_enemy_msg = format!(
        "{}5,{},Unnamed,",
        CONST::COMM_START_NEW_MESS,
        player_id
    );
    
    let mut full_enemy_msg = new_enemy_msg;
    for (i, node) in player_snake.nodes.iter().enumerate() {
        full_enemy_msg.push_str(&format!("{:.4},{:.4}", node.x, node.y));
        if i < player_snake.nodes.len() - 1 {
            full_enemy_msg.push(',');
        }
    }
    
    // Send all other players to this new player
    let player_keys = player::keys();
    let mut data = String::new();
    
    for &i in &player_keys {
        if let Some(other_player) = player::read(i) {
            if other_player.id != player_id {
                data.push_str(&format!(
                    "{}{}{}",
                    CONST::COMM_START_NEW_MESS,
                    CONST::COMM_NEW_ENEMY,
                    other_player.id
                ));
                data.push_str(&format!(",{},", other_player.name));
                
                for (j, node) in other_player.snake.nodes.iter().enumerate() {
                    data.push_str(&format!("{},{}", node.x, node.y));
                    if j < other_player.snake.nodes.len() - 1 {
                        data.push(',');
                    }
                }
            }
        }
    }
    
    if !data.is_empty() {
        let _ = tx.send(UdpPacket {
            addr,
            data: data.into_bytes(),
        }).await;
    }
    
    // Send new player to all other players
    for &i in &player_keys {
        if let Some(other_player) = player::read(i) {
            if other_player.id != player_id {
                let _ = tx.send(UdpPacket {
                    addr: other_player.addr,
                    data: full_enemy_msg.clone().into_bytes(),
                }).await;
            }
        }
    }
    
    // Send all baits to the new player
    let bait_keys = bait::keys();
    for &i in &bait_keys {
        if let Some(bait) = bait::read(i) {
            let bait_msg = format!(
                "{}3,{},{},{}",
                CONST::COMM_START_NEW_MESS,
                bait.x,
                bait.y,
                bait.size
            );
            
            let _ = tx.send(UdpPacket {
                addr,
                data: bait_msg.into_bytes(),
            }).await;
        }
    }
    
    println!("Total player(s): {}", player::length());
    player_id
}

// Delete a player
pub async fn delete_player(player_id: usize, tx: &mpsc::Sender<UdpPacket>) {
    // Inform all players about the dead/closed player
    let player_keys = player::keys();
    let data = format!("{}7,{}", CONST::COMM_START_NEW_MESS, player_id);
    
    for &i in &player_keys {
        if i != player_id {
            if let Some(player) = player::read(i) {
                let _ = tx.send(UdpPacket {
                    addr: player.addr,
                    data: data.clone().into_bytes(),
                }).await;
            }
        }
    }
    
    player::destroy(player_id);
    println!("Total player(s): {}", player::length());
}

// Start the game server
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    println!("game_server is running");
    
    // Bind to UDP socket
    let socket = UdpSocket::bind(format!("{}:{}", CONST::SERVER_IP, CONST::SERVER_PORT)).await?;
    let socket = Arc::new(socket);
    
    // Create a channel for sending UDP packets
    let (tx, mut rx) = mpsc::channel::<UdpPacket>(1000);
    
    // Start the game loop
    let game_loop_socket = socket.clone();
    let game_loop_tx = tx.clone();
    let sender_socket = socket.clone();

    let mut buf = [0u8; 1024];
    tokio::spawn(async move {

        while let (size, addr) = socket.recv_from(&mut buf).await.unwrap() {
            if size > 0 {
                let rx_data = &buf[..size];
                println!("Going to process packet");
                process_packet(rx_data, addr, &tx).await;
            } else {
                println!("Error: no data received.");
            }
        }
    });
    
    // Start the packet sender task
    tokio::spawn(async move {
        while let Some(packet) = rx.recv().await {
            let _ = sender_socket.send_to(&packet.data, packet.addr).await;
        }
    });
    
    // Main receive loop
    

    game_loop(game_loop_socket, game_loop_tx).await;

    Ok(())

} 