//! Renderer Direct3D 11 — device, swap chain, shaders HLSL, constant buffer.

use crate::graphics::backend::GfxBackend;
use crate::graphics::shaders::{FRAGMENT_HLSL, LIGHT_DIRECTION, VERTEX_HLSL};
use crate::graphics::{Camera, Color, GpuMesh, Mesh};
use crate::math::Mat4;
use std::collections::HashMap;
use std::ffi::c_void;
use windows::core::PCSTR;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_HARDWARE;
use windows::Win32::Graphics::Direct3D::D3D_FEATURE_LEVEL_11_0;
use windows::Win32::Graphics::Direct3D::D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Direct3D::Fxc::{D3DCompile, D3DCOMPILE_ENABLE_STRICTNESS};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R32G32B32_FLOAT, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{
    DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_DISCARD, DXGI_USAGE_RENDER_TARGET_OUTPUT, IDXGISwapChain,
};
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::Window;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct TransformBuffer {
    mvp: [[f32; 4]; 4],
    model: [[f32; 4]; 4],
    light_dir: [f32; 3],
    _pad: f32,
}

struct DxMesh {
    vertex_buffer: ID3D11Buffer,
    index_buffer: ID3D11Buffer,
    index_count: u32,
}

pub struct DirectX11Renderer {
    device: ID3D11Device,
    context: ID3D11DeviceContext,
    swap_chain: IDXGISwapChain,
    render_target: ID3D11RenderTargetView,
    depth_stencil: ID3D11Texture2D,
    depth_view: ID3D11DepthStencilView,
    vertex_shader: ID3D11VertexShader,
    pixel_shader: ID3D11PixelShader,
    input_layout: ID3D11InputLayout,
    constant_buffer: ID3D11Buffer,
    meshes: HashMap<u64, DxMesh>,
    next_id: u64,
    width: u32,
    height: u32,
}

