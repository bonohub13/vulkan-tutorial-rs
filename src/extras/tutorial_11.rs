use anyhow::Result;
use nalgebra_glm as glm;
use winit::event_loop::{ControlFlow, EventLoop};

const GRID_COUNT: usize = 40;

pub struct GravityPhysicsDemo {
    app: crate::App,
    gravity_system: lve_rs::extras::gravity_physics_system::GravityPhysicsSystem,
    vec_field_system: lve_rs::extras::gravity_physics_system::Vec2FieldSystem,
    physics_objects: Vec<lve_rs::GameObject>,
    vector_field: Vec<lve_rs::GameObject>,
}

impl GravityPhysicsDemo {
    pub fn new<T>(
        event_loop: &EventLoop<T>,
        width: Option<i32>,
        height: Option<i32>,
    ) -> Result<Self> {
        let app = crate::App::new(event_loop, width, height)?;
        let gravity_system =
            lve_rs::extras::gravity_physics_system::GravityPhysicsSystem::new(0.405);
        let vec_field_system = lve_rs::extras::gravity_physics_system::Vec2FieldSystem;
        let mut physics_objects = vec![];
        let mut vector_field = vec![];
        let red_sphere = {
            let model =
                lve_rs::extras::gravity_physics_system::create_circle_model(app.device(), 64)?;
            let mut sphere = unsafe { lve_rs::GameObject::create_game_object(model) };

            sphere.transform_2d.scale = glm::vec2(0.05, 0.05);
            sphere.transform_2d.translation = glm::vec2(0.5, 0.5);
            sphere.color = glm::vec3(1., 0., 0.);
            sphere.rigit_body_2d.velocity = glm::vec2(-0.5, 0.);

            sphere
        };
        let blue_sphere = {
            let model =
                lve_rs::extras::gravity_physics_system::create_circle_model(app.device(), 64)?;
            let mut sphere = unsafe { lve_rs::GameObject::create_game_object(model) };

            sphere.transform_2d.scale = glm::vec2(0.05, 0.05);
            sphere.transform_2d.translation = glm::vec2(-0.45, -0.25);
            sphere.color = glm::vec3(0., 0., 1.);
            sphere.rigit_body_2d.velocity = glm::vec2(0.5, 0.);

            sphere
        };

        physics_objects.push(red_sphere);
        physics_objects.push(blue_sphere);

        for i in 0..GRID_COUNT {
            for j in 0..GRID_COUNT {
                let model = lve_rs::extras::gravity_physics_system::create_square_model(
                    app.device(),
                    glm::vec2(0.5, 0.),
                )?;
                let mut vf = unsafe { lve_rs::GameObject::create_game_object(model) };

                vf.transform_2d.scale = glm::vec2(0.005, 0.005);
                vf.transform_2d.translation = glm::vec2(
                    -1.0 + (i as f32 + 0.5) * 2.0 / GRID_COUNT as f32,
                    -1.0 + (j as f32 + 0.5) * 2.0 / GRID_COUNT as f32,
                );
                vf.color = glm::vec3(1., 1., 1.);

                vector_field.push(vf);
            }
        }

        Ok(Self {
            app,
            gravity_system,
            vec_field_system,
            physics_objects,
            vector_field,
        })
    }

    #[inline]
    pub fn window(&self) -> &lve_rs::Window {
        self.app.window()
    }

    #[inline]
    pub fn window_resized(&mut self, width: i32, height: i32) {
        self.app.window_resized(width, height)
    }

    #[inline]
    pub fn device(&self) -> &lve_rs::Device {
        self.app.device()
    }

    pub fn draw_frame(&mut self, mut control_flow: Option<&mut ControlFlow>) -> Result<()> {
        if let Ok(command_buffer) =
            self.app
                .begin_frame(if let Some(ref mut cf_ref_mut) = control_flow {
                    Some(cf_ref_mut)
                } else {
                    None
                })
        {
            self.gravity_system
                .update(&mut self.physics_objects, 1.0 / 60.0, Some(5));
            self.vec_field_system.update(
                &self.gravity_system,
                &self.physics_objects,
                &mut self.vector_field,
            );
            unsafe {
                self.app.begin_swap_chain_render_pass(&command_buffer);
                self.app
                    .render_game_objects(&command_buffer, &mut self.physics_objects);
                self.app
                    .render_game_objects(&command_buffer, &mut self.vector_field);
                self.app.end_swap_chain_render_pass(&command_buffer);
            }
            self.app
                .end_frame(if let Some(ref mut cf_ref_mut) = control_flow {
                    Some(cf_ref_mut)
                } else {
                    None
                })?;
        }

        unsafe { self.app.device_wait_idle() }?;

        Ok(())
    }
}

impl Drop for GravityPhysicsDemo {
    fn drop(&mut self) {
        for obj in self.vector_field.iter() {
            unsafe { (*obj.model).borrow_mut().destroy(self.device()) }
        }
        for obj in self.physics_objects.iter() {
            unsafe { (*obj.model).borrow_mut().destroy(self.device()) }
        }
    }
}
