// src/triangle.rs

use nalgebra_glm::{Vec3, dot, Vec4};
use crate::fragment::{Fragment, CelestialType};
use crate::vertex::Vertex;
use crate::color::Color;

/// Genera los fragmentos de un triángulo para rasterización
pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex, celestial_type: CelestialType) -> Vec<Fragment> {
    let mut fragments = Vec::new();
    let (a, b, c) = (v1.transformed_position, v2.transformed_position, v3.transformed_position);

    let (min_x, min_y, max_x, max_y) = calculate_bounding_box(&a, &b, &c);

    let light_dir = Vec3::new(0.0, 0.0, 1.0).normalize();

    let triangle_area = edge_function(&a, &b, &c);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let point = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);

            let (w1, w2, w3) = barycentric_coordinates(&point, &a, &b, &c, triangle_area);

            // Solo considerar puntos dentro del triángulo
            if w1 >= 0.0 && w2 >= 0.0 && w3 >= 0.0 {
                // Opcional: Verificar si la suma de los pesos es aproximadamente 1.0
                let sum = w1 + w2 + w3;
                if sum > 0.999 && sum < 1.001 {
                    // Interpolar normales
                    let interpolated_normal = (v1.transformed_normal * w1 + v2.transformed_normal * w2 + v3.transformed_normal * w3).normalize();

                    // Calcular intensidad de luz usando el producto punto
                    let intensity = dot(&interpolated_normal, &light_dir).max(0.0);

                    // Definir color base y aplicar la intensidad
                    let base_color = Color::new(100, 100, 100); // Puedes ajustar esto según tus necesidades
                    let lit_color = base_color * intensity;

                    // Interpolar profundidad (z)
                    let depth = a.z * w1 + b.z * w2 + c.z * w3;

                    // Interpolar posición del vértice en el espacio 3D
                    let vertex_position = v1.position * w1 + v2.position * w2 + v3.position * w3;

                    // Convert vertex_position to 4D
                    let vertex_position = Vec4::new(vertex_position.x, vertex_position.y, vertex_position.z, 1.0);

                    // Determinar si el fragmento es emisivo basado en el tipo celestial
                    let emissive = match celestial_type {
                        CelestialType::Star => true,
                        _ => false,
                    };

                    // Crear el fragmento con todos los parámetros requeridos
                    let fragment = Fragment::new(
                        x as f32,
                        y as f32,
                        lit_color,
                        depth,
                        interpolated_normal,
                        intensity,
                        vertex_position,
                        celestial_type,
                    );

                    fragments.push(fragment);
                }
            }
        }
    }

    fragments
}

/// Calcula el bounding box de un triángulo
fn calculate_bounding_box(v1: &Vec3, v2: &Vec3, v3: &Vec3) -> (i32, i32, i32, i32) {
    let min_x = v1.x.min(v2.x).min(v3.x).floor() as i32;
    let min_y = v1.y.min(v2.y).min(v3.y).floor() as i32;
    let max_x = v1.x.max(v2.x).max(v3.x).ceil() as i32;
    let max_y = v1.y.max(v2.y).max(v3.y).ceil() as i32;

    (min_x, min_y, max_x, max_y)
}

/// Calcula las coordenadas barycentric de un punto dentro de un triángulo
fn barycentric_coordinates(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3, area: f32) -> (f32, f32, f32) {
    let w1 = edge_function(&b, &c, p) / area;
    let w2 = edge_function(&c, &a, p) / area;
    let w3 = edge_function(&a, &b, p) / area;

    (w1, w2, w3)
}

/// Calcula la función de borde para determinación de inclusión
fn edge_function(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}
