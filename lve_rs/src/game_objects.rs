use anyhow::Result;
use std::{cell::RefCell, rc::Rc};

pub type ObjectId = u32;

pub struct TransformComponent2D {
    pub translation: glm::Vec2,
    pub scale: glm::Vec2,
    pub rotation: f32,
}

pub struct GameObject {
    pub color: glm::Vec3,
    pub model: Rc<RefCell<crate::Model>>,
    pub transform_2d: TransformComponent2D,
    id: ObjectId,
}

impl TransformComponent2D {
    pub fn mat2(&self) -> glm::Mat2 {
        let rotation_mat = {
            let s = (-self.rotation).sin();
            let c = (-self.rotation).cos();

            glm::mat2(c, s, -s, c)
        };
        let scale_mat = glm::mat2(self.scale.x, 0., 0., self.scale.y);

        rotation_mat * scale_mat
    }
}

impl GameObject {
    pub fn new(object_id: ObjectId, model: Rc<RefCell<crate::Model>>) -> Self {
        Self {
            id: object_id,
            model,
            color: glm::Vec3::default(),
            transform_2d: TransformComponent2D {
                ..Default::default()
            },
        }
    }

    pub unsafe fn multiple_triangles(device: &crate::Device) -> Result<Vec<Self>> {
        let vertices = crate::Vertex::serpinski(
            &crate::Vertex::new(&[0.0f32, -0.5f32], &[1.0, 0., 0.]),
            &crate::Vertex::new(&[0.5f32, 0.5f32], &[0., 1., 0.]),
            &crate::Vertex::new(&[-0.5f32, 0.5f32], &[0., 0., 1.]),
            0,
        );
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
            let mut triangle = Self::create_game_object(model);
            let offset = i as f32;

            triangle.transform_2d.scale = (0.5 + offset * 0.025) * glm::vec2(1., 1.);
            triangle.transform_2d.rotation = std::f32::consts::PI * 0.025 * offset;
            triangle.color = colors[i as usize % colors.len()];

            triangles.push(triangle);
        }

        Ok(triangles)
    }

    pub unsafe fn create_game_object(model: Rc<RefCell<crate::Model>>) -> Self {
        static mut CURRENT_ID: ObjectId = 0;

        CURRENT_ID += 1;

        Self::new(CURRENT_ID, model)
    }

    pub const fn id(&self) -> ObjectId {
        self.id
    }
}

impl Default for TransformComponent2D {
    fn default() -> Self {
        Self {
            translation: glm::Vec2::default(),
            scale: glm::vec2(1., 1.),
            rotation: 0.,
        }
    }
}
