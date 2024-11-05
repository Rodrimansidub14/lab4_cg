use nalgebra_glm::{Vec3, Vec4, Mat3};
use nalgebra::Point3; // Añadido para resolver el error E0433
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::{Fragment, CelestialType};
use crate::color::Color;
use nalgebra_glm::dot;
// Vertex Shader
pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = Mat3::from_columns(&uniforms.model_matrix.fixed_view::<3, 3>(0, 0).column_iter().collect::<Vec<_>>());
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = (normal_matrix * vertex.normal).normalize();

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}

// Fragment Shader Dispatcher
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    match fragment.celestial_type {
        CelestialType::Star => star_shader(fragment, uniforms),
        CelestialType::Planet => rocky_planet_shader(fragment, uniforms),
        CelestialType::GasGiant => gas_giant_shader(fragment, uniforms),
        CelestialType::Ringed => rings_shader(fragment, uniforms),

        CelestialType::Moon => moon_shader(fragment, uniforms),
        CelestialType::Comet => comet_shader(fragment, uniforms),
        CelestialType::Atmosphere => atmosphere_shader(fragment, uniforms),
        // Agrega otros tipos según sea necesario
    }
}

// Shader para Estrella
fn star_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    let noise_scale = 150.0;

    // Obtenemos el valor del ruido en 3D para la posición del fragmento
    let noise_value = uniforms.noise.get_noise_3d(
        fragment.vertex_position.x * noise_scale,
        fragment.vertex_position.y * noise_scale,
        fragment.vertex_position.z * noise_scale,
    );

    // Escalamos el valor de ruido para obtener un rango de 0 a 1
    let normalized_noise = (noise_value + 1.0) * 0.5;

    // Definimos colores base para simular la apariencia de una estrella
    let base_red = Color::new(253, 46, 4);       // Naranja-rojo
    let hot_yellow = Color::new(248, 96, 13);  // Amarillo claro

    // Interpolamos entre los colores según el valor de ruido
    let star_color = base_red.lerp(&hot_yellow, normalized_noise);

    // La emisividad será directamente el color generado
    (star_color, true) // `true` indica que es emisivo
}

