pub struct Camera {
    projection_matrix: glm::Mat4,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            projection_matrix: glm::Mat4::identity(),
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
        // In the tutorial, each value is indexed [row][column]
        //  however, in nalgebra_glm, it is represented as m[column][row]
        //  Be carefull not to get it mixed up!
        self.projection_matrix.m43 = 1.0;
        self.projection_matrix.m34 = -(far * near) / (far - near);
    }

    pub const fn projection(&self) -> &glm::Mat4 {
        &self.projection_matrix
    }
}
