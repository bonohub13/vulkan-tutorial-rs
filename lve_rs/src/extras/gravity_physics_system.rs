use anyhow::Result;
use std::{cell::RefCell, rc::Rc};

pub struct GravityPhysicsSystem {
    pub strength_gravity: f32,
}

pub struct Vec2FieldSystem;

pub fn create_square_model(
    device: &crate::Device,
    offset: glm::Vec2,
) -> Result<Rc<RefCell<crate::Model>>> {
    let vertices = [
        crate::Vertex {
            position: glm::vec2(-0.5, -0.5),
            color: glm::Vec3::default(),
        },
        crate::Vertex {
            position: glm::vec2(0.5, 0.5),
            color: glm::Vec3::default(),
        },
        crate::Vertex {
            position: glm::vec2(-0.5, 0.5),
            color: glm::Vec3::default(),
        },
        crate::Vertex {
            position: glm::vec2(-0.5, -0.5),
            color: glm::Vec3::default(),
        },
        crate::Vertex {
            position: glm::vec2(0.5, -0.5),
            color: glm::Vec3::default(),
        },
        crate::Vertex {
            position: glm::vec2(0.5, 0.5),
            color: glm::Vec3::default(),
        },
    ]
    .iter_mut()
    .map(|v| {
        v.position += offset;

        *v
    })
    .collect::<Vec<_>>();

    Ok(Rc::new(RefCell::new(crate::Model::new(device, &vertices)?)))
}

pub fn create_circle_model(
    device: &crate::Device,
    num_sides: usize,
) -> Result<Rc<RefCell<crate::Model>>> {
    let unique_vertices = (0..=num_sides)
        .into_iter()
        .map(|i| {
            if i < num_sides {
                let angle = i as f32 * 2.0 * std::f32::consts::PI / num_sides as f32;

                crate::Vertex::new(&[angle.cos(), angle.sin()], &[0.0, 0.0, 0.0])
            } else {
                crate::Vertex::new(&[0.0, 0.0], &[0.0, 0.0, 0.0])
            }
        })
        .collect::<Vec<_>>();
    let vertices = (0..(num_sides * 3))
        .into_iter()
        .map(|i| {
            let offset = i % 3;
            let index = i / 3;
            match offset {
                0 => unique_vertices[index],
                1 => unique_vertices[(index + 1) % num_sides],
                2 | _ => unique_vertices[num_sides],
            }
        })
        .collect::<Vec<_>>();

    Ok(Rc::new(RefCell::new(crate::Model::new(device, &vertices)?)))
}

impl GravityPhysicsSystem {
    pub fn new(strength: f32) -> Self {
        Self {
            strength_gravity: strength,
        }
    }

    pub fn update(
        &self,
        objects: &mut [crate::GameObject],
        delta_time: f32,
        substeps: Option<u32>,
    ) {
        let substeps = if let Some(substeps) = substeps {
            substeps
        } else {
            1
        };
        let step_delta = delta_time / substeps as f32;

        for _ in 0..substeps {
            self.step_simulation(objects, step_delta)
        }
    }

    pub fn compute_force(
        &self,
        from_object: &crate::GameObject,
        to_object: &crate::GameObject,
    ) -> glm::Vec2 {
        let offset = from_object.transform_2d.translation - to_object.transform_2d.translation;
        let distance_squared = offset.dot(&offset);

        if distance_squared.abs() < 1e-10f32 {
            return glm::vec2(0., 0.);
        }

        // F = gain * (m1 * m2) / d;
        let force =
            self.strength_gravity * to_object.rigit_body_2d.mass * from_object.rigit_body_2d.mass
                / distance_squared;

        force * offset / distance_squared.sqrt()
    }

    fn step_simulation(&self, physics_objects: &mut [crate::GameObject], delta_time: f32) {
        for iter_a in 0..physics_objects.len() {
            for iter_b in 0..physics_objects.len() {
                if iter_a == iter_b {
                    continue;
                }

                let force = self.compute_force(&physics_objects[iter_a], &physics_objects[iter_b]);

                physics_objects[iter_a].rigit_body_2d.velocity +=
                    delta_time * -force / physics_objects[iter_a].rigit_body_2d.mass;
                physics_objects[iter_b].rigit_body_2d.velocity +=
                    delta_time * force / physics_objects[iter_b].rigit_body_2d.mass;
            }
        }

        for obj in physics_objects.iter_mut() {
            obj.transform_2d.translation += delta_time * obj.rigit_body_2d.velocity;
        }
    }
}

impl Vec2FieldSystem {
    pub fn update(
        &self,
        physics_system: &GravityPhysicsSystem,
        physics_objects: &[crate::GameObject],
        vector_field: &mut [crate::GameObject],
    ) {
        vector_field.iter_mut().for_each(|vf| {
            let direction = physics_objects
                .iter()
                .map(|obj| physics_system.compute_force(obj, vf))
                .sum::<glm::Vec2>();

            vf.transform_2d.scale.x = 0.005
                + 0.045
                    * ((glm::length(&direction) + 1.0).log(std::f32::consts::E) / 3.0)
                        .clamp(0.0, 1.0);
            vf.transform_2d.rotation = direction.y.atan2(direction.x);
        });
    }
}
