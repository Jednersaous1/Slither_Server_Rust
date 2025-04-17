use std::sync::Mutex;
use once_cell::sync::Lazy;

pub struct Bait {
    pub x: f64,
    pub y: f64,
    pub color: String,
    pub size: f64,
}

static BAITS: Lazy<Mutex<Vec<Bait>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn create(x: f64, y: f64, color: String, size: f64) -> Bait {
    let new_bait = Bait {
        x,
        y,
        color,
        size,
    };
    
    BAITS.lock().unwrap().push(new_bait.clone());
    new_bait
}

pub fn read(id: usize) -> Option<Bait> {
    BAITS.lock().unwrap().get(id).cloned()
}

pub fn destroy(id: usize) {
    let mut baits = BAITS.lock().unwrap();
    if id < baits.len() {
        baits.remove(id);
    }
}

pub fn keys() -> Vec<usize> {
    let baits = BAITS.lock().unwrap();
    (0..baits.len()).collect()
}

pub fn length() -> usize {
    BAITS.lock().unwrap().len()
}

impl Clone for Bait {
    fn clone(&self) -> Self {
        Bait {
            x: self.x,
            y: self.y,
            color: self.color.clone(),
            size: self.size,
        }
    }
} 