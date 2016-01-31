use math::Point;

#[derive(Clone, Copy, Debug)]
pub enum Light {
    Point(PointLight),
}

impl Light {
    pub fn point() -> Light {
        Light::Point(PointLight {
            position: Point::origin(),
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Point,
}
