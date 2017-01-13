use anchor::AnchorId;
use math::{Color, Vector3};

#[derive(Clone, Copy, Debug)]
pub struct Light {
    pub data: LightData,
    pub color: Color,
    pub strength: f32,
    anchor: Option<AnchorId>,
}

impl Light {
    pub fn point(radius: f32, strength: f32, color: Color) -> Light {
        Light {
            data: LightData::Point { radius: radius },
            color: color,
            strength: strength,
            anchor: None,
        }
    }

    pub fn directional(direction: Vector3, strength: f32, color: Color) -> Light {
        Light {
            data: LightData::Directional { direction: direction.normalized() },
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
    Point { radius: f32 },
    Directional { direction: Vector3 },
}

/// Identifies a light that has been registered with the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct LightId(usize);
derive_Counter!(LightId);
