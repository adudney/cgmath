// Copyright 2014 The CGMath Developers. For a full listing of the authors,
// refer to the Cargo.toml file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt;

use angle::{Angle, Rad};
use approx::ApproxEq;
use matrix::SquareMatrix;
use matrix::{Matrix2, Matrix3};
use num::BaseFloat;
use point::{Point, Point2, Point3};
use quaternion::Quaternion;
use vector::{InnerSpace, Vector2, Vector3};

/// A trait for a generic rotation. A rotation is a transformation that
/// creates a circular motion, and preserves at least one point in the space.
pub trait Rotation<P: Point>: PartialEq + Sized where
    // FIXME: Ugly type signatures - blocked by rust-lang/rust#24092
    Self: ApproxEq<Epsilon = <P as Point>::Scalar>,
    <P as Point>::Scalar: BaseFloat,
{
    /// Create the identity transform (causes no transformation).
    fn one() -> Self;

    /// Create a rotation to a given direction with an 'up' vector
    fn look_at(dir: P::Vector, up: P::Vector) -> Self;

    /// Create a shortest rotation to transform vector 'a' into 'b'.
    /// Both given vectors are assumed to have unit length.
    fn between_vectors(a: P::Vector, b: P::Vector) -> Self;

    /// Rotate a vector using this rotation.
    fn rotate_vector(&self, vec: P::Vector) -> P::Vector;

    /// Rotate a point using this rotation, by converting it to its
    /// representation as a vector.
    #[inline]
    fn rotate_point(&self, point: P) -> P {
        P::from_vec(self.rotate_vector(point.to_vec()))
    }

    /// Create a new rotation which combines both this rotation, and another.
    fn concat(&self, other: &Self) -> Self;

    /// Create a new rotation which "un-does" this rotation. That is,
    /// `r.concat(r.invert())` is the identity.
    fn invert(&self) -> Self;

    /// Modify this rotation in-place by combining it with another.
    #[inline]
    fn concat_self(&mut self, other: &Self) {
        *self = Self::concat(self, other);
    }

    /// Invert this rotation in-place.
    #[inline]
    fn invert_self(&mut self) {
        *self = self.invert();
    }
}

/// A two-dimensional rotation.
pub trait Rotation2<S: BaseFloat>: Rotation<Point2<S>>
                                 + Into<Matrix2<S>>
                                 + Into<Basis2<S>> {
    /// Create a rotation by a given angle. Thus is a redundant case of both
    /// from_axis_angle() and from_euler() for 2D space.
    fn from_angle(theta: Rad<S>) -> Self;
}

/// A three-dimensional rotation.
pub trait Rotation3<S: BaseFloat>: Rotation<Point3<S>>
                                 + Into<Matrix3<S>>
                                 + Into<Basis3<S>>
                                 + Into<Quaternion<S>> {
    /// Create a rotation using an angle around a given axis.
    fn from_axis_angle(axis: Vector3<S>, angle: Rad<S>) -> Self;

    /// Create a rotation from a set of euler angles.
    ///
    /// # Parameters
    ///
    /// - `x`: the angular rotation around the `x` axis (pitch).
    /// - `y`: the angular rotation around the `y` axis (yaw).
    /// - `z`: the angular rotation around the `z` axis (roll).
    fn from_euler(x: Rad<S>, y: Rad<S>, z: Rad<S>) -> Self;

    /// Create a rotation from an angle around the `x` axis (pitch).
    #[inline]
    fn from_angle_x(theta: Rad<S>) -> Self {
        Rotation3::from_axis_angle(Vector3::unit_x(), theta)
    }

    /// Create a rotation from an angle around the `y` axis (yaw).
    #[inline]
    fn from_angle_y(theta: Rad<S>) -> Self {
        Rotation3::from_axis_angle(Vector3::unit_y(), theta)
    }

    /// Create a rotation from an angle around the `z` axis (roll).
    #[inline]
    fn from_angle_z(theta: Rad<S>) -> Self {
        Rotation3::from_axis_angle(Vector3::unit_z(), theta)
    }
}


