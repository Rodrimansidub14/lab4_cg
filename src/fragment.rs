use nalgebra_glm::{Vec2, Vec3};
use crate::color::Color;

#[derive(Clone, Copy)]
pub enum CelestialType {
    Star,
    Planet,
    GasGiant,
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
    pub vertex_position: Vec3,
    pub celestial_type: CelestialType, // Tipo de cuerpo celeste
}

impl Fragment {
    pub fn new(
        x: f32, 
        y: f32, 
        color: Color, 
        depth: f32, 
        normal: Vec3, 
        intensity: f32, 
        vertex_position: Vec3, 
        celestial_type: CelestialType
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
