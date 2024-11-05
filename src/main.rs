// src/main.rs

use nalgebra_glm::{Vec3, Mat4};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::sync::Arc;
use nalgebra_glm as glm;

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
use crate::color::Color;
use fastnoise_lite::{FastNoiseLite, NoiseType, CellularDistanceFunction, FractalType,DomainWarpType};
use renderer::render;
use fragment::{CelestialType};
use uniforms::Uniforms;

// Enumeración para los cuerpos celestes
#[derive(Clone, PartialEq)]
enum CelestialBody {
    Star,
    Planet,
    GasGiant,
    Ringed,
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
            CelestialBody::Ringed => CelestialType::Ringed,
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
                CelestialBody::Ringed,
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

    fn select(&mut self, index: usize) {
        if index < self.all_bodies.len() {
            self.current_index = index; // Set the current index to the selected body
        }
    }
}

// Función para obtener la posición del cuerpo celeste (todos en el origen)
fn get_body_position(_body: &CelestialBody, _time: u32) -> Vec3 {
    // Todos los cuerpos se renderizarán en el origen
    Vec3::new(0.0, 0.0, 0.0)
}

// Funciones para crear generadores de ruido específicos para cada cuerpo celeste
fn create_noise_star() -> Arc<FastNoiseLite> {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::Perlin)); // Ruido Perlin para variaciones suaves
    noise.set_frequency(Some(1.0)); // Alta frecuencia para detalles finos
    Arc::new(noise)
}

fn create_noise_planet() -> Arc<FastNoiseLite> {
    let mut noise = FastNoiseLite::with_seed(1338);
    noise.set_noise_type(Some(NoiseType::Perlin)); // Cambiado a Ruido Perlin
    noise.set_frequency(Some(0.35)); // Frecuencia media para detalles moderados
    Arc::new(noise)
}
// src/main.rs

fn create_noise_gas_giant() -> Arc<FastNoiseLite> {
    let mut noise = FastNoiseLite::with_seed(1637);
    noise.set_noise_type(Some(NoiseType::Perlin));
    noise.set_frequency(Some(0.025)); // Ajusta según la densidad de detalles deseados

    // Configurar el fractal FBm para detalles adicionales
    noise.set_fractal_type(Some(FractalType::FBm));
    noise.set_fractal_octaves(Some(3));            // Detalle de octavas
    noise.set_fractal_lacunarity(Some(2.000));     // Separación de detalles
    noise.set_fractal_gain(Some(0.500));           // Ganancia para contrastes

    Arc::new(noise)
}

fn create_noise_moon() -> Arc<FastNoiseLite> {
    let mut noise = FastNoiseLite::with_seed(1340);
    noise.set_noise_type(Some(NoiseType::Perlin)); // Ruido Perlin para cráteres y detalles superficiales
    noise.set_frequency(Some(0.5)); // Alta frecuencia para detalles más finos
    Arc::new(noise)
}

fn create_noise_comet() -> Arc<FastNoiseLite> {
    let mut noise = FastNoiseLite::with_seed(1341);
    noise.set_noise_type(Some(NoiseType::Perlin)); // Ruido Perlin para patrones irregulares
    noise.set_frequency(Some(0.4)); // Frecuencia media
    Arc::new(noise)
}

