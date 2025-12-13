use rayon::prelude::*;
use std::{fs::File, io::Write, path::Path, time::Instant};
use wide::f32x8;

mod glsl_types;
use glsl_types::*;

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

// Функция для вычисления 8 пикселей SIMD
fn calculate_8_pixels_color_simd(x_start: u32, y: u32, time: f32, buffer: &mut [u8]) {
    let x_coords =
        f32x8::splat(x_start as f32) + f32x8::from([0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]);

    let y_coords: f32x8 = f32x8::splat((HEIGHT - y) as f32);
    let r_x = f32x8::splat(WIDTH as f32);
    let r_y = f32x8::splat(HEIGHT as f32);

    // GLSL: vec2 p = (I+I-r) / r.y;
    let in_vec2_i: Vec2 = Vec2::new(x_coords, y_coords);
    let resolution: Vec2 = Vec2::new(r_x, r_y);
    let position: Vec2 = (in_vec2_i + in_vec2_i - resolution) / resolution.y;

    // GLSL: z = 4. - 4.*abs(.7-dot(p,p))
    let z_scalar: f32x8 =
        (f32x8::splat(0.7) - position.dot(position)).abs() * f32x8::splat(-4.0) + f32x8::splat(4.0);

    // GLSL: f = p*(z+=...) -> f = p * z.x
    let mut fluid_coordinstes: Vec2 = position * z_scalar;

    // GLSL: O *= 0. -> O = vec4(0.)
    let mut out_vec4_o: Vec4 = Vec4::ZERO;

    // GLSL: for(...; i.y++<8.; ...)
    for step in 1..=8 {
        let i_y: f32x8 = f32x8::splat(step as f32);

        // GLSL: O += (sin(f)+1.).xyyx * abs(f.x-f.y)
        out_vec4_o = out_vec4_o
            + (fluid_coordinstes.sin() + Vec2::splat_float(1.0)).xyyx()
                * (fluid_coordinstes.x - fluid_coordinstes.y).abs();

        // GLSL: f += cos(f.yx*i.y+i+iTime)/i.y+.7;
        fluid_coordinstes = fluid_coordinstes
            + ((fluid_coordinstes.yx() * i_y
                + Vec2::new(f32x8::ZERO, i_y)
                + Vec2::splat_float(time))
            .cos()
                / i_y)
            + Vec2::splat_float(0.7);
    }

    // GLSL: O = tanh(7.*exp(z.x-4.-p.y*vec4(-1,1,2,0))/O);
    let color_gradient = Vec4::new(
        // p.y*vec4(-1,1,2,0
        -position.y,
        position.y,
        position.y * f32x8::splat(2.0),
        f32x8::ZERO,
    );

    out_vec4_o = (((Vec4::splat_f32x8(z_scalar - f32x8::splat(4.0)) - color_gradient).exp()
        * f32x8::splat(7.0))
        / out_vec4_o)
        .tanh();

    let pixels = vec4_to_rgb_arrow(out_vec4_o);

    for i in 0..8 {
        let offset = (x_start as usize + i) * 3; // 3 байта на пиксель
        buffer[offset + 0] = pixels.r[i]; // R
        buffer[offset + 1] = pixels.g[i]; // G
        buffer[offset + 2] = pixels.b[i]; // B
    }
}

fn shader(pixels: &mut [u8], t: f32) {
    assert_eq!(pixels.len(), (WIDTH * HEIGHT * 3) as usize);

    pixels
        .par_chunks_mut((WIDTH * 3) as usize)
        .enumerate()
        .for_each(|(row_idx, row_slice)| {
            let y = row_idx as u32;

            let mut x = 0;
            while x + 8 <= row_slice.len() / 3 {
                calculate_8_pixels_color_simd(x as u32, y, t, row_slice);
                x += 8;
            }
        });
}

fn dump_ppm<T: AsRef<Path>>(
    filename: T,
    pixels: &[u8],
    width: u32,
    height: u32,
) -> std::io::Result<()> {
    let mut out = File::create(filename)?;
    writeln!(out, "P6 {} {} 255", width, height)?;
    out.write_all(pixels)?;
    Ok(())
}

// код бенчмарка в видео на ютуб 6:58:54
// fn cycles_to_seconds(cycles: u64) -> f32 {
//     cycles as f32 / 4000000000.0
// }

const COUNT: usize = 100;
fn main() {
    let mut pixels = vec![0u8; (WIDTH * HEIGHT * 3) as usize];
    let mut time = 0.0;

    let mut total_frame_time_ns = 0;

    // let mut total_frame_time_an = 0;

    for _ in 0..COUNT {
        let start = Instant::now();

        // let start_an = unsafe { std::arch::x86_64::_rdtsc() };

        shader(&mut pixels, time);
        time += 1.0;

        let elapsed = start.elapsed();
        total_frame_time_ns += elapsed.as_nanos() as u64;

        // let end_an = unsafe { std::arch::x86_64::_rdtsc() };
        // total_frame_time_an += end_an - start_an;
    }

    let avg = (total_frame_time_ns / COUNT as u64) as f64 / 1.0e+6;
    let took = (total_frame_time_ns / COUNT as u64) as f64 / 1.0e+9;
    let fps = 1000.0 / avg;
    println!("Took {took} s to render {COUNT} frames (Avg: {avg}, FPS: {fps})");

    // let frame_time_an =
    //     cycles_to_seconds((total_frame_time_an as f64 / COUNT as f64) as u64) * 1000.;
    // println!(
    //     "Took {} s to render {} frames (Avg: {}, FPS: {})",
    //     cycles_to_seconds(total_frame_time_an),
    //     COUNT,
    //     frame_time_an,
    //     1000.0 / frame_time_an
    // );

    shader(&mut pixels, 0.0);
    dump_ppm("output.ppm", &pixels, WIDTH, HEIGHT).unwrap();
}
