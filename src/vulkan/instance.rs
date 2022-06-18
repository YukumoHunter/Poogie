use anyhow::Result;
use ash::{extensions::ext, vk};
use log;
use std::ffi::{c_void, CStr, CString};

#[derive(Default)]
pub struct InstanceBuilder {
    pub required_extensions: Vec<&'static str>,
    pub debug_graphics: bool,
}

impl InstanceBuilder {
    pub fn build(self) -> Result<Instance> {
        Instance::create(self)
    }

    pub fn required_extensions(mut self, required_extensions: Vec<&'static str>) -> Self {
        self.required_extensions = required_extensions;
        self
    }

    pub fn debug_graphics(mut self, debug_graphics: bool) -> Self {
        self.debug_graphics = debug_graphics;
        self
    }
}

pub struct Instance {
    #[allow(dead_code)]
    pub(crate) entry: ash::Entry,
    pub raw_instance: ash::Instance,
    #[allow(dead_code)]
    pub(crate) debug_utils: Option<ext::DebugUtils>,
    #[allow(dead_code)]
    pub(crate) debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Instance {
    pub fn builder() -> InstanceBuilder {
        InstanceBuilder::default()
    }

    pub fn create(builder: InstanceBuilder) -> Result<Self> {
        let app_name = CString::new("PoogieApp").unwrap();
        let engine_name = CString::new("PoogieEngine").unwrap();

        let entry = unsafe { ash::Entry::load()? };

        let app_info = vk::ApplicationInfo::builder()
            .api_version(vk::make_api_version(0, 1, 3, 0))
            .application_name(&app_name)
            .engine_name(&engine_name);

        let required_extensions = builder
            .required_extensions
            .into_iter()
            .map(|ext| CString::new(ext).unwrap())
            .collect::<Vec<_>>();

        let extension_ptrs = required_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();

        let debug_messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback))
            .build();

        // TEMP
        let name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
        let layernames = vec![name.as_ptr()];

        let mut create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layernames)
            .enabled_extension_names(&extension_ptrs)
            .build();
        create_info.p_next = &debug_messenger_info as *const _ as *const c_void;

        let raw_instance = unsafe { entry.create_instance(&create_info, None)? };
        log::info!("Created vulkan instance!");

        let (debug_utils, debug_messenger) = if builder.debug_graphics {
            let debug_utils = ext::DebugUtils::new(&entry, &raw_instance);

            let debug_messenger =
                unsafe { debug_utils.create_debug_utils_messenger(&debug_messenger_info, None)? };

            (Some(debug_utils), Some(debug_messenger))
        } else {
            (None, None)
        };

        Ok(Self {
            entry,
            raw_instance,
            debug_utils,
            debug_messenger,
        })
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    _msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    _msg_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> u32 {
    let message = CStr::from_ptr((*_msg_data).p_message).to_str().unwrap();

    match _msg_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            log::info!("{}\n", message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("{}\n", message)
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("{}\n", message)
        }
        _ => (),
    }

    vk::FALSE
}
