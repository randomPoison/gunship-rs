use anchor::AnchorId;
use math::Color;

#[derive(Clone, Debug)]
pub struct Light {
    pub data: LightData,
    pub color: Color,
    pub strength: f32,
    anchor: Option<AnchorId>,
}

impl Light {
    pub fn point(radius: f32, strength: f32, color: Color) -> Light {
        Light {
            data: LightData::Point(PointLight {
                radius: radius,
            }),
            color: color,
            strength: strength,
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
}

/// Identifies a light that has been registered with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LightId(usize);
derive_Counter!(LightId);
