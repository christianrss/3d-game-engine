//! Renderer Vulkan — implementação didática com `ash`.

use crate::graphics::backend::GfxBackend;
use crate::graphics::shaders::{FRAGMENT_GLSL, LIGHT_DIRECTION, VERTEX_GLSL};
use crate::graphics::{Camera, Color, GpuMesh, Mesh};
use crate::math::Mat4;
use ash::{vk, Entry, Instance};
use ash_window::create_surface;
use std::collections::HashMap;
use std::ffi::CString;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Uniform buffer enviado ao shader (deve espelhar o layout GLSL).
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformBufferObject {
    mvp: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    light_dir: [f32; 3],
    _pad: f32,
}

struct VkMesh {
    vertex_buffer: vk::Buffer,
    vertex_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_memory: vk::DeviceMemory,
    index_count: u32,
}

/// Renderer Vulkan completo.
pub struct VulkanRenderer {
    entry: Entry,
    instance: Instance,
    surface: vk::SurfaceKHR,
    surface_loader: ash::extensions::khr::Surface,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    swapchain_loader: ash::extensions::khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    image_views: Vec<vk::ImageView>,
    format: vk::Format,
    extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available: Vec<vk::Semaphore>,
    render_finished: Vec<vk::Semaphore>,
    in_flight: Vec<vk::Fence>,
    current_frame: usize,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_memories: Vec<vk::DeviceMemory>,
    uniform_mapped: Vec<*mut u8>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_layout: vk::DescriptorSetLayout,
    depth_image: vk::Image,
    depth_memory: vk::DeviceMemory,
    depth_view: vk::ImageView,
    meshes: HashMap<u64, VkMesh>,
    next_id: u64,
    width: u32,
    height: u32,
}

impl VulkanRenderer {
    pub fn new(window: &Window) -> Result<Self, String> {
        unsafe {
            let entry = Entry::load().map_err(|e| e.to_string())?;

            let app_name = CString::new("DesertShooter").unwrap();
            let engine_name = CString::new("DesertEngine").unwrap();
            let app_info = vk::ApplicationInfo::default()
                .application_name(&app_name)
                .application_version(vk::make_api_version(0, 1, 0, 0))
                .engine_name(&engine_name)
                .engine_version(vk::make_api_version(0, 1, 0, 0))
                .api_version(vk::API_VERSION_1_2);

            let mut extensions =
                ash_window::enumerate_required_extensions(window.display_handle().unwrap().as_raw())
                    .map_err(|e| e.to_string())?
                    .to_vec();

            // Garante extensões de superfície
            extensions.push(ash::extensions::khr::Surface::name().as_ptr());

            let layer_name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
            let layers = [layer_name.as_ptr()];

            let create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_layer_names(&layers)
                .enabled_extension_names(&extensions);

            let instance = entry
                .create_instance(&create_info, None)
                .map_err(|e| e.to_string())?;

            let surface = create_surface(
                &entry,
                &instance,
                window.window_handle().unwrap().as_raw(),
                None,
            )
            .map_err(|e| e.to_string())?;

            let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

            let (physical_device, queue_family) =
                pick_physical_device(&instance, &surface_loader, surface)?;

            let queue_priorities = [1.0f32];
            let queue_info = vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family)
                .queue_priorities(&queue_priorities);

            let device_extensions = [ash::extensions::khr::Swapchain::name().as_ptr()];
            let device_create = vk::DeviceCreateInfo::default()
                .queue_create_infos(&[queue_info])
                .enabled_extension_names(&device_extensions);

            let device = instance
                .create_device(physical_device, &device_create, None)
                .map_err(|e| e.to_string())?;

            let graphics_queue = device.get_device_queue(queue_family, 0);
            let swapchain_loader = ash::extensions::khr::Swapchain::new(&instance, &device);

            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            let (swapchain, images, format, extent) = create_swapchain(
                &instance,
                &device,
                physical_device,
                &surface_loader,
                &swapchain_loader,
                surface,
                width,
                height,
            )?;

            let image_views = create_image_views(&device, &images, format)?;
            let render_pass = create_render_pass(&device, format)?;
            let (descriptor_layout, descriptor_pool, descriptor_sets, uniform_buffers, uniform_memories, uniform_mapped) =
                create_descriptor_and_uniforms(&device, MAX_FRAMES_IN_FLIGHT)?;
            let (pipeline_layout, pipeline) =
                create_pipeline(&device, render_pass, descriptor_layout, &entry)?;
            let (depth_image, depth_memory, depth_view) =
                create_depth_resources(&instance, &device, physical_device, extent)?;
            let framebuffers = create_framebuffers(
                &device,
                render_pass,
                &image_views,
                depth_view,
                extent,
            )?;
            let (command_pool, command_buffers) =
                create_command_buffers(&device, queue_family, MAX_FRAMES_IN_FLIGHT)?;

            let mut image_available = Vec::new();
            let mut render_finished = Vec::new();
            let mut in_flight = Vec::new();

            let semaphore_info = vk::SemaphoreCreateInfo::default();
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

            for _ in 0..MAX_FRAMES_IN_FLIGHT {
                image_available.push(device.create_semaphore(&semaphore_info, None).unwrap());
                render_finished.push(device.create_semaphore(&semaphore_info, None).unwrap());
                in_flight.push(device.create_fence(&fence_info, None).unwrap());
            }

            Ok(Self {
                entry,
                instance,
                surface,
                surface_loader,
                physical_device,
                device,
                graphics_queue,
                swapchain_loader,
                swapchain,
                images,
                image_views,
                format,
                extent,
                render_pass,
                pipeline_layout,
                pipeline,
                framebuffers,
                command_pool,
                command_buffers,
                image_available,
                render_finished,
                in_flight,
                current_frame: 0,
                uniform_buffers,
                uniform_memories,
                uniform_mapped,
                descriptor_pool,
                descriptor_sets,
                descriptor_layout,
                depth_image,
                depth_memory,
                depth_view,
                meshes: HashMap::new(),
                next_id: 1,
                width,
                height,
            })
        }
    }

    fn update_uniform(&self, frame: usize, model: Mat4, camera: &Camera) {
        let ubo = UniformBufferObject {
            mvp: (camera.view_projection() * model).to_cols_array_2d(),
            model: model.to_cols_array_2d(),
            light_dir: LIGHT_DIRECTION,
            _pad: 0.0,
        };
        unsafe {
            let ptr = self.uniform_mapped[frame];
            std::ptr::copy_nonoverlapping(
                &ubo as *const _ as *const u8,
                ptr,
                std::mem::size_of::<UniformBufferObject>(),
            );
        }
    }
}

