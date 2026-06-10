//! Renderer Vulkan — implementação didática com `ash` 0.38.

use crate::graphics::backend::GfxBackend;
use crate::graphics::shaders::{FRAGMENT_GLSL, LIGHT_DIRECTION, VERTEX_GLSL};
use crate::graphics::{Camera, Color, GpuMesh, Mesh};
use crate::math::Mat4;
use ash::vk;
use ash::{Entry, Instance};
use ash_window::create_surface;
use std::collections::HashMap;
use std::ffi::CString;
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

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

pub struct VulkanRenderer {
    entry: Entry,
    instance: Instance,
    surface: vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    swapchain_loader: ash::khr::swapchain::Device,
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
            let app_info = vk::ApplicationInfo {
                p_application_name: app_name.as_ptr(),
                application_version: vk::make_api_version(0, 1, 0, 0),
                p_engine_name: engine_name.as_ptr(),
                engine_version: vk::make_api_version(0, 1, 0, 0),
                api_version: vk::API_VERSION_1_2,
                ..Default::default()
            };

            let display = window
                .display_handle()
                .map_err(|e| e.to_string())?
                .as_raw();
            let extensions = ash_window::enumerate_required_extensions(display)
                .map_err(|e| e.to_string())?
                .to_vec();

            let layer_name = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
            let layers = [layer_name.as_ptr()];

            let create_info = vk::InstanceCreateInfo {
                p_application_info: &app_info,
                enabled_layer_count: layers.len() as u32,
                pp_enabled_layer_names: layers.as_ptr(),
                enabled_extension_count: extensions.len() as u32,
                pp_enabled_extension_names: extensions.as_ptr(),
                ..Default::default()
            };

            let instance = entry
                .create_instance(&create_info, None)
                .map_err(|e| e.to_string())?;

            let win = window.window_handle().map_err(|e| e.to_string())?.as_raw();
            let surface = create_surface(&entry, &instance, display, win, None)
                .map_err(|e| e.to_string())?;

            let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
            let (physical_device, queue_family) =
                pick_physical_device(&instance, &surface_loader, surface)?;

            let queue_priorities = [1.0f32];
            let queue_info = vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family,
                queue_count: 1,
                p_queue_priorities: queue_priorities.as_ptr(),
                ..Default::default()
            };

            let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
            let device_create = vk::DeviceCreateInfo {
                queue_create_info_count: 1,
                p_queue_create_infos: &queue_info,
                enabled_extension_count: device_extensions.len() as u32,
                pp_enabled_extension_names: device_extensions.as_ptr(),
                ..Default::default()
            };

            let device = instance
                .create_device(physical_device, &device_create, None)
                .map_err(|e| e.to_string())?;

            let graphics_queue = device.get_device_queue(queue_family, 0);
            let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);

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
            let (
                descriptor_layout,
                descriptor_pool,
                descriptor_sets,
                uniform_buffers,
                uniform_memories,
                uniform_mapped,
            ) = create_descriptor_and_uniforms(&instance, &device, physical_device, MAX_FRAMES_IN_FLIGHT)?;
            let (pipeline_layout, pipeline) =
                create_pipeline(&device, render_pass, descriptor_layout, extent)?;
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

            let semaphore_info = vk::SemaphoreCreateInfo::default();
            let fence_info = vk::FenceCreateInfo {
                flags: vk::FenceCreateFlags::SIGNALED,
                ..Default::default()
            };

            let mut image_available = Vec::new();
            let mut render_finished = Vec::new();
            let mut in_flight = Vec::new();
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
            std::ptr::copy_nonoverlapping(
                &ubo as *const _ as *const u8,
                self.uniform_mapped[frame],
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
    }

    fn upload_mesh(&mut self, mesh: &Mesh) -> Result<GpuMesh, String> {
        let id = self.next_id;
        self.next_id += 1;

        let (vb, vm, ib, im) = unsafe {
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
            albedo_tex: None,
        })
    }

    fn begin_frame(&mut self, _clear: Color) {}

    fn upload_texture(
        &mut self,
        _data: &crate::graphics::TextureData,
    ) -> Result<crate::graphics::GpuTexture, String> {
        Ok(crate::graphics::GpuTexture {
            gpu_id: 0,
            width: 1,
            height: 1,
        })
    }

    fn draw(
        &mut self,
        gpu_mesh: &GpuMesh,
        model: Mat4,
        camera: &Camera,
        _material: crate::graphics::DrawMaterial,
    ) -> Result<(), String> {
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

            self.device
                .begin_command_buffer(self.command_buffers[frame], &vk::CommandBufferBeginInfo::default())
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
            let clears = [clear_color, clear_depth];

            let render_pass_info = vk::RenderPassBeginInfo {
                render_pass: self.render_pass,
                framebuffer: self.framebuffers[image_index as usize],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: self.extent,
                },
                clear_value_count: clears.len() as u32,
                p_clear_values: clears.as_ptr(),
                ..Default::default()
            };

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
            let cmd_bufs = [self.command_buffers[frame]];

            let submit_info = vk::SubmitInfo {
                wait_semaphore_count: wait_semaphores.len() as u32,
                p_wait_semaphores: wait_semaphores.as_ptr(),
                p_wait_dst_stage_mask: wait_stages.as_ptr(),
                command_buffer_count: cmd_bufs.len() as u32,
                p_command_buffers: cmd_bufs.as_ptr(),
                signal_semaphore_count: signal_semaphores.len() as u32,
                p_signal_semaphores: signal_semaphores.as_ptr(),
                ..Default::default()
            };

            self.device
                .queue_submit(self.graphics_queue, &[submit_info], self.in_flight[frame])
                .map_err(|e| e.to_string())?;

            let swapchains = [self.swapchain];
            let image_indices = [image_index];
            let present_info = vk::PresentInfoKHR {
                wait_semaphore_count: signal_semaphores.len() as u32,
                p_wait_semaphores: signal_semaphores.as_ptr(),
                swapchain_count: swapchains.len() as u32,
                p_swapchains: swapchains.as_ptr(),
                p_image_indices: image_indices.as_ptr(),
                ..Default::default()
            };

            let _ = self
                .swapchain_loader
                .queue_present(self.graphics_queue, &present_info);
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

