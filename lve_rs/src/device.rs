use anyhow::{bail, Context, Result};
use winit::window::Window;

pub struct Device {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    physical_device: wgpu::Adapter,
    device: wgpu::Device,
}

impl Device {
    pub async fn new(window: &Window) -> Result<Self> {
        let instance = Self::create_instance();
        let surface = unsafe { Self::create_surface(&instance, window) }?;
        let physical_device = Self::pick_physical_device(&instance, &surface)?;
        let (device, queue) = Self::create_logical_device(&physical_device).await?;

        Ok(Self {
            instance,
            surface,
            physical_device,
            device,
        })
    }

    fn create_instance() -> wgpu::Instance {
        let instance_descriptor = wgpu::InstanceDescriptor {
            backends: Self::backends(),
            ..Default::default()
        };

        wgpu::Instance::new(instance_descriptor)
    }

    unsafe fn create_surface(instance: &wgpu::Instance, window: &Window) -> Result<wgpu::Surface> {
        Ok(instance.create_surface(window)?)
    }

    fn pick_physical_device(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> Result<wgpu::Adapter> {
        let device_count = instance.enumerate_adapters(Self::backends()).count();

        if device_count == 0 {
            bail!("Failed to find GPUs with Vulkan support!");
        }

        println!("Device count: {}", device_count);

        let device = instance
            .enumerate_adapters(Self::backends())
            .filter(|adapter| adapter.is_surface_supported(surface))
            .next()
            .context("Failed to find a suitable GPU!")?;
        let properties = device.get_info();

        println!("Physical device: {}", properties.name);

        Ok(device)
    }

    async fn create_logical_device(
        physical_device: &wgpu::Adapter,
    ) -> Result<(wgpu::Device, wgpu::Queue)> {
        let (device, queue) = physical_device
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::SPIRV_SHADER_PASSTHROUGH,
                    label: None,
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;

        Ok((device, queue))
    }

    /* ---- Helper functions ---- */
    fn backends() -> wgpu::Backends {
        let mut backends = wgpu::Backends::all();

        backends.remove(wgpu::Backends::BROWSER_WEBGPU);

        backends
    }
}
