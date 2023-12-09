use crate::utils as lve_utils;
use anyhow::Result;
use ash::{extensions::ext as vk_ext, vk};
use std::ffi::{c_void, CStr};

pub struct DebugUtilsMessenger {
    extension: vk_ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

unsafe extern "system" fn debug_callback(
    msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_cb_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let msg = CStr::from_ptr((*p_cb_data).p_message);
    let msg_severity = match msg_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[Verbose]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[Info]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[Warning]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[Error]",
        _ => "[Unknown]",
    };
    let msg_type = match msg_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[GENERAL]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[VALIDATION]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[PERFORMANCE]",
        vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => "[DEVICE_ADDRESS_BINDING]",
        _ => "[Unknown]",
    };

    eprintln!(
        "validation layers ({} | {}): {:?}",
        msg_severity, msg_type, msg
    );

    vk::FALSE
}

impl DebugUtilsMessenger {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Result<Self> {
        let extension = vk_ext::DebugUtils::new(entry, instance);
        let debug_utils_messenger = {
            let create_info = Self::populate_debug_message_create_info();

            unsafe { extension.create_debug_utils_messenger(&create_info, None) }
        }?;

        Ok(Self {
            extension,
            debug_utils_messenger,
        })
    }

    #[inline]
    pub fn null(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        Self {
            extension: vk_ext::DebugUtils::new(entry, instance),
            debug_utils_messenger: vk::DebugUtilsMessengerEXT::null(),
        }
    }

    #[inline]
    pub const fn extension_name() -> &'static CStr {
        vk_ext::DebugUtils::name()
    }

    #[inline]
    pub fn populate_debug_message_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
        vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE, // | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING,
            )
            .pfn_user_callback(Some(debug_callback))
            .build()
    }
}

impl Drop for DebugUtilsMessenger {
    fn drop(&mut self) {
        if !lve_utils::is_release_build() {
            unsafe {
                self.extension
                    .destroy_debug_utils_messenger(self.debug_utils_messenger, None)
            };
        }
    }
}
