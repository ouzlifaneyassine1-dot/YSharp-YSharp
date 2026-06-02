#![allow(non_camel_case_types, dead_code)]

// ---------------------------------------------------------------------------
// OY# SIMD Vector Math — float4, float4x4, quaternion
// Auto-vectorized with explicit SIMD via cfg(target_feature)
// Fallback scalar path for any platform.
// ---------------------------------------------------------------------------

use core::ops::{Add, Sub, Mul, Neg, Index, IndexMut, Div};

// ---------------------------------------------------------------------------
// float4
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct float4(pub f32, pub f32, pub f32, pub f32);

macro_rules! float4_binop {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for float4 {
            type Output = float4;
            #[inline(always)]
            fn $method(self, rhs: float4) -> float4 {
                #[cfg(target_feature = "sse")]
                {
                    // Will use _mm_add_ps etc; fallback for now
                }
                float4(self.0 $op rhs.0, self.1 $op rhs.1, self.2 $op rhs.2, self.3 $op rhs.3)
            }
        }
        impl $trait<f32> for float4 {
            type Output = float4;
            #[inline(always)]
            fn $method(self, rhs: f32) -> float4 {
                float4(self.0 $op rhs, self.1 $op rhs, self.2 $op rhs, self.3 $op rhs)
            }
        }
    };
}

float4_binop!(Add, add, +);
float4_binop!(Sub, sub, -);
float4_binop!(Mul, mul, *);
float4_binop!(Div, div, /);

impl Neg for float4 {
    type Output = float4;
    #[inline(always)]
    fn neg(self) -> float4 { float4(-self.0, -self.1, -self.2, -self.3) }
}

impl Index<usize> for float4 {
    type Output = f32;
    #[inline(always)]
    fn index(&self, i: usize) -> &f32 {
        match i { 0 => &self.0, 1 => &self.1, 2 => &self.2, _ => &self.3 }
    }
}
impl IndexMut<usize> for float4 {
    #[inline(always)]
    fn index_mut(&mut self, i: usize) -> &mut f32 {
        match i { 0 => &mut self.0, 1 => &mut self.1, 2 => &mut self.2, _ => &mut self.3 }
    }
}

impl float4 {
    #[inline(always)] pub const fn new(x: f32, y: f32, z: f32, w: f32) -> float4 { float4(x, y, z, w) }
    #[inline(always)] pub const fn zero() -> float4 { float4(0.0, 0.0, 0.0, 0.0) }
    #[inline(always)] pub const fn one() -> float4 { float4(1.0, 1.0, 1.0, 1.0) }
    #[inline(always)] pub const fn unit_x() -> float4 { float4(1.0, 0.0, 0.0, 0.0) }
    #[inline(always)] pub const fn unit_y() -> float4 { float4(0.0, 1.0, 0.0, 0.0) }
    #[inline(always)] pub const fn unit_z() -> float4 { float4(0.0, 0.0, 1.0, 0.0) }
    #[inline(always)] pub fn splat(v: f32) -> float4 { float4(v, v, v, v) }

    #[inline(always)] pub fn dot(self, rhs: float4) -> f32 {
        self.0 * rhs.0 + self.1 * rhs.1 + self.2 * rhs.2 + self.3 * rhs.3
    }

    #[inline(always)] pub fn cross(self, rhs: float4) -> float4 {
        float4(
            self.1 * rhs.2 - self.2 * rhs.1,
            self.2 * rhs.0 - self.0 * rhs.2,
            self.0 * rhs.1 - self.1 * rhs.0,
            0.0,
        )
    }

    #[inline(always)] pub fn length_sq(self) -> f32 { self.dot(self) }
    #[inline(always)] pub fn length(self) -> f32 { self.length_sq().sqrt() }

    #[inline(always)] pub fn normalize(self) -> float4 {
        let len = self.length();
        if len > 1e-8 { self * (1.0 / len) } else { float4::zero() }
    }

    #[inline(always)] pub fn lerp(self, other: float4, t: f32) -> float4 {
        self + (other - self) * t
    }

    #[inline(always)] pub fn abs(self) -> float4 {
        float4(self.0.abs(), self.1.abs(), self.2.abs(), self.3.abs())
    }

    #[inline(always)] pub fn min(self, other: float4) -> float4 {
        float4(self.0.min(other.0), self.1.min(other.1), self.2.min(other.2), self.3.min(other.3))
    }

    #[inline(always)] pub fn max(self, other: float4) -> float4 {
        float4(self.0.max(other.0), self.1.max(other.1), self.2.max(other.2), self.3.max(other.3))
    }

    #[inline(always)] pub fn sqrt(self) -> float4 {
        float4(self.0.sqrt(), self.1.sqrt(), self.2.sqrt(), self.3.sqrt())
    }

    #[inline(always)] pub fn truncate_to_xyz(self) -> float4 { float4(self.0, self.1, self.2, 0.0) }

    /// Convert to pointer for GPU upload
    #[inline(always)] pub fn as_ptr(&self) -> *const f32 { self as *const float4 as *const f32 }
}

