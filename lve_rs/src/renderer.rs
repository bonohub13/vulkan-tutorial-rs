use anyhow::{bail, Result};
use ash::vk;
use winit::event_loop::ControlFlow;

pub struct Renderer {
    swap_chain: Box<crate::SwapChain>,
    command_buffers: Vec<vk::CommandBuffer>,
    current_image_index: usize,
    current_frame_index: usize,
    frame_started: bool,
}

impl Renderer {
    pub fn new(window: &crate::Window, device: &crate::Device) -> Result<Self> {
        let swap_chain = Self::recreate_swap_chain(&window, &device, None, None)?;
        let command_buffers = Self::create_command_buffers(&device)?;

        Ok(Self {
            swap_chain,
            command_buffers,
            current_image_index: 0,
            current_frame_index: 0,
            frame_started: false,
        })
    }

    pub unsafe fn destroy(&mut self, device: &crate::Device) {
        Self::free_command_buffers(device, &mut self.command_buffers);
        self.swap_chain.destroy(device);
    }

    pub const fn swap_chain_render_pass(&self) -> &vk::RenderPass {
        self.swap_chain.render_pass()
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.swap_chain.extent_aspect_ratio() as f32
    }

    pub const fn swap_chain(&self) -> &crate::SwapChain {
        &self.swap_chain
    }

    pub const fn frame_started(&self) -> bool {
        self.frame_started
    }

    pub fn current_command_buffer(&self) -> &vk::CommandBuffer {
        assert!(
            self.frame_started,
            "Cannot get command buffer when frame not in progress"
        );

        &self.command_buffers[self.current_frame_index]
    }

    pub const fn frame_index(&self) -> usize {
        assert!(
            self.frame_started,
            "Cannot get frame index when frame not in progress"
        );

        self.current_frame_index
    }

    pub fn begin_frame(
        &mut self,
        window: &crate::Window,
        device: &crate::Device,
        control_flow: Option<&mut ControlFlow>,
    ) -> Result<vk::CommandBuffer> {
        assert!(
            !self.frame_started,
            "Can't call begin_frame while already in progress"
        );

        let device_ref = device.device();
        (self.current_image_index, _) = match self.swap_chain.acquire_next_image(device) {
            Ok((image_index, result)) => {
                if result {
                    let swap_chain = Self::recreate_swap_chain(
                        window,
                        device,
                        Some(&self.swap_chain),
                        control_flow,
                    )?;
                    unsafe { device_ref.device_wait_idle() }?;
                    unsafe {
                        self.swap_chain.destroy(device);
                    }
                    self.swap_chain = swap_chain;

                    return Ok(vk::CommandBuffer::null());
                }

                Ok((image_index, result)) as Result<(usize, bool)>
            }
            Err(_) => bail!("Failed to acquire swap chain image!"),
        }?;

        self.frame_started = true;

        let command_buffer = *self.current_command_buffer();
        let begin_info = vk::CommandBufferBeginInfo::builder();

        unsafe {
            device
                .device()
                .begin_command_buffer(command_buffer, &begin_info)
        }?;

        Ok(command_buffer)
    }

    pub fn end_frame(
        &mut self,
        window: &mut crate::Window,
        device: &crate::Device,
        control_flow: Option<&mut ControlFlow>,
    ) -> Result<()> {
        assert!(
            self.frame_started,
            "Can't call end_frame while frame is not in progress"
        );

        let device_ref = device.device();
        let command_buffer = *self.current_command_buffer();

        unsafe { device.device().end_command_buffer(command_buffer) }?;
        match self.swap_chain.submit_command_buffers(
            device,
            &command_buffer,
            self.current_image_index,
        ) {
            Ok(window_resized) => {
                if window_resized || window.was_window_resized() {
                    window.reset_window_resized_flag();
                    let swap_chain = Self::recreate_swap_chain(
                        window,
                        device,
                        Some(&self.swap_chain),
                        control_flow,
                    )?;
                    unsafe { device_ref.device_wait_idle() }?;
                    unsafe {
                        self.swap_chain.destroy(device);
                    }
                    self.swap_chain = swap_chain;
                }
            }

            Err(_) => {
                if window.was_window_resized() {
                    window.reset_window_resized_flag();
                    let swap_chain = Self::recreate_swap_chain(
                        window,
                        device,
                        Some(&self.swap_chain),
                        control_flow,
                    )?;
                    unsafe { device_ref.device_wait_idle() }?;
                    unsafe {
                        self.swap_chain.destroy(device);
                    }
                    self.swap_chain = swap_chain;
                } else {
                    bail!("Failed to present swap chain image!")
                }
            }
        };

        self.frame_started = false;
        self.current_frame_index =
            (self.current_frame_index + 1) % crate::SwapChain::MAX_FRAMES_IN_FLIGHT as usize;

        Ok(())
    }

