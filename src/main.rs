// src/main.rs

use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::sync::Arc;

// Importa tus módulos aquí
mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;
mod uniforms;
mod renderer;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use shaders::{fragment_shader, vertex_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType, CellularDistanceFunction};
use renderer::render;
use fragment::{CelestialType, Fragment};
use uniforms::Uniforms;

// Enumeración para los cuerpos celestes
#[derive(Clone, PartialEq)]
enum CelestialBody {
    Star,
    Planet,
    GasGiant,
    Moon,
    Comet,
    Nebula, // Cuerpo celeste adicional
}

impl CelestialBody {
    fn to_celestial_type(&self) -> CelestialType {
        match self {
            CelestialBody::Star => CelestialType::Star,
            CelestialBody::Planet => CelestialType::Planet,
            CelestialBody::GasGiant => CelestialType::GasGiant,
            CelestialBody::Moon => CelestialType::Moon,
            CelestialBody::Comet => CelestialType::Comet,
            CelestialBody::Nebula => CelestialType::Atmosphere, // Asumiendo que Atmosphere es para Nebulas
        }
    }
}

// Estructura para manejar los cuerpos visibles de manera secuencial
struct BodyManager {
    all_bodies: Vec<CelestialBody>,
    current_index: usize,
    zoom_level: f32,
}

impl BodyManager {
    fn new() -> Self {
        BodyManager {
            all_bodies: vec![
                CelestialBody::Star,
                CelestialBody::Planet,
                CelestialBody::GasGiant,
                CelestialBody::Moon,
                CelestialBody::Comet,
                CelestialBody::Nebula,
            ],
            current_index: 0,
            zoom_level: 10.0, // Distancia inicial de la cámara
        }
    }

    fn next(&mut self) {
        self.current_index = (self.current_index + 1) % self.all_bodies.len();
    }

    fn current(&self) -> CelestialBody {
        self.all_bodies[self.current_index].clone()
    }

    fn zoom_in(&mut self) {
        self.zoom_level = (self.zoom_level / 1.1).max(1.0); // Limitar zoom mínimo
    }

    fn zoom_out(&mut self) {
        self.zoom_level *= 1.1;
    }
}

// Función para obtener la posición del cuerpo celeste (todos en el origen)
fn get_body_position(_body: &CelestialBody, _time: u32) -> Vec3 {
    // Todos los cuerpos se renderizarán en el origen
    Vec3::new(0.0, 0.0, 0.0)
}

// Función para crear el ruido
// En la función create_noise dentro de main.rs
fn create_noise() -> Arc<FastNoiseLite> {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Cellular));
    noise.set_cellular_distance_function(Some(CellularDistanceFunction::EuclideanSq));
    noise.set_frequency(Some(0.35)); // Aumenta la frecuencia para mayor detalle
    Arc::new(noise)
}


// Función para crear la matriz de modelo
fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

// Función para crear la matriz de vista
fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    nalgebra_glm::look_at(&eye, &center, &up)
}

// Función para crear la matriz de perspectiva
fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * std::f32::consts::PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    nalgebra_glm::perspective(fov, aspect_ratio, near, far)
}

// Función para crear la matriz de viewport
fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

// Función para aplicar el post-procesamiento
fn post_process(framebuffer: &mut Framebuffer) {
    for i in 0..framebuffer.buffer.len() {
        let emissive = framebuffer.emissive_buffer[i];
        let final_color = blend_add(framebuffer.buffer[i], emissive);
        framebuffer.buffer[i] = final_color;
    }
}

// Función para mezclar colores usando blend add
fn blend_add(base: u32, emissive: u32) -> u32 {
    let r = ((base >> 16) & 0xFF).saturating_add((emissive >> 16) & 0xFF);
    let g = ((base >> 8) & 0xFF).saturating_add((emissive >> 8) & 0xFF);
    let b = (base & 0xFF).saturating_add(emissive & 0xFF);
    (r << 16) | (g << 8) | b
}