// ---------------------------------------------------------------------------
// float4x4 — column-major 4x4 matrix
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct float4x4(pub [float4; 4]);

impl float4x4 {
    #[inline(always)] pub const fn identity() -> float4x4 {
        float4x4([
            float4(1.0, 0.0, 0.0, 0.0),
            float4(0.0, 1.0, 0.0, 0.0),
            float4(0.0, 0.0, 1.0, 0.0),
            float4(0.0, 0.0, 0.0, 1.0),
        ])
    }

    #[inline(always)] pub fn zero() -> float4x4 { float4x4([float4::zero(); 4]) }

    #[inline(always)] pub fn transpose(&self) -> float4x4 {
        let a = self.0;
        float4x4([
            float4(a[0].0, a[1].0, a[2].0, a[3].0),
            float4(a[0].1, a[1].1, a[2].1, a[3].1),
            float4(a[0].2, a[1].2, a[2].2, a[3].2),
            float4(a[0].3, a[1].3, a[2].3, a[3].3),
        ])
    }

    #[inline(always)] pub fn mul_vec(&self, v: float4) -> float4 {
        let c0 = self.0[0]; let c1 = self.0[1]; let c2 = self.0[2]; let c3 = self.0[3];
        c0 * v.0 + c1 * v.1 + c2 * v.2 + c3 * v.3
    }

    #[inline(always)] pub fn translate(x: f32, y: f32, z: f32) -> float4x4 {
        float4x4([
            float4(1.0, 0.0, 0.0, 0.0),
            float4(0.0, 1.0, 0.0, 0.0),
            float4(0.0, 0.0, 1.0, 0.0),
            float4(x, y, z, 1.0),
        ])
    }

    #[inline(always)] pub fn scale(x: f32, y: f32, z: f32) -> float4x4 {
        float4x4([
            float4(x, 0.0, 0.0, 0.0),
            float4(0.0, y, 0.0, 0.0),
            float4(0.0, 0.0, z, 0.0),
            float4(0.0, 0.0, 0.0, 1.0),
        ])
    }

    #[inline(always)] pub fn rotate_x(angle: f32) -> float4x4 {
        let (s, c) = angle.sin_cos();
        float4x4([
            float4(1.0, 0.0, 0.0, 0.0),
            float4(0.0, c, s, 0.0),
            float4(0.0, -s, c, 0.0),
            float4(0.0, 0.0, 0.0, 1.0),
        ])
    }

    #[inline(always)] pub fn rotate_y(angle: f32) -> float4x4 {
        let (s, c) = angle.sin_cos();
        float4x4([
            float4(c, 0.0, -s, 0.0),
            float4(0.0, 1.0, 0.0, 0.0),
            float4(s, 0.0, c, 0.0),
            float4(0.0, 0.0, 0.0, 1.0),
        ])
    }

    #[inline(always)] pub fn rotate_z(angle: f32) -> float4x4 {
        let (s, c) = angle.sin_cos();
        float4x4([
            float4(c, s, 0.0, 0.0),
            float4(-s, c, 0.0, 0.0),
            float4(0.0, 0.0, 1.0, 0.0),
            float4(0.0, 0.0, 0.0, 1.0),
        ])
    }

    #[inline(always)] pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> float4x4 {
        let f = 1.0 / (fov_y * 0.5).tan();
        let range_inv = 1.0 / (near - far);
        float4x4([
            float4(f / aspect, 0.0, 0.0, 0.0),
            float4(0.0, f, 0.0, 0.0),
            float4(0.0, 0.0, (near + far) * range_inv, -1.0),
            float4(0.0, 0.0, near * far * range_inv * 2.0, 0.0),
        ])
    }

    #[inline(always)] pub fn look_at(eye: float4, target: float4, up: float4) -> float4x4 {
        let f = (target - eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);
        float4x4([
            float4(s.0, u.0, -f.0, 0.0),
            float4(s.1, u.1, -f.1, 0.0),
            float4(s.2, u.2, -f.2, 0.0),
            float4(-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0),
        ])
    }
}