unsafe fn pick_physical_device(
    instance: &Instance,
    surface_loader: &ash::khr::surface::Instance,
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
    surface_loader: &ash::khr::surface::Instance,
    swapchain_loader: &ash::khr::swapchain::Device,
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

    let create_info = vk::SwapchainCreateInfoKHR {
        surface,
        min_image_count: caps.min_image_count.max(2),
        image_format: format.format,
        image_color_space: format.color_space,
        image_extent: extent,
        image_array_layers: 1,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode: vk::SharingMode::EXCLUSIVE,
        pre_transform: caps.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode: vk::PresentModeKHR::FIFO,
        clipped: vk::TRUE,
        ..Default::default()
    };

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
        let create_info = vk::ImageViewCreateInfo {
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };
        views.push(
            device
                .create_image_view(&create_info, None)
                .map_err(|e| e.to_string())?,
        );
    }
    Ok(views)
}

unsafe fn create_render_pass(device: &ash::Device, format: vk::Format) -> Result<vk::RenderPass, String> {
    let color_attachment = vk::AttachmentDescription {
        format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        ..Default::default()
    };

    let depth_attachment = vk::AttachmentDescription {
        format: vk::Format::D32_SFLOAT,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::DONT_CARE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let color_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };
    let depth_ref = vk::AttachmentReference {
        attachment: 1,
        layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        ..Default::default()
    };

    let subpass = vk::SubpassDescription {
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        color_attachment_count: 1,
        p_color_attachments: &color_ref,
        p_depth_stencil_attachment: &depth_ref,
        ..Default::default()
    };

    let dependency = vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        ..Default::default()
    };

    let attachments = [color_attachment, depth_attachment];
    let subpasses = [subpass];
    let dependencies = [dependency];

    let create_info = vk::RenderPassCreateInfo {
        attachment_count: attachments.len() as u32,
        p_attachments: attachments.as_ptr(),
        subpass_count: subpasses.len() as u32,
        p_subpasses: subpasses.as_ptr(),
        dependency_count: dependencies.len() as u32,
        p_dependencies: dependencies.as_ptr(),
        ..Default::default()
    };

    device
        .create_render_pass(&create_info, None)
        .map_err(|e| e.to_string())
}

