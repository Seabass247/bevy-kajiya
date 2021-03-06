pub mod camera;
mod frame;
pub mod mesh;
pub mod plugin;
pub mod render_resources;
mod world_renderer;

pub use camera::{EnvironmentSettings, KajiyaCamera, KajiyaCameraBundle};
pub use mesh::{KajiyaMesh, KajiyaMeshInstance, KajiyaMeshInstanceBundle};
pub use plugin::KajiyaRenderPlugin;

const DEFAULT_SCENE_NAME: &str = "car";

#[derive(Clone)]
pub struct KajiyaSceneDescriptor {
    pub scene_name: String,
    pub gi_volume_scale: f32,
}

impl Default for KajiyaSceneDescriptor {
    fn default() -> Self {
        Self {
            scene_name: DEFAULT_SCENE_NAME.to_string(),
            gi_volume_scale: 1.0,
        }
    }
}
