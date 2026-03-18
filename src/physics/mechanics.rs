/// Kinetic Energy
/// Ek = 0.5 * m * v²
/// m: mass in kg
/// v: velocity in m/s
/// returns: energy in Joules
pub fn kinetic_energy(mass: f32, velocity: f32) -> f32 {
    0.5 * mass * velocity.powi(2)
}
