use std::sync::Mutex;
use std::net::SocketAddr;
use once_cell::sync::Lazy;
use crate::models::snake::Snake;
use tokio::sync::Mutex as TokioMutex;

pub struct Player {
    pub id: String,
    pub name: String,
    pub score: i32,
    pub current_rank: String,
    pub snake: Snake,
    pub addr: SocketAddr,
    pub move_x: f64,
    pub move_y: f64,
    pub window_w: f64,
    pub window_h: f64,
    pub last_seen: std::time::Instant,
}

impl Clone for Player {
    fn clone(&self) -> Self {
        Player {
            id: self.id.clone(),
            name: self.name.clone(),
            score: self.score,
            current_rank: self.current_rank.clone(),
            snake: self.snake.clone(),
            addr: self.addr,
            move_x: self.move_x,
            move_y: self.move_y,
            window_w: self.window_w,
            window_h: self.window_h,
            last_seen: self.last_seen,
        }
    }
}

// Use pub here to make it accessible from game_server.rs
pub static PLAYERS: Lazy<Mutex<Vec<Option<Player>>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn create(id: String, name: String, score: i32, current_rank: String, snake: Snake, addr: SocketAddr) -> Player {
    let player = Player {
        id,
        name,
        score,
        current_rank,
        snake,
        addr,
        move_x: 0.0,
        move_y: 0.0,
        window_w: 0.0,
        window_h: 0.0,
        last_seen: std::time::Instant::now(),
    };
    
    // Add player to collection
    let mut players = PLAYERS.lock().unwrap();
    players.push(Some(player.clone()));
    
    println!("{}:{}", addr.ip(), addr.port());
    player
}

pub fn get_snake(id: usize) -> Option<Snake> {
    let players = PLAYERS.lock().unwrap();
    if id < players.len() && players[id].is_some() {
        Some(players[id].as_ref().unwrap().snake.clone())
    } else {
        None
    }
}

pub fn read(id: usize) -> Option<Player> {
    let players = PLAYERS.lock().unwrap();
    if id < players.len() {
        players[id].clone()
    } else {
        None
    }
}

pub fn destroy(id: usize) {
    let mut players = PLAYERS.lock().unwrap();
    players.remove(id);
}

pub fn keys() -> Vec<usize> {
    let players = PLAYERS.lock().unwrap();
    players.iter()
        .enumerate()
        .filter(|(_, player)| player.is_some())
        .map(|(i, _)| i)
        .collect()
}

pub fn length() -> usize {
    keys().len()
}

pub fn update_xy(player: &mut Player, x: f64, y: f64) {
    player.move_x = x;
    player.move_y = y;
}

pub fn update_player_xy(id: usize, x: f64, y: f64, window_w: f64, window_h: f64) {
    let mut players = PLAYERS.lock().unwrap();
    if id < players.len() && players[id].is_some() {
        if let Some(ref mut player) = players[id] {
            player.move_x = x;
            player.move_y = y;
            player.window_w = window_w;
            player.window_h = window_h;
            player.last_seen = std::time::Instant::now();
        }
    }
}

pub fn update_player_name(id: usize, name: String) {
    let mut players = PLAYERS.lock().unwrap();
    if id < players.len() && players[id].is_some() {
        if let Some(ref mut player) = players[id] {
            player.name = name;
            player.last_seen = std::time::Instant::now();
        }
    }
}

pub fn update_player_acceleration(id: usize, accelerate: bool) {
    let mut players = PLAYERS.lock().unwrap();
    if id < players.len() && players[id].is_some() {
        if let Some(ref mut player) = players[id] {
            player.snake.accelerate = accelerate;
            player.last_seen = std::time::Instant::now();
        }
    }
}

pub fn update_player_snake(id: usize, new_snake: Snake) {
    let mut players = PLAYERS.lock().unwrap();
    if id < players.len() && players[id].is_some() {
        if let Some(ref mut player) = players[id] {
            player.snake = new_snake;
        }
    }
}

pub fn grow_player_snake(id: usize) {
    let mut players = PLAYERS.lock().unwrap();
    if id < players.len() && players[id].is_some() {
        if let Some(ref mut player) = players[id] {
            crate::models::snake::grow(&mut player.snake);
        }
    }
}

pub fn find_id_by_addr(addr: &SocketAddr) -> Option<usize> {
    let players = PLAYERS.lock().unwrap();
    for (i, player_opt) in players.iter().enumerate() {
        if let Some(player) = player_opt {
            if player.addr == *addr {
                return Some(i);
            }
        }
    }
    None
}

pub fn update_last_seen(id: usize) {
    let mut players = PLAYERS.lock().unwrap();
    if id < players.len() && players[id].is_some() {
        if let Some(player) = &mut players[id] {
            player.last_seen = std::time::Instant::now();
        }
    }
}

// Remove players that haven't been seen in a while (UDP connection management)
pub fn clean_inactive_players(timeout_secs: u64) -> Vec<usize> {
    let mut inactive_ids = Vec::new();
    let mut players = PLAYERS.lock().unwrap();
    
    for (i, player_opt) in players.iter().enumerate() {
        if let Some(player) = player_opt {
            let elapsed = player.last_seen.elapsed().as_secs();
            if elapsed > timeout_secs {
                inactive_ids.push(i);
            }
        }
    }
    
    // Remove inactive players
    for id in &inactive_ids {
        // if *id < players.len() {
        //     players[*id] = None;
        // }
        players.remove(*id);
    }
    
    inactive_ids
} 