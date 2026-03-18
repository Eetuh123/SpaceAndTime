use glam::Vec3;

/// Kinetic Energy
/// Ek = 0.5 * m * v²
/// m: mass in kg
/// v: velocity in m/s
/// returns: energy (E) in Joules
pub fn kinetic_energy(mass: f32, velocity: f32) -> f32 {
    0.5 * mass * velocity.powi(2)
}
// Newton's law of gravitation | Newtonin gravitaatiolaki
// F = G × (m1 × m2) / r²
// G: Gravitational constant
// m1: mass of object 1
// m2: mass of object 2
// r: distance between the objects (Middle point)
// returns: gravitational force (F) in Newtons
pub fn universal_gravitation(g: f32, m1: f32, m2: f32, r: f32) -> f32 {
    g * (m1 * m2) / r.powf(2.0)
}
// Standard velocity (v) formula | Standardi nopeus kaava
// v = v + (F / m) * Δt
// v: velocity / current velocity
// F: force being applied
// m: mass of the object
// Δt: time from last interval
// returns: new velocity value
pub fn velocity(v: Vec3, f: Vec3, m: f32, t: f32,) -> Vec3 {
    (v + (f / m) * t)
}
// Position update | Paikan päivitys
// x = x + v * Δt
// x: current position (cordinates)
// v: current velocity
// Δt: time from last interval
// returns: new position
pub fn position(x: Vec3, v: Vec3, t: f32) -> Vec3 {
    (x + v * t)
}