use AnchorId;

#[derive(Clone, Debug)]
pub struct Light {
    pub data: LightData,
    anchor: Option<AnchorId>,
}

impl Light {
    pub fn point() -> Light {
        Light {
            data: LightData::Point(PointLight {
                radius: 1.0,
                strength: 1.0,
            }),
            anchor: None,
        }
    }

    pub fn anchor(&self) -> Option<&AnchorId> {
        self.anchor.as_ref()
    }

    pub fn set_anchor(&mut self, anchor_id: AnchorId) {
        self.anchor = Some(anchor_id);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum LightData {
    Point(PointLight),
}

#[derive(Clone, Copy, Debug)]
pub struct PointLight {
    pub radius: f32,
    pub strength: f32,
}