/// A two-dimensional rotation matrix.
///
/// The matrix is guaranteed to be orthogonal, so some operations can be
/// implemented more efficiently than the implementations for `math::Matrix2`. To
/// enforce orthogonality at the type level the operations have been restricted
/// to a subset of those implemented on `Matrix2`.
///
/// ## Example
///
/// Suppose we want to rotate a vector that lies in the x-y plane by some
/// angle. We can accomplish this quite easily with a two-dimensional rotation
/// matrix:
///
/// ```no_run
/// use cgmath::rad;
/// use cgmath::Vector2;
/// use cgmath::{Matrix, Matrix2};
/// use cgmath::{Rotation, Rotation2, Basis2};
/// use cgmath::ApproxEq;
/// use std::f64;
///
/// // For simplicity, we will rotate the unit x vector to the unit y vector --
/// // so the angle is 90 degrees, or π/2.
/// let unit_x: Vector2<f64> = Vector2::unit_x();
/// let rot: Basis2<f64> = Rotation2::from_angle(rad(0.5f64 * f64::consts::PI));
///
/// // Rotate the vector using the two-dimensional rotation matrix:
/// let unit_y = rot.rotate_vector(unit_x);
///
/// // Since sin(π/2) may not be exactly zero due to rounding errors, we can
/// // use cgmath's approx_eq() feature to show that it is close enough.
/// assert!(unit_y.approx_eq(&Vector2::unit_y()));
///
/// // This is exactly equivalent to using the raw matrix itself:
/// let unit_y2: Matrix2<_> = rot.into();
/// let unit_y2 = unit_y2 * unit_x;
/// assert_eq!(unit_y2, unit_y);
///
/// // Note that we can also concatenate rotations:
/// let rot_half: Basis2<f64> = Rotation2::from_angle(rad(0.25f64 * f64::consts::PI));
/// let unit_y3 = rot_half.concat(&rot_half).rotate_vector(unit_x);
/// assert!(unit_y3.approx_eq(&unit_y2));
/// ```
#[derive(PartialEq, Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct Basis2<S> {
    mat: Matrix2<S>
}

impl<S: BaseFloat> AsRef<Matrix2<S>> for Basis2<S> {
    #[inline]
    fn as_ref(&self) -> &Matrix2<S> {
        &self.mat
    }
}

impl<S: BaseFloat> From<Basis2<S>> for Matrix2<S> {
    #[inline]
    fn from(b: Basis2<S>) -> Matrix2<S> { b.mat }
}

impl<S: BaseFloat> Rotation<Point2<S>> for Basis2<S> {
    #[inline]
    fn one() -> Basis2<S> { Basis2 { mat: Matrix2::identity() } }

    #[inline]
    fn look_at(dir: Vector2<S>, up: Vector2<S>) -> Basis2<S> {
        Basis2 { mat: Matrix2::look_at(dir, up) }
    }

    #[inline]
    fn between_vectors(a: Vector2<S>, b: Vector2<S>) -> Basis2<S> {
        Rotation2::from_angle(Rad::acos(a.dot(b)) )
    }

    #[inline]
    fn rotate_vector(&self, vec: Vector2<S>) -> Vector2<S> { self.mat * vec }

    #[inline]
    fn concat(&self, other: &Basis2<S>) -> Basis2<S> { Basis2 { mat: self.mat * other.mat } }

    #[inline]
    fn concat_self(&mut self, other: &Basis2<S>) { self.mat = self.mat * other.mat; }

    // TODO: we know the matrix is orthogonal, so this could be re-written
    // to be faster
    #[inline]
    fn invert(&self) -> Basis2<S> { Basis2 { mat: self.mat.invert().unwrap() } }

    // TODO: we know the matrix is orthogonal, so this could be re-written
    // to be faster
    #[inline]
    fn invert_self(&mut self) { self.mat.invert_self(); }
}

impl<S: BaseFloat> ApproxEq for Basis2<S> {
    type Epsilon = S;

    #[inline]
    fn approx_eq_eps(&self, other: &Basis2<S>, epsilon: &S) -> bool {
        self.mat.approx_eq_eps(&other.mat, epsilon)
    }
}

impl<S: BaseFloat> Rotation2<S> for Basis2<S> {
    fn from_angle(theta: Rad<S>) -> Basis2<S> { Basis2 { mat: Matrix2::from_angle(theta) } }
}

impl<S: fmt::Debug> fmt::Debug for Basis2<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Basis2 "));
        <[[S; 2]; 2] as fmt::Debug>::fmt(self.mat.as_ref(), f)
    }
}

