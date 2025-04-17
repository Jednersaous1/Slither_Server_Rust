use std::sync::Mutex;
use std::f64::consts::PI;
use once_cell::sync::Lazy;
use rand::Rng;
use crate::game::constants as CONST;

pub struct Node {
    pub x: f64,
    pub y: f64,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            x: self.x,
            y: self.y,
        }
    }
}

pub struct Snake {
    pub length: f64,
    pub skin: i32,
    pub speed: f64,
    pub current_speed_sec: f64,
    pub nodes: Vec<Node>,
    pub current_angle: f64,
    pub rotate_angle: f64,
    pub is_dead: bool,
    pub accelerate: bool,
    pub accelerate_time: f64,
}

impl Clone for Snake {
    fn clone(&self) -> Self {
        Snake {
            length: self.length,
            skin: self.skin,
            speed: self.speed,
            current_speed_sec: self.current_speed_sec,
            nodes: self.nodes.clone(),
            current_angle: self.current_angle,
            rotate_angle: self.rotate_angle,
            is_dead: self.is_dead,
            accelerate: self.accelerate,
            accelerate_time: self.accelerate_time,
        }
    }
}

static SNAKES: Lazy<Mutex<Vec<Option<Snake>>>> = Lazy::new(|| Mutex::new(Vec::new()));

fn random(low: f64, high: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(low..high)
}

fn create_first_five_nodes(initial_x: f64, initial_y: f64) -> Vec<Node> {
    let mut nodes = Vec::new();
    
    nodes.push(Node {
        x: initial_x,
        y: initial_y,
    });
    
    for _ in 1..CONST::SNAKE_INITIAL_LENGTH {
        nodes.push(Node {
            x: nodes.last().unwrap().x + CONST::SNAKE_NODE_SPACE,
            y: nodes.last().unwrap().y + CONST::SNAKE_NODE_SPACE,
        });
    }
    
    nodes
}

pub fn create(length: f64, skin: i32, speed: f64) -> Snake {
    let map_border_w = CONST::BORDER_WIDTH - CONST::MAP_WIDTH;
    let map_border_h = CONST::BORDER_HEIGHT - CONST::MAP_HEIGHT;
    
    let initial_x = random(
        map_border_w / 2.0 + 500.0,
        map_border_w / 2.0 + CONST::MAP_WIDTH - 500.0
    );
    
    let initial_y = random(
        map_border_h / 2.0 + 500.0,
        map_border_h / 2.0 + CONST::MAP_HEIGHT - 500.0
    );
    
    let default_nodes = create_first_five_nodes(initial_x, initial_y);
    
    let snake = Snake {
        length,
        skin,
        speed,
        current_speed_sec: 0.0,
        nodes: default_nodes,
        current_angle: 0.0,
        rotate_angle: 0.0,
        is_dead: false,
        accelerate: false,
        accelerate_time: 0.0,
    };
    
    // Add snake to collection
    let mut snakes = SNAKES.lock().unwrap();
    snakes.push(Some(snake.clone()));
    
    snake
}

pub fn read(id: usize) -> Option<Snake> {
    let snakes = SNAKES.lock().unwrap();
    if id < snakes.len() {
        snakes[id].clone()
    } else {
        None
    }
}

pub fn destroy(id: usize) {
    let mut snakes = SNAKES.lock().unwrap();
    if id < snakes.len() {
        snakes[id] = None;
    }
}

pub fn keys() -> Vec<usize> {
    let snakes = SNAKES.lock().unwrap();
    snakes.iter()
        .enumerate()
        .filter(|(_, snake)| snake.is_some())
        .map(|(i, _)| i)
        .collect()
}

pub fn length() -> usize {
    let snakes = SNAKES.lock().unwrap();
    snakes.len()
}

pub fn grow(snake: &mut Snake) {
    if snake.nodes.len() < 500 {
        let nodes = &snake.nodes;
        let last_node = nodes.last().unwrap();
        
        snake.nodes.push(Node {
            x: last_node.x,
            y: last_node.y,
        });
    }
}

pub fn new_rotate_angle(snake: &mut Snake, angle: f64) {
    snake.rotate_angle = angle;
}

pub fn rotate(snake: &mut Snake) {
    if snake.rotate_angle > snake.current_angle {
        snake.current_angle = f64::min(
            snake.rotate_angle, 
            snake.current_angle + CONST::SNAKE_ROTATE_SPEED
        );
    } else {
        snake.current_angle = f64::max(
            snake.rotate_angle, 
            snake.current_angle - CONST::SNAKE_ROTATE_SPEED
        );
    }
}

