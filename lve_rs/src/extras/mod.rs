use anyhow::Result;
use std::{cell::RefCell, rc::Rc};

pub fn serpinski(
    top: &crate::Vertex,
    right: &crate::Vertex,
    left: &crate::Vertex,
    depth: u32,
) -> Vec<crate::Vertex> {
    let mut vertices = vec![];

    serpinski_triangle(&mut vertices, top, right, left, depth);

    vertices
}

pub unsafe fn multiple_triangles(device: &crate::Device) -> Result<Vec<crate::GameObject>> {
    let vertices = [
        crate::Vertex::new(&[0.0f32, -0.5f32, 0.0], &[1.0, 0., 0.]),
        crate::Vertex::new(&[0.5f32, 0.5f32, 0.0], &[0., 1., 0.]),
        crate::Vertex::new(&[-0.5f32, 0.5f32, 0.0], &[0., 0., 1.]),
    ];
    let colors = [
        glm::vec3(1., 0.7, 0.73),
        glm::vec3(1., 0.87, 0.73),
        glm::vec3(1., 1., 0.73),
        glm::vec3(0.73, 1., 0.8),
        glm::vec3(0.73, 0.88, 1.),
    ]
    .iter()
    .map(|color| glm::pow(color, &glm::vec3(2.2, 2.2, 2.2)))
    .collect::<Vec<_>>();
    let mut triangles = vec![];

    for i in 0..40 {
        let model = Rc::new(RefCell::new(crate::Model::new(device, &vertices)?));
        let mut triangle = crate::GameObject::create_game_object(model);
        let offset = i as f32;

        triangle.transform.scale = (0.5 + offset * 0.025) * glm::vec3(1., 1., 1.);
        triangle.transform.rotation.y = std::f32::consts::PI * 0.025 * offset;
        triangle.color = colors[i as usize % colors.len()];

        triangles.push(triangle);
    }

    Ok(triangles)
}

fn serpinski_triangle(
    vertices: &mut Vec<crate::Vertex>,
    top: &crate::Vertex,
    right: &crate::Vertex,
    left: &crate::Vertex,
    depth: u32,
) {
    if depth <= 0 {
        vertices.push(*top);
        vertices.push(*right);
        vertices.push(*left);
    } else {
        let top_right = crate::Vertex {
            position: 0.5f32 * (top.position + right.position),
            color: 0.5f32 * (top.color + right.color),
        };
        let right_left = crate::Vertex {
            position: 0.5f32 * (right.position + left.position),
            color: 0.5f32 * (right.color + left.color),
        };
        let left_top = crate::Vertex {
            position: 0.5f32 * (left.position + top.position),
            color: 0.5f32 * (left.color + top.color),
        };

        serpinski_triangle(vertices, &left_top, &right_left, &left, depth - 1);
        serpinski_triangle(vertices, &top_right, &right, &right_left, depth - 1);
        serpinski_triangle(vertices, top, &top_right, &left_top, depth - 1);
    }
}