unsafe fn compile_glsl(
    compiler: &shaderc::Compiler,
    src: &str,
    kind: shaderc::ShaderKind,
) -> Result<Vec<u32>, String> {
    let compiled = compiler
        .compile_into_spirv(src, kind, "shader", "main", None)
        .map_err(|e| e.to_string())?;
    Ok(compiled.as_binary().to_vec())
}

unsafe fn create_pipeline(
    device: &ash::Device,
    render_pass: vk::RenderPass,
    descriptor_layout: vk::DescriptorSetLayout,
    extent: vk::Extent2D,
) -> Result<(vk::PipelineLayout, vk::Pipeline), String> {
    let compiler = shaderc::Compiler::new().ok_or("shaderc: falha ao inicializar")?;
    let vert_spv = compile_glsl(&compiler, VERTEX_GLSL, shaderc::ShaderKind::Vertex)?;
    let frag_spv = compile_glsl(&compiler, FRAGMENT_GLSL, shaderc::ShaderKind::Fragment)?;

    let vert_module = {
        let create_info = vk::ShaderModuleCreateInfo {
            code_size: vert_spv.len() * 4,
            p_code: vert_spv.as_ptr(),
            ..Default::default()
        };
        device
            .create_shader_module(&create_info, None)
            .map_err(|e| e.to_string())?
    };
    let frag_module = {
        let create_info = vk::ShaderModuleCreateInfo {
            code_size: frag_spv.len() * 4,
            p_code: frag_spv.as_ptr(),
            ..Default::default()
        };
        device
            .create_shader_module(&create_info, None)
            .map_err(|e| e.to_string())?
    };

    let entry_main = CString::new("main").unwrap();
    let vert_stage = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::VERTEX,
        module: vert_module,
        p_name: entry_main.as_ptr(),
        ..Default::default()
    };
    let frag_stage = vk::PipelineShaderStageCreateInfo {
        stage: vk::ShaderStageFlags::FRAGMENT,
        module: frag_module,
        p_name: entry_main.as_ptr(),
        ..Default::default()
    };

    let binding = vk::VertexInputBindingDescription {
        binding: 0,
        stride: std::mem::size_of::<crate::graphics::Vertex>() as u32,
        input_rate: vk::VertexInputRate::VERTEX,
        ..Default::default()
    };

    let attrs = [
        vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 0,
            ..Default::default()
        },
        vk::VertexInputAttributeDescription {
            location: 1,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 12,
            ..Default::default()
        },
        vk::VertexInputAttributeDescription {
            location: 2,
            binding: 0,
            format: vk::Format::R32G32_SFLOAT,
            offset: 24,
            ..Default::default()
        },
        vk::VertexInputAttributeDescription {
            location: 3,
            binding: 0,
            format: vk::Format::R32G32B32_SFLOAT,
            offset: 32,
            ..Default::default()
        },
    ];

    let vertex_input = vk::PipelineVertexInputStateCreateInfo {
        vertex_binding_description_count: 1,
        p_vertex_binding_descriptions: &binding,
        vertex_attribute_description_count: attrs.len() as u32,
        p_vertex_attribute_descriptions: attrs.as_ptr(),
        ..Default::default()
    };

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo {
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        ..Default::default()
    };

    let viewport = vk::Viewport {
        width: extent.width as f32,
        height: extent.height as f32,
        max_depth: 1.0,
        ..Default::default()
    };
    let scissor = vk::Rect2D {
        extent,
        ..Default::default()
    };
    let viewport_state = vk::PipelineViewportStateCreateInfo {
        viewport_count: 1,
        p_viewports: &viewport,
        scissor_count: 1,
        p_scissors: &scissor,
        ..Default::default()
    };

    let rasterizer = vk::PipelineRasterizationStateCreateInfo {
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::BACK,
        front_face: vk::FrontFace::COUNTER_CLOCKWISE,
        line_width: 1.0,
        ..Default::default()
    };

    let multisample = vk::PipelineMultisampleStateCreateInfo {
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        ..Default::default()
    };

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo {
        depth_test_enable: vk::TRUE,
        depth_write_enable: vk::TRUE,
        depth_compare_op: vk::CompareOp::LESS,
        ..Default::default()
    };

    let color_blend = vk::PipelineColorBlendAttachmentState {
        color_write_mask: vk::ColorComponentFlags::RGBA,
        ..Default::default()
    };
    let blend = vk::PipelineColorBlendStateCreateInfo {
        attachment_count: 1,
        p_attachments: &color_blend,
        ..Default::default()
    };

    let layouts = [descriptor_layout];
    let layout_info = vk::PipelineLayoutCreateInfo {
        set_layout_count: layouts.len() as u32,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };
    let pipeline_layout = device
        .create_pipeline_layout(&layout_info, None)
        .map_err(|e| e.to_string())?;

    let stages = [vert_stage, frag_stage];
    let pipeline_info = vk::GraphicsPipelineCreateInfo {
        stage_count: stages.len() as u32,
        p_stages: stages.as_ptr(),
        p_vertex_input_state: &vertex_input,
        p_input_assembly_state: &input_assembly,
        p_viewport_state: &viewport_state,
        p_rasterization_state: &rasterizer,
        p_multisample_state: &multisample,
        p_depth_stencil_state: &depth_stencil,
        p_color_blend_state: &blend,
        layout: pipeline_layout,
        render_pass,
        ..Default::default()
    };

    let pipelines = device
        .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        .map_err(|(_, e)| e.to_string())?;
    let pipeline = pipelines[0];

    device.destroy_shader_module(vert_module, None);
    device.destroy_shader_module(frag_module, None);

    Ok((pipeline_layout, pipeline))
}