impl Mul for float4x4 {
    type Output = float4x4;
    #[inline(always)]
    fn mul(self, rhs: float4x4) -> float4x4 {
        let a = self; let b = rhs.transpose();
        float4x4([
            float4(a.0[0].dot(b.0[0]), a.0[0].dot(b.0[1]), a.0[0].dot(b.0[2]), a.0[0].dot(b.0[3])),
            float4(a.0[1].dot(b.0[0]), a.0[1].dot(b.0[1]), a.0[1].dot(b.0[2]), a.0[1].dot(b.0[3])),
            float4(a.0[2].dot(b.0[0]), a.0[2].dot(b.0[1]), a.0[2].dot(b.0[2]), a.0[2].dot(b.0[3])),
            float4(a.0[3].dot(b.0[0]), a.0[3].dot(b.0[1]), a.0[3].dot(b.0[2]), a.0[3].dot(b.0[3])),
        ])
    }
}

impl Mul<float4> for float4x4 {
    type Output = float4;
    #[inline(always)] fn mul(self, rhs: float4) -> float4 { self.mul_vec(rhs) }
}

impl Mul<f32> for float4x4 {
    type Output = float4x4;
    #[inline(always)]
    fn mul(self, rhs: f32) -> float4x4 {
        float4x4([self.0[0] * rhs, self.0[1] * rhs, self.0[2] * rhs, self.0[3] * rhs])
    }
}

// ---------------------------------------------------------------------------
// Quaternion
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct Quat(pub float4);

impl Quat {
    #[inline(always)] pub const fn identity() -> Quat { Quat(float4(0.0, 0.0, 0.0, 1.0)) }
    #[inline(always)] pub fn from_axis_angle(axis: float4, angle: f32) -> Quat {
        let half = angle * 0.5;
        let s = half.sin();
        Quat(axis.normalize() * s + float4(0.0, 0.0, 0.0, half.cos()))
    }

    #[inline(always)] pub fn from_euler(x: f32, y: f32, z: f32) -> Quat {
        let (sx, cx) = (x * 0.5).sin_cos();
        let (sy, cy) = (y * 0.5).sin_cos();
        let (sz, cz) = (z * 0.5).sin_cos();
        Quat(float4(
            sx * cy * cz + cx * sy * sz,
            cx * sy * cz - sx * cy * sz,
            cx * cy * sz + sx * sy * cz,
            cx * cy * cz - sx * sy * sz,
        ))
    }

    #[inline(always)] pub fn to_matrix(&self) -> float4x4 {
        let q = self.0;
        let (x2, y2, z2) = (q.0 + q.0, q.1 + q.1, q.2 + q.2);
        let (xx, xy, xz) = (q.0 * x2, q.0 * y2, q.0 * z2);
        let (yy, yz, zz) = (q.1 * y2, q.1 * z2, q.2 * z2);
        let (wx, wy, wz) = (q.3 * x2, q.3 * y2, q.3 * z2);
        float4x4([
            float4(1.0 - (yy + zz), xy + wz, xz - wy, 0.0),
            float4(xy - wz, 1.0 - (xx + zz), yz + wx, 0.0),
            float4(xz + wy, yz - wx, 1.0 - (xx + yy), 0.0),
            float4(0.0, 0.0, 0.0, 1.0),
        ])
    }

    #[inline(always)] pub fn rotate(&self, v: float4) -> float4 {
        let q = self.0;
        let t = q.cross(v) * 2.0;
        v + t * q.3 + q.cross(t)
    }

    #[inline(always)] pub fn slerp(&self, other: Quat, t: f32) -> Quat {
        let mut cos_half = self.0.dot(other.0);
        let mut other_q = other;
        if cos_half < 0.0 { other_q = Quat(-other_q.0); cos_half = -cos_half; }
        if cos_half >= 1.0 { return *self; }
        let half = cos_half.acos();
        let sin_half = half.sin().sqrt();
        if sin_half < 1e-6 { return Quat((self.0 * 0.5 + other_q.0 * 0.5).normalize()); }
        let a = ((1.0 - t) * half).sin() / sin_half;
        let b = (t * half).sin() / sin_half;
        Quat(self.0 * a + other_q.0 * b)
    }
}