// Shader para Planeta Rocoso
pub fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    // Colores base para los diferentes tipos de terreno
    let ocean_color = Color::new(10, 115, 252);        // Azul para océano
    let continent_color = Color::new(34, 139, 34);    // Verde para tierra/continente
    let snow_color = Color::new(255, 250, 250); // Blanco nieve

    let mountain_color = Color::new(97, 77, 63);     // Marrón para montañas
    let cloud_color = Color::new(255, 255, 255);      // Blanco para nubes
    let atmosphere_color = Color::new(173, 216, 230); // Azul claro para atmósfera

    
    // Parámetros de escala y umbrales de ruido para el terreno
    let noise_scale = uniforms.noise_scale;
    let ocean_threshold = uniforms.ocean_threshold;
    let mountain_threshold = uniforms.mountain_threshold;
    let continent_threshold = uniforms.continent_threshold;
    let snow_threshold = uniforms.snow_threshold; // Nuevo umbral para nieve

    // Simulación de rotación y cálculo de ruido
    let rotation_speed = 0.8; // Ajusta según sea necesario
    let angle = uniforms.time * rotation_speed;
    let rotated_x = fragment.vertex_position.x * angle.cos() - fragment.vertex_position.z * angle.sin();
    let rotated_z = fragment.vertex_position.x * angle.sin() + fragment.vertex_position.z * angle.cos();
    let rotated_position = nalgebra_glm::Vec3::new(rotated_x, fragment.vertex_position.y, rotated_z);

    // Generar ruido para definir el tipo de terreno usando ruido 3D
    let terrain_noise_value = uniforms.noise.get_noise_3d(
        rotated_position.x * noise_scale,
        rotated_position.y * noise_scale,
        rotated_position.z * noise_scale,
    );

    // Clasificar las zonas usando los umbrales para definir océano, continente y montaña
    let mut surface_color = if terrain_noise_value < ocean_threshold {
        ocean_color
    } else if terrain_noise_value < mountain_threshold {
        mountain_color
    } else if terrain_noise_value < continent_threshold {
        continent_color
    } else {
        mountain_color.blend_subtract(&Color::new(50, 50, 50))
    };

    // Añadir capa de nieve en regiones de alta altitud
    if terrain_noise_value > snow_threshold {
        // Calcular la cantidad de nieve basada en la altitud
        let snow_factor = ((terrain_noise_value - snow_threshold) / (1.0 - snow_threshold)).clamp(0.0, 1.0);
        
        // Mezclar el color de la superficie con el color de la nieve
        surface_color = surface_color.lerp(&snow_color, snow_factor);
    }

    // Parámetros para las nubes
    let cloud_scale = 7.0;          // Escala de ruido para nubes
    let cloud_speed = 0.3;          // Velocidad de movimiento de nubes
    let cloud_threshold: f32 = 0.1; // Umbral para generar nubes

    // Generar ruido 2D para las nubes
    let cloud_noise_value = uniforms.noise.get_noise_2d(
        rotated_position.x * cloud_scale + uniforms.time * cloud_speed,
        rotated_position.y * cloud_scale + uniforms.time * cloud_speed,
    );

    // Calcular opacidad de las nubes (mapear de [-1,1] a [0,1])
    let cloud_opacity = ((cloud_noise_value + 1.0) / 2.0).clamp(0.0, 1.0);

    // Aplicar nubes sobre el terreno usando interpolación lineal (lerp)
    if cloud_noise_value > cloud_threshold {
        surface_color = surface_color.lerp(&cloud_color, cloud_opacity * 1.0); // Opacidad incrementada a 1.0
    }

    // **Iluminación Básica (Lambertiana)**
    // Definir la dirección de la luz
    let light_dir = uniforms.light_direction.normalize();
    
    // Calcular la intensidad de la luz basada en la normal del fragmento
    let intensity = dot(&fragment.normal, &light_dir).max(0.0);

    // Aplicar la iluminación al color final
    surface_color = surface_color * intensity;

    // **Atmósfera (Halo)**
    let distance = rotated_position.magnitude();
    
    if distance > 1.0 && distance < 1.0 + 0.3 { // Grosor de atmósfera incrementado a 0.3
        // Generar ruido para la atmósfera
        let atmosphere_noise = uniforms.noise.get_noise_2d(
            rotated_position.x * 20.0 + uniforms.time * 0.02,
            rotated_position.y * 20.0 + uniforms.time * 0.02,
        );

        // Calcular opacidad de la atmósfera
        let atmosphere_opacity: f32 = ((atmosphere_noise + 1.0) / 2.0).clamp(0.0, 1.0) * 0.7; // Opacidad incrementada a 0.7

        // Mezclar el color de la atmósfera con el color actual
        surface_color = surface_color.lerp(&atmosphere_color, atmosphere_opacity);
    }

    // Devolver el color del terreno con las nubes y atmósfera, y marcarlo como no emisivo
    (surface_color, false)
}
// Shader para Gigante Gaseoso
// src/shaders.rs
pub fn gas_giant_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    // Colores base para las bandas del gigante gaseoso
    let band_color_1 = Color::new(200, 160, 100); // Beige claro
    let band_color_2 = Color::new(150, 100, 50);  // Marrón rojizo

    // Color para las tormentas
    let storm_color = Color::new(255, 69, 0);     // Rojo intenso para tormentas

    // Parámetros para el movimiento sinusoidal de las bandas
    let band_frequency = 7.0;
    let wave_speed = 0.9;
    let wave_amplitude = 0.5;

    // Crear las bandas usando un desplazamiento sinusoidal en función del tiempo
    let latitude = fragment.vertex_position.y + (uniforms.time as f32 * wave_speed).sin() * wave_amplitude;
    let band_factor = ((latitude * band_frequency).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let mut surface_color = band_color_1.lerp(&band_color_2, band_factor);

    // **Capa de tormentas independiente**
    // Generamos ruido para la capa de tormentas. Esto se hace en una escala menor que las bandas
    // para crear áreas focalizadas y localizadas en el planeta.
    let storm_scale = 7.0; // Controla el tamaño de las tormentas
    let storm_movement_speed = 0.7; // Velocidad de desplazamiento de las tormentas

    // Posición para el ruido de tormentas, usando una rotación lenta para que parezcan estables
    let storm_noise_value = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * storm_scale + uniforms.time * storm_movement_speed,
        fragment.vertex_position.z * storm_scale + uniforms.time * storm_movement_speed,
    );

    // Definir umbral para la intensidad de las tormentas
    let storm_threshold = 0.5;
    if storm_noise_value > storm_threshold {
        let storm_intensity = (storm_noise_value - storm_threshold) / (1.0 - storm_threshold);
        surface_color = surface_color.lerp(&storm_color, storm_intensity.clamp(0.0, 1.0));
    }

    // **Iluminación Básica (Lambertiana)**
    let light_dir = uniforms.light_direction.normalize();
    let intensity = (fragment.normal.dot(&light_dir)).max(0.0);

    // Aplicar iluminación al color de la superficie
    surface_color = surface_color * intensity;

    // **Atmósfera Exterior (Halo)**
    let distance_from_center = fragment.vertex_position.magnitude();
    if distance_from_center > 1.0 && distance_from_center < 1.2 {
        let atmosphere_noise = uniforms.noise.get_noise_2d(
            fragment.vertex_position.x * 15.0 + uniforms.time * 0.05,
            fragment.vertex_position.y * 15.0 + uniforms.time * 0.05,
        );

        let atmosphere_opacity = ((atmosphere_noise + 1.0) / 2.0).clamp(0.0, 0.6);
        let atmosphere_color = Color::new(173, 216, 230); // Azul claro para el halo de atmósfera

        // Mezclar atmósfera con el color de la superficie
        surface_color = surface_color.lerp(&atmosphere_color, atmosphere_opacity);
    }

    (surface_color, false)
}



