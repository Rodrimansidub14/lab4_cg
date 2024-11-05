// src/uniforms.rs

use nalgebra_glm::{Mat4, Vec3};
use fastnoise_lite::FastNoiseLite;
use std::sync::Arc;
use crate::color::Color;

/// Estructura que contiene todos los parámetros uniformes para los shaders
pub struct Uniforms {
    pub model_matrix: Mat4,
    pub view_matrix: Mat4,
    pub projection_matrix: Mat4,
    pub viewport_matrix: Mat4,
    pub time: f32,
    pub noise: Arc<FastNoiseLite>,
    pub light_direction: Vec3,
    pub noise_scale: f32,
    pub ocean_threshold: f32,
    pub continent_threshold: f32,
    pub mountain_threshold: f32,
    pub snow_threshold: f32,
    // **Parámetros para los anillos**
    pub ring_inner_radius: f32,
    pub ring_outer_radius: f32,
    pub ring_color: Color,
    pub ring_opacity: f32,
    pub ring_frequency: f32,
    pub ring_wave_speed: f32,
    pub ring_rotation_matrix: Mat4, // Nueva matriz de rotación para los anillos
}

impl Uniforms {
    /// Constructor para facilitar la creación de Uniforms
    pub fn new(
        model_matrix: Mat4,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        viewport_matrix: Mat4,
        time: f32,
        noise: Arc<FastNoiseLite>,
        light_direction: Vec3,
        noise_scale: f32,
        ocean_threshold: f32,
        continent_threshold: f32,
        mountain_threshold: f32,
        snow_threshold: f32,
        ring_inner_radius: f32,
        ring_outer_radius: f32,
        ring_color: Color,
        ring_opacity: f32,
        ring_frequency: f32,
        ring_wave_speed: f32,
        ring_rotation_matrix: Mat4, // Asegúrate de pasar la matriz de rotación
    ) -> Self {
        Uniforms {
            model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            noise,
            light_direction,
            noise_scale,
            ocean_threshold,
            continent_threshold,
            mountain_threshold,
            snow_threshold,
            ring_inner_radius,
            ring_outer_radius,
            ring_color,
            ring_opacity,
            ring_frequency,
            ring_wave_speed,
            ring_rotation_matrix, // Inicializar el campo
        }
    }
}