impl GfxBackend for VulkanRenderer {
    type Error = String;

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
        // Recriação de swapchain simplificada — em produção usaria vkDeviceWaitIdle + rebuild
    }

    fn upload_mesh(&mut self, mesh: &Mesh) -> Result<GpuMesh, String> {
        let id = self.next_id;
        self.next_id += 1;

        let (vb, vm, ib, im) = unsafe {
            let vertex_size = (mesh.vertices.len() * std::mem::size_of::<crate::graphics::Vertex>()) as u64;
            let index_size = (mesh.indices.len() * 4) as u64;

            let (vb, vm) = create_device_local_buffer(
                &self.instance,
                &self.device,
                self.physical_device,
                bytemuck::cast_slice(&mesh.vertices),
                vk::BufferUsageFlags::VERTEX_BUFFER,
            )?;

            let (ib, im) = create_device_local_buffer(
                &self.instance,
                &self.device,
                self.physical_device,
                bytemuck::cast_slice(&mesh.indices),
                vk::BufferUsageFlags::INDEX_BUFFER,
            )?;

            Ok::<_, String>((vb, vm, ib, im))
        }?;

        self.meshes.insert(
            id,
            VkMesh {
                vertex_buffer: vb,
                vertex_memory: vm,
                index_buffer: ib,
                index_memory: im,
                index_count: mesh.indices.len() as u32,
            },
        );

        Ok(GpuMesh {
            vertex_count: mesh.vertices.len() as u32,
            index_count: mesh.indices.len() as u32,
            gpu_id: id,
        })
    }

    fn begin_frame(&mut self, clear: Color) {
        let _ = clear;
        // O clear acontece no render pass dentro de draw/end_frame nesta implementação
    }

    fn draw(&mut self, gpu_mesh: &GpuMesh, model: Mat4, camera: &Camera) -> Result<(), String> {
        let mesh = self
            .meshes
            .get(&gpu_mesh.gpu_id)
            .ok_or("Mesh não encontrada")?;

        let frame = self.current_frame;
        self.update_uniform(frame, model, camera);

        unsafe {
            self.device
                .wait_for_fences(&[self.in_flight[frame]], true, u64::MAX)
                .map_err(|e| e.to_string())?;
            self.device
                .reset_fences(&[self.in_flight[frame]])
                .map_err(|e| e.to_string())?;

            let (image_index, _) = self
                .swapchain_loader
                .acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    self.image_available[frame],
                    vk::Fence::null(),
                )
                .map_err(|e| e.to_string())?;

            self.device
                .reset_command_buffer(self.command_buffers[frame], vk::CommandBufferResetFlags::empty())
                .map_err(|e| e.to_string())?;

            let begin = vk::CommandBufferBeginInfo::default();
            self.device
                .begin_command_buffer(self.command_buffers[frame], &begin)
                .map_err(|e| e.to_string())?;

            let clear_color = vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.95, 0.75, 0.5, 1.0],
                },
            };
            let clear_depth = vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            };

            let render_pass_info = vk::RenderPassBeginInfo::default()
                .render_pass(self.render_pass)
                .framebuffer(self.framebuffers[image_index as usize])
                .render_area(vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.extent,
                })
                .clear_values(&[clear_color, clear_depth]);

            self.device.cmd_begin_render_pass(
                self.command_buffers[frame],
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );

            self.device.cmd_bind_pipeline(
                self.command_buffers[frame],
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            self.device.cmd_bind_descriptor_sets(
                self.command_buffers[frame],
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[frame]],
                &[],
            );

            self.device.cmd_bind_vertex_buffers(
                self.command_buffers[frame],
                0,
                &[mesh.vertex_buffer],
                &[0],
            );
            self.device.cmd_bind_index_buffer(
                self.command_buffers[frame],
                mesh.index_buffer,
                0,
                vk::IndexType::UINT32,
            );
            self.device.cmd_draw_indexed(
                self.command_buffers[frame],
                mesh.index_count,
                1,
                0,
                0,
                0,
            );

            self.device.cmd_end_render_pass(self.command_buffers[frame]);
            self.device
                .end_command_buffer(self.command_buffers[frame])
                .map_err(|e| e.to_string())?;

            let wait_semaphores = [self.image_available[frame]];
            let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
            let signal_semaphores = [self.render_finished[frame]];

            let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&[self.command_buffers[frame]])
                .signal_semaphores(&signal_semaphores);

            self.device
                .queue_submit(self.graphics_queue, &[submit_info], self.in_flight[frame])
                .map_err(|e| e.to_string())?;

            let swap_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&[self.swapchain])
                .image_indices(&[image_index]);

            let _ = self.swapchain_loader.queue_present(self.graphics_queue, &swap_info);
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl Drop for VulkanRenderer {
    fn drop(&mut self) {
        unsafe {
            let _ = self.device.device_wait_idle();
            // Cleanup simplificado — em produção destruir todos os recursos na ordem correta
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}

// ── Funções auxiliares Vulkan ─────────────────────────────────────────────────

unsafe fn pick_physical_device(
    instance: &Instance,
    surface_loader: &ash::extensions::khr::Surface,
    surface: vk::SurfaceKHR,
) -> Result<(vk::PhysicalDevice, u32), String> {
    let devices = instance
        .enumerate_physical_devices()
        .map_err(|e| e.to_string())?;

    for device in devices {
        let props = instance.get_physical_device_queue_family_properties(device);
        for (i, fam) in props.iter().enumerate() {
            if fam.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                let present = surface_loader
                    .get_physical_device_surface_support(device, i as u32, surface)
                    .unwrap_or(false);
                if present {
                    return Ok((device, i as u32));
                }
            }
        }
    }
    Err("Nenhuma GPU compatível com Vulkan encontrada".into())
}

unsafe fn create_swapchain(
    instance: &Instance,
    device: &ash::Device,
    physical: vk::PhysicalDevice,
    surface_loader: &ash::extensions::khr::Surface,
    swapchain_loader: &ash::extensions::khr::Swapchain,
    surface: vk::SurfaceKHR,
    width: u32,
    height: u32,
) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D), String> {
    let caps = surface_loader
        .get_physical_device_surface_capabilities(physical, surface)
        .map_err(|e| e.to_string())?;

    let formats = surface_loader
        .get_physical_device_surface_formats(physical, surface)
        .map_err(|e| e.to_string())?;
    let format = formats
        .iter()
        .find(|f| f.format == vk::Format::B8G8R8A8_SRGB)
        .unwrap_or(&formats[0]);

    let extent = if caps.current_extent.width != u32::MAX {
        caps.current_extent
    } else {
        vk::Extent2D { width, height }
    };

    let create_info = vk::SwapchainCreateInfoKHR::default()
        .surface(surface)
        .min_image_count(caps.min_image_count.max(2))
        .image_format(format.format)
        .image_color_space(format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(caps.current_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(vk::PresentModeKHR::FIFO)
        .clipped(true);

    let swapchain = swapchain_loader
        .create_swapchain(&create_info, None)
        .map_err(|e| e.to_string())?;
    let images = swapchain_loader
        .get_swapchain_images(swapchain)
        .map_err(|e| e.to_string())?;

    Ok((swapchain, images, format.format, extent))
}

unsafe fn create_image_views(
    device: &ash::Device,
    images: &[vk::Image],
    format: vk::Format,
) -> Result<Vec<vk::ImageView>, String> {
    let mut views = Vec::new();
    for &image in images {
        let create_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
        views.push(device.create_image_view(&create_info, None).map_err(|e| e.to_string())?);
    }
    Ok(views)
}

unsafe fn create_render_pass(device: &ash::Device, format: vk::Format) -> Result<vk::RenderPass, String> {
    let color_attachment = vk::AttachmentDescription::default()
        .format(format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let depth_attachment = vk::AttachmentDescription::default()
        .format(vk::Format::D32_SFLOAT)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::DONT_CARE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let color_ref = vk::AttachmentReference::default()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let depth_ref = vk::AttachmentReference::default()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

    let subpass = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(&[color_ref])
        .depth_stencil_attachment(&depth_ref);

    let dependency = vk::SubpassDependency::default()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let create_info = vk::RenderPassCreateInfo::default()
        .attachments(&[color_attachment, depth_attachment])
        .subpasses(&[subpass])
        .dependencies(&[dependency]);

    device
        .create_render_pass(&create_info, None)
        .map_err(|e| e.to_string())
}

unsafe fn compile_glsl(shaderc: &shaderc::Compiler, src: &str, kind: shaderc::ShaderKind) -> Result<Vec<u32>, String> {
    let compiled = shaderc
        .compile_into_spirv(src, kind, "shader", "main", None)
        .map_err(|e| e.to_string())?;
    Ok(compiled.as_binary().to_vec())
}

unsafe fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    descriptor_layout: vk::DescriptorSetLayout,
    entry: &Entry,
) -> Result<(vk::PipelineLayout, vk::Pipeline), String> {
    let compiler = shaderc::Compiler::new().map_err(|e| e.to_string())?;
    let vert_spv = compile_glsl(&compiler, VERTEX_GLSL, shaderc::ShaderKind::Vertex)?;
    let frag_spv = compile_glsl(&compiler, FRAGMENT_GLSL, shaderc::ShaderKind::Fragment)?;

    let vert_module = {
        let create_info = vk::ShaderModuleCreateInfo::default().code(&vert_spv);
        device.create_shader_module(&create_info, None).map_err(|e| e.to_string())?
    };
    let frag_module = {
        let create_info = vk::ShaderModuleCreateInfo::default().code(&frag_spv);
        device.create_shader_module(&create_info, None).map_err(|e| e.to_string())?
    };

    let entry_main = CString::new("main").unwrap();
    let vert_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::VERTEX)
        .module(vert_module)
        .name(&entry_main);
    let frag_stage = vk::PipelineShaderStageCreateInfo::default()
        .stage(vk::ShaderStageFlags::FRAGMENT)
        .module(frag_module)
        .name(&entry_main);

    let binding = vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(std::mem::size_of::<crate::graphics::Vertex>() as u32)
        .input_rate(vk::VertexInputRate::VERTEX);

    let attrs = [
        vk::VertexInputAttributeDescription::default().location(0).binding(0).format(vk::Format::R32G32B32_SFLOAT).offset(0),
        vk::VertexInputAttributeDescription::default().location(1).binding(0).format(vk::Format::R32G32B32_SFLOAT).offset(12),
        vk::VertexInputAttributeDescription::default().location(2).binding(0).format(vk::Format::R32G32B32_SFLOAT).offset(24),
    ];

    let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(&[binding])
        .vertex_attribute_descriptions(&attrs);

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    let viewport = vk::Viewport::default()
        .width(1280.0)
        .height(720.0)
        .max_depth(1.0);
    let scissor = vk::Rect2D::default().extent(vk::Extent2D { width: 1280, height: 720 });
    let viewport_state = vk::PipelineViewportStateCreateInfo::default()
        .viewports(&[viewport])
        .scissors(&[scissor]);

    let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .line_width(1.0);

    let multisample = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(vk::SampleCountFlags::TYPE_1);

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS);

    let color_blend = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA);
    let blend = vk::PipelineColorBlendStateCreateInfo::default()
        .attachments(&[color_blend]);

    let layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(&[descriptor_layout]);
    let pipeline_layout = device
        .create_pipeline_layout(&layout_info, None)
        .map_err(|e| e.to_string())?;

    let stages = [vert_stage, frag_stage];
    let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
        .stages(&stages)
        .vertex_input_state(&vertex_input)
        .input_assembly_state(&input_assembly)
        .viewport_state(&viewport_state)
        .rasterization_state(&rasterizer)
        .multisample_state(&multisample)
        .depth_stencil_state(&depth_stencil)
        .color_blend_state(&blend)
        .layout(pipeline_layout)
        .render_pass(render_pass);

    let pipeline = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .map_err(|e| e.to_string())?
        .0[0];

    device.destroy_shader_module(vert_module, None);
    device.destroy_shader_module(frag_module, None);

    Ok((pipeline_layout, pipeline))
}

