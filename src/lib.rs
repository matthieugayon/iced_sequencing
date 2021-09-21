pub mod core;
pub mod graphics;
pub mod native;
pub mod style;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    #[doc(no_inline)]
    pub use crate::graphics::grid;
    pub use crate::graphics::h_list;
    pub use crate::graphics::multi_slider;
    pub use crate::graphics::snapshot;

    #[doc(no_inline)]
    pub use grid::Grid;
    pub use h_list::HList;
    pub use multi_slider::MultiSlider;
    pub use snapshot::Snapshot;
}

#[doc(no_inline)]
pub use platform::*;
