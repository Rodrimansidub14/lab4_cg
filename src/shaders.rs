use nalgebra_glm::{Vec3, Vec4, Mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::{Fragment, CelestialType};
use crate::color::Color;
use nalgebra_glm::dot;

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

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    match fragment.celestial_type {
        CelestialType::Star => star_shader(fragment, uniforms),
        CelestialType::Planet => rocky_planet_shader(fragment, uniforms),
        CelestialType::GasGiant => gas_giant_shader(fragment, uniforms),
        CelestialType::Moon => moon_shader(fragment, uniforms),
        CelestialType::Comet => comet_shader(fragment, uniforms),
        CelestialType::Atmosphere => atmosphere_shader(fragment, uniforms),
        // Agrega otros tipos según sea necesario
    }
}
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
}pub fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    // Colores base para los diferentes tipos de terreno
    let ocean_color = Color::new(0, 105, 148);        // Azul para océano
    let continent_color = Color::new(34, 139, 34);    // Verde para tierra/continente
    let mountain_color = Color::new(139, 69, 19);     // Marrón para montañas
    let cloud_color = Color::new(255, 255, 255);      // Blanco para nubes
    let atmosphere_color = Color::new(173, 216, 230); // Azul claro para atmósfera

    // Parámetros de escala y umbrales de ruido para el terreno
    let noise_scale = uniforms.noise_scale;
    let ocean_threshold = uniforms.ocean_threshold;
    let continent_threshold = uniforms.continent_threshold;
    let mountain_threshold = uniforms.mountain_threshold;

    // Simulación de rotación del planeta
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
    } else if terrain_noise_value < continent_threshold {
        continent_color
    } else if terrain_noise_value < mountain_threshold {
        mountain_color
    } else {
        mountain_color.blend_subtract(&Color::new(50, 50, 50)) // Zonas más elevadas, ligeramente más oscuras
    };

    // Parámetros para las nubes
    let cloud_scale = 7.0;          // Escala de ruido incrementada para nubes más grandes
    let cloud_speed = 0.1;          // Velocidad incrementada para nubes más dinámicas
    let cloud_threshold = 0.3;      // Umbral disminuido para más nubes

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
        let atmosphere_opacity = ((atmosphere_noise + 1.0) / 2.0).clamp(0.0, 1.0) * 0.5; // Opacidad incrementada a 0.5

        // Mezclar el color de la atmósfera con el color actual
        surface_color = surface_color.lerp(&atmosphere_color, atmosphere_opacity);
    }

    // Devolver el color del terreno con las nubes y atmósfera, y marcarlo como no emisivo
    (surface_color, false)
}




fn gas_giant_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    // Capa 1: Bandas de colores alternados
    let bands = ((fragment.vertex_position.y * 5.0).sin() + 1.0) / 2.0;
    let band_color = Color::new(
        (bands * 255.0) as u8,
        ((1.0 - bands) * 255.0) as u8,
        (bands * 128.0) as u8,
    );

    // Capa 2: Tormentas o patrones nubosos con ruido
    let storm_noise = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * 5.0,
        fragment.vertex_position.y * 5.0 + uniforms.time as f32 * 0.02,
    );
    let storm_color = if storm_noise > 0.5 { Color::new(255, 0, 0) } else { Color::black() };

    // Capa 3: Anillos
    let rings_color = rings_shader(fragment, uniforms).0;

    // Mezclar capas usando blend modes
    let blended_color = band_color.blend_add(&storm_color).blend_add(&rings_color);

    (blended_color, false)
}

fn rings_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    let distance = (fragment.vertex_position.x.powi(2) + fragment.vertex_position.y.powi(2)).sqrt();
    let ring_inner = 1.5;
    let ring_outer = 2.0;
    let ring_width = ring_outer - ring_inner;

    if distance > ring_inner && distance < ring_outer {
        let noise_value = uniforms.noise.get_noise_2d(
            fragment.vertex_position.x * 10.0 + uniforms.time as f32 * 0.1,
            fragment.vertex_position.y * 10.0 + uniforms.time as f32 * 0.1,
        );
        let alpha = noise_value.clamp(0.0, 1.0);
        let ring_color = Color::new(200, 200, 200); // Gris claro

        (ring_color * alpha * fragment.intensity, false)
    } else {
        (Color::black(), false)
    }
}

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

fn comet_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    let distance = (fragment.vertex_position.x.powi(2) + fragment.vertex_position.y.powi(2)).sqrt();
    let tail_length = 3.0;
    let tail_color = Color::new(255, 255, 255); // Blanco

    let alpha = ((distance - tail_length).abs() / 0.5).clamp(0.0, 1.0);
    let final_color = tail_color * alpha;

    (final_color, false)
}

fn cloud_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    let zoom = 10.0;
    let ox = uniforms.time as f32 * 0.1;
    let oy = 0.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox, y * zoom + oy);

    let cloud_threshold = 0.5;
    let cloud_color = Color::new(255, 255, 255); // Blanco
    let sky_color = Color::new(30, 97, 145);     // Azul cielo

    let final_color = if noise_value > cloud_threshold {
        cloud_color
    } else {
        sky_color
    };

    (final_color * fragment.intensity, false)
}

fn surface_animation_shader(fragment: &Fragment, uniforms: &Uniforms) -> (Color, bool) {
    let animation_factor = uniforms.time as f32 * 0.05;
    let noise_value = uniforms.noise.get_noise_2d(
        fragment.vertex_position.x * 5.0 + animation_factor,
        fragment.vertex_position.y * 5.0 + animation_factor,
    );

    let base_color = Color::new(34, 139, 34); // Verde bosque
    let lava_color = Color::new(255, 69, 0); // Rojo anaranjado

    let final_color = base_color.lerp(&lava_color, (noise_value + 1.0) / 2.0);

    (final_color * fragment.intensity, false)
}

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
