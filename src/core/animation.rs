#[derive(Debug, Clone, Copy)]
pub struct BasicTileAnimation {
    pub steps: i32,
    pub delay: f32,
}

impl BasicTileAnimation {
    pub fn new_smart(sprite_len: u32, one_frame_size: u32, delay: f32) -> Self {
        let steps = sprite_len / one_frame_size;
        Self {
            steps: steps as i32,
            delay,
        }
    }

    pub fn new(steps: i32, delay: f32) -> Self {
        Self { steps, delay }
    }
}

pub struct AdditionalAnimationDescr {
    pub animation_name: String,
    pub atlas_name: String,
    pub steps: i32,
    pub delay: f32,
}

impl AdditionalAnimationDescr {
    pub fn new(animation_name: String, atlas_name: String, steps: i32, delay: f32) -> Self {
        Self {
            animation_name,
            atlas_name,
            steps,
            delay,
        }
    }
}
