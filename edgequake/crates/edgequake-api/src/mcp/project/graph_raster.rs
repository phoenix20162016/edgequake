//! Rasterize a laid-out graph to PNG. Does not query the graph.

use image::{ImageBuffer, ImageFormat, Rgb, RgbImage};

use super::graph_layout::{LayoutEdge, LayoutNode};
use super::graph_theme::{
    BG, CANVAS_H, CANVAS_W, EDGE, FOCUS_FILL, FOCUS_RADIUS, FOCUS_RING, FOCUS_RING_WIDTH,
    NODE_FILL, NODE_RADIUS,
};

pub fn raster_png(nodes: &[LayoutNode], edges: &[LayoutEdge]) -> Vec<u8> {
    let mut img: RgbImage = ImageBuffer::from_pixel(CANVAS_W, CANVAS_H, Rgb(BG));
    for e in edges {
        let Some(a) = nodes.iter().find(|n| n.id == e.source) else {
            continue;
        };
        let Some(b) = nodes.iter().find(|n| n.id == e.target) else {
            continue;
        };
        draw_line(
            &mut img,
            a.x as i32,
            a.y as i32,
            b.x as i32,
            b.y as i32,
            Rgb(EDGE),
        );
    }
    for n in nodes {
        let r = if n.is_focus {
            FOCUS_RADIUS
        } else {
            NODE_RADIUS
        };
        let fill = if n.is_focus {
            Rgb(FOCUS_FILL)
        } else {
            Rgb(NODE_FILL)
        };
        fill_circle(&mut img, n.x as i32, n.y as i32, r, fill);
        if n.is_focus {
            stroke_circle(
                &mut img,
                n.x as i32,
                n.y as i32,
                r + FOCUS_RING_WIDTH,
                FOCUS_RING_WIDTH,
                Rgb(FOCUS_RING),
            );
        }
    }
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, ImageFormat::Png)
        .expect("png encode");
    buf.into_inner()
}

fn fill_circle(img: &mut RgbImage, cx: i32, cy: i32, r: i32, color: Rgb<u8>) {
    let r2 = r * r;
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            if (x - cx) * (x - cx) + (y - cy) * (y - cy) <= r2 {
                put(img, x, y, color);
            }
        }
    }
}

fn stroke_circle(img: &mut RgbImage, cx: i32, cy: i32, r: i32, width: i32, color: Rgb<u8>) {
    let outer = r * r;
    let inner = (r - width).max(0);
    let inner2 = inner * inner;
    for y in (cy - r)..=(cy + r) {
        for x in (cx - r)..=(cx + r) {
            let d = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            if d <= outer && d >= inner2 {
                put(img, x, y, color);
            }
        }
    }
}

fn draw_line(img: &mut RgbImage, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgb<u8>) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        put(img, x, y, color);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn put(img: &mut RgbImage, x: i32, y: i32, color: Rgb<u8>) {
    if x >= 0 && y >= 0 && (x as u32) < img.width() && (y as u32) < img.height() {
        img.put_pixel(x as u32, y as u32, color);
    }
}