impl DirectX11Renderer {
    pub fn new(window: &Window) -> Result<Self, String> {
        unsafe {
            let hwnd = window_to_hwnd(window)?;

            let swap_desc = DXGI_SWAP_CHAIN_DESC {
                BufferDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_MODE_DESC {
                    Width: window.inner_size().width,
                    Height: window.inner_size().height,
                    RefreshRate: windows::Win32::Graphics::Dxgi::Common::DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1,
                    },
                    Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R8G8B8A8_UNORM,
                    ScanlineOrdering:
                        windows::Win32::Graphics::Dxgi::Common::DXGI_MODE_SCANLINE_ORDER_UNSPECIFIED,
                    Scaling: windows::Win32::Graphics::Dxgi::Common::DXGI_MODE_SCALING_UNSPECIFIED,
                },
                SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                BufferCount: 1,
                OutputWindow: hwnd,
                Windowed: true.into(),
                SwapEffect: DXGI_SWAP_EFFECT_DISCARD,
                Flags: 0,
            };

            let mut device = None;
            let mut context = None;
            let mut swap_chain = None;

            D3D11CreateDeviceAndSwapChain(
                None,
                D3D_DRIVER_TYPE_HARDWARE,
                None,
                D3D11_CREATE_DEVICE_FLAG(0),
                Some(&[D3D_FEATURE_LEVEL_11_0]),
                D3D11_SDK_VERSION,
                Some(&swap_desc),
                Some(&mut swap_chain),
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .map_err(|e| format!("D3D11CreateDeviceAndSwapChain: {e}"))?;

            let device = device.ok_or("Device nulo")?;
            let context = context.ok_or("Context nulo")?;
            let swap_chain = swap_chain.ok_or("SwapChain nulo")?;

            let back_buffer: ID3D11Texture2D = swap_chain
                .GetBuffer(0)
                .map_err(|e| format!("GetBuffer: {e}"))?;

            let mut render_target = None;
            device
                .CreateRenderTargetView(&back_buffer, None, Some(&mut render_target))
                .map_err(|e| format!("CreateRenderTargetView: {e}"))?;
            let render_target = render_target.ok_or("RTV nulo")?;

            let (depth_stencil, depth_view) = create_depth_stencil(
                &device,
                window.inner_size().width,
                window.inner_size().height,
            )?;

            let (vertex_shader, vs_blob) = compile_vs(&device, VERTEX_HLSL)?;
            let pixel_shader = compile_ps(&device, FRAGMENT_HLSL)?;
            let input_layout = create_input_layout(&device, &vs_blob)?;

            let cb_desc = D3D11_BUFFER_DESC {
                ByteWidth: std::mem::size_of::<TransformBuffer>() as u32,
                Usage: D3D11_USAGE_DYNAMIC,
                BindFlags: D3D11_BIND_CONSTANT_BUFFER.0 as u32,
                CPUAccessFlags: D3D11_CPU_ACCESS_WRITE.0 as u32,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            let mut constant_buffer = None;
            device
                .CreateBuffer(&cb_desc, None, Some(&mut constant_buffer))
                .map_err(|e| format!("CreateBuffer CB: {e}"))?;
            let constant_buffer = constant_buffer.ok_or("CB nulo")?;

            Ok(Self {
                device,
                context,
                swap_chain,
                render_target,
                depth_stencil,
                depth_view,
                vertex_shader,
                pixel_shader,
                input_layout,
                constant_buffer,
                meshes: HashMap::new(),
                next_id: 1,
                width: window.inner_size().width,
                height: window.inner_size().height,
            })
        }
    }

    fn update_constant_buffer(&self, model: Mat4, camera: &Camera) {
        let data = TransformBuffer {
            mvp: (camera.view_projection() * model).to_cols_array_2d(),
            model: model.to_cols_array_2d(),
            light_dir: LIGHT_DIRECTION,
            _pad: 0.0,
        };

        unsafe {
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            self.context
                .Map(
                    &self.constant_buffer,
                    0,
                    D3D11_MAP_WRITE_DISCARD,
                    0,
                    Some(&mut mapped),
                )
                .ok();
            std::ptr::copy_nonoverlapping(
                &data as *const _ as *const u8,
                mapped.pData as *mut u8,
                std::mem::size_of::<TransformBuffer>(),
            );
            self.context.Unmap(&self.constant_buffer, 0);
        }
    }
}

impl GfxBackend for DirectX11Renderer {
    type Error = String;

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width.max(1);
        self.height = height.max(1);
    }