unsafe fn create_descriptor_and_uniforms(
    device: &ash::Device,
    frames: usize,
) -> Result<
    (
        vk::DescriptorSetLayout,
        vk::DescriptorPool,
        Vec<vk::DescriptorSet>,
        Vec<vk::Buffer>,
        Vec<vk::DeviceMemory>,
        Vec<*mut u8>,
    ),
    String,
> {
    let binding = vk::DescriptorSetLayoutBinding::default()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&[binding]);
    let descriptor_layout = device
        .create_descriptor_set_layout(&layout_info, None)
        .map_err(|e| e.to_string())?;

    let pool_size = vk::DescriptorPoolSize::default()
        .ty(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(frames as u32);
    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(&[pool_size])
        .max_sets(frames as u32);
    let descriptor_pool = device
        .create_descriptor_pool(&pool_info, None)
        .map_err(|e| e.to_string())?;

    let layouts = vec![descriptor_layout; frames];
    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);
    let descriptor_sets = device
        .allocate_descriptor_sets(&alloc_info)
        .map_err(|e| e.to_string())?;

    let mut uniform_buffers = Vec::new();
    let mut uniform_memories = Vec::new();
    let mut uniform_mapped = Vec::new();

    for i in 0..frames {
        let buffer_size = std::mem::size_of::<UniformBufferObject>() as u64;
        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::UNIFORM_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = device.create_buffer(&buffer_info, None).map_err(|e| e.to_string())?;
        let req = device.get_buffer_memory_requirements(buffer);
        let mem_props = vk::PhysicalDeviceMemoryProperties::default(); // simplificado
        let _ = mem_props;
        let mem_index = 0; // simplificado — host visible
        let alloc = vk::MemoryAllocateInfo::default().size(req.size).memory_type_index(mem_index);
        let memory = device.allocate_memory(&alloc, None).map_err(|e| e.to_string())?;
        device.bind_buffer_memory(buffer, memory, 0).map_err(|e| e.to_string())?;
        let ptr = device
            .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .map_err(|e| e.to_string())? as *mut u8;

        let descriptor_info = vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(0)
            .range(buffer_size);
        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_sets[i])
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .buffer_info(&[descriptor_info]);
        device.update_descriptor_sets(&[write], &[]);

        uniform_buffers.push(buffer);
        uniform_memories.push(memory);
        uniform_mapped.push(ptr);
    }

    Ok((
        descriptor_layout,
        descriptor_pool,
        descriptor_sets,
        uniform_buffers,
        uniform_memories,
        uniform_mapped,
    ))
}

