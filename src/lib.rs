pub mod graphics;
pub mod native;
pub mod style;
pub mod core;

#[cfg(not(target_arch = "wasm32"))]
mod platform {
    #[doc(no_inline)]
    pub use crate::graphics::grid;
    pub use crate::graphics::snapshot;
    pub use crate::graphics::h_list;


    #[doc(no_inline)]
    pub use {
        grid::Grid
    };

    pub use {
        snapshot::Snapshot
    };

    pub use {
        h_list::HList
    };
}

#[doc(no_inline)]
pub use platform::*;