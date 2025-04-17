// Helper module for collision detection

pub struct Rect {
    pub top: f64,
    pub left: f64,
    pub right: f64,
    pub bottom: f64,
}

pub fn rect_intersect(r1: &Rect, r2: &Rect) -> bool {
    !(r2.left > r1.right || 
      r2.right < r1.left || 
      r2.top > r1.bottom ||
      r2.bottom < r1.top)
} 