pub fn move_snake(snake: &mut Snake, to_x: f64, to_y: f64, center_x: f64, center_y: f64) {
    if CONST::SERVER_CURRENT_UPDATE_PLAYER_METHOD == 1 {
        let n = snake.nodes.len();
        
        for i in (1..n).rev() {
            snake.nodes[i].x = snake.nodes[i - 1].x;
            snake.nodes[i].y = snake.nodes[i - 1].y;
        }
        
        let dx = to_x - center_x / 2.0;
        let dy = to_y - center_y / 2.0;
        
        let dist = (dx * dx + dy * dy).sqrt();
        
        let norm_x = dx / if dist == 0.0 { 1.0 } else { dist };
        let norm_y = dy / if dist == 0.0 { 1.0 } else { dist };
        
        let vel_x = norm_x * CONST::SNAKE_SPEED;
        let vel_y = norm_y * CONST::SNAKE_SPEED;
        
        snake.nodes[0].x = snake.nodes[0].x + vel_x;
        snake.nodes[0].y = snake.nodes[0].y + vel_y;
        
        // Limit by MAP_BORDER
        if snake.nodes[0].x - CONST::SNAKE_INITIAL_SIZE / 2.0 < CONST::OFFSET_X {
            snake.nodes[0].x = CONST::OFFSET_X + CONST::SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].y - CONST::SNAKE_INITIAL_SIZE / 2.0 < CONST::OFFSET_Y {
            snake.nodes[0].y = CONST::OFFSET_Y + CONST::SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].x + CONST::SNAKE_INITIAL_SIZE / 2.0 > CONST::TRUE_MAP_WIDTH {
            snake.nodes[0].x = CONST::TRUE_MAP_WIDTH - CONST::SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].y + CONST::SNAKE_INITIAL_SIZE / 2.0 > CONST::TRUE_MAP_HEIGHT {
            snake.nodes[0].y = CONST::TRUE_MAP_HEIGHT - CONST::SNAKE_INITIAL_SIZE / 2.0;
        }
    } else if CONST::SERVER_CURRENT_UPDATE_PLAYER_METHOD == 2 {
        // new method
        let n = snake.nodes.len();
        
        for i in (1..n).rev() {
            let dx = snake.nodes[i - 1].x - snake.nodes[i].x;
            let dy = snake.nodes[i - 1].y - snake.nodes[i].y;
            let dist = (dx * dx + dy * dy).sqrt();
            let node_dist = dist / CONST::SNAKE_NODE_INITIAL_DISTANCE;
            
            let speed = if snake.accelerate {
                CONST::SNAKE_SPEED_ACCELERATE * CONST::SNAKE_SPEED * node_dist
            } else {
                CONST::SNAKE_SPEED * node_dist
            };
            
            let norm_x = dx / if dist == 0.0 { 0.1 } else { dist };
            let norm_y = dy / if dist == 0.0 { 0.1 } else { dist };
            let vel_x = norm_x * speed;
            let vel_y = norm_y * speed;
            
            snake.nodes[i].x += vel_x;
            snake.nodes[i].y += vel_y;
            
            // Limit by MAP_BORDER
            if snake.nodes[i].x - CONST::SNAKE_INITIAL_SIZE / 2.0 < CONST::OFFSET_X {
                snake.nodes[i].x = CONST::OFFSET_X + CONST::SNAKE_INITIAL_SIZE / 2.0;
            }
            if snake.nodes[i].y - CONST::SNAKE_INITIAL_SIZE / 2.0 < CONST::OFFSET_Y {
                snake.nodes[i].y = CONST::OFFSET_Y + CONST::SNAKE_INITIAL_SIZE / 2.0;
            }
            if snake.nodes[i].x + CONST::SNAKE_INITIAL_SIZE / 2.0 > CONST::TRUE_MAP_WIDTH {
                snake.nodes[i].x = CONST::TRUE_MAP_WIDTH - CONST::SNAKE_INITIAL_SIZE / 2.0;
            }
            if snake.nodes[i].y + CONST::SNAKE_INITIAL_SIZE / 2.0 > CONST::TRUE_MAP_HEIGHT {
                snake.nodes[i].y = CONST::TRUE_MAP_HEIGHT - CONST::SNAKE_INITIAL_SIZE / 2.0;
            }
        }
        
        let dx = to_x - center_x / 2.0;
        let dy = to_y - center_y / 2.0;
        let dist = (dx * dx + dy * dy).sqrt();
        
        let norm_x = dx / if dist == 0.0 { 1.0 } else { dist };
        let norm_y = dy / if dist == 0.0 { 1.0 } else { dist };
        
        let vel_x = norm_x * if snake.accelerate {
            CONST::SNAKE_SPEED_ACCELERATE * CONST::SNAKE_SPEED
        } else {
            CONST::SNAKE_SPEED
        };
        
        let vel_y = norm_y * if snake.accelerate {
            CONST::SNAKE_SPEED_ACCELERATE * CONST::SNAKE_SPEED
        } else {
            CONST::SNAKE_SPEED
        };
        
        snake.nodes[0].x = snake.nodes[0].x + vel_x;
        snake.nodes[0].y = snake.nodes[0].y + vel_y;
        
        // Limit by MAP_BORDER
        if snake.nodes[0].x - CONST::SNAKE_INITIAL_SIZE / 2.0 < CONST::OFFSET_X {
            snake.nodes[0].x = CONST::OFFSET_X + CONST::SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].y - CONST::SNAKE_INITIAL_SIZE / 2.0 < CONST::OFFSET_Y {
            snake.nodes[0].y = CONST::OFFSET_Y + CONST::SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].x + CONST::SNAKE_INITIAL_SIZE / 2.0 > CONST::TRUE_MAP_WIDTH {
            snake.nodes[0].x = CONST::TRUE_MAP_WIDTH - CONST::SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].y + CONST::SNAKE_INITIAL_SIZE / 2.0 > CONST::TRUE_MAP_HEIGHT {
            snake.nodes[0].y = CONST::TRUE_MAP_HEIGHT - CONST::SNAKE_INITIAL_SIZE / 2.0;
        }
    }
}

pub fn shorter(snake: &mut Snake) {
    if !snake.nodes.is_empty() {
        snake.nodes.pop();
    }
} 