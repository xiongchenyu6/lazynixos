use image::{DynamicImage, Rgba, RgbaImage};

const SIZE: u32 = 128;
const CENTER: f64 = 63.5;
const ARM_COLOR: Rgba<u8> = Rgba([116, 198, 224, 255]);
const BRANCH_COLOR: Rgba<u8> = Rgba([78, 176, 210, 255]);
const CORE_COLOR: Rgba<u8> = Rgba([150, 215, 235, 255]);

pub fn generate_snowflake() -> DynamicImage {
    let mut img = RgbaImage::from_fn(SIZE, SIZE, |_, _| Rgba([0, 0, 0, 0]));

    // 6-fold fractal arms
    for i in 0..6 {
        let angle = std::f64::consts::FRAC_PI_2 + (i as f64) * std::f64::consts::FRAC_PI_3;
        draw_branch(&mut img, CENTER, CENTER, 48.0, angle, 4);
    }

    // Hexagonal core
    let core_r = 6.0;
    for i in 0..6 {
        let a1 = (i as f64) * std::f64::consts::FRAC_PI_3;
        let a2 = ((i + 1) as f64) * std::f64::consts::FRAC_PI_3;
        draw_line_aa(
            &mut img,
            CENTER + core_r * a1.cos(),
            CENTER - core_r * a1.sin(),
            CENTER + core_r * a2.cos(),
            CENTER - core_r * a2.sin(),
            CORE_COLOR,
        );
    }

    // Bright center dot
    let _ = img
        .get_pixel_mut_checked(CENTER as u32, CENTER as u32)
        .map(|p| *p = CORE_COLOR);

    DynamicImage::ImageRgba8(img)
}

fn draw_branch(img: &mut RgbaImage, x: f64, y: f64, len: f64, angle: f64, depth: u32) {
    if depth == 0 || len < 1.5 {
        return;
    }

    let ex = x + len * angle.cos();
    let ey = y - len * angle.sin();

    let color = if depth >= 3 { ARM_COLOR } else { BRANCH_COLOR };
    draw_line_aa(img, x, y, ex, ey, color);

    // Continue main arm
    draw_branch(img, ex, ey, len * 0.55, angle, depth - 1);

    // Sub-branches at 1/3 and 2/3 of the arm
    let bx1 = x + len * 0.33 * angle.cos();
    let by1 = y - len * 0.33 * angle.sin();
    let bx2 = x + len * 0.66 * angle.cos();
    let by2 = y - len * 0.66 * angle.sin();
    let blen = len * 0.38;

    draw_branch(
        img,
        bx1,
        by1,
        blen,
        angle + std::f64::consts::FRAC_PI_3,
        depth - 1,
    );
    draw_branch(
        img,
        bx1,
        by1,
        blen,
        angle - std::f64::consts::FRAC_PI_3,
        depth - 1,
    );
    draw_branch(
        img,
        bx2,
        by2,
        blen * 0.7,
        angle + std::f64::consts::FRAC_PI_3,
        depth - 1,
    );
    draw_branch(
        img,
        bx2,
        by2,
        blen * 0.7,
        angle - std::f64::consts::FRAC_PI_3,
        depth - 1,
    );
}

fn draw_line_aa(img: &mut RgbaImage, x0: f64, y0: f64, x1: f64, y1: f64, color: Rgba<u8>) {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let steps = ((dx * dx + dy * dy).sqrt() * 2.0).max(1.0) as u32;

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let px = (x0 + dx * t).round() as i32;
        let py = (y0 + dy * t).round() as i32;

        if px >= 0 && px < SIZE as i32 && py >= 0 && py < SIZE as i32 {
            img.put_pixel(px as u32, py as u32, color);
        }
    }
}