    fn upload_mesh(&mut self, mesh: &Mesh) -> Result<GpuMesh, String> {
        let id = self.next_id;
        self.next_id += 1;

        unsafe {
            let vb_desc = D3D11_BUFFER_DESC {
                ByteWidth: (mesh.vertices.len() * std::mem::size_of::<crate::graphics::Vertex>()) as u32,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_VERTEX_BUFFER.0 as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            let vb_data = D3D11_SUBRESOURCE_DATA {
                pSysMem: mesh.vertices.as_ptr() as *const c_void,
                SysMemPitch: 0,
                SysMemSlicePitch: 0,
            };
            let mut vertex_buffer = None;
            self.device
                .CreateBuffer(&vb_desc, Some(&vb_data), Some(&mut vertex_buffer))
                .map_err(|e| format!("VB: {e}"))?;
            let vertex_buffer = vertex_buffer.ok_or("VB nulo")?;

            let ib_desc = D3D11_BUFFER_DESC {
                ByteWidth: (mesh.indices.len() * 4) as u32,
                Usage: D3D11_USAGE_DEFAULT,
                BindFlags: D3D11_BIND_INDEX_BUFFER.0 as u32,
                CPUAccessFlags: 0,
                MiscFlags: 0,
                StructureByteStride: 0,
            };
            let ib_data = D3D11_SUBRESOURCE_DATA {
                pSysMem: mesh.indices.as_ptr() as *const c_void,
                SysMemPitch: 0,
                SysMemSlicePitch: 0,
            };
            let mut index_buffer = None;
            self.device
                .CreateBuffer(&ib_desc, Some(&ib_data), Some(&mut index_buffer))
                .map_err(|e| format!("IB: {e}"))?;
            let index_buffer = index_buffer.ok_or("IB nulo")?;

            self.meshes.insert(
                id,
                DxMesh {
                    vertex_buffer,
                    index_buffer,
                    index_count: mesh.indices.len() as u32,
                },
            );
        }

        Ok(GpuMesh {
            vertex_count: mesh.vertices.len() as u32,
            index_count: mesh.indices.len() as u32,
            gpu_id: id,
        })
    }

    fn begin_frame(&mut self, clear: Color) {
        unsafe {
            let color = [clear.r, clear.g, clear.b, clear.a];
            self.context.ClearRenderTargetView(&self.render_target, &color);
            self.context.ClearDepthStencilView(
                &self.depth_view,
                D3D11_CLEAR_DEPTH.0 as u32,
                1.0,
                0,
            );

            self.context.OMSetRenderTargets(
                Some(&[Some(self.render_target.clone())]),
                Some(&self.depth_view),
            );

            let viewport = D3D11_VIEWPORT {
                TopLeftX: 0.0,
                TopLeftY: 0.0,
                Width: self.width as f32,
                Height: self.height as f32,
                MinDepth: 0.0,
                MaxDepth: 1.0,
            };
            self.context.RSSetViewports(Some(&[viewport]));

            self.context.IASetInputLayout(&self.input_layout);
            self.context.IASetPrimitiveTopology(D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
            self.context.VSSetShader(&self.vertex_shader, None);
            self.context.PSSetShader(&self.pixel_shader, None);
            self.context
                .VSSetConstantBuffers(0, Some(&[Some(self.constant_buffer.clone())]));
            self.context
                .PSSetConstantBuffers(0, Some(&[Some(self.constant_buffer.clone())]));
        }
    }

    fn upload_texture(
        &mut self,
        data: &crate::graphics::TextureData,
    ) -> Result<crate::graphics::GpuTexture, String> {
        let _ = data;
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

        self.update_constant_buffer(model, camera);

        unsafe {
            let stride = std::mem::size_of::<crate::graphics::Vertex>() as u32;
            let offset = 0u32;
            self.context.IASetVertexBuffers(
                0,
                1,
                Some(&Some(mesh.vertex_buffer.clone())),
                Some(&stride),
                Some(&offset),
            );
            self.context.IASetIndexBuffer(
                &mesh.index_buffer,
                windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_R32_UINT,
                0,
            );
            self.context.DrawIndexed(mesh.index_count, 0, 0);
        }
        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), String> {
        unsafe {
            self.swap_chain
                .Present(1, windows::Win32::Graphics::Dxgi::DXGI_PRESENT(0))
                .ok()
                .map_err(|e| format!("Present: {e}"))?;
        }
        Ok(())
    }
}

fn window_to_hwnd(window: &Window) -> Result<HWND, String> {
    let handle = window.window_handle().map_err(|e| e.to_string())?;
    match handle.as_raw() {
        RawWindowHandle::Win32(h) => Ok(HWND(h.hwnd.get() as *mut c_void)),
        other => Err(format!("Handle não é Win32: {other:?}")),
    }
}

unsafe fn compile_shader(source: &str, entry: &str, profile: &str) -> Result<Vec<u8>, String> {
    let source_null = format!("{source}\0");
    let entry_null = format!("{entry}\0");
    let profile_null = format!("{profile}\0");

    let mut blob = None;
    let mut error_blob = None;

    D3DCompile(
        source_null.as_ptr() as *const c_void,
        source_null.len(),
        PCSTR::null(),
        None,
        None,
        PCSTR(entry_null.as_ptr() as *const u8),
        PCSTR(profile_null.as_ptr() as *const u8),
        D3DCOMPILE_ENABLE_STRICTNESS,
        0,
        &mut blob,
        Some(&mut error_blob),
    )
    .map_err(|e| format!("D3DCompile: {e}"))?;

    let blob = blob.ok_or("Shader blob nulo")?;
    let ptr = blob.GetBufferPointer() as *const u8;
    let size = blob.GetBufferSize();
    Ok(std::slice::from_raw_parts(ptr, size).to_vec())
}

unsafe fn compile_vs(
    device: &ID3D11Device,
    source: &str,
) -> Result<(ID3D11VertexShader, Vec<u8>), String> {
    let bytecode = compile_shader(source, "main", "vs_5_0")?;
    let mut shader = None;
    device
        .CreateVertexShader(&bytecode, None, Some(&mut shader))
        .map_err(|e| format!("VS: {e}"))?;
    Ok((shader.ok_or("VS nulo")?, bytecode))
}

unsafe fn compile_ps(device: &ID3D11Device, source: &str) -> Result<ID3D11PixelShader, String> {
    let bytecode = compile_shader(source, "main", "ps_5_0")?;
    let mut shader = None;
    device
        .CreatePixelShader(&bytecode, None, Some(&mut shader))
        .map_err(|e| format!("PS: {e}"))?;
    shader.ok_or("PS nulo".into())
}

unsafe fn create_input_layout(
    device: &ID3D11Device,
    vs_blob: &[u8],
) -> Result<ID3D11InputLayout, String> {
    let layout = [
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: windows::core::PCSTR(b"POSITION\0".as_ptr()),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 0,
            InputSlotClass: D3D11_INPUT_CLASSIFICATION(D3D11_INPUT_PER_VERTEX_DATA.0 as i32),
            InstanceDataStepRate: 0,
        },
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: windows::core::PCSTR(b"NORMAL\0".as_ptr()),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 12,
            InputSlotClass: D3D11_INPUT_CLASSIFICATION(D3D11_INPUT_PER_VERTEX_DATA.0 as i32),
            InstanceDataStepRate: 0,
        },
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: windows::core::PCSTR(b"TEXCOORD\0".as_ptr()),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 24,
            InputSlotClass: D3D11_INPUT_CLASSIFICATION(D3D11_INPUT_PER_VERTEX_DATA.0 as i32),
            InstanceDataStepRate: 0,
        },
        D3D11_INPUT_ELEMENT_DESC {
            SemanticName: windows::core::PCSTR(b"COLOR\0".as_ptr()),
            SemanticIndex: 0,
            Format: DXGI_FORMAT_R32G32B32_FLOAT,
            InputSlot: 0,
            AlignedByteOffset: 32,
            InputSlotClass: D3D11_INPUT_CLASSIFICATION(D3D11_INPUT_PER_VERTEX_DATA.0 as i32),
            InstanceDataStepRate: 0,
        },
    ];

    let mut layout_out = None;
    device
        .CreateInputLayout(&layout, vs_blob, Some(&mut layout_out))
        .map_err(|e| format!("InputLayout: {e}"))?;
    layout_out.ok_or("Layout nulo".into())
}

unsafe fn create_depth_stencil(
    device: &ID3D11Device,
    width: u32,
    height: u32,
) -> Result<(ID3D11Texture2D, ID3D11DepthStencilView), String> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: windows::Win32::Graphics::Dxgi::Common::DXGI_FORMAT_D24_UNORM_S8_UINT,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: D3D11_BIND_DEPTH_STENCIL.0 as u32,
        CPUAccessFlags: 0,
        MiscFlags: 0,
    };

    let mut texture = None;
    device
        .CreateTexture2D(&desc, None, Some(&mut texture))
        .map_err(|e| format!("Depth tex: {e}"))?;
    let texture = texture.ok_or("Depth tex nulo")?;

    let mut view = None;
    device
        .CreateDepthStencilView(&texture, None, Some(&mut view))
        .map_err(|e| format!("DSV: {e}"))?;
    let view = view.ok_or("DSV nulo")?;

    Ok((texture, view))
}
