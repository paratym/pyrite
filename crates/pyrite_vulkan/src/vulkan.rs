use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::Arc;

use ash::Entry;
use ash::{extensions, Device};
use ash::{vk, Instance};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use pyrite_app::resource::Resource;

static DEFAULT_QUEUE: QueueConfig = QueueConfig::new(
    "pyrite_vulkan::default_queue",
    1.0,
    &[
        QueueType::Graphics,
        QueueType::Compute,
        QueueType::Transfer,
        QueueType::Present,
    ],
);

pub type VulkanDep = Arc<dyn VulkanInstance>;

pub type VulkanRef<'a> = &'a dyn VulkanInstance;

pub trait VulkanInstance: Send + Sync {
    fn instance(&self) -> &Instance;
    fn surface_loader(&self) -> &extensions::khr::Surface;
    fn surface(&self) -> &vk::SurfaceKHR;
    fn physical_device(&self) -> &PhysicalDevice;
    fn device(&self) -> &Device;
    fn get_queue(&self, name: &str) -> &Queue;
    fn default_queue(&self) -> &Queue {
        self.get_queue(DEFAULT_QUEUE.name)
    }
}

#[derive(Resource)]
pub struct Vulkan {
    internal_instance: Arc<dyn VulkanInstance>,
}

impl VulkanInstance for Vulkan {
    fn instance(&self) -> &Instance {
        self.internal_instance.instance()
    }

    fn surface_loader(&self) -> &extensions::khr::Surface {
        self.internal_instance.surface_loader()
    }

    fn surface(&self) -> &vk::SurfaceKHR {
        self.internal_instance.surface()
    }

    fn physical_device(&self) -> &PhysicalDevice {
        self.internal_instance.physical_device()
    }

    fn device(&self) -> &Device {
        self.internal_instance.device()
    }

    fn get_queue(&self, name: &str) -> &Queue {
        self.internal_instance.get_queue(name)
    }
}

pub struct InternalVulkanInstance {
    _entry: Entry,
    instance: Instance,
    physical_device: PhysicalDevice,
    device: Device,
    surface_loader: extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    queues: HashMap<&'static str, Queue>,
}

impl VulkanInstance for InternalVulkanInstance {
    fn instance(&self) -> &Instance {
        &self.instance
    }

    fn surface_loader(&self) -> &extensions::khr::Surface {
        &self.surface_loader
    }

    fn surface(&self) -> &vk::SurfaceKHR {
        &self.surface
    }

    fn physical_device(&self) -> &PhysicalDevice {
        &self.physical_device
    }

    fn device(&self) -> &Device {
        &self.device
    }

    fn get_queue(&self, name: &str) -> &Queue {
        self.queues.get(name).unwrap()
    }
}

pub struct PhysicalDevice {
    physical_device: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    features: vk::PhysicalDeviceFeatures,
    memory_properties: vk::PhysicalDeviceMemoryProperties,
    queue_families: Vec<vk::QueueFamilyProperties>,
}

pub struct Queue {
    queue_family_index: u32,
    queue: vk::Queue,
}

impl Drop for InternalVulkanInstance {
    fn drop(&mut self) {
        unsafe {
            self.device().device_wait_idle().unwrap();

            self.surface_loader().destroy_surface(self.surface, None);
            self.device().destroy_device(None);
            self.instance().destroy_instance(None);
        }
    }
}

pub struct VulkanConfig<'a> {
    pub app_name: String,
    pub surface_window: Box<&'a dyn SurfaceWindow>,
    pub queues: Vec<&'static QueueConfig>,
}

impl Vulkan {
    pub fn new(config: VulkanConfig) -> Self {
        let entry = unsafe { Entry::load() }.expect("Vulkan could not be loaded.");

        let instance = {
            let app_name = CString::new("Pyrite").unwrap();
            let engine_name = CString::new("Pyrite").unwrap();

            let app_info = vk::ApplicationInfo::builder()
                .application_name(&app_name)
                .application_version(vk::make_api_version(0, 0, 1, 0))
                .engine_name(&engine_name)
                .engine_version(vk::make_api_version(0, 0, 1, 0))
                .api_version(vk::make_api_version(0, 1, 2, 0));

            let instance_extensions = ash_window::enumerate_required_extensions(
                config.surface_window.raw_display_handle(),
            )
            .expect("Could not enumerate required window extensions. Is the display handle valid?")
            .into_iter()
            .map(|ext| *ext as *const i8)
            .collect::<Vec<_>>();

            let mut instance_layers = vec![];
            #[cfg(debug_assertions)]
            {
                instance_layers.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());
            }

            let c_ptr_instance_layers = instance_layers
                .iter()
                .map(|layer| layer.as_ptr())
                .collect::<Vec<_>>();

            let instance_create_info = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .enabled_extension_names(&instance_extensions)
                .enabled_layer_names(&c_ptr_instance_layers);

            unsafe { entry.create_instance(&instance_create_info, None) }
                .expect("Vulkan instance could not be created.")
        };

