use math::Point;

#[derive(Clone, Copy, Debug)]
pub enum Light {
    Point(PointLight),
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub position: Point,
}
