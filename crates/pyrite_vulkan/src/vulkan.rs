use std::{collections::HashMap, ffi::CString, sync::Arc};

use ash::vk;
use pyrite_app::resource::Resource;
use raw_window_handle::HasWindowHandle;

// The default queue name.
pub const DEFAULT_QUEUE: &str = "pyrite_vulkan_default";

// The Vulkan application info engine name.
const ENGINE_NAME: &str = "pyrite";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum QueueCapability {
    Graphics,
    Compute,
    Transfer,
    Present,
}

/// How the queue should resolve if it can't be constructed.
#[derive(Clone, Debug)]
pub enum QueueResolution {
    /// Don't care if the queue was not constructed.
    DontCare,

    /// The fallback queue to use if this queue can't be constructed.
    Fallback(String),

    // Panic if the queue can't be constructed.
    Panic,
}

/// Configuration for a virtual queue.
///
/// A virtual queue is managed by the vulkan application and will be constructed
/// either on its own queue family or shared with other virtual queues.
#[derive(Debug, Clone)]
pub struct QueueConfig {
    /// The unique name of the queue.
    pub name: String,

    /// The required capabilities of the queue.
    pub capabilities: Vec<QueueCapability>,

    /// The priority of the queue, ranging from 0.0 to 1.0.
    /// Queues will be sorted by priority, those with higher priority will be constructed first.
    pub priority: f32,

    // The queue resolution strategy to use if the queue can't be constructed.
    pub resolution: QueueResolution,
}

pub enum SwapchainSupport<'a> {
    None,
    Supported(
        &'a dyn raw_window_handle::HasDisplayHandle,
        &'a dyn HasWindowHandle,
    ),
}

pub struct VulkanConfig<'a> {
    pub app_name: String,
    pub queues: Vec<QueueConfig>,
    pub enable_validation: bool,
    pub swapchain_support: SwapchainSupport<'a>,
}

impl Default for VulkanConfig<'_> {
    fn default() -> Self {
        Self {
            app_name: "Pyrite".to_string(),
            queues: vec![QueueConfig {
                name: DEFAULT_QUEUE.to_string(),
                capabilities: vec![
                    QueueCapability::Graphics,
                    QueueCapability::Compute,
                    QueueCapability::Transfer,
                    QueueCapability::Present,
                ],
                priority: 1.0,
                resolution: QueueResolution::Panic,
            }],
            enable_validation: true,
            swapchain_support: SwapchainSupport::None,
        }
    }
}

pub struct VulkanDebugUtils {
    debug_utils_loader: ash::extensions::ext::DebugUtils,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
}

pub struct VulkanSurface {
    surface_loader: ash::extensions::khr::Surface,
    surface: ash::vk::SurfaceKHR,
}

pub struct VulkanPhysicalDevice {
    physical_device: ash::vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_families: Vec<vk::QueueFamilyProperties>,
}

pub struct VulkanQueue {
    queue_family_index: u32,
    queue: vk::Queue,
}

pub struct VulkanInstance {
    entry: ash::Entry,
    instance: ash::Instance,
    debug_utils: Option<VulkanDebugUtils>,
    surface: Option<VulkanSurface>,
    physical_device: VulkanPhysicalDevice,
    device: ash::Device,
    queues: HashMap<String, VulkanQueue>,
    queue_aliases: HashMap<String, String>,
}

