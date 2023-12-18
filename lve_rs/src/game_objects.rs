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
        let mut transform = glm::translation(&self.translation);

        // YXZ (Tait-Bryan method)
        transform = glm::rotate(&transform, self.rotation.y, &glm::vec3(0., 1., 0.));
        transform = glm::rotate(&transform, self.rotation.x, &glm::vec3(1., 0., 0.));
        transform = glm::rotate(&transform, self.rotation.z, &glm::vec3(0., 0., 1.));

        glm::scale(&transform, &self.scale)
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
