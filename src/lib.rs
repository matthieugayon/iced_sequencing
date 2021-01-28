pub mod graphics;
pub mod native;
pub mod style;
pub mod core;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    #[doc(no_inline)]
    pub use crate::graphics::grid;

    #[doc(no_inline)]
    pub use {
        grid::Grid
    };

    #[doc(no_inline)]
    pub use crate::graphics::multi_slider;

    #[doc(no_inline)]
    pub use {
        multi_slider::MultiSlider
    };
}

#[doc(no_inline)]
pub use platform::*;