impl VulkanInstance {
    pub fn new(config: &VulkanConfig) -> Self {
        if config.enable_validation {
            println!("[pyrite_vulkan]: Validation enabled.");
        }

        let entry = unsafe { ash::Entry::load().expect("Failed to load Vulkan.") };

        let instance = {
            let app_name = CString::new(config.app_name.clone()).unwrap();
            let engine_name = CString::new(ENGINE_NAME).unwrap();

            let app_info = vk::ApplicationInfo::default()
                .application_name(&app_name)
                .application_version(vk::make_api_version(0, 0, 1, 0))
                .engine_name(&engine_name)
                .engine_version(vk::make_api_version(0, 0, 1, 0))
                .api_version(vk::make_api_version(0, 1, 2, 0));

            let mut instance_extensions = Vec::new();
            let mut instance_layers = Vec::new();

            // Add validation layers and debug utils if validation is enabled.
            if config.enable_validation {
                instance_extensions.push(ash::extensions::ext::DebugUtils::NAME.to_owned());
                instance_layers.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());
            }

            let mut ptr_instance_extensions = instance_extensions
                .iter()
                .map(|s| s.as_ptr())
                .collect::<Vec<_>>();
            let ptr_instance_layers = instance_layers
                .iter()
                .map(|s| s.as_ptr())
                .collect::<Vec<_>>();

            // Add window extensions if swapchain support is enabled.
            if let SwapchainSupport::Supported(has_display_handle, _) = config.swapchain_support {
                let window_extensions = ash_window::enumerate_required_extensions(
                    has_display_handle.display_handle().unwrap(),
                )
                .unwrap();

                ptr_instance_extensions.extend(window_extensions);
            }

            let instance_create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(&ptr_instance_extensions)
                .enabled_layer_names(&ptr_instance_layers);

            unsafe {
                entry
                    .create_instance(&instance_create_info, None)
                    .expect("Failed to create Vulkan instance.")
            }
        };

        let debug_utils = match config.enable_validation {
            true => {
                let debug_utils_loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);
                let debug_utils_messenger = {
                    let debug_utils_messenger_create_info =
                        vk::DebugUtilsMessengerCreateInfoEXT::default()
                            .message_severity(
                                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                                    | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
                            )
                            .message_type(
                                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                            )
                            .pfn_user_callback(Some(Self::debug_messenger_callback));

                    unsafe {
                        debug_utils_loader
                            .create_debug_utils_messenger(&debug_utils_messenger_create_info, None)
                            .expect("Failed to create Vulkan debug utils messenger.")
                    }
                };

                Some(VulkanDebugUtils {
                    debug_utils_loader,
                    debug_utils_messenger,
                })
            }
            false => None,
        };

        let surface = match config.swapchain_support {
            SwapchainSupport::None => None,
            SwapchainSupport::Supported(has_display_handle, has_window_handle) => {
                let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);
                let surface = unsafe {
                    ash_window::create_surface(
                        &entry,
                        &instance,
                        has_display_handle.display_handle().unwrap(),
                        has_window_handle.window_handle().unwrap(),
                        None,
                    )
                    .expect("Failed to create Vulkan surface.")
                };

                Some(VulkanSurface {
                    surface_loader,
                    surface,
                })
            }
        };

        let physical_device = {
            let physical_devices = unsafe {
                instance
                    .enumerate_physical_devices()
                    .expect("Failed to enumerate physical devices.")
            };

            let chosen_device = physical_devices.first().unwrap().clone();

            VulkanPhysicalDevice {
                physical_device: chosen_device,
                properties: unsafe { instance.get_physical_device_properties(chosen_device) },
                features: unsafe { instance.get_physical_device_features(chosen_device) },
                memory_properties: unsafe {
                    instance.get_physical_device_memory_properties(chosen_device)
                },
                queue_families: unsafe {
                    instance.get_physical_device_queue_family_properties(chosen_device)
                },
            }
        };

        let (device, queues, queue_aliases) = {
            let resolved_queue_definitions =
                utils::resolve_queue_definitions(&physical_device, &config, &surface);
            println!(
                "[pyrite_vulkan]: Resolved queue definitions: {:?}",
                resolved_queue_definitions
            );

            // Collect all the queue priorities for each queue family definition.
            let mut queue_definition_priorities = Vec::new();
            for (_, queue_configs) in resolved_queue_definitions.queue_family_indices() {
                let queue_priorities = queue_configs
                    .iter()
                    .map(|queue_config| queue_config.priority.clone())
                    .collect::<Vec<_>>();

                queue_definition_priorities.push(queue_priorities);
            }

            // Collect all the queue family definitions.
            let mut queue_definitions = Vec::new();
            for ((queue_family_index, _), queue_priorities) in resolved_queue_definitions
                .queue_family_indices()
                .iter()
                .zip(queue_definition_priorities.iter())
            {
                queue_definitions.push(
                    vk::DeviceQueueCreateInfo::default()
                        .queue_family_index(*queue_family_index)
                        .queue_priorities(queue_priorities),
                );
            }

            let device_create_info =
                vk::DeviceCreateInfo::default().queue_create_infos(&queue_definitions);

            let device = unsafe {
                instance
                    .create_device(physical_device.physical_device, &device_create_info, None)
                    .expect("Failed to create Vulkan device.")
            };

            let mut queues = HashMap::new();
            for (queue_family_index, queue_configs) in
                resolved_queue_definitions.queue_family_indices()
            {
                for (local_queue_index, queue_config) in queue_configs.iter().enumerate() {
                    let queue = unsafe {
                        device.get_device_queue(*queue_family_index, local_queue_index as u32)
                    };

                    queues.insert(
                        queue_config.name.clone(),
                        VulkanQueue {
                            queue_family_index: queue_family_index.clone(),
                            queue,
                        },
                    );
                }
            }
            let queue_aliases = resolved_queue_definitions.virtual_queue_aliases().clone();

            (device, queues, queue_aliases)
        };

        Self {
            entry,
            instance,
            debug_utils,
            surface,
            physical_device,
            device,
            queues,
            queue_aliases,
        }
    }

    unsafe extern "system" fn debug_messenger_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _p_user_data: *mut std::ffi::c_void,
    ) -> vk::Bool32 {
        let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
        println!(
            "[pyrite_vulkan]: {:?} {:?} {:?}",
            message_severity, message_type, message
        );

        vk::FALSE
    }
}

