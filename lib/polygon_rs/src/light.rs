use math::Point;

#[derive(Debug)]
pub enum Light {
    Point(PointLight),
}

#[derive(Debug)]
pub struct PointLight {
    pub position: Point,
}
