/* LICENSE BEGIN
    This file is part of the SixtyFPS Project -- https://sixtyfps.io
    Copyright (c) 2020 Olivier Goffart <olivier.goffart@sixtyfps.io>
    Copyright (c) 2020 Simon Hausmann <simon.hausmann@sixtyfps.io>

    SPDX-License-Identifier: GPL-3.0-only
    This file is also available under commercial licensing terms.
    Please contact info@sixtyfps.io for more information.
LICENSE END */
/*!
This module contains brush related types for the run-time library.
*/

use super::{Color, Point};
use crate::properties::InterpolatedPropertyValue;
use crate::SharedVector;

/// A brush is a data structure that is used to describe how
/// a shape, such as a rectangle, path or even text, shall be filled.
/// A brush can also be applied to the outline of a shape, that means
/// the fill of the outline itself.
#[derive(Clone, PartialEq, Debug)]
#[repr(C)]
pub enum Brush {
    /// The brush will not produce any fill.
    NoBrush,
    /// The color variant of brush is a plain color that is to be used for the fill.
    SolidColor(Color),
    /// The linear gradient variant of a brush describes the gradient stops for a fill
    /// where all color stops are along a line that's rotated by the specified angle.
    LinearGradient(LinearGradientBrush),
}

impl Default for Brush {
    fn default() -> Self {
        Self::NoBrush
    }
}

impl std::fmt::Display for Brush {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Brush::NoBrush => write!(f, "Brush::NoBrush()"),
            Brush::SolidColor(col) => write!(f, "Brush::SolidColor({})", col),
            Brush::LinearGradient(_) => write!(f, "Brush::LinearGradient(todo)"),
        }
    }
}

impl Brush {
    /// If the brush is NoBrush, the default constructed color is returned.
    /// If the brush is SolidColor, the contained color is returned.
    /// If the brush is a LinearGradient, the color of the first stop is returned.
    pub fn color(&self) -> Color {
        match self {
            Brush::NoBrush => Default::default(),
            Brush::SolidColor(col) => *col,
            Brush::LinearGradient(gradient) => {
                gradient.stops().next().map(|stop| stop.color).unwrap_or_default()
            }
        }
    }
}

/// The LinearGradientBrush describes a way of filling a shape with different colors, which
/// are interpolated between different stops. The colors are aligned with a line that's rotated
/// by the LinearGradient's angle.
#[derive(Clone, PartialEq, Debug)]
#[repr(transparent)]
pub struct LinearGradientBrush(SharedVector<GradientStop>);

impl LinearGradientBrush {
    /// Creates a new linear gradient, described by the specified angle and the provided color stops.
    pub fn new(angle: f32, stops: impl IntoIterator<Item = GradientStop>) -> Self {
        let stop_iter = stops.into_iter();
        let mut encoded_angle_and_stops = SharedVector::with_capacity(stop_iter.size_hint().0 + 1);
        // The gradient's first stop is a fake stop to store the angle
        encoded_angle_and_stops.push(GradientStop { color: Default::default(), position: angle });
        encoded_angle_and_stops.extend(stop_iter);
        Self(encoded_angle_and_stops)
    }
    /// Returns the angle of the linear gradient in degrees.
    pub fn angle(&self) -> f32 {
        self.0[0].position
    }
    /// Returns the color stops of the linear gradient.
    pub fn stops<'a>(&'a self) -> impl Iterator<Item = &'a GradientStop> + 'a {
        // skip the first fake stop that just contains the angle
        self.0.iter().skip(1)
    }

    /// Returns the start / end points of the gradient within the [-0.5; 0.5] unit square, based on the angle.
    pub fn start_end_points(&self) -> (Point, Point) {
        let angle = self.angle().to_radians();
        let r = (angle.sin().abs() + angle.cos().abs()) / 2.;
        let (y, x) = (angle - std::f32::consts::PI / 2.).sin_cos();
        let (y, x) = (y * r, x * r);
        let start = Point::new(0.5 - x, 0.5 - y);
        let end = Point::new(0.5 + x, 0.5 + y);
        (start, end)
    }
}

/// GradientStop describes a single color stop in a gradient. The colors between multiple
/// stops are interpolated.
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub struct GradientStop {
    /// The color to draw at this stop.
    pub color: Color,
    /// The position of this stop on the entire shape, as a normalized value between 0 and 1.
    pub position: f32,
}

impl InterpolatedPropertyValue for Brush {
    fn interpolate(&self, target_value: &Self, t: f32) -> Self {
        match (self, target_value) {
            (Brush::NoBrush, Brush::NoBrush) => Brush::NoBrush,
            (Brush::NoBrush, Brush::SolidColor(col)) => {
                Brush::SolidColor(Color::default().interpolate(col, t))
            }
            (Brush::NoBrush, Brush::LinearGradient(_)) => unimplemented!(),
            (Brush::SolidColor(_), Brush::NoBrush) => unimplemented!(),
            (Brush::SolidColor(source_col), Brush::SolidColor(target_col)) => {
                Brush::SolidColor(source_col.interpolate(target_col, t))
            }
            (Brush::SolidColor(_), Brush::LinearGradient(_)) => unimplemented!(),
            (Brush::LinearGradient(_), Brush::NoBrush) => unimplemented!(),
            (Brush::LinearGradient(_), Brush::SolidColor(_)) => unimplemented!(),
            (Brush::LinearGradient(_), Brush::LinearGradient(_)) => unimplemented!(),
        }
    }
}

#[test]
fn test_linear_gradient_encoding() {
    let stops: SharedVector<GradientStop> = [
        GradientStop { position: 0.0, color: Color::from_argb_u8(255, 255, 0, 0) },
        GradientStop { position: 0.5, color: Color::from_argb_u8(255, 0, 255, 0) },
        GradientStop { position: 1.0, color: Color::from_argb_u8(255, 0, 0, 255) },
    ]
    .into();
    let grad = LinearGradientBrush::new(256., stops.clone());
    assert_eq!(grad.angle(), 256.);
    assert!(grad.stops().eq(stops.iter()));
}