    pub unsafe fn begin_swap_chain_render_pass(
        &self,
        device: &crate::Device,
        command_buffer: &vk::CommandBuffer,
    ) {
        assert!(
            self.frame_started,
            "Can't call begin_swap_chain_render_pass while frame is not in progress"
        );
        assert!(
            command_buffer == self.current_command_buffer(),
            "Can't begin render pass on command buffer from a different frame"
        );

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.01f32, 0.01f32, 0.01f32, 1.0f32],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue::builder()
                    .depth(1.0f32)
                    .stencil(0)
                    .build(),
            },
        ];
        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(*self.swap_chain.render_pass())
            .framebuffer(*self.swap_chain.framebuffer(self.current_image_index))
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swap_chain.swap_chain_extent(),
            })
            .clear_values(&clear_values);
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.swap_chain.width() as f32,
            height: self.swap_chain.height() as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            extent: self.swap_chain.swap_chain_extent(),
            offset: vk::Offset2D { x: 0, y: 0 },
        };

        device.device().cmd_begin_render_pass(
            *command_buffer,
            &render_pass_info,
            vk::SubpassContents::INLINE,
        );
        device
            .device()
            .cmd_set_viewport(*command_buffer, 0, std::slice::from_ref(&viewport));
        device
            .device()
            .cmd_set_scissor(*command_buffer, 0, std::slice::from_ref(&scissor));
    }

    pub unsafe fn end_swap_chain_render_pass(
        &self,
        device: &crate::Device,
        command_buffer: &vk::CommandBuffer,
    ) {
        assert!(
            self.frame_started,
            "Can't call end_swap_chain_render_pass while frame is not in progress"
        );
        assert!(
            command_buffer == self.current_command_buffer(),
            "Can't end render pass on command buffer from a different frame"
        );

        device.device().cmd_end_render_pass(*command_buffer);
    }

    fn recreate_swap_chain(
        window: &crate::Window,
        device: &crate::Device,
        old_swap_chain: Option<&crate::SwapChain>,
        mut control_flow: Option<&mut ControlFlow>,
    ) -> Result<Box<crate::SwapChain>> {
        let device_ref = device.device();
        let mut extent = window.extent()?;

        while extent.width == 0 || extent.height == 0 {
            extent = window.extent()?;
            if let Some(ref mut control_flow_mut_ref) = control_flow {
                **control_flow_mut_ref = ControlFlow::Wait;
            }
        }
        // Wait until current swap chain is out of use
        unsafe { device_ref.device_wait_idle() }?;

        let swap_chain = if let Some(old_swap_chain) = old_swap_chain {
            let swap_chain = crate::SwapChain::with_previous_swap_chain(
                device,
                extent,
                old_swap_chain.swap_chain(),
            )?;

            if !old_swap_chain.compare_swap_formats(&swap_chain) {
                bail!("Swap chain image or depth format has changed!");
            }

            swap_chain
        } else {
            crate::SwapChain::new(device, extent)?
        };

        Ok(Box::new(swap_chain))
    }

    fn create_command_buffers(device: &crate::Device) -> Result<Vec<vk::CommandBuffer>> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(*device.command_pool())
            .command_buffer_count(crate::SwapChain::MAX_FRAMES_IN_FLIGHT as u32);
        let command_buffers = unsafe { device.device().allocate_command_buffers(&allocate_info) }?;

        Ok(command_buffers)
    }

    #[inline]
    unsafe fn free_command_buffers(
        device: &crate::Device,
        command_buffers: &mut Vec<vk::CommandBuffer>,
    ) {
        device
            .device()
            .free_command_buffers(*device.command_pool(), command_buffers);
        command_buffers.clear()
    }
}
