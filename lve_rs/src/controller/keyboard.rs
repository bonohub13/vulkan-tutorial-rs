use winit::event::VirtualKeyCode;

pub struct KeyMappings {
    pub move_left: VirtualKeyCode,
    pub move_right: VirtualKeyCode,
    pub move_forward: VirtualKeyCode,
    pub move_back: VirtualKeyCode,
    pub move_up: VirtualKeyCode,
    pub move_down: VirtualKeyCode,
    pub look_left: VirtualKeyCode,
    pub look_right: VirtualKeyCode,
    pub look_up: VirtualKeyCode,
    pub look_down: VirtualKeyCode,
}

pub struct KeyboardMovementController {
    pub keys: KeyMappings,
    pub move_speed: f32,
    pub look_speed: f32,
}

impl KeyboardMovementController {
    pub fn new(move_speed: f32, look_speed: f32) -> Self {
        Self {
            keys: KeyMappings::default(),
            move_speed,
            look_speed,
        }
    }

    pub fn move_in_plane_xz(
        &self,
        delta_time: f32,
        game_object: &mut crate::GameObject,
        keys: &[Option<VirtualKeyCode>],
    ) {
        let keys_pressed = keys
            .iter()
            .filter(|key| key.is_some())
            .map(|key| key.unwrap())
            .collect::<Vec<_>>();

        if keys_pressed.len() == 0 {
            return;
        }
        let mut rotate = glm::Vec3::zeros();

        if keys_pressed.contains(&self.keys.look_right) {
            rotate.y += 1.0;
        }
        if keys_pressed.contains(&self.keys.look_left) {
            rotate.y -= 1.0;
        }
        if keys_pressed.contains(&self.keys.look_up) {
            rotate.x += 1.0;
        }
        if keys_pressed.contains(&self.keys.look_down) {
            rotate.x -= 1.0;
        }

        if rotate.dot(&rotate) > std::f32::EPSILON {
            game_object.transform.rotation +=
                self.look_speed * delta_time * glm::normalize(&rotate);
        }

        game_object.transform.rotation.x = game_object.transform.rotation.x.clamp(-1.5, 1.5);
        game_object.transform.rotation.y =
            glm::modf(game_object.transform.rotation.y, 2.0 * std::f32::consts::PI);

        let yaw = game_object.transform.rotation.y;
        let forward_direction = glm::vec3(yaw.sin(), 0.0, yaw.cos());
        let right_direction = glm::vec3(forward_direction.z, 0.0, -forward_direction.x);
        let up_direction = glm::vec3(0.0, -1.0, 0.0);
        let mut move_direction = glm::Vec3::zeros();

        if keys_pressed.contains(&self.keys.move_forward) {
            move_direction += forward_direction;
        }
        if keys_pressed.contains(&self.keys.move_back) {
            move_direction -= forward_direction;
        }
        if keys_pressed.contains(&self.keys.move_right) {
            move_direction += right_direction;
        }
        if keys_pressed.contains(&self.keys.move_left) {
            move_direction -= right_direction;
        }
        if keys_pressed.contains(&self.keys.move_up) {
            move_direction += up_direction;
        }
        if keys_pressed.contains(&self.keys.move_down) {
            move_direction -= up_direction;
        }

        if move_direction.dot(&move_direction) > std::f32::EPSILON {
            game_object.transform.translation +=
                self.move_speed * delta_time * glm::normalize(&move_direction);
        }
    }
}

impl Default for KeyMappings {
    fn default() -> Self {
        Self {
            move_left: VirtualKeyCode::A,
            move_right: VirtualKeyCode::D,
            move_forward: VirtualKeyCode::W,
            move_back: VirtualKeyCode::S,
            move_up: VirtualKeyCode::E,
            move_down: VirtualKeyCode::Q,
            look_left: VirtualKeyCode::Left,
            look_right: VirtualKeyCode::Right,
            look_up: VirtualKeyCode::Up,
            look_down: VirtualKeyCode::Down,
        }
    }
}

impl Default for KeyboardMovementController {
    fn default() -> Self {
        Self {
            keys: KeyMappings::default(),
            move_speed: 3.0,
            look_speed: 1.5,
        }
    }
}
