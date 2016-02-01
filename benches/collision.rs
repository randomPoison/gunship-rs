#![feature(test)]

extern crate gunship;
extern crate rand;
extern crate test;

use gunship::*;
use gunship::component::collider::{Sphere, OBB};
use gunship::component::collider::bounding_volume::AABB;
use std::f32::consts::PI;
use test::Bencher;

#[bench]
fn sphere_sphere_x100(bencher: &mut Bencher) {
    let mut spheres = Vec::new();
    for _ in 0..10 {
        spheres.push(Sphere {
            center: Point::new(
                random_range(-5.0, 5.0),
                random_range(-5.0, 5.0),
                random_range(-5.0, 5.0),
            ),
            radius: random_range(0.5, 2.0),
        });
    }

    bencher.iter(|| {
        for lhs in &spheres {
            for rhs in &spheres {
                test::black_box(lhs.test_sphere(rhs));
            }
        }
    });
}

#[bench]
fn obb_obb_x100(bencher: &mut Bencher) {
    let mut obbs = Vec::new();
    for _ in 0..10 {
        obbs.push(OBB {
            center: Point::new(
                random_range(-5.0, 5.0),
                random_range(-5.0, 5.0),
                random_range(-5.0, 5.0),
            ),
            orientation: Matrix3::rotation(
                random_range(0.0, PI),
                random_range(0.0, PI),
                random_range(0.0, PI),
            ),
            half_widths: Vector3::new(
                random_range(0.5, 2.0),
                random_range(0.5, 2.0),
                random_range(0.5, 2.0),
            ),
        });
    }

    bencher.iter(|| {
        for lhs in &obbs {
            for rhs in &obbs {
                test::black_box(lhs.test_obb(rhs));
            }
        }
    });
}

#[bench]
fn sphere_obb_x100(bencher: &mut Bencher) {
        let mut spheres = Vec::new();
        for _ in 0..10 {
            spheres.push(Sphere {
                center: Point::new(
                    random_range(-5.0, 5.0),
                    random_range(-5.0, 5.0),
                    random_range(-5.0, 5.0),
                ),
                radius: random_range(0.5, 2.0),
            });
        }

        let mut obbs = Vec::new();
        for _ in 0..10 {
            obbs.push(OBB {
                center: Point::new(
                    random_range(-5.0, 5.0),
                    random_range(-5.0, 5.0),
                    random_range(-5.0, 5.0),
                ),
                orientation: Matrix3::rotation(
                    random_range(0.0, PI),
                    random_range(0.0, PI),
                    random_range(0.0, PI),
                ),
                half_widths: Vector3::new(
                    random_range(0.5, 2.0),
                    random_range(0.5, 2.0),
                    random_range(0.5, 2.0),
                ),
            });
        }

        bencher.iter(|| {
            for lhs in &spheres {
                for rhs in &obbs {
                    test::black_box(lhs.test_obb(rhs));
                }
            }
        });
}

#[bench]
fn abb_abb_x100(bencher: &mut Bencher) {
    let mut aabbs = Vec::new();
    for _ in 0..10 {
        let min = Point::new(
            random_range(-5.0, 5.0),
            random_range(-5.0, 5.0),
            random_range(-5.0, 5.0),
        );
        let offset = Vector3::new(
            random_range(0.1, 2.0),
            random_range(0.1, 2.0),
            random_range(0.1, 2.0),
        );
        aabbs.push(AABB {
            min: min,
            max: min + offset,
        });
    }

    bencher.iter(|| {
        for lhs in &aabbs {
            for rhs in &aabbs {
                test::black_box(lhs.test_aabb(rhs));
            }
        }
    });
}

fn random_range(min: f32, max: f32) -> f32 {
    let range = max - min;
    rand::random::<f32>() * range + min
}
