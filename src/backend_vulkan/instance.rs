use anyhow::Result;
use ash::{
    extensions::{ext, khr},
    vk,
};
use log;
use std::ffi::{c_void, CStr, CString};
use std::sync::Arc;

#[derive(Default)]
pub struct InstanceBuilder {
    pub required_extensions: Vec<&'static str>,
    pub debug_graphics: bool,
}

impl InstanceBuilder {
    pub fn build(self) -> Result<Arc<Instance>> {
        Ok(Arc::new(Instance::create(self)?))
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
    pub raw: ash::Instance,
    #[allow(dead_code)]
    pub(crate) debug_utils: Option<ext::DebugUtils>,
    #[allow(dead_code)]
    pub(crate) debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
}

impl Instance {
    pub fn builder() -> InstanceBuilder {
        InstanceBuilder::default()
    }

    fn internal_extension_names(builder: &InstanceBuilder) -> Vec<CString> {
        let mut names = vec![khr::Surface::name().to_owned()];
        if builder.debug_graphics {
            names.push(ext::DebugUtils::name().to_owned());
        }
        names
    }

    fn internal_layer_names(builder: &InstanceBuilder) -> Vec<CString> {
        let mut names = Vec::new();
        if builder.debug_graphics {
            names.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());
        }
        names
    }

    fn create(builder: InstanceBuilder) -> Result<Self> {
        let app_name = CString::new("PoogieApp").unwrap();
        let engine_name = CString::new("PoogieEngine").unwrap();

        let entry = unsafe { ash::Entry::load()? };

        let app_info = vk::ApplicationInfo::builder()
            .api_version(vk::make_api_version(0, 1, 3, 0))
            .application_name(&app_name)
            .engine_name(&engine_name);

        // add all extensions to vector
        let mut extension_names = Self::internal_extension_names(&builder);
        let mut req_extension_names = builder
            .required_extensions
            .clone()
            .into_iter()
            .map(|ext| CString::new(ext).unwrap())
            .collect::<Vec<_>>();
        extension_names.append(&mut req_extension_names);

        // and get extension pointers
        let extension_names = extension_names
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<*const i8>>();

        let layer_names = Self::internal_layer_names(&builder);
        let layer_names = layer_names
            .iter()
            .map(|lay| lay.as_ptr())
            .collect::<Vec<*const i8>>();

        let mut debug_messenger_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            )
            .message_type(
                // vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));

        let mut validation_features = vk::ValidationFeaturesEXT::builder()
            .enabled_validation_features(&[vk::ValidationFeatureEnableEXT::BEST_PRACTICES]);

        let create_info = vk::InstanceCreateInfo::builder()
            .push_next(&mut debug_messenger_info)
            .application_info(&app_info)
            .enabled_extension_names(&extension_names)
            .enabled_layer_names(&layer_names)
            .push_next(&mut validation_features);

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
            raw: raw_instance,
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
            log::info!("{message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            log::warn!("{message}");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            log::error!("{message}");
        }
        _ => (),
    }

    vk::FALSE
}
