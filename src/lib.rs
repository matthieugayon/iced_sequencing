pub mod graphics;
pub mod native;
pub mod style;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    #[doc(no_inline)]
    pub use crate::graphics::grid;

    #[doc(no_inline)]
    pub use {
        grid::Grid
    };
}

#[doc(no_inline)]
pub use platform::*;