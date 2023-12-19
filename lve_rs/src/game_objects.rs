use std::{cell::RefCell, rc::Rc};

pub type ObjectId = u32;

pub struct TransformComponent {
    pub translation: glm::Vec3,
    pub scale: glm::Vec3,
    pub rotation: glm::Vec3,
}

pub struct GameObject {
    pub color: glm::Vec3,
    pub model: Rc<RefCell<crate::Model>>,
    pub transform: TransformComponent,
    id: ObjectId,
}

impl TransformComponent {
    pub fn mat4(&self) -> glm::Mat4 {
        let cosine = self
            .rotation
            .into_iter()
            .map(|xyz| xyz.cos())
            .collect::<Vec<_>>();
        let sine = self
            .rotation
            .into_iter()
            .map(|xyz| xyz.sin())
            .collect::<Vec<_>>();

        glm::mat4(
            self.scale.x * (cosine[1] * cosine[2] + sine[1] * sine[0] * sine[2]),
            self.scale.y * (cosine[2] * sine[1] * sine[0] - cosine[1] * sine[2]),
            self.scale.z * (cosine[0] * sine[1]),
            self.translation.x,
            self.scale.x * (cosine[0] * sine[2]),
            self.scale.y * (cosine[0] * cosine[2]),
            self.scale.z * (-sine[0]),
            self.translation.y,
            self.scale.x * (cosine[1] * sine[0] * sine[2] - cosine[2] * sine[1]),
            self.scale.y * (cosine[1] * cosine[2] * sine[0] + sine[1] * sine[2]),
            self.scale.z * (cosine[1] * cosine[0]),
            self.translation.z,
            0.0,
            0.0,
            0.0,
            1.0,
        )
    }
}

impl GameObject {
    pub fn new(object_id: ObjectId, model: Rc<RefCell<crate::Model>>) -> Self {
        Self {
            id: object_id,
            model,
            color: glm::Vec3::default(),
            transform: TransformComponent {
                ..Default::default()
            },
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

impl Default for TransformComponent {
    fn default() -> Self {
        Self {
            translation: glm::Vec3::default(),
            scale: glm::vec3(1., 1., 1.),
            rotation: glm::Vec3::default(),
        }
    }
}