unsafe fn create_descriptor_and_uniforms(
    instance: &Instance,
    device: &ash::Device,
    physical: vk::PhysicalDevice,
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
    let binding = vk::DescriptorSetLayoutBinding {
        binding: 0,
        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: 1,
        stage_flags: vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
        ..Default::default()
    };

    let layout_info = vk::DescriptorSetLayoutCreateInfo {
        binding_count: 1,
        p_bindings: &binding,
        ..Default::default()
    };
    let descriptor_layout = device
        .create_descriptor_set_layout(&layout_info, None)
        .map_err(|e| e.to_string())?;

    let pool_size = vk::DescriptorPoolSize {
        ty: vk::DescriptorType::UNIFORM_BUFFER,
        descriptor_count: frames as u32,
        ..Default::default()
    };
    let pool_info = vk::DescriptorPoolCreateInfo {
        pool_size_count: 1,
        p_pool_sizes: &pool_size,
        max_sets: frames as u32,
        ..Default::default()
    };
    let descriptor_pool = device
        .create_descriptor_pool(&pool_info, None)
        .map_err(|e| e.to_string())?;

    let layouts = vec![descriptor_layout; frames];
    let alloc_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool,
        descriptor_set_count: frames as u32,
        p_set_layouts: layouts.as_ptr(),
        ..Default::default()
    };
    let descriptor_sets = device
        .allocate_descriptor_sets(&alloc_info)
        .map_err(|e| e.to_string())?;

    let mut uniform_buffers = Vec::new();
    let mut uniform_memories = Vec::new();
    let mut uniform_mapped = Vec::new();

    for i in 0..frames {
        let buffer_size = std::mem::size_of::<UniformBufferObject>() as u64;
        let buffer_info = vk::BufferCreateInfo {
            size: buffer_size,
            usage: vk::BufferUsageFlags::UNIFORM_BUFFER,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            ..Default::default()
        };
        let buffer = device
            .create_buffer(&buffer_info, None)
            .map_err(|e| e.to_string())?;
        let req = device.get_buffer_memory_requirements(buffer);
        let mem_index = find_memory_type(
            instance,
            physical,
            req.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;
        let alloc = vk::MemoryAllocateInfo {
            allocation_size: req.size,
            memory_type_index: mem_index,
            ..Default::default()
        };
        let memory = device
            .allocate_memory(&alloc, None)
            .map_err(|e| e.to_string())?;
        device
            .bind_buffer_memory(buffer, memory, 0)
            .map_err(|e| e.to_string())?;
        let ptr = device
            .map_memory(memory, 0, buffer_size, vk::MemoryMapFlags::empty())
            .map_err(|e| e.to_string())? as *mut u8;

        let descriptor_info = vk::DescriptorBufferInfo {
            buffer,
            range: buffer_size,
            ..Default::default()
        };
        let write = vk::WriteDescriptorSet {
            dst_set: descriptor_sets[i],
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
            p_buffer_info: &descriptor_info,
            ..Default::default()
        };
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
    let image_info = vk::ImageCreateInfo {
        image_type: vk::ImageType::TYPE_2D,
        format: vk::Format::D32_SFLOAT,
        extent: vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        },
        mip_levels: 1,
        array_layers: 1,
        samples: vk::SampleCountFlags::TYPE_1,
        tiling: vk::ImageTiling::OPTIMAL,
        usage: vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
        ..Default::default()
    };

    let image = device
        .create_image(&image_info, None)
        .map_err(|e| e.to_string())?;
    let req = device.get_image_memory_requirements(image);
    let mem_type = find_memory_type(
        instance,
        physical,
        req.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    let alloc = vk::MemoryAllocateInfo {
        allocation_size: req.size,
        memory_type_index: mem_type,
        ..Default::default()
    };
    let memory = device
        .allocate_memory(&alloc, None)
        .map_err(|e| e.to_string())?;
    device
        .bind_image_memory(image, memory, 0)
        .map_err(|e| e.to_string())?;

    let view_info = vk::ImageViewCreateInfo {
        image,
        view_type: vk::ImageViewType::TYPE_2D,
        format: vk::Format::D32_SFLOAT,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::DEPTH,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };
    let view = device
        .create_image_view(&view_info, None)
        .map_err(|e| e.to_string())?;
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
        let info = vk::FramebufferCreateInfo {
            render_pass,
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            width: extent.width,
            height: extent.height,
            layers: 1,
            ..Default::default()
        };
        fbs.push(
            device
                .create_framebuffer(&info, None)
                .map_err(|e| e.to_string())?,
        );
    }
    Ok(fbs)
}

