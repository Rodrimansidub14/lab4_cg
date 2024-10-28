// src/uniforms.rs

use nalgebra_glm::Mat4;
use fastnoise_lite::FastNoiseLite;
use std::sync::Arc;

/// Estructura que contiene todos los parámetros uniformes para los shaders
pub struct Uniforms {
    pub model_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub viewport_matrix: Mat4,
    pub time: f32, // Cambiado a f32 para mayor precisión en cálculos de tiempo
    pub noise: Arc<FastNoiseLite>,
    pub noise_scale: f32,
    pub ocean_threshold: f32,
    pub continent_threshold: f32,
    pub mountain_threshold: f32,
    pub light_direction: nalgebra_glm::Vec3, // Dirección de la luz
}
