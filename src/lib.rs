pub mod core;
pub mod graphics;
pub mod native;
pub mod style;

// republish the native widgets as mods
pub use native::h_list;
pub use native::grid;
pub use native::snapshot_view;

pub use grid::*;
pub use h_list::*;
pub use snapshot_view::*;

// #[cfg(not(target_arch = "wasm32"))]
// mod platform {
//     #[doc(no_inline)]
//     pub use crate::graphics::grid;

//     #[doc(no_inline)]
//     pub use grid::Grid;
//     pub use crate::native::h_list::HList;
//     // pub use crate::native::multi_slider::MultiSlider;
//     pub use crate::native::snapshot::Snapshot;
// }

// #[doc(no_inline)]
// pub use platform::*;
