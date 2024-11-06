use nalgebra_glm::{Vec2, Vec3,Vec4};
use crate::color::Color;

#[derive(Clone, Copy)]
pub enum CelestialType {
    Star,
    Planet,
    GasGiant,
    Ringed,
    Rings,
    Planet2,
    Mars,
    Moon,
    Comet,
    Atmosphere, 
    // Para efectos atmosféricos
    // Agrega otros tipos según sea necesario
}


pub struct Fragment {
    pub position: Vec2,
    pub color: Color,
    pub depth: f32,
    pub normal: Vec3,
    pub intensity: f32,
    pub vertex_position: Vec4, // Cambiado a Vec4
    pub celestial_type: CelestialType,
}

impl Fragment {
    pub fn new(
        x: f32,
        y: f32,
        color: Color,
        depth: f32,
        normal: Vec3,
        intensity: f32,
        vertex_position: Vec4, // Ahora es Vec4
        celestial_type: CelestialType,
    ) -> Self {
        Fragment {
            position: Vec2::new(x, y),
            color,
            depth,
            normal,
            intensity,
            vertex_position,
            celestial_type,
        }
    }
}