// Shader para Gigante Gaseoso con Patrones de Seno e Interpolación de Colores


// Shader para Anillos del Gigante Gaseoso
pub fn rings_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    // Colores base para las bandas del gigante gaseoso
    let band_color_1 = Color::new(111, 171, 191); // Beige claro
    let band_color_2 = Color::new(163, 106, 150); // Marrón rojizo

    // Color para las tormentas
    let storm_color = Color::new(255, 69, 0); // Rojo intenso para tormentas

    // Parámetros para el movimiento sinusoidal de las bandas
    let band_frequency = uniforms.ring_frequency; // Número de bandas, ajustable
    let wave_speed = uniforms.ring_wave_speed;   // Velocidad de las ondas, ajustable
    let wave_amplitude = 0.5;                     // Amplitud fija para las ondas

    // Crear las bandas usando un desplazamiento sinusoidal en función del tiempo
    let latitude = fragment.vertex_position.y + (uniforms.time * wave_speed).sin() * wave_amplitude;
    let band_factor = ((latitude * band_frequency).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let mut surface_color = band_color_1.lerp(&band_color_2, band_factor);

    // **Capa de tormentas independiente**
    // Generamos ruido para la capa de tormentas. Esto se hace en una escala menor que las bandas
    // para crear áreas focalizadas y localizadas en el planeta.
    let storm_scale = 7.0;             // Controla el tamaño de las tormentas
    let storm_movement_speed = 0.7;    // Velocidad de desplazamiento de las tormentas

    // Posición para el ruido de tormentas, usando una rotación lenta para que parezcan estables
    let storm_noise_value = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * storm_scale + uniforms.time * storm_movement_speed,
        fragment.vertex_position.z * storm_scale + uniforms.time * storm_movement_speed,
    );

    // Definir umbral para la intensidad de las tormentas
    let storm_threshold = 0.5;
    if storm_noise_value > storm_threshold {
        let storm_intensity = (storm_noise_value - storm_threshold) / (1.0 - storm_threshold);
        surface_color = surface_color.lerp(&storm_color, storm_intensity.clamp(0.0, 1.0));
    }

    // **Renderizado de Anillos**
    // Convertir la posición del fragmento a coordenadas polares
    // Primero, aplicamos la rotación de los anillos
    let fragment_position = Point3::new(
        fragment.vertex_position.x,
        fragment.vertex_position.y,
        fragment.vertex_position.z,
    );

    // Aplicar la matriz de rotación a la posición del fragmento
    let rotated_position = uniforms.ring_rotation_matrix.transform_point(&fragment_position);

    let pos = rotated_position.coords.xy(); // Vec2
    let distance = pos.magnitude(); // Ahora válido
    let angle_polar = pos.y.atan2(pos.x); // Ángulo en radianes

    // Definir los parámetros de los anillos
    let inner_radius = uniforms.ring_inner_radius;
    let outer_radius = uniforms.ring_outer_radius;
    
    // Verificar si el fragmento está dentro de la región de los anillos
    if distance > inner_radius && distance < outer_radius {
        // Calcular el patrón de los anillos usando una función sinusoidal y ruido
        let ring_pattern = ((angle_polar * band_frequency + uniforms.time * wave_speed).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
        
        // Incorporar ruido para variar la intensidad de las bandas de los anillos
        let ring_noise = uniforms.noise.get_noise_2d(angle_polar * band_frequency + uniforms.time * wave_speed, 0.0);
        let ring_intensity = (ring_pattern * 0.5 + 0.5) * (ring_noise * 0.5 + 0.5);
        let ring_intensity = ring_intensity.clamp(0.0, 1.0);

        // Definir el color del anillo mezclando el color base con negro
        let final_ring_color = uniforms.ring_color.lerp(&Color::black(), 1.0 - ring_intensity);
        let final_ring_color = Color {
            r: (final_ring_color.r as f32 * uniforms.ring_opacity).clamp(0.0, 255.0) as u8,
            g: (final_ring_color.g as f32 * uniforms.ring_opacity).clamp(0.0, 255.0) as u8,
            b: (final_ring_color.b as f32 * uniforms.ring_opacity).clamp(0.0, 255.0) as u8,
        };

        // Aplicar blending con el color de la superficie
        surface_color = surface_color.lerp(&final_ring_color, ring_intensity * uniforms.ring_opacity);
    }

    // **Iluminación Básica (Lambertiana)**
    let light_dir = uniforms.light_direction.normalize();
    let intensity = (dot(&fragment.normal, &light_dir)).max(0.0);

    // Aplicar iluminación al color de la superficie
    surface_color = surface_color * intensity;

    // **Atmósfera Exterior (Halo)**
    let distance_from_center = fragment.vertex_position.magnitude();
    if distance_from_center > 1.0 && distance_from_center < 1.2 {
        let atmosphere_noise = uniforms.noise.get_noise_2d(
            fragment.vertex_position.x * 15.0 + uniforms.time * 0.05,
            fragment.vertex_position.y * 15.0 + uniforms.time * 0.05,
        );

        let atmosphere_opacity = ((atmosphere_noise + 1.0) / 2.0).clamp(0.0, 0.6);
        let atmosphere_color = Color::new(173, 216, 230); // Azul claro para el halo de atmósfera

        // Mezclar atmósfera con el color de la superficie
        surface_color = surface_color.lerp(&atmosphere_color, atmosphere_opacity);
    }

    (surface_color, false)
}

