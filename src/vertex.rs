use nalgebra_glm::{Vec3, Vec2};
use crate::color::Color;
#[derive(Clone)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub tex_coords: Vec2,
    pub color: Color,
    pub transformed_position: Vec3,
    pub transformed_normal: Vec3,
}

impl Vertex {
    pub fn new(position: Vec3, normal: Vec3, tex_coords: Vec2) -> Self {
        Vertex {
            position,
            normal,
            tex_coords,
            color: Color::black(),
            transformed_position: Vec3::zeros(),
            transformed_normal: Vec3::zeros(),
        }
    }
}
