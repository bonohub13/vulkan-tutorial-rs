use std::{cell::RefCell, rc::Rc};

pub type ObjectId = u32;

#[derive(PartialEq)]
pub struct TransformComponent2D {
    pub translation: glm::Vec2,
    pub scale: glm::Vec2,
    pub rotation: f32,
}

#[derive(PartialEq)]
pub struct RigidBodyComponent2D {
    pub velocity: glm::Vec2,
    pub mass: f32,
}

pub struct GameObject {
    pub color: glm::Vec3,
    pub model: Rc<RefCell<crate::Model>>,
    pub transform_2d: TransformComponent2D,
    pub rigit_body_2d: RigidBodyComponent2D,
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
            transform_2d: TransformComponent2D::default(),
            rigit_body_2d: RigidBodyComponent2D::default(),
        }
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

impl Default for RigidBodyComponent2D {
    fn default() -> Self {
        Self {
            velocity: glm::vec2(0., 0.),
            mass: 1.0,
        }
    }
}