// Función para manejar la entrada del usuario
fn handle_input(window: &Window, camera: &mut Camera, body_manager: &mut BodyManager) {
    let rotation_speed = 0.05; // Ajusta este valor para controlar la velocidad de rotación

    // Cambiar al siguiente cuerpo celeste al presionar 'N'
    if window.is_key_down(Key::N) {
        body_manager.next();
    }

    // Zoom In y Zoom Out
    if window.is_key_down(Key::Z) {
        body_manager.zoom_in();
    }

    if window.is_key_down(Key::X) {
        body_manager.zoom_out();
    }

    // Rotación de la cámara
    if window.is_key_down(Key::Up) {
        camera.orbit(0.0, rotation_speed);
    }
    if window.is_key_down(Key::Down) {
        camera.orbit(0.0, -rotation_speed);
    }
    if window.is_key_down(Key::Left) {
        camera.orbit(-rotation_speed, 0.0);
    }
    if window.is_key_down(Key::Right) {
        camera.orbit(rotation_speed, 0.0);

        
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16); // Aproximadamente 60 FPS

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Animated Fragment Shader",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("Failed to create window: {}", e);
    });

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x000000);

    // Parámetros de la cámara
    let mut camera = Camera::new(
        Vec3::new(0.0, 0.0, 10.0), // Eye
        Vec3::new(0.0, 0.0, 0.0),  // Center
        Vec3::new(0.0, 1.0, 0.0)   // Up
    );
    // Parámetros de ruido y umbrales para el shader
    let noise_scale = 3.0; // Reduce la escala para agrandar los continentes
    let ocean_threshold = -0.6; // Ajusta para que los océanos no ocupen demasiado espacio
    let continent_threshold = 0.65; // Ajusta para que los continentes ocupen más espacio
    let mountain_threshold = 0.1; // Define el inicio de las montañas
    // Crear el ruido
    let noise = create_noise();

    // Cargar modelos
    let star_obj = Obj::load("assets/models/planet.obj").expect("Failed to load star.obj");
    let star_vertex_array = star_obj.get_vertex_array();

    let planet_obj = Obj::load("assets/models/planet.obj").expect("Failed to load planet.obj");
    let planet_vertex_array = planet_obj.get_vertex_array();

    let gas_giant_obj = Obj::load("assets/models/ringed2.obj").expect("Failed to load gas_giant.obj");
    let gas_giant_vertex_array = gas_giant_obj.get_vertex_array();

    let moon_obj = Obj::load("assets/models/planet.obj").expect("Failed to load moon.obj");
    let moon_vertex_array = moon_obj.get_vertex_array();

    let comet_obj = Obj::load("assets/models/planet.obj").expect("Failed to load comet.obj");
    let comet_vertex_array = comet_obj.get_vertex_array();

    let nebula_obj = Obj::load("assets/models/planet.obj").expect("Failed to load nebula.obj"); // Asegúrate de tener un modelo para la nebulosa
    let nebula_vertex_array = nebula_obj.get_vertex_array();

    let mut time = 0;

    // Inicializar BodyManager
    let mut body_manager = BodyManager::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        time += 1;

        // Manejar entradas
        handle_input(&window, &mut camera, &mut body_manager);

        framebuffer.clear();

        // Obtener el cuerpo celeste actual
        let current_body = body_manager.current();
        let body_position = get_body_position(&current_body, time);

        // Configurar la cámara para enfocar el cuerpo celeste actual
        camera.center = body_position;
        let direction = Vec3::new(0.0, 0.0, 1.0); // Dirección hacia el cuerpo celeste
        camera.eye = body_position + direction * body_manager.zoom_level;

        // Crear matrices de transformación
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

        let uniforms = Uniforms {
            model_matrix: Mat4::identity(),
            view_matrix: create_view_matrix(camera.eye, camera.center, camera.up),
            projection_matrix: create_perspective_matrix(window_width as f32, window_height as f32),
            viewport_matrix: create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32),
            time,
            noise: noise.clone(),
            noise_scale, // Configuración de escala de ruido
            ocean_threshold, // Configuración para océanos
            continent_threshold, // Configuración para continentes
            mountain_threshold, // Configuración para montañas
        };
    

        // Renderizar el cuerpo celeste actual
        match current_body {
            CelestialBody::Star => {
                let star_translation = body_position;
                let star_rotation = Vec3::new(0.0, (time as f32 * 0.01).sin(), 0.0); // Rotación ejemplo
                let star_scale = 3.0;
                let star_model_matrix = create_model_matrix(star_translation, star_scale, star_rotation);
                let star_uniforms = Uniforms { 
                    model_matrix: star_model_matrix, 
                    view_matrix: view_matrix, 
                    projection_matrix: projection_matrix, 
                    viewport_matrix: viewport_matrix,
                    time,
                    noise: uniforms.noise.clone(),
                    noise_scale,
                    ocean_threshold,
                    continent_threshold,
                    mountain_threshold,
                };
                render(&mut framebuffer, &star_uniforms, &star_vertex_array, CelestialBody::Star.to_celestial_type());
            },
            CelestialBody::Planet => {
                let planet_translation = body_position;
                let planet_rotation = Vec3::new(0.0, (time as f32 * 0.02).sin(), 0.0); // Rotación ejemplo
                let planet_scale = 1.0;
                let planet_model_matrix = create_model_matrix(planet_translation, planet_scale, planet_rotation);
                let planet_uniforms = Uniforms { 
                    model_matrix: planet_model_matrix, 
                    view_matrix: view_matrix, 
                    projection_matrix: projection_matrix, 
                    viewport_matrix: viewport_matrix,
                    time,
                    noise: uniforms.noise.clone(),
                    noise_scale,
                    ocean_threshold,
                    continent_threshold,
                    mountain_threshold,
                };
                render(&mut framebuffer, &planet_uniforms, &planet_vertex_array, CelestialBody::Planet.to_celestial_type());
            },
            CelestialBody::GasGiant => {
                let gas_giant_translation = body_position;
                let gas_giant_rotation = Vec3::new(0.0, (time as f32 * 0.015).sin(), 0.0); // Rotación ejemplo
                let gas_giant_scale = 1.5;
                let gas_giant_model_matrix = create_model_matrix(gas_giant_translation, gas_giant_scale, gas_giant_rotation);
                let gas_giant_uniforms = Uniforms { 
                    model_matrix: gas_giant_model_matrix, 
                    view_matrix: view_matrix, 
                    projection_matrix: projection_matrix, 
                    viewport_matrix: viewport_matrix,
                    time,
                    noise: uniforms.noise.clone(),
                    noise_scale,
                    ocean_threshold,
                    continent_threshold,
                    mountain_threshold,
                };
                render(&mut framebuffer, &gas_giant_uniforms, &gas_giant_vertex_array, CelestialBody::GasGiant.to_celestial_type());
            },
            CelestialBody::Moon => {
                let moon_translation = body_position;
                let moon_rotation = Vec3::new(0.0, (time as f32 * 0.05).sin(), 0.0); // Rotación ejemplo
                let moon_scale = 0.5;
                let moon_model_matrix = create_model_matrix(moon_translation, moon_scale, moon_rotation);
                let moon_uniforms = Uniforms { 
                    model_matrix: moon_model_matrix, 
                    view_matrix: view_matrix, 
                    projection_matrix: projection_matrix, 
                    viewport_matrix: viewport_matrix,
                    time,
                    noise: uniforms.noise.clone(),
                    noise_scale,
                    ocean_threshold,
                    continent_threshold,
                    mountain_threshold,
                };
                render(&mut framebuffer, &moon_uniforms, &moon_vertex_array, CelestialBody::Moon.to_celestial_type());
            },
            CelestialBody::Comet => {
                let comet_translation = body_position;
                let comet_rotation = Vec3::new(0.0, (time as f32 * 0.03).sin(), 0.0); // Rotación ejemplo
                let comet_scale = 0.7;
                let comet_model_matrix = create_model_matrix(comet_translation, comet_scale, comet_rotation);
                let comet_uniforms = Uniforms { 
                    model_matrix: comet_model_matrix, 
                    view_matrix: view_matrix, 
                    projection_matrix: projection_matrix, 
                    viewport_matrix: viewport_matrix,
                    time,
                    noise: uniforms.noise.clone(),
                    noise_scale,
                    ocean_threshold,
                    continent_threshold,
                    mountain_threshold,
                };
                render(&mut framebuffer, &comet_uniforms, &comet_vertex_array, CelestialBody::Comet.to_celestial_type());
            },
            CelestialBody::Nebula => {
                let nebula_translation = body_position;
                let nebula_rotation = Vec3::new(0.0, (time as f32 * 0.005).sin(), 0.0); // Rotación ejemplo
                let nebula_scale = 2.0;
                let nebula_model_matrix = create_model_matrix(nebula_translation, nebula_scale, nebula_rotation);
                let nebula_uniforms = Uniforms { 
                    model_matrix: nebula_model_matrix, 
                    view_matrix: view_matrix, 
                    projection_matrix: projection_matrix, 
                    viewport_matrix: viewport_matrix,
                    time,
                    noise: uniforms.noise.clone(),
                    noise_scale,
                    ocean_threshold,
                    continent_threshold,
                    mountain_threshold,
                };
                render(&mut framebuffer, &nebula_uniforms, &nebula_vertex_array, CelestialBody::Nebula.to_celestial_type());
            },
        }

        // Post-Procesamiento para Emisión
        post_process(&mut framebuffer);

        // Actualizar la ventana con el buffer final
        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
