use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    sync::Arc,
};

use ash::{extensions, vk, Device, Entry, Instance};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use pyrite_app::resource::Resource;

pub type QueueName = &'static str;

pub static DEFAULT_QUEUE: QueueConfig = QueueConfig::new(
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
    fn queue(&self, name: QueueName) -> Option<&Queue>;
    fn default_queue(&self) -> &Queue {
        self.queue(DEFAULT_QUEUE.queue_name()).unwrap()
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

    fn queue(&self, name: QueueName) -> Option<&Queue> {
        self.internal_instance.queue(name)
    }
}

pub struct InternalVulkanInstance {
    _entry: Entry,
    instance: Instance,
    physical_device: PhysicalDevice,
    device: Device,
    surface_loader: extensions::khr::Surface,
    surface: vk::SurfaceKHR,
    queues: HashMap<QueueName, Queue>,
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

    fn queue(&self, name: QueueName) -> Option<&Queue> {
        self.queues.get(name)
    }
}

#[derive(Clone)]
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
            println!("Queue families: {:?}", queue_families);

            PhysicalDevice {
                physical_device: device,
                properties,
                features,
                memory_properties,
                queue_families,
            }
        };

        let (device, queues) = Self::create_device(
            config,
            &physical_device,
            &surface_loader,
            surface,
            &instance,
        );

        Self {
            internal_instance: Arc::new(InternalVulkanInstance {
                _entry: entry,
                instance,
                physical_device,
                device,
                surface_loader,
                surface,
                queues,
            }),
        }
    }

    fn create_device(
        config: VulkanConfig<'_>,
        physical_device: &PhysicalDevice,
        surface_loader: &extensions::khr::Surface,
        surface: vk::SurfaceKHR,
        instance: &Instance,
    ) -> (Device, HashMap<QueueName, Queue>) {
        // Maps queue family index to a queue.
        let mut queue_family_indices = HashMap::<u32, Vec<QueueConfig>>::new();

        // Find queue families we need and queues we need.
        config.queues.into_iter().for_each(|queue| {
            let queue_family_index = physical_device.queue_families.iter().enumerate().find_map(
                |(index, properties)| {
                    // Check if queue family has all the required types requested.
                    if !queue.required_types().iter().all(|flag| {
                        *flag == QueueType::Present || properties.queue_flags.contains(flag.to_vk())
                    }) {
                        return None;
                    }

                    // Check if queue family supports surface if present is requested.
                    if queue.required_types().contains(&QueueType::Present) {
                        let supports_surface = unsafe {
                            surface_loader
                                .get_physical_device_surface_support(
                                    physical_device.physical_device,
                                    index as u32,
                                    surface,
                                )
                                .expect("Could not get physical device surface support.")
                        };

                        if !supports_surface {
                            return None;
                        }
                    }

                    // Check if the queue family has enough capacity for the queue.
                    if queue_family_indices.contains_key(&(index as u32))
                        && queue_family_indices.get(&(index as u32)).unwrap().len()
                            >= properties.queue_count as usize
                    {
                        return None;
                    }

                    Some(index as u32)
                },
            );

            // Add queue to queue family.
            if let Some(queue_family_index) = queue_family_index {
                if queue_family_indices.contains_key(&queue_family_index) {
                    queue_family_indices
                        .get_mut(&queue_family_index)
                        .unwrap()
                        .push(queue.clone());
                } else {
                    queue_family_indices.insert(queue_family_index, vec![queue.clone()]);
                }
            } else {
                // Queue family isn't found so toss queue out.
                println!("Queue family not found for queue: {}", queue.queue_name());
            }
        });

        let mut queue_priorities = Vec::new();

        let queue_create_infos = queue_family_indices
            .iter()
            .map(|(index, queues)| {
                queue_priorities.push(
                    queues
                        .iter()
                        .map(|queue| queue.priority)
                        .collect::<Vec<_>>(),
                );
                vk::DeviceQueueCreateInfo::builder()
                    .queue_family_index(*index)
                    .queue_priorities(queue_priorities.get(queue_priorities.len() - 1).unwrap())
                    .build()
            })
            .collect::<Vec<_>>();

        let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions);

        let device = unsafe {
            instance.create_device(physical_device.physical_device, &device_create_info, None)
        }
        .unwrap();

        let queues = queue_family_indices
            .iter()
            .map(|(index, queues)| {
                queues
                    .iter()
                    .enumerate()
                    .map(|(i, queue)| {
                        let vk_queue = unsafe { device.get_device_queue(*index, i as u32) };
                        (
                            queue.name,
                            Queue {
                                queue_family_index: *index,
                                queue: vk_queue,
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<HashMap<_, _>>();

        (device, queues)
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

    pub fn queue(mut self, queue: &'static QueueConfig) -> Self {
        self.queues.push(queue);
        self
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum QueueType {
    Graphics = 0b00000001,
    Compute = 0b00000010,
    Transfer = 0b00000100,
    Present = 0b00001000,
}

impl QueueType {
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn to_vk(&self) -> vk::QueueFlags {
        match self {
            Self::Graphics => vk::QueueFlags::GRAPHICS,
            Self::Compute => vk::QueueFlags::COMPUTE,
            Self::Transfer => vk::QueueFlags::TRANSFER,
            Self::Present => vk::QueueFlags::empty(),
        }
    }
}

#[derive(Clone)]
pub struct QueueConfig {
    /// The queue name, used as an identifier for the queue.
    name: QueueName,

    /// The priority of the queue. This is a value between 0.0 and 1.0.
    priority: f32,

    /// The command types that the queue needs.
    ///
    /// This is used to determine which queue families are suitable for the queue,
    /// however this functionality is not supported yet and only one queue family is used for now.
    _required_types: &'static [QueueType],
}

impl QueueConfig {
    pub const fn new(name: QueueName, priority: f32, required_types: &'static [QueueType]) -> Self {
        Self {
            name,
            priority,
            _required_types: required_types,
        }
    }

    pub fn queue_name(&self) -> &'static str {
        self.name
    }

    pub fn priority(&self) -> f32 {
        self.priority
    }

    pub fn required_types(&self) -> &'static [QueueType] {
        self._required_types
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