pub type VulkanDep = Arc<VulkanInstance>;

#[derive(Resource)]
pub struct Vulkan {
    instance: Arc<VulkanInstance>,
}

impl std::ops::Deref for Vulkan {
    type Target = VulkanInstance;

    fn deref(&self) -> &Self::Target {
        &self.instance
    }
}

impl Vulkan {
    pub fn new(config: &VulkanConfig) -> Self {
        Self {
            instance: Arc::new(VulkanInstance::new(config)),
        }
    }

    pub fn create_dep(&self) -> VulkanDep {
        Arc::clone(&self.instance)
    }
}

pub(super) mod utils {
    use std::collections::HashSet;

    use super::*;

    #[derive(Debug)]
    pub(super) struct ResolvedQueueDefinitions {
        /// Mapping the unique queue family index to it's list of virtual queue configs.
        queue_family_indices: HashMap<u32, Vec<QueueConfig>>,
        /// Mapping the virtual queue name to it's fallback queue name.
        virtual_queue_aliases: HashMap<String, String>,
    }

    impl ResolvedQueueDefinitions {
        pub(super) fn queue_family_indices(&self) -> &HashMap<u32, Vec<QueueConfig>> {
            &self.queue_family_indices
        }

        pub(super) fn virtual_queue_aliases(&self) -> &HashMap<String, String> {
            &self.virtual_queue_aliases
        }
    }

