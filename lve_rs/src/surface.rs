use crate::Window;
use anyhow::Result;
use ash::{extensions::khr as vk_khr, vk};
use ash_window;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use std::ffi::CStr;

pub struct Surface {
    extension: vk_khr::Surface,
    surface: vk::SurfaceKHR,
}

pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

impl Surface {
    pub fn new(window: &Window, entry: &ash::Entry, instance: &ash::Instance) -> Result<Self> {
        let extension = vk_khr::Surface::new(entry, instance);
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.window().raw_display_handle(),
                window.window().raw_window_handle(),
                None,
            )
        }?;

        Ok(Self { extension, surface })
    }

    #[inline]
    pub const fn extension_name() -> &'static CStr {
        vk_khr::Surface::name()
    }

    #[inline]
    pub fn surface(&self) -> &vk::SurfaceKHR {
        &self.surface
    }

    #[inline]
    pub unsafe fn get_physical_device_surface_support(
        &self,
        physical_device: &vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<bool> {
        Ok(self.extension.get_physical_device_surface_support(
            *physical_device,
            queue_family_index,
            self.surface,
        )?)
    }

    #[inline]
    pub unsafe fn get_physical_device_surface_capabilities(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<vk::SurfaceCapabilitiesKHR> {
        Ok(self
            .extension
            .get_physical_device_surface_capabilities(*physical_device, self.surface)?)
    }

    #[inline]
    pub unsafe fn get_physical_device_surface_formats(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<Vec<vk::SurfaceFormatKHR>> {
        Ok(self
            .extension
            .get_physical_device_surface_formats(*physical_device, self.surface)?)
    }

    #[inline]
    pub unsafe fn get_physical_device_surface_present_modes(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<Vec<vk::PresentModeKHR>> {
        Ok(self
            .extension
            .get_physical_device_surface_present_modes(*physical_device, self.surface)?)
    }

    #[inline]
    pub unsafe fn query_swap_chain_support(
        &self,
        physical_device: &vk::PhysicalDevice,
    ) -> Result<SwapChainSupportDetails> {
        Ok(SwapChainSupportDetails {
            capabilities: self.get_physical_device_surface_capabilities(physical_device)?,
            formats: self.get_physical_device_surface_formats(physical_device)?,
            present_modes: self.get_physical_device_surface_present_modes(physical_device)?,
        })
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.extension.destroy_surface(self.surface, None);
        }
    }
}