unsafe fn create_command_buffers(
    device: &ash::Device,
    queue_family: u32,
    count: usize,
) -> Result<(vk::CommandPool, Vec<vk::CommandBuffer>), String> {
    let pool_info = vk::CommandPoolCreateInfo {
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: queue_family,
        ..Default::default()
    };
    let pool = device
        .create_command_pool(&pool_info, None)
        .map_err(|e| e.to_string())?;
    let alloc_info = vk::CommandBufferAllocateInfo {
        command_pool: pool,
        level: vk::CommandBufferLevel::PRIMARY,
        command_buffer_count: count as u32,
        ..Default::default()
    };
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
    let mem_props = instance.get_physical_device_memory_properties(physical);
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
    let buffer_info = vk::BufferCreateInfo {
        size,
        usage: usage | vk::BufferUsageFlags::TRANSFER_DST,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let buffer = device
        .create_buffer(&buffer_info, None)
        .map_err(|e| e.to_string())?;
    let req = device.get_buffer_memory_requirements(buffer);
    let mem_type = find_memory_type(
        instance,
        physical,
        req.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    let alloc = vk::MemoryAllocateInfo {
        allocation_size: req.size,
        memory_type_index: mem_type,
        ..Default::default()
    };
    let memory = device
        .allocate_memory(&alloc, None)
        .map_err(|e| e.to_string())?;
    device
        .bind_buffer_memory(buffer, memory, 0)
        .map_err(|e| e.to_string())?;

    // Upload via staging (simplificado — produção usaria command buffer de cópia)
    let staging_info = vk::BufferCreateInfo {
        size,
        usage: vk::BufferUsageFlags::TRANSFER_SRC,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let staging = device
        .create_buffer(&staging_info, None)
        .map_err(|e| e.to_string())?;
    let staging_req = device.get_buffer_memory_requirements(staging);
    let staging_mem_type = find_memory_type(
        instance,
        physical,
        staging_req.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    let staging_alloc = vk::MemoryAllocateInfo {
        allocation_size: staging_req.size,
        memory_type_index: staging_mem_type,
        ..Default::default()
    };
    let staging_mem = device
        .allocate_memory(&staging_alloc, None)
        .map_err(|e| e.to_string())?;
    device
        .bind_buffer_memory(staging, staging_mem, 0)
        .map_err(|e| e.to_string())?;

    let ptr = device
        .map_memory(staging_mem, 0, size, vk::MemoryMapFlags::empty())
        .map_err(|e| e.to_string())?;
    std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
    device.unmap_memory(staging_mem);

    device.destroy_buffer(staging, None);
    device.free_memory(staging_mem, None);

    Ok((buffer, memory))
}
