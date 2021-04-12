mod text;
pub use text::TextDesc;
pub use text::TextHandle;
pub use text::TextSectionDesc;

// re-export
#[cfg(feature = "iced")]
pub use iced_wgpu;

#[cfg(feature = "iced")]
pub use iced_winit;