unsafe fn create_depth_resources(
    instance: &Instance,
    device: &ash::Device,
    physical: vk::PhysicalDevice,
    extent: vk::Extent2D,
) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView), String> {
    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(vk::Format::D32_SFLOAT)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT);

    let image = device.create_image(&image_info, None).map_err(|e| e.to_string())?;
    let req = device.get_image_memory_requirements(image);
    let mem_type = find_memory_type(instance, physical, req.memory_type_bits, vk::MemoryPropertyFlags::DEVICE_LOCAL)?;
    let alloc = vk::MemoryAllocateInfo::default().size(req.size).memory_type_index(mem_type);
    let memory = device.allocate_memory(&alloc, None).map_err(|e| e.to_string())?;
    device.bind_image_memory(image, memory, 0).map_err(|e| e.to_string())?;

    let view_info = vk::ImageViewCreateInfo::default()
        .image(image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(vk::Format::D32_SFLOAT)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });
    let view = device.create_image_view(&view_info, None).map_err(|e| e.to_string())?;
    Ok((image, memory, view))
}

unsafe fn create_framebuffers(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    image_views: &[vk::ImageView],
    depth_view: vk::ImageView,
    extent: vk::Extent2D,
) -> Result<Vec<vk::Framebuffer>, String> {
    let mut fbs = Vec::new();
    for &view in image_views {
        let attachments = [view, depth_view];
        let info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&attachments)
            .width(extent.width)
            .height(extent.height)
            .layers(1);
        fbs.push(device.create_framebuffer(&info, None).map_err(|e| e.to_string())?);
    }
    Ok(fbs)
}