fn create_noise_nebula() -> Arc<FastNoiseLite> {
    let mut noise = FastNoiseLite::with_seed(1342);
    noise.set_noise_type(Some(NoiseType::Cellular)); // Ruido Cellular para patrones nebulosos
    noise.set_cellular_distance_function(Some(CellularDistanceFunction::EuclideanSq));
    noise.set_frequency(Some(0.1)); // Muy baja frecuencia para grandes estructuras
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
// src/main.rs


// Función para crear una matriz de rotación a partir de ángulos de Euler (en grados)
fn create_rotation_matrix(pitch: f32, yaw: f32, roll: f32) -> Mat4 {
    Mat4::from_euler_angles(
        pitch * std::f32::consts::PI / 180.0, 
        yaw * std::f32::consts::PI / 180.0, 
        roll * std::f32::consts::PI / 180.0
    )
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

    // Selección de cuerpos celestes con teclas numéricas
    for num in 1..=9 {
        let key = match num {
            1 => Key::Key1,
            2 => Key::Key2,
            3 => Key::Key3,
            4 => Key::Key4,
            5 => Key::Key5,
            6 => Key::Key6,
            7 => Key::Key7,

            _ => continue,
        };

        if window.is_key_down(key) {
            body_manager.select(num - 1); // Índice basado en cero
        }
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

    // Parámetros de ruido y umbrales para el shader (pueden ser ajustados por cuerpo celeste)
    let noise_scale = 3.0;          // Ajusta según sea necesario
    let ocean_threshold = -0.3;     // Incrementado para más océanos
    let mountain_threshold = 0.2;   // Ajustado para mantener la ordenación
    let continent_threshold = 0.65; // Mantiene su valor

    // Crear generadores de ruido separados para cada cuerpo celeste
    let noise_star = create_noise_star();
    let noise_planet = create_noise_planet();
    let noise_gas_giant = create_noise_gas_giant();
    let noise_moon = create_noise_moon();
    let noise_comet = create_noise_comet();
    let noise_nebula = create_noise_nebula();

    // Cargar modelos (asegúrate de que los paths y modelos sean correctos)
    let star_obj = Obj::load("assets/models/planet.obj").expect("Failed to load star.obj");
    let star_vertex_array = star_obj.get_vertex_array();

    let planet_obj = Obj::load("assets/models/planet.obj").expect("Failed to load planet.obj");
    let planet_vertex_array = planet_obj.get_vertex_array();

    let gas_giant_obj = Obj::load("assets/models/planet.obj").expect("Failed to load gas_giant.obj");
    let gas_giant_vertex_array = gas_giant_obj.get_vertex_array();
    
    let ringed_obj = Obj::load("assets/models/planet.obj").expect("Failed to load ringed.obj");
    let ringed_vertex_array = ringed_obj.get_vertex_array();

    let moon_obj = Obj::load("assets/models/planet.obj").expect("Failed to load moon.obj");
    let moon_vertex_array = moon_obj.get_vertex_array();

    let comet_obj = Obj::load("assets/models/planet.obj").expect("Failed to load comet.obj");
    let comet_vertex_array = comet_obj.get_vertex_array();

    let nebula_obj = Obj::load("assets/models/planet.obj").expect("Failed to load nebula.obj"); // Asegúrate de tener un modelo para la nebulosa
    let nebula_vertex_array = nebula_obj.get_vertex_array();

    let mut time = 0.0; // Usar f32 para mayor precisión en cálculos de tiempo

    // Inicializar BodyManager
    let mut body_manager = BodyManager::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        time += 0.016; // Incrementar el tiempo en cada frame (aprox. 60 FPS)

        // Manejar entradas
        handle_input(&window, &mut camera, &mut body_manager);

        framebuffer.clear();

        // Obtener el cuerpo celeste actual
        let current_body = body_manager.current();
        let body_position = get_body_position(&current_body, time as u32); // Convertir tiempo a u32

        // Configurar la cámara para enfocar el cuerpo celeste actual
        camera.center = body_position;
        let direction = Vec3::new(0.0, 0.0, 1.0); // Dirección hacia el cuerpo celeste
        camera.eye = body_position + direction * body_manager.zoom_level;

        // Crear matrices de transformación
        let view_matrix = create_view_matrix(camera.eye, camera.center, camera.up);
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

        // Definir la dirección de la luz (por ejemplo, hacia la derecha y arriba)
        let light_direction = Vec3::new(1.0, 1.0, 1.0).normalize();

        // Renderizar el cuerpo celeste actual con su propio generador de ruido
        match current_body {
            CelestialBody::Star => {
                let star_translation = body_position;
                let star_rotation = Vec3::new(0.0, (time as f32 * 0.01).sin(), 0.0); // Rotación ejemplo
                let star_scale = 3.0;
                let star_model_matrix = create_model_matrix(star_translation, star_scale, star_rotation);
                let star_uniforms = Uniforms::new(
                    star_model_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                    noise_star.clone(),
                    light_direction,
                    1.0,                        // noise_scale
                    -0.6,                       // ocean_threshold
                    0.65,                       // continent_threshold
                    0.1,                        // mountain_threshold
                    0.0,                        // snow_threshold
                    0.0,                        // ring_inner_radius
                    0.0,                        // ring_outer_radius
                    Color::black(),             // ring_color
                    0.0,                        // ring_opacity
                    0.0,                        // ring_frequency
                    0.0,                        // ring_wave_speed
                    glm::identity::<f32, 4>(),  // ring_rotation_matrix (corregido)
                );
                render(&mut framebuffer, &star_uniforms, &star_vertex_array, CelestialBody::Star.to_celestial_type());
            },
            CelestialBody::Planet => {
                let planet_translation = body_position;
                let planet_rotation = Vec3::new(0.0, (time * 0.02).sin(), 0.0); // Rotación ejemplo
                let planet_scale = 1.0;
                let planet_model_matrix = create_model_matrix(planet_translation, planet_scale, planet_rotation);
                let planet_uniforms = Uniforms::new(
                    planet_model_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                    noise_planet.clone(),
                    light_direction,
                    3.0,                        // noise_scale
                    -0.038,                     // ocean_threshold
                    0.85,                       // continent_threshold
                    0.2,                        // mountain_threshold
                    0.05,                       // snow_threshold
                    0.0,                        // ring_inner_radius
                    0.0,                        // ring_outer_radius
                    Color::black(),             // ring_color
                    0.0,                        // ring_opacity
                    0.0,                        // ring_frequency
                    0.0,                        // ring_wave_speed
                    glm::identity::<f32, 4>(),  // ring_rotation_matrix (corregido)
                );
                render(&mut framebuffer, &planet_uniforms, &planet_vertex_array, CelestialBody::Planet.to_celestial_type());
            },
            CelestialBody::GasGiant => {
                let gas_giant_translation = body_position;
                let gas_giant_rotation = Vec3::new(0.0, (time * 0.02).sin(), 0.0); // Rotación ejemplo
                let gas_giant_scale = 1.5;
                let gas_giant_model_matrix = create_model_matrix(gas_giant_translation, gas_giant_scale, gas_giant_rotation);
                let gas_giant_uniforms = Uniforms::new(
                    gas_giant_model_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                    noise_gas_giant.clone(),
                    light_direction,
                    15.0,                       // noise_scale
                    -0.6,                       // ocean_threshold
                    0.65,                       // continent_threshold
                    0.1,                        // mountain_threshold
                    0.0,                        // snow_threshold
                    0.0,                        // ring_inner_radius
                    0.0,                        // ring_outer_radius
                    Color::black(),             // ring_color
                    0.0,                        // ring_opacity
                    0.0,                        // ring_frequency
                    0.0,                        // ring_wave_speed
                    glm::identity::<f32, 4>(),  // ring_rotation_matrix (corregido)
                );
                render(&mut framebuffer, &gas_giant_uniforms, &gas_giant_vertex_array, CelestialBody::GasGiant.to_celestial_type());
            },
            CelestialBody::Ringed => {
                let ringed_translation = body_position;
                let ringed_rotation = Vec3::new(0.0, (time * 0.02).sin(), 0.0); // Rotación ejemplo
                let ringed_scale = 1.5;
                let ringed_model_matrix = create_model_matrix(ringed_translation, ringed_scale, ringed_rotation);
                
                // Crear una matriz de rotación para inclinar los anillos
                // Por ejemplo, inclinar los anillos 30 grados alrededor del eje X y 45 grados alrededor del eje Y
                let ring_pitch = 30.0; // Ángulo en grados
                let ring_yaw = 45.0;   // Ángulo en grados
                let ring_roll = 0.0;   // Ángulo en grados
                let ring_rotation_matrix = create_rotation_matrix(ring_pitch, ring_yaw, ring_roll);
                
                let ringed_uniforms = Uniforms::new(
                    ringed_model_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                    noise_gas_giant.clone(),
                    light_direction,
                    15.0,                       // noise_scale
                    -0.6,                       // ocean_threshold
                    0.65,                       // continent_threshold
                    0.1,                        // mountain_threshold
                    0.0,                        // snow_threshold
                    0.5,                        // ring_inner_radius
                    1.0,                        // ring_outer_radius
                    Color::new(255, 255, 255),  // ring_color (blanco)
                    0.5,                        // ring_opacity
                    20.0,                       // ring_frequency
                    0.5,                        // ring_wave_speed
                    ring_rotation_matrix,       // ring_rotation_matrix
                );
                render(&mut framebuffer, &ringed_uniforms, &ringed_vertex_array, CelestialBody::Ringed.to_celestial_type());
            },
            CelestialBody::Moon => {
                let moon_translation = body_position;
                let moon_rotation = Vec3::new(0.0, (time * 0.05).sin(), 0.0); // Rotación ejemplo
                let moon_scale = 0.5;
                let moon_model_matrix = create_model_matrix(moon_translation, moon_scale, moon_rotation);
                let moon_uniforms = Uniforms::new(
                    moon_model_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                    noise_moon.clone(),
                    light_direction,
                    2.5,                        // noise_scale
                    -0.6,                       // ocean_threshold
                    0.65,                       // continent_threshold
                    0.1,                        // mountain_threshold
                    0.0,                        // snow_threshold
                    0.0,                        // ring_inner_radius
                    0.0,                        // ring_outer_radius
                    Color::black(),             // ring_color
                    0.0,                        // ring_opacity
                    0.0,                        // ring_frequency
                    0.0,                        // ring_wave_speed
                    glm::identity::<f32, 4>(),  // ring_rotation_matrix (corregido)
                );
                render(&mut framebuffer, &moon_uniforms, &moon_vertex_array, CelestialBody::Moon.to_celestial_type());
            },
            CelestialBody::Comet => {
                let comet_translation = body_position;
                let comet_rotation = Vec3::new(0.0, (time * 0.03).sin(), 0.0); // Rotación ejemplo
                let comet_scale = 0.7;
                let comet_model_matrix = create_model_matrix(comet_translation, comet_scale, comet_rotation);
                let comet_uniforms = Uniforms::new(
                    comet_model_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                    noise_comet.clone(),
                    light_direction,
                    2.0,                        // noise_scale
                    -0.6,                       // ocean_threshold
                    0.65,                       // continent_threshold
                    0.1,                        // mountain_threshold
                    0.0,                        // snow_threshold
                    0.0,                        // ring_inner_radius
                    0.0,                        // ring_outer_radius
                    Color::black(),             // ring_color
                    0.0,                        // ring_opacity
                    0.0,                        // ring_frequency
                    0.0,                        // ring_wave_speed
                    glm::identity::<f32, 4>(),  // ring_rotation_matrix (corregido)
                );
                render(&mut framebuffer, &comet_uniforms, &comet_vertex_array, CelestialBody::Comet.to_celestial_type());
            },
            CelestialBody::Nebula => {
                let nebula_translation = body_position;
                let nebula_rotation = Vec3::new(0.0, (time * 0.005).sin(), 0.0); // Rotación ejemplo
                let nebula_scale = 2.0;
                let nebula_model_matrix = create_model_matrix(nebula_translation, nebula_scale, nebula_rotation);
                let nebula_uniforms = Uniforms::new(
                    nebula_model_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time,
                    noise_nebula.clone(),
                    light_direction,
                    1.5,                        // noise_scale
                    -0.6,                       // ocean_threshold
                    0.65,                       // continent_threshold
                    0.1,                        // mountain_threshold
                    0.0,                        // snow_threshold
                    0.0,                        // ring_inner_radius
                    0.0,                        // ring_outer_radius
                    Color::black(),             // ring_color
                    0.0,                        // ring_opacity
                    0.0,                        // ring_frequency
                    0.0,                        // ring_wave_speed
                    glm::identity::<f32, 4>(),  // ring_rotation_matrix (corregido)
                );
                render(&mut framebuffer, &nebula_uniforms, &nebula_vertex_array, CelestialBody::Nebula.to_celestial_type());
            },
        } 
         // Post-Procesamiento para Emisión
        post_process(&mut framebuffer);

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
