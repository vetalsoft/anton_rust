use wide::f32x4;
use rayon::prelude::*;
use std::{
    fs::File,
    io::Write,
    path::Path,
    time::Instant,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

const TANH_TABLE_SIZE: usize = 1200;
const TANH_MIN_INPUT: f32 = 0.0;
const TANH_MAX_INPUT: f32 = 5.0;

static TANH_TABLE: std::sync::LazyLock<Vec<u8>> = std::sync::LazyLock::new(|| build_tanh_table());

fn build_tanh_table() -> Vec<u8> {
    let mut table = Vec::with_capacity(TANH_TABLE_SIZE);
    let scale = (TANH_TABLE_SIZE - 1) as f32 / (TANH_MAX_INPUT - TANH_MIN_INPUT);
    for i in 0..TANH_TABLE_SIZE {
        let x = TANH_MIN_INPUT + (i as f32) / scale;
        let tanh_x = x.tanh(); // стандартный tanh из f32
        let clamped = tanh_x.max(0.0).min(1.0); // clamp(0, 1)
        let result = (clamped * 255.0).round() as u8; // * 255 и округление
        table.push(result);
    }
    table
}

fn lookup_tanh_clamped(x: f32, table: &[u8]) -> u8 {
    // Приводим x к индексу
    let scaled = (x - TANH_MIN_INPUT) * (TANH_TABLE_SIZE - 1) as f32 / (TANH_MAX_INPUT - TANH_MIN_INPUT);
    let scaled = scaled.max(0.0).min((TANH_TABLE_SIZE - 1) as f32);

    let idx_f = scaled;
    let idx0 = idx_f.floor() as usize;
    let idx1 = (idx0 + 1).min(TANH_TABLE_SIZE - 1);
    let fract = idx_f - idx_f.floor();

    let val0 = table[idx0] as f32;
    let val1 = table[idx1] as f32;

    ((val0 * (1.0 - fract) + val1 * fract).round()) as u8
}

fn lookup_tanh_clamped_f32x4(v: f32x4, table: &[u8]) -> [u8; 4] {
    let arr = v.to_array();
    [   lookup_tanh_clamped(arr[0], table),
        lookup_tanh_clamped(arr[1], table),
        lookup_tanh_clamped(arr[2], table),
        lookup_tanh_clamped(arr[3], table),
    ]
}

// fn simd_vec4_tanh(v: f32x4) -> f32x4 {
//     let two_x = v * f32x4::splat(2.0);
//     let e2x = two_x.exp(); // e^(2x)
//     (e2x - f32x4::splat(1.0)) / (e2x + f32x4::splat(1.0))
// }

// fn simd_clamp_th(v: f32x4, min_val: f32x4, max_val: f32x4) -> f32x4 {
//     simd_vec4_tanh(v).max(min_val).min(max_val)
// }

// Функция для вычисления 4 пикселей SIMD
fn calculate_4_pixels_color_simd(x_start: u32, y: u32, t: f32, buffer: &mut [u8]) {
    let x_coords = f32x4::from(
        [x_start as f32,
        (x_start + 1) as f32,
        (x_start + 2) as f32,
        (x_start + 3) as f32]
    );

    let y_coord_scalar = (HEIGHT - y) as f32;
    let y_coords = f32x4::splat(y_coord_scalar);
    let r_x_scalar = WIDTH as f32;
    let r_y_scalar = HEIGHT as f32;
    let r_x = f32x4::splat(r_x_scalar);
    let r_y = f32x4::splat(r_y_scalar);

    let fc_x = x_coords;
    let fc_y = y_coords;
    let p_x = (fc_x * 2.0 - r_x) / r_y;
    let p_y = (fc_y * 2.0 - r_y) / r_y;

    let p_dot_p = p_x * p_x + p_y * p_y;
    let l_val = (f32x4::splat(0.7) - p_dot_p).abs();
    let scale = (f32x4::splat(1.0) - l_val) * f32x4::splat(5.0);
    let mut v_x = p_x * scale;
    let mut v_y = p_y * scale;

    let mut o_x = f32x4::ZERO;
    let mut o_y = f32x4::ZERO;
    let mut o_z = f32x4::ZERO;
    let mut o_w = f32x4::ZERO;

    for step in 1..=8 {
        let i_y_scalar = step as f32;
        let i_y_vec = f32x4::splat(i_y_scalar);

        let abs_diff = (v_x - v_y).abs() * 0.2;
        let sin_v_x = (v_x).sin();
        let sin_v_y = (v_y).sin();

        o_x = o_x + (sin_v_x + f32x4::splat(1.0)) * abs_diff;
        o_y = o_y + (sin_v_y + f32x4::splat(1.0)) * abs_diff;
        o_z = o_z + (sin_v_y + f32x4::splat(1.0)) * abs_diff;
        o_w = o_w + (sin_v_x + f32x4::splat(1.0)) * abs_diff; 

        let cos_x = (v_y * i_y_vec + f32x4::splat(t)).cos();
        let cos_y = (v_x * i_y_vec + i_y_vec + f32x4::splat(t)).cos();

        v_x = v_x + cos_x / i_y_vec + f32x4::splat(0.7);
        v_y = v_y + cos_y / i_y_vec + f32x4::splat(0.7);
    }

    let exp_l = (-f32x4::splat(4.0) * l_val).exp();

    let r = (p_y.exp() * exp_l) / o_x;
    let g = ((-p_y).exp() * exp_l) / o_y;
    let b = ((-p_y * 2.0).exp() * exp_l) / o_z;

    let r_arr = lookup_tanh_clamped_f32x4(r, &TANH_TABLE);
    let g_arr = lookup_tanh_clamped_f32x4(g, &TANH_TABLE);
    let b_arr = lookup_tanh_clamped_f32x4(b, &TANH_TABLE);

    for i in 0..4 {
        let offset = (x_start as usize + i) * 3; // 3 байта на пиксель
        buffer[offset + 0] = r_arr[i]; // R
        buffer[offset + 1] = g_arr[i]; // G
        buffer[offset + 2] = b_arr[i]; // B
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
            while x + 4 <= row_slice.len() / 3 {
                calculate_4_pixels_color_simd(x as u32, y, t, row_slice);
                x += 4;
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
