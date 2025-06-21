use std::sync::{Arc, Mutex};

use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage}, command_buffer::{allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo}, AutoCommandBufferBuilder, CommandBufferUsage}, descriptor_set::{allocator::{StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo}, DescriptorSet, WriteDescriptorSet}, device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo, QueueFlags}, format::Format, image::{view::ImageView, Image, ImageCreateInfo, ImageType, ImageUsage}, instance::{Instance, InstanceCreateFlags, InstanceCreateInfo, InstanceExtensions}, memory::allocator::{AllocationCreateInfo, FreeListAllocator, GenericMemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator}, pipeline::{compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo, ComputePipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo}, render_pass::{Framebuffer, FramebufferCreateInfo}, swapchain::{self, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo}, sync::{self, GpuFuture}, Validated, VulkanError, VulkanLibrary};
use winit::{application::ApplicationHandler, event::WindowEvent, window::Window};

use crate::render::{decoder_shader::TriTask, primitives::Tri};

pub mod primitives;

pub struct State {
    pub window: Option<Arc<Window>>,
    pub renderer: Renderer,
    pub minimized: bool,
}

impl ApplicationHandler for State {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Some(Arc::new(event_loop.create_window(Window::default_attributes()).unwrap()));
        self.renderer.setup_swapchain(window.clone().unwrap().clone());
        self.window = window;
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested if !event_loop.exiting() && !self.minimized => self.renderer.render(),
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    self.minimized = true;
                } else {
                    self.minimized = false;
                    self.renderer.recreate_swapchain = true;
                }
            }
            _ => {}
        }
    }

    fn user_event(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop, _event: ()) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

pub struct Renderer {
    queue: Arc<Queue>,
    memory_allocator: Arc<GenericMemoryAllocator<FreeListAllocator>>,
    descriptor_allocator: Arc<StandardDescriptorSetAllocator>,
    command_allocator: Arc<StandardCommandBufferAllocator>,
    decoder_pipeline: Arc<ComputePipeline>,
    raster_pipeline: Arc<ComputePipeline>,

    tris: Arc<Mutex<Vec<Tri>>>,
    display_range: Arc<Mutex<((u32, u32), (u32, u32))>>,

    recreate_swapchain: bool,

    surface: Option<Arc<Surface>>,
    swapchain: Option<Arc<Swapchain>>,
    swapchain_images: Option<Vec<Arc<Image>>>,
    render_pass: Option<Arc<vulkano::render_pass::RenderPass>>,
    framebuffers: Option<Vec<Arc<Framebuffer>>>,
}

