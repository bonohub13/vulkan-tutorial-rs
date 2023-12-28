use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type ObjectId = u32;
pub type Map = HashMap<ObjectId, GameObject>;

pub struct TransformComponent {
    pub translation: glm::Vec3,
    pub scale: glm::Vec3,
    pub rotation: glm::Vec3,
}

#[derive(Clone, Copy)]
pub struct PointLightComponent {
    pub light_intensity: f32,
}

pub struct GameObject {
    pub color: glm::Vec3,
    pub model: Option<Rc<RefCell<crate::Model>>>,
    pub transform: TransformComponent,
    pub point_light: Option<PointLightComponent>,
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

    pub fn normal_matrix(&self) -> glm::Mat4 {
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
        let inv_scale = glm::Vec3::from_column_slice(
            &self.scale.iter().map(|xyz| 1.0 / xyz).collect::<Vec<_>>(),
        );

        /* NOTE:
         *  The size of normal matrix must be Mat 4x4!
         *  nalgebra_glm does not convert mat3x3 into mat4x4 with padding,
         *  how the C++ glm library does.
         *  This causes a bug.
         */
        glm::mat4(
            inv_scale.x * (cosine[1] * cosine[2] + sine[1] * sine[0] * sine[2]),
            inv_scale.y * (cosine[2] * sine[1] * sine[0] - cosine[1] * sine[2]),
            inv_scale.z * (cosine[0] * sine[1]),
            0.0,
            inv_scale.x * (cosine[0] * sine[2]),
            inv_scale.y * (cosine[0] * cosine[2]),
            inv_scale.z * (-sine[0]),
            0.0,
            inv_scale.x * (cosine[1] * sine[0] * sine[2] - cosine[2] * sine[1]),
            inv_scale.y * (cosine[1] * cosine[2] * sine[0] + sine[1] * sine[2]),
            inv_scale.z * (cosine[1] * cosine[0]),
            0.0,
            0.0,
            0.0,
            0.0,
            1.0,
        )
    }
}

impl GameObject {
    pub fn new(object_id: ObjectId, model: Option<Rc<RefCell<crate::Model>>>) -> Self {
        Self {
            id: object_id,
            model,
            color: glm::Vec3::default(),
            transform: TransformComponent {
                ..Default::default()
            },
            point_light: None,
        }
    }

    pub unsafe fn create_game_object(model: Option<Rc<RefCell<crate::Model>>>) -> Self {
        static mut CURRENT_ID: ObjectId = 0;

        CURRENT_ID += 1;

        Self::new(CURRENT_ID, model)
    }

    pub unsafe fn make_point_light(
        intensity: Option<f32>,
        radius: Option<f32>,
        color: Option<glm::Vec3>,
    ) -> Self {
        let intensity = intensity.unwrap_or(10.0);
        let radius = radius.unwrap_or(0.1);
        let color = color.unwrap_or(glm::vec3(1.0, 1.0, 1.0));
        let mut game_object = Self::create_game_object(None);

        game_object.color = color;
        game_object.transform.scale.x = radius;
        game_object.point_light = Some(PointLightComponent {
            light_intensity: intensity,
        });

        game_object
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

impl Default for PointLightComponent {
    fn default() -> Self {
        Self {
            light_intensity: 1.0,
        }
    }
}
