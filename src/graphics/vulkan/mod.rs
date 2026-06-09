//! # Backend Vulkan 1.2
//!
//! Implementação direta com a crate `ash` (bindings Rust para Vulkan).
//! Vulkan é verboso, mas este módulo mostra cada etapa com comentários:
//!
//! ```text
//! Instance → Surface → Device → Swapchain → Pipeline → Draw
//! ```

mod renderer;

pub use renderer::VulkanRenderer;
