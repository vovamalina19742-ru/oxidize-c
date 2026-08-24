//! Oxidize-C Transpiled Module
//! Source: tagged_union.c
//! Pattern: Rust Tagged Algebraic Enum

use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub radius: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

/// 100% Safe Algebraic Enum replacing C union and ShapeType discriminator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}

impl Shape {
    #[must_use]
    pub fn circle(radius: f64) -> Self {
        Self::Circle(Circle { radius })
    }

    #[must_use]
    pub fn rectangle(width: f64, height: f64) -> Self {
        Self::Rectangle(Rectangle { width, height })
    }

    #[must_use]
    pub fn calculate_area(&self) -> f64 {
        match self {
            Self::Circle(c) => PI * c.radius * c.radius,
            Self::Rectangle(r) => r.width * r.height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_area_calculation() {
        let circle = Shape::circle(10.0);
        let rect = Shape::rectangle(4.0, 5.0);

        assert!((circle.calculate_area() - 314.1592653589793).abs() < 1e-6);
        assert_eq!(rect.calculate_area(), 20.0);
    }
}
