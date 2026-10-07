//! Graph PNG visual tokens (SPEC-161 LENS-front).

pub const CANVAS_W: u32 = 1024;
pub const CANVAS_H: u32 = 768;
pub const BG: [u8; 3] = [255, 255, 255];
pub const FOCUS_FILL: [u8; 3] = [37, 99, 235];
pub const NODE_FILL: [u8; 3] = [100, 116, 139];
pub const FOCUS_RING: [u8; 3] = [15, 23, 42];
pub const EDGE: [u8; 3] = [148, 163, 184];
pub const FOCUS_RADIUS: i32 = 14;
pub const NODE_RADIUS: i32 = 10;
pub const FOCUS_RING_WIDTH: i32 = 3;
pub const HOP1_FRAC: f32 = 0.28;
pub const HOP2_FRAC: f32 = 0.55;
pub const LABEL_MAX: usize = 24;
