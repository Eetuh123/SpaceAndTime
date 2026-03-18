use glam::Vec3;

// Euclidean Distance | Euklidinen etäisyys
// d = √((x2-x1)² + (y2-y1)² + (z2-z1)²)
// a: First point coordinates
// b: Second point coordinates
// returns: returns distance between A and B
pub fn distance(a: Vec3, b: Vec3 ) -> f32 {
    (f32::sqrt((a.x - b.x).powi(2)+(a.y - b.y).powi(2)+(a.z - b.z).powi(2)))
}