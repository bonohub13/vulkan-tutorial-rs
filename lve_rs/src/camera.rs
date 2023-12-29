pub struct Camera {
    projection_matrix: glm::Mat4,
    view_matrix: glm::Mat4,
    inverse_view_matrix: glm::Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            projection_matrix: glm::Mat4::identity(),
            view_matrix: glm::Mat4::identity(),
            inverse_view_matrix: glm::Mat4::identity(),
        }
    }

    pub fn set_orthographic_projection(
        &mut self,
        left: f32,
        right: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    ) {
        self.projection_matrix = glm::Mat4::identity();
        self.projection_matrix.m11 = 2.0 / (right - left);
        self.projection_matrix.m22 = 2.0 / (bottom - top);
        self.projection_matrix.m33 = 1.0 / (far - near);
        self.projection_matrix.m41 = -(right + left) / (right - left);
        self.projection_matrix.m42 = -(bottom + top) / (bottom - top);
        self.projection_matrix.m43 = -near / (far - near);
    }

    pub fn set_perspective_projection(&mut self, fovy: f32, aspect: f32, near: f32, far: f32) {
        assert!((aspect - std::f32::EPSILON) > 0.0);

        let tan_half_fovy = (0.5 * fovy).tan();

        self.projection_matrix = glm::Mat4::zeros();
        self.projection_matrix.m11 = 1.0 / (aspect * tan_half_fovy);
        self.projection_matrix.m22 = 1.0 / tan_half_fovy;
        self.projection_matrix.m33 = far / (far - near);
        // In the tutorial, each value is indexed [column][row]
        //  however, in nalgebra_glm, it is represented as m[row][column]
        //  Be carefull not to get it mixed up!
        self.projection_matrix.m43 = 1.0;
        self.projection_matrix.m34 = -(far * near) / (far - near);
    }

    pub fn set_view_direction(
        &mut self,
        position: &[f32; 3],
        direction: &[f32; 3],
        up: Option<&[f32; 3]>,
    ) {
        let up = match up {
            Some(up) => glm::Vec3::from_row_slice(up),
            None => glm::vec3(0.0, -1.0, 0.0),
        };
        let w = glm::normalize(&glm::Vec3::from_row_slice(direction));
        let u = glm::normalize(&glm::cross(&w, &up));
        let v = glm::normalize(&glm::cross(&w, &u));
        let position = glm::Vec3::from_row_slice(position);

        self.view_matrix = glm::Mat4::identity();
        self.view_matrix.m11 = u.x;
        self.view_matrix.m12 = u.y;
        self.view_matrix.m13 = u.z;
        self.view_matrix.m21 = v.x;
        self.view_matrix.m22 = v.y;
        self.view_matrix.m23 = v.z;
        self.view_matrix.m31 = w.x;
        self.view_matrix.m32 = w.y;
        self.view_matrix.m33 = w.z;
        self.view_matrix.m14 = -glm::dot(&u, &position);
        self.view_matrix.m24 = -glm::dot(&v, &position);
        self.view_matrix.m34 = -glm::dot(&w, &position);

        self.inverse_view_matrix = glm::Mat4::identity();
        self.inverse_view_matrix.m11 = u.x;
        self.inverse_view_matrix.m21 = u.y;
        self.inverse_view_matrix.m31 = u.z;
        self.inverse_view_matrix.m12 = v.x;
        self.inverse_view_matrix.m22 = v.y;
        self.inverse_view_matrix.m32 = v.z;
        self.inverse_view_matrix.m13 = w.x;
        self.inverse_view_matrix.m23 = w.y;
        self.inverse_view_matrix.m33 = w.z;
        self.inverse_view_matrix.m14 = position.x;
        self.inverse_view_matrix.m24 = position.y;
        self.inverse_view_matrix.m34 = position.z;
    }

    pub fn set_view_target(
        &mut self,
        position: &[f32; 3],
        target: &[f32; 3],
        up: Option<&[f32; 3]>,
    ) {
        let target_delta = (0..3)
            .into_iter()
            .map(|idx| target[idx] - position[idx])
            .collect::<Vec<_>>();

        self.set_view_direction(
            position,
            &[target_delta[0], target_delta[1], target_delta[2]],
            up,
        );
    }

    pub fn set_view_xyz(&mut self, position: &[f32; 3], rotation: &[f32; 3]) {
        let cosine = [rotation[1].cos(), rotation[0].cos(), rotation[2].cos()];
        let sine = [rotation[1].sin(), rotation[0].sin(), rotation[2].sin()];
        let u = glm::vec3(
            cosine[0] * cosine[2] + sine.iter().product::<f32>(),
            cosine[1] * sine[2],
            cosine[0] * sine[1] * sine[2] - cosine[2] * sine[0],
        );
        let v = glm::vec3(
            cosine[2] * sine[0] * sine[1] - cosine[0] * sine[2],
            cosine[1] * cosine[2],
            cosine[0] * cosine[2] * sine[1] + sine[0] * sine[2],
        );
        let w = glm::vec3(cosine[1] * sine[0], -sine[1], cosine[0] * cosine[1]);
        let position = glm::Vec3::from_row_slice(position);

        self.view_matrix = glm::Mat4::identity();
        self.view_matrix.m11 = u.x;
        self.view_matrix.m12 = u.y;
        self.view_matrix.m13 = u.z;
        self.view_matrix.m21 = v.x;
        self.view_matrix.m22 = v.y;
        self.view_matrix.m23 = v.z;
        self.view_matrix.m31 = w.x;
        self.view_matrix.m32 = w.y;
        self.view_matrix.m33 = w.z;
        self.view_matrix.m14 = -glm::dot(&u, &position);
        self.view_matrix.m24 = -glm::dot(&v, &position);
        self.view_matrix.m34 = -glm::dot(&w, &position);

        self.inverse_view_matrix = glm::Mat4::identity();
        self.inverse_view_matrix.m11 = u.x;
        self.inverse_view_matrix.m21 = u.y;
        self.inverse_view_matrix.m31 = u.z;
        self.inverse_view_matrix.m12 = v.x;
        self.inverse_view_matrix.m22 = v.y;
        self.inverse_view_matrix.m32 = v.z;
        self.inverse_view_matrix.m13 = w.x;
        self.inverse_view_matrix.m23 = w.y;
        self.inverse_view_matrix.m33 = w.z;
        self.inverse_view_matrix.m14 = position.x;
        self.inverse_view_matrix.m24 = position.y;
        self.inverse_view_matrix.m34 = position.z;
    }

    pub const fn projection(&self) -> &glm::Mat4 {
        &self.projection_matrix
    }

    pub const fn view(&self) -> &glm::Mat4 {
        &self.view_matrix
    }

    pub const fn inverse_view(&self) -> &glm::Mat4 {
        &self.inverse_view_matrix
    }

    pub fn position(&self) -> glm::Vec3 {
        self.inverse_view_matrix.column(3).xyz()
    }
}
