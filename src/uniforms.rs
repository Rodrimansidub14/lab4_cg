use nalgebra_glm::Mat4;
use fastnoise_lite::FastNoiseLite;
use std::sync::Arc;

pub struct Uniforms {
    pub model_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub viewport_matrix: Mat4,
    pub time: u32,
    pub noise: Arc<FastNoiseLite>,
    pub noise_scale: f32, // Nueva escala de ruido
    pub ocean_threshold: f32, // Nuevo umbral para océano
    pub continent_threshold: f32, // Nuevo umbral para continente
    pub mountain_threshold: f32, // Nuevo umbral para montañas
}