    pub(super) fn resolve_queue_definitions(
        physical_device: &VulkanPhysicalDevice,
        vulkan_config: &VulkanConfig,
        vulkan_surface: &Option<VulkanSurface>,
    ) -> ResolvedQueueDefinitions {
        // Check if the vulkan config queue definitions are valid.
        {
            let mut queue_names = HashSet::new();
            for queue_config in &vulkan_config.queues {
                // Check if the queue name is unique.
                if queue_names.contains(&queue_config.name) {
                    panic!(
                        "[pyrite_vulkan]: Queue name '{}' is not unique. Queue names must be uniquely named.",
                        queue_config.name
                    );
                }

                // Check for duplicate capabilities.
                let mut capabilities = HashSet::new();
                for capability in &queue_config.capabilities {
                    if capabilities.contains(capability) {
                        panic!(
                            "[pyrite_vulkan]: Queue capability '{:?}' is duplicated in queue '{}'. Queue capabilities must be unique.",
                            capability, queue_config.name
                        );
                    }

                    capabilities.insert(capability);
                }

                // Check if the queue priority is valid.
                if queue_config.priority < 0.0 || queue_config.priority > 1.0 {
                    panic!(
                        "[pyrite_vulkan]: Queue priority value '{}' is invalid. Queue priority must be between 0.0 and 1.0.",
                        queue_config.priority
                    );
                }

                // Check if the queue fallback is valid if specified.
                if let QueueResolution::Fallback(fallback_queue_name) = &queue_config.resolution {
                    if !vulkan_config
                        .queues
                        .iter()
                        .any(|queue| &queue.name == fallback_queue_name)
                    {
                        panic!(
                            "[pyrite_vulkan]: Queue fallback '{}' is invalid. If specified, the fallback queue be valid, otherwise set it to None.",
                            fallback_queue_name
                        );
                    }

                    // Ensure the fallback queue doesn't have a circular dependency to this queue.
                    let mut visited_queue_names = HashSet::new();
                    let mut current_queue_name = queue_config.name.clone();
                    while let QueueResolution::Fallback(current_fallback_queue_name) =
                        &vulkan_config
                            .queues
                            .iter()
                            .find(|queue| queue.name == current_queue_name)
                            .unwrap()
                            .resolution
                    {
                        // Check for circular dependencies.
                        if visited_queue_names.contains(current_fallback_queue_name) {
                            panic!(
                                "[pyrite_vulkan]: Circular dependency detected in queue fallbacks. Queue '{}' is dependent on itself.",
                                current_fallback_queue_name
                            );
                        }

                        current_queue_name = current_fallback_queue_name.clone();
                        visited_queue_names.insert(current_fallback_queue_name);
                    }
                }

                queue_names.insert(&queue_config.name);
            }
        }

        // Resolve the queue definitions
        // This will map each queue family index to it's list of virtual queue configs.
        let sorted_queue_configs = {
            let mut queue_configs = vulkan_config.queues.clone();

            // Sort the queue configs by descending priority.
            queue_configs.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
            queue_configs
        };

        let mut queue_family_indices = HashMap::new();
        let mut virtual_queue_aliases: HashMap<String, String> = HashMap::new();

        let mut queue_family_count: HashMap<u32, u32> = HashMap::new();
        for queue_config in &sorted_queue_configs {
            // Search for all the valid queue families that match the queue config.
            let valid_queue_family_indices = (0..physical_device.queue_families.len() as u32)
                .filter(|queue_family_index| {
                    is_queue_family_valid(
                        physical_device,
                        *queue_family_index,
                        queue_config,
                        vulkan_surface,
                    )
                })
                .collect::<Vec<_>>();

            // We will search within the queues valid queue family indices to find the queue
            // family with the least amount of queues. If no queue family is found or there were no
            // valid queue families, then this will return None.
            let chosen_queue_family_index: Option<u32> = valid_queue_family_indices.iter().fold(
                None,
                |min_family_index: Option<u32>, current_family_index| {
                    // Zero cost abstractions... right?
                    let min_family_count = min_family_index
                        .as_ref()
                        .map(|index| {
                            queue_family_count
                                .get(index)
                                .map(|qf| qf.clone())
                                .unwrap_or(0)
                        })
                        .unwrap_or(0);
                    let current_family_count = queue_family_count
                        .get(current_family_index)
                        .map(|qf| qf.clone())
                        .unwrap_or(0);

                    // Check if the current queue family has space for another queue.
                    if current_family_count
                        < physical_device.queue_families[current_family_index.clone() as usize]
                            .queue_count
                        || current_family_count >= min_family_count
                    {
                        if min_family_index.is_none() {
                            // The minimum queue family hasn't been set yet.
                            return Some(current_family_index.clone());
                        } else if min_family_count > current_family_count {
                            // The current queue family has less queues than the minimum queue family.
                            return Some(current_family_index.clone());
                        }
                    }

                    // The current queue family is full or has a higher queue count than the minimum queue family.
                    min_family_index
                },
            );

            // If no queue family was found, then use the queue config's specified resolution strategy.
            if chosen_queue_family_index.is_none() {
                match &queue_config.resolution {
                    QueueResolution::DontCare => {
                        // Don't care if the queue can't be constructed.
                        continue;
                    }
                    QueueResolution::Fallback(fallback_queue_name) => {
                        // Use the fallback queue if it can't be constructed.
                        virtual_queue_aliases
                            .insert(queue_config.name.clone(), fallback_queue_name.clone());
                        continue;
                    }
                    QueueResolution::Panic => {
                        // Panic if the queue can't be constructed.
                        panic!(
                            "[pyrite_vulkan]: Queue config '{}' is invalid. No queue families found that match the queue config.",
                            queue_config.name
                        );
                    }
                }
            }

            let chosen_queue_family_index = chosen_queue_family_index.unwrap();

            // Insert the into chosen queue family index's queue configs.
            queue_family_indices
                .entry(chosen_queue_family_index)
                .or_insert(Vec::new())
                .push(queue_config.clone());

            // Update the chosen queue family's queue count.
            queue_family_count.insert(
                chosen_queue_family_index,
                queue_family_count
                    .get(&chosen_queue_family_index)
                    .unwrap_or(&0)
                    + 1,
            );
        }

        // Validate and flatten virtual queue aliases.
        let virtual_queue_aliases = {
            let mut flattened_virtual_queue_aliases = HashMap::new();

            for (alias, definition) in &virtual_queue_aliases {
                // If the resolved queue name is also an alias, then resolve it to its final constructed
                // virtual queue name.
                let mut final_alias = alias.clone();
                while let Some(current_alias) = virtual_queue_aliases.get(definition) {
                    final_alias = current_alias.clone();
                }

                let final_definition = virtual_queue_aliases.get(&final_alias);

                // Validate that the final alias's definition was constructed.
                if final_definition.is_none() {
                    panic!(
                        "[pyrite_vulkan]: Virtual queue alias '{}' is invalid. The resolved virtual queue '{}' was not constructed.",
                        alias, final_alias
                    );
                }

                flattened_virtual_queue_aliases
                    .insert(final_alias, final_definition.unwrap().clone());
            }

            flattened_virtual_queue_aliases
        };

        ResolvedQueueDefinitions {
            queue_family_indices,
            virtual_queue_aliases,
        }
    }

    fn is_queue_family_valid(
        physical_device: &VulkanPhysicalDevice,
        queue_family_index: u32,
        queue_config: &QueueConfig,
        vulkan_surface: &Option<VulkanSurface>,
    ) -> bool {
        let queue_family = physical_device.queue_families[queue_family_index as usize];

        // Check if the queue family supports all the capabilities required by the queue config.
        for capability in &queue_config.capabilities {
            let capability_supported = match capability {
                QueueCapability::Graphics => {
                    queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                }
                QueueCapability::Compute => {
                    queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                }
                QueueCapability::Transfer => {
                    queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER)
                }
                QueueCapability::Present => {
                    if let Some(vulkan_surface) = vulkan_surface {
                        let surface_loader = &vulkan_surface.surface_loader;
                        let surface = vulkan_surface.surface;

                        unsafe {
                            surface_loader
                                .get_physical_device_surface_support(
                                    physical_device.physical_device,
                                    queue_family_index as u32,
                                    surface,
                                )
                                .unwrap()
                        }
                    } else {
                        false
                    }
                }
            };

            // If the queue family doesn't support one of the capabilities, then it is not valid.
            if !capability_supported {
                return false;
            }
        }

        // All capabilities are supported by the queue family.
        return true;
    }
}