impl Mul for Quat {
    type Output = Quat;
    #[inline(always)]
    fn mul(self, rhs: Quat) -> Quat {
        Quat(float4(
            self.0.3 * rhs.0.0 + self.0.0 * rhs.0.3 + self.0.1 * rhs.0.2 - self.0.2 * rhs.0.1,
            self.0.3 * rhs.0.1 + self.0.1 * rhs.0.3 + self.0.2 * rhs.0.0 - self.0.0 * rhs.0.2,
            self.0.3 * rhs.0.2 + self.0.2 * rhs.0.3 + self.0.0 * rhs.0.1 - self.0.1 * rhs.0.0,
            self.0.3 * rhs.0.3 - self.0.0 * rhs.0.0 - self.0.1 * rhs.0.1 - self.0.2 * rhs.0.2,
        ))
    }
}

// ---------------------------------------------------------------------------
// AABB — for broadphase collision
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AABB { pub min: float4, pub max: float4 }

impl AABB {
    #[inline(always)] pub fn new(min: float4, max: float4) -> AABB { AABB { min, max } }
    #[inline(always)] pub fn from_center(center: float4, half_ext: float4) -> AABB {
        AABB { min: center - half_ext, max: center + half_ext }
    }
    #[inline(always)] pub fn intersects(&self, other: &AABB) -> bool {
        self.min.0 <= other.max.0 && self.max.0 >= other.min.0
        && self.min.1 <= other.max.1 && self.max.1 >= other.min.1
        && self.min.2 <= other.max.2 && self.max.2 >= other.min.2
    }
    #[inline(always)] pub fn contains(&self, point: float4) -> bool {
        point.0 >= self.min.0 && point.0 <= self.max.0
        && point.1 >= self.min.1 && point.1 <= self.max.1
        && point.2 >= self.min.2 && point.2 <= self.max.2
    }
    #[inline(always)] pub fn center(&self) -> float4 { (self.min + self.max) * 0.5 }
    #[inline(always)] pub fn half_extents(&self) -> float4 { (self.max - self.min) * 0.5 }
}

// ---------------------------------------------------------------------------
// Ray
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct Ray { pub origin: float4, pub dir: float4 }

impl Ray {
    #[inline(always)] pub fn new(origin: float4, dir: float4) -> Ray { Ray { origin, dir: dir.normalize() } }
    #[inline(always)] pub fn at(&self, t: f32) -> float4 { self.origin + self.dir * t }
    #[inline(always)] pub fn intersect_aabb(&self, aabb: &AABB) -> Option<f32> {
        let inv = float4(1.0, 1.0, 1.0, 0.0) / self.dir;
        let t1 = (aabb.min - self.origin) * inv;
        let t2 = (aabb.max - self.origin) * inv;
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);
        let t0 = tmin.0.max(tmin.1.max(tmin.2));
        let t1 = tmax.0.min(tmax.1.min(tmax.2));
        if t1 >= t0 && t1 >= 0.0 { Some(t0.max(0.0)) } else { None }
    }
}