/// A three-dimensional rotation matrix.
///
/// The matrix is guaranteed to be orthogonal, so some operations, specifically
/// inversion, can be implemented more efficiently than the implementations for
/// `math::Matrix3`. To ensure orthogonality is maintained, the operations have
/// been restricted to a subeset of those implemented on `Matrix3`.
#[derive(PartialEq, Copy, Clone, RustcEncodable, RustcDecodable)]
pub struct Basis3<S> {
    mat: Matrix3<S>
}

impl<S: BaseFloat> Basis3<S> {
    /// Create a new rotation matrix from a quaternion.
    #[inline]
    pub fn from_quaternion(quaternion: &Quaternion<S>) -> Basis3<S> {
        Basis3 { mat: quaternion.clone().into() }
    }
}

impl<S> AsRef<Matrix3<S>> for Basis3<S> {
    #[inline]
    fn as_ref(&self) -> &Matrix3<S> {
        &self.mat
    }
}

impl<S: BaseFloat> From<Basis3<S>> for Matrix3<S> {
    #[inline]
    fn from(b: Basis3<S>) -> Matrix3<S> { b.mat }
}

impl<S: BaseFloat> From<Basis3<S>> for Quaternion<S> {
    #[inline]
    fn from(b: Basis3<S>) -> Quaternion<S> { b.mat.into() }
}

impl<S: BaseFloat> Rotation<Point3<S>> for Basis3<S> {
    #[inline]
    fn one() -> Basis3<S> { Basis3 { mat: Matrix3::identity() } }

    #[inline]
    fn look_at(dir: Vector3<S>, up: Vector3<S>) -> Basis3<S> {
        Basis3 { mat: Matrix3::look_at(dir, up) }
    }

    #[inline]
    fn between_vectors(a: Vector3<S>, b: Vector3<S>) -> Basis3<S> {
        let q: Quaternion<S> = Rotation::between_vectors(a, b);
        q.into()
    }

    #[inline]
    fn rotate_vector(&self, vec: Vector3<S>) -> Vector3<S> { self.mat * vec }

    #[inline]
    fn concat(&self, other: &Basis3<S>) -> Basis3<S> { Basis3 { mat: self.mat * other.mat } }

    #[inline]
    fn concat_self(&mut self, other: &Basis3<S>) { self.mat = self.mat * other.mat; }

    // TODO: we know the matrix is orthogonal, so this could be re-written
    // to be faster
    #[inline]
    fn invert(&self) -> Basis3<S> { Basis3 { mat: self.mat.invert().unwrap() } }

    // TODO: we know the matrix is orthogonal, so this could be re-written
    // to be faster
    #[inline]
    fn invert_self(&mut self) { self.mat.invert_self(); }
}

impl<S: BaseFloat> ApproxEq for Basis3<S> {
    type Epsilon = S;

    #[inline]
    fn approx_eq_eps(&self, other: &Basis3<S>, epsilon: &S) -> bool {
        self.mat.approx_eq_eps(&other.mat, epsilon)
    }
}

impl<S: BaseFloat> Rotation3<S> for Basis3<S> {
    fn from_axis_angle(axis: Vector3<S>, angle: Rad<S>) -> Basis3<S> {
        Basis3 { mat: Matrix3::from_axis_angle(axis, angle) }
    }

    fn from_euler(x: Rad<S>, y: Rad<S>, z: Rad<S>) -> Basis3<S> {
        Basis3 { mat: Matrix3::from_euler(x, y ,z) }
    }

    fn from_angle_x(theta: Rad<S>) -> Basis3<S> {
        Basis3 { mat: Matrix3::from_angle_x(theta) }
    }

    fn from_angle_y(theta: Rad<S>) -> Basis3<S> {
        Basis3 { mat: Matrix3::from_angle_y(theta) }
    }

    fn from_angle_z(theta: Rad<S>) -> Basis3<S> {
        Basis3 { mat: Matrix3::from_angle_z(theta) }
    }
}

impl<S: fmt::Debug> fmt::Debug for Basis3<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "Basis3 "));
        <[[S; 3]; 3] as fmt::Debug>::fmt(self.mat.as_ref(), f)
    }
}