        let surface_loader = extensions::khr::Surface::new(&entry, &instance);
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                config.surface_window.raw_display_handle(),
                config.surface_window.raw_window_handle(),
                None,
            )
        }
        .expect("Could not create Vulkan surface.");

        let physical_device = {
            let physical_devices = unsafe { instance.enumerate_physical_devices() }
                .expect("Could not enumerate physical devices.");

            // TODO: Pick physical device based on scoring system.
            let device = physical_devices[0];
            let properties = unsafe { instance.get_physical_device_properties(device) };
            let features = unsafe { instance.get_physical_device_features(device) };
            let memory_properties =
                unsafe { instance.get_physical_device_memory_properties(device) };
            let queue_families =
                unsafe { instance.get_physical_device_queue_family_properties(device) };

            PhysicalDevice {
                physical_device: device,
                properties,
                features,
                memory_properties,
                queue_families,
            }
        };

        let (device, queues) = {
            let queue_family_index = physical_device
                .queue_families
                .iter()
                .enumerate()
                .find_map(|(index, properties)| {
                    let supports_graphics =
                        properties.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                            && properties.queue_flags.contains(vk::QueueFlags::TRANSFER)
                            && properties.queue_flags.contains(vk::QueueFlags::COMPUTE);
                    let supports_surface = unsafe {
                        surface_loader.get_physical_device_surface_support(
                            physical_device.physical_device,
                            index as u32,
                            surface,
                        )
                    }
                    .unwrap();

                    if supports_graphics && supports_surface {
                        Some(index as u32)
                    } else {
                        None
                    }
                })
                .expect("Could not find a suitable queue family.");

            // TODO: Make queues search for different families to spread performance out on the gpu and make it actually asynchronous.
            let queue_priorities = config
                .queues
                .iter()
                .map(|queue| queue.priority)
                .collect::<Vec<_>>();
            let queue_create_infos = [vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&queue_priorities)
                .build()];

            let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(&queue_create_infos)
                .enabled_extension_names(&device_extensions);

            let device = unsafe {
                instance.create_device(physical_device.physical_device, &device_create_info, None)
            }
            .unwrap();

            let queues = config
                .queues
                .iter()
                .enumerate()
                .map(|(i, queue)| {
                    let vk_queue = unsafe { device.get_device_queue(queue_family_index, i as u32) };
                    (
                        queue.name,
                        Queue {
                            queue_family_index,
                            queue: vk_queue,
                        },
                    )
                })
                .collect::<HashMap<_, _>>();

            (device, queues)
        };

        Self {
            internal_instance: Arc::new(InternalVulkanInstance {
                _entry: entry,
                instance,
                device,
                physical_device,
                surface_loader,
                surface,
                queues,
            }),
        }
    }

    pub fn create_dep(&self) -> VulkanDep {
        self.internal_instance.clone()
    }
}

impl Queue {
    pub fn queue_family_index(&self) -> u32 {
        self.queue_family_index
    }

    pub fn queue(&self) -> vk::Queue {
        self.queue
    }
}

pub trait SurfaceWindow: HasRawDisplayHandle + HasRawWindowHandle {}

impl<'a> VulkanConfig<'a> {
    pub fn from_window(app_name: String, window: &'a impl SurfaceWindow) -> Self {
        Self {
            app_name,
            surface_window: Box::new(window),
            queues: vec![&DEFAULT_QUEUE],
        }
    }
}

pub enum QueueType {
    Graphics = 0b00000001,
    Compute = 0b00000010,
    Transfer = 0b00000100,
    Present = 0b00001000,
}

pub struct QueueConfig {
    /// The queue name, used as an identifier for the queue.
    name: &'static str,

    /// The priority of the queue. This is a value between 0.0 and 1.0.
    priority: f32,

    /// The command types that the queue needs.
    ///
    /// This is used to determine which queue families are suitable for the queue,
    /// however this functionality is not supported yet and only one queue family is used for now.
    _required_types: &'static [QueueType],
}

impl QueueConfig {
    pub const fn new(
        name: &'static str,
        priority: f32,
        required_types: &'static [QueueType],
    ) -> Self {
        Self {
            name,
            priority,
            _required_types: required_types,
        }
    }
}

impl PhysicalDevice {
    pub fn physical_device(&self) -> vk::PhysicalDevice {
        self.physical_device
    }

    pub fn name(&self) -> String {
        unsafe { CStr::from_ptr(self.properties.device_name.as_ptr()) }
            .to_str()
            .unwrap()
            .to_owned()
    }

    pub fn properties(&self) -> &vk::PhysicalDeviceProperties {
        &self.properties
    }

    pub fn features(&self) -> &vk::PhysicalDeviceFeatures {
        &self.features
    }

    pub fn memory_properties(&self) -> &vk::PhysicalDeviceMemoryProperties {
        &self.memory_properties
    }

    pub fn queue_families(&self) -> &[vk::QueueFamilyProperties] {
        &self.queue_families
    }
}
