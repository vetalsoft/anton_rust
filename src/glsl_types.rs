use std::ops::{Add, Div, Mul, Sub};
use wide::f32x8;


#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32x8,
    pub y: f32x8,
}

impl Vec2 {
    pub fn new(x: f32x8, y: f32x8) -> Self {
        Vec2 { x, y }
    }

    pub fn dot(self, other: Self) -> f32x8 {
        self.x * other.x + self.y * other.y
    }

    pub fn sin(self) -> Self {
        Vec2 { x: self.x.sin(), y: self.y.sin() }
    }

    pub fn splat_float(float: f32) -> Self {
        let sp = f32x8::splat(float);
        Vec2 {
            x: sp,
            y: sp,
        }
    }

    pub fn xyyx(self) -> Vec4 {
        Vec4 { x: self.x, y: self.y, z: self.y, w: self.x }
    }

    pub fn yx(self) -> Self {
        Vec2 { x: self.y, y: self.x }
    }

    pub fn cos(self) -> Self {
        Vec2 { x: self.x.cos(), y: self.y.cos() }
    }
}

impl Add for Vec2{
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Vec2 { x: self.x + other.x, y: self.y + other.y }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Vec2 { x: self.x - other.x, y: self.y - other.y }
    }
}

impl Div<f32x8> for Vec2 {
    type Output = Self;
    fn div(self, scalar: f32x8) -> Self::Output {
        Vec2 { x: self.x / scalar, y: self.y / scalar }
    }
}

impl  Mul<f32x8> for Vec2 {
    type Output = Self;
    fn mul(self, scalar: f32x8) -> Self::Output {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vec4 {
    pub x: f32x8,
    pub y: f32x8,
    pub z: f32x8,
    pub w: f32x8,
}

impl Vec4 {
    pub fn new(x: f32x8, y: f32x8, z: f32x8, w: f32x8) -> Self {
        Vec4 { x, y, z, w }
    }

    pub fn splat_f32x8(scalar: f32x8) -> Vec4 {
        Vec4 { x: scalar, y: scalar, z: scalar, w: scalar }
    }

    pub const ZERO: Self = Self {
        x: f32x8::ZERO,
        y: f32x8::ZERO,
        z: f32x8::ZERO,
        w: f32x8::ZERO,
    };

    pub fn exp(self) -> Self {
        Vec4 { x: self.x.exp(), y: self.y.exp(), z: self.z.exp(), w: self.w.exp() }
    }

    pub fn tanh(self) -> Self {
        Vec4 {
            x: simd_vec8_tanh(self.x),
            y: simd_vec8_tanh(self.y),
            z: simd_vec8_tanh(self.z),
            w: simd_vec8_tanh(self.w),
        }
    }
}

fn simd_vec8_tanh(v: f32x8) -> f32x8 {
    let two_x = v * f32x8::splat(2.0);
    let e2x = two_x.exp(); // e^(2x)
    (e2x - f32x8::splat(1.0)) / (e2x + f32x8::splat(1.0))
}

impl  Mul<f32x8> for Vec4 {
    type Output = Self;
    fn mul(self, scalar: f32x8) -> Self::Output {
        Vec4 {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl Add for Vec4{
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        Vec4 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Vec4 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        Vec4 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Div<Vec4> for Vec4 {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        Vec4 {
            x: self.x / other.x,
            y: self.y / other.y,
            z: self.z / other.z,
            w: self.w / other.w,
        }
    }
}

pub struct Color {
    pub r: [u8; 8],
    pub g: [u8; 8],
    pub b: [u8; 8],
}

impl Color {
    
}

pub fn vec4_to_rgb_arrow(vec: Vec4) -> Color {
    let convet = |x: f32x8|
        ((x * 255.0).max(f32x8::ZERO).min(f32x8::splat(255.0)))
        .round().to_array().map(|e| e as u8);
    Color {
        r: convet(vec.x),
        g: convet(vec.y),
        b: convet(vec.z),
    }
}