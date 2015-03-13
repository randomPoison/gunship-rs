use point::Point;

pub struct Mesh {
    pub vertices: Vec<Point>
}

impl Mesh {
    pub fn new() -> Mesh {
        Mesh {
            vertices: Vec::new()
        }
    }

    pub fn from_slice(data: &[Point]) -> Mesh {
        let mut vec_data: Vec<Point> = Vec::new();
        for point in data {
            vec_data.push(Point {
                x: point.x,
                y: point.y,
                z: point.z,
                w: point.w
            });
        }
        Mesh {
            vertices: vec_data
        }
    }
}