// Shader para Luna
fn moon_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    // Capa 1: Color base
    let base_color = Color::new(169, 169, 169); // Gris oscuro

    // Capa 2: Cráteres con ruido
    let crater_noise = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * 15.0 + uniforms.time as f32 * 0.05,
        fragment.vertex_position.y * 15.0 + uniforms.time as f32 * 0.05,
    );
    let crater_color = if crater_noise > 0.3 { Color::black() } else { base_color };

    // Capa 3: Superficie iluminada
    let intensity = fragment.intensity;
    let lit_color = crater_color * intensity;

    (lit_color, false)
}

// Shader para Cometa
fn comet_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    let distance = (fragment.vertex_position.x.powi(2) + fragment.vertex_position.y.powi(2)).sqrt();
    let tail_length = 3.0;
    let tail_color = Color::new(255, 255, 255); // Blanco

    let alpha = ((distance - tail_length).abs() / 0.5).clamp(0.0, 1.0);
    let final_color = tail_color * alpha;

    (final_color, false)
}

// Shader para Nubes (si es necesario)

// Shader para Atmósfera
fn atmosphere_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    let distance = (fragment.vertex_position.x.powi(2) + fragment.vertex_position.y.powi(2)).sqrt();
    let atmosphere_radius = 1.2; // Radio de la atmósfera
    let thickness = 0.2; // Grosor de la atmósfera

    if distance > atmosphere_radius && distance < atmosphere_radius + thickness {
        let noise_value = uniforms.noise.get_noise_2d(
            fragment.vertex_position.x * 20.0 + uniforms.time as f32 * 0.05,
            fragment.vertex_position.y * 20.0 + uniforms.time as f32 * 0.05,
        );
        let alpha = noise_value.abs().clamp(0.0, 1.0);
        let atmosphere_color = Color::new(135, 206, 235); // Azul cielo

        (atmosphere_color * alpha * fragment.intensity, false)
    } else {
        (Color::black(), false)
    }
}
