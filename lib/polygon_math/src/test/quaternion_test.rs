use std::f32::consts::PI;

use quaternion::Quaternion;
use vector::Vector3;
use matrix::Matrix4;

#[test]
fn multiplication() {
    // Test that multiplication against the identity quaternion does yields the correct result.
    let identity = Quaternion::identity();
    assert_eq!(identity * identity, identity);

    let quat = Quaternion::axis_angle(Vector3::new(1.0, 0.0, 0.0), PI);
    assert_eq!(identity * quat, quat);
    assert_eq!(quat * identity, quat);
}

#[test]
fn as_matrix() {
    assert_eq!(Quaternion::identity().as_matrix(), Matrix4::identity());

    assert_eq!(Quaternion::axis_angle(Vector3::new(1.0, 0.0, 0.0), PI).as_matrix(), Matrix4::rotation(PI, 0.0, 0.0));
    assert_eq!(Quaternion::axis_angle(Vector3::new(0.0, 1.0, 0.0), PI).as_matrix(), Matrix4::rotation(0.0, PI, 0.0));
    assert_eq!(Quaternion::axis_angle(Vector3::new(0.0, 0.0, 1.0), PI).as_matrix(), Matrix4::rotation(0.0, 0.0, PI));

    assert_eq!(Quaternion::axis_angle(Vector3::new(1.0, 0.0, 0.0), PI * 0.5).as_matrix(), Matrix4::rotation(PI * 0.5, 0.0, 0.0));
    assert_eq!(Quaternion::axis_angle(Vector3::new(0.0, 1.0, 0.0), PI * 0.5).as_matrix(), Matrix4::rotation(0.0, PI * 0.5, 0.0));
    assert_eq!(Quaternion::axis_angle(Vector3::new(0.0, 0.0, 1.0), PI * 0.5).as_matrix(), Matrix4::rotation(0.0, 0.0, PI * 0.5));

    assert_eq!(Quaternion::axis_angle(Vector3::new(1.0, 0.0, 0.0), 0.5).as_matrix(), Matrix4::rotation(0.5, 0.0, 0.0));
    assert_eq!(Quaternion::axis_angle(Vector3::new(0.0, 1.0, 0.0), 0.5).as_matrix(), Matrix4::rotation(0.0, 0.5, 0.0));
    assert_eq!(Quaternion::axis_angle(Vector3::new(0.0, 0.0, 1.0), 0.5).as_matrix(), Matrix4::rotation(0.0, 0.0, 0.5));
}
