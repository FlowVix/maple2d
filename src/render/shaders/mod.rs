#![allow(clippy::approx_constant)]

mod out;

pub use out::common as wgsl_common;
pub use out::draw as wgsl_draw;
pub use out::{make_fragment_state, make_vertex_state};