impl Renderer {
    pub fn new(tris: Arc<Mutex<Vec<Tri>>>, display_range: Arc<Mutex<((u32, u32), (u32, u32))>>) -> Self {
        println!("renderer Tri ptr = {:p}", Arc::as_ptr(&tris));
        let library = VulkanLibrary::new().expect("no local Vulkan library/DLL");
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: InstanceExtensions {
                    khr_win32_surface: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        ).expect("failed to create Vulkan instance");

        let physical_device = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .next()
            .expect("no devices available");

        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)| {
                queue_family_properties.queue_flags.contains(QueueFlags::COMPUTE) &&
                queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
            }).expect("couldn't find a queue family");

        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![
                    QueueCreateInfo {
                        queue_family_index: queue_family_index as u32,
                        ..Default::default()
                    },
                ],
                enabled_extensions: DeviceExtensions {
                    khr_16bit_storage: true,
                    khr_swapchain: true,
                    ..Default::default()
                },
                enabled_features: DeviceFeatures {
                    storage_buffer16_bit_access: true,
                    storage_input_output16: true,
                    uniform_and_storage_buffer16_bit_access: true,
                    shader_int16: true,
                    ..Default::default()
                },
                ..Default::default()
            }
        ).expect("failed to create Vulkan device");

        let queue = queues.next().unwrap();

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let descriptor_allocator = Arc::new(StandardDescriptorSetAllocator::new(device.clone(), StandardDescriptorSetAllocatorCreateInfo::default()));
        let command_allocator = Arc::new(StandardCommandBufferAllocator::new(device.clone(), StandardCommandBufferAllocatorCreateInfo::default()));

        let reorder = decoder_shader::load(device.clone()).expect("failed to create shader module");
        let reorder_entry = reorder.entry_point("main").unwrap();
        let reorder_stage = PipelineShaderStageCreateInfo::new(reorder_entry);

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&reorder_stage])
                .into_pipeline_layout_create_info(device.clone())
                .unwrap()
        ).unwrap();

        let decoder_pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(reorder_stage, layout)
        ).expect("failed to create compute pipeline");

        let raster = raster_shader::load(device.clone()).expect("failed to create shader module");
        let raster_entry = raster.entry_point("main").unwrap();
        let raster_stage = PipelineShaderStageCreateInfo::new(raster_entry);

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages([&raster_stage])
                .into_pipeline_layout_create_info(device.clone())
                .unwrap()
        ).unwrap();

        let raster_pipeline = ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(raster_stage, layout)
        ).expect("failed to create compute pipeline");

        Self {
            queue,
            memory_allocator,
            descriptor_allocator,
            command_allocator,
            decoder_pipeline,
            raster_pipeline,

            tris,
            display_range,

            recreate_swapchain: false,

            surface: None,
            swapchain: None,
            swapchain_images: None,
            render_pass: None,
            framebuffers: None,
        }
    }

    fn setup_swapchain(&mut self, window: Arc<Window>) {
        let surface = Surface::from_window(self.queue.device().instance().clone(), window).unwrap();

        let (swapchain, images) = Swapchain::new(
            self.queue.device().clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: 2,
                image_format: Format::B8G8R8A8_UNORM,
                image_extent: [640, 480],
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                present_mode: PresentMode::Fifo,
                ..Default::default()
            },
        ).unwrap();

        let render_pass = vulkano::single_pass_renderpass!(
            self.queue.device().clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        ).unwrap();

        let framebuffers = images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone()).unwrap();
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                ).unwrap()
            })
            .collect::<Vec<_>>();

        self.surface = Some(surface);
        self.swapchain = Some(swapchain);
        self.swapchain_images = Some(images);
        self.render_pass = Some(render_pass);
        self.framebuffers = Some(framebuffers);
    }

    pub fn render(&mut self) {
        let num_tris = self.tris.lock().unwrap().len();
        if num_tris == 0 {return}
        println!("Redraw!");

        let tris_buf = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            self.tris.lock().unwrap().iter().cloned()
        ).unwrap();

        let ((min_x, max_x), (min_y, max_y)) = *self.display_range.lock().unwrap();
        let size = [max_x - min_x, max_y - min_y];

        let output_img = Image::new(
            self.memory_allocator.clone(),
            ImageCreateInfo {
                image_type: ImageType::Dim2d,
                format: Format::R8G8B8A8_UNORM,
                extent: [640, 480, 1],
                usage: ImageUsage::TRANSFER_DST | ImageUsage::STORAGE,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
        ).unwrap();
        let output_view = ImageView::new_default(output_img.clone()).unwrap();

        let draw_area_buf = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            [min_x as i32, min_y as i32, max_x as i32, max_y as i32]
        ).unwrap();

        let task_buf = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            std::iter::repeat(0u8).take(num_tris * size_of::<TriTask>())
        ).unwrap();

        let counter_buf = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            0u32
        ).unwrap();

        let decoder_set = DescriptorSet::new(
            self.descriptor_allocator.clone(),
            self.decoder_pipeline.layout().set_layouts()[0].clone(),
            [
                WriteDescriptorSet::buffer(0, tris_buf.clone()),
                WriteDescriptorSet::buffer(1, draw_area_buf.clone()),
                WriteDescriptorSet::buffer(3, task_buf.clone()),
                WriteDescriptorSet::buffer(4, counter_buf.clone()),
            ],
            None
        ).unwrap();

        let raster_set = DescriptorSet::new(
            self.descriptor_allocator.clone(),
            self.raster_pipeline.layout().set_layouts()[0].clone(),
            [
                WriteDescriptorSet::image_view(2, output_view.clone()),
                WriteDescriptorSet::buffer(3, task_buf.clone()),
                WriteDescriptorSet::buffer(4, counter_buf.clone()),
            ],
            None,
        ).unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        ).unwrap();

        let group_count = (((self.tris.lock().unwrap().len() as u32 as usize) + 63) / 64) as u32;
        unsafe { 
            builder
                .bind_pipeline_compute(self.decoder_pipeline.clone()).unwrap()
                .bind_descriptor_sets(PipelineBindPoint::Compute, self.decoder_pipeline.layout().clone(), 0, decoder_set.clone()).unwrap()
                .dispatch([group_count, 1, 1]).unwrap()
                .bind_pipeline_compute(self.raster_pipeline.clone()).unwrap()
                .bind_descriptor_sets(PipelineBindPoint::Compute, self.raster_pipeline.layout().clone(), 0, raster_set.clone()).unwrap()
                .dispatch([size[0] / 8, size[1] / 8, 1]).unwrap();
        }
        
        let command_buffer = builder.build().unwrap();
        let future = sync::now(self.queue.device().clone())
            .then_execute(self.queue.clone(), command_buffer.clone()).unwrap()
            .then_signal_fence_and_flush().unwrap();

        future.wait(None).unwrap();

        let swapchain = self.swapchain.as_mut().unwrap();
        let (image_i, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(swapchain.clone(), None)
            .map_err(Validated::unwrap)
        {
            Ok(r) => r,
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                return;
            }
            Err(e) => panic!("failed to acquire next image: {e}"),
        };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        let execution = sync::now(self.queue.device().clone())
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer.clone())
            .unwrap()
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(swapchain.clone(), image_i)
            ).then_signal_fence_and_flush();

        match execution.map_err(Validated::unwrap) {
            Ok(future) => {
                future.wait(None).unwrap();
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
            }
            Err(e) => {
                println!("failed to flush future: {e}");
            }
        }

        self.tris.lock().unwrap().clear();
    }
}

mod decoder_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        path: "src\\render\\shaders\\decoder.comp"
    }
}

mod raster_shader {
    vulkano_shaders::shader!{
        ty: "compute",
        path: "src\\render\\shaders\\raster.comp"
    }
}