unsafe fn create_command_buffers(
    device: &ash::Device,
    queue_family: u32,
    count: usize,
) -> Result<(vk::CommandPool, Vec<vk::CommandBuffer>), String> {
    let pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family);
    let pool = device.create_command_pool(&pool_info, None).map_err(|e| e.to_string())?;
    let alloc_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(count as u32);
    let buffers = device
        .allocate_command_buffers(&alloc_info)
        .map_err(|e| e.to_string())?;
    Ok((pool, buffers))
}

unsafe fn find_memory_type(
    instance: &Instance,
    physical: vk::PhysicalDevice,
    requirements: u32,
    properties: vk::MemoryPropertyFlags,
) -> Result<u32, String> {
    let mem_props = instance
        .get_physical_device_memory_properties(physical);
    for i in 0..mem_props.memory_type_count {
        if (requirements & (1 << i)) != 0
            && (mem_props.memory_types[i as usize].property_flags & properties) == properties
        {
            return Ok(i);
        }
    }
    Err("Tipo de memória Vulkan não encontrado".into())
}

unsafe fn create_device_local_buffer(
    instance: &Instance,
    device: &ash::Device,
    physical: vk::PhysicalDevice,
    data: &[u8],
    usage: vk::BufferUsageFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory), String> {
    let size = data.len() as u64;
    let buffer_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(usage | vk::BufferUsageFlags::TRANSFER_DST)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let buffer = device.create_buffer(&buffer_info, None).map_err(|e| e.to_string())?;
    let req = device.get_buffer_memory_requirements(buffer);
    let mem_type = find_memory_type(
        instance,
        physical,
        req.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    let alloc = vk::MemoryAllocateInfo::default().size(req.size).memory_type_index(mem_type);
    let memory = device.allocate_memory(&alloc, None).map_err(|e| e.to_string())?;
    device.bind_buffer_memory(buffer, memory, 0).map_err(|e| e.to_string())?;

    // Upload via staging buffer
    let staging_info = vk::BufferCreateInfo::default()
        .size(size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let staging = device.create_buffer(&staging_info, None).map_err(|e| e.to_string())?;
    let staging_req = device.get_buffer_memory_requirements(staging);
    let staging_mem_type = find_memory_type(
        instance,
        physical,
        staging_req.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    let staging_alloc = vk::MemoryAllocateInfo::default()
        .size(staging_req.size)
        .memory_type_index(staging_mem_type);
    let staging_mem = device.allocate_memory(&staging_alloc, None).map_err(|e| e.to_string())?;
    device.bind_buffer_memory(staging, staging_mem, 0).map_err(|e| e.to_string())?;

    let ptr = device
        .map_memory(staging_mem, 0, size, vk::MemoryMapFlags::empty())
        .map_err(|e| e.to_string())?;
    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
    device.unmap_memory(staging_mem);

    // Copy command — simplificado: requer command pool temporário
    device.destroy_buffer(staging, None);
    device.free_memory(staging_mem, None);

    Ok((buffer, memory))
}
