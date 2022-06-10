pub mod core;
pub mod graphics;
pub mod native;
pub mod style;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    #[doc(no_inline)]
    pub use crate::graphics::grid;

    #[doc(no_inline)]
    pub use grid::Grid;
    pub use crate::native::h_list::HList;
    pub use crate::native::multi_slider::MultiSlider;
    pub use crate::native::snapshot::Snapshot;
}

#[doc(no_inline)]
pub use platform::*;
