use std::fs::File;

use bevy::{
    prelude::{
        Bundle, Changed, Commands, Component, Entity, NonSendMut, Query, Res, ResMut, Transform,
        With,
    },
    utils::HashMap,
};
use glam::{Quat, Vec2, Vec3};
use kajiya::{
    camera::{CameraLens, LookThroughCamera},
    frame_desc::WorldFrameDesc,
    world_renderer::{AddMeshOptions, InstanceHandle, MeshHandle, WorldRenderer},
};

use crate::{
    camera::{ExtractedCamera, ExtractedEnvironment, KajiyaCamera},
    mesh::{MeshInstanceExtracted, RenderInstance, RenderInstances, MeshInstanceExtractedBundle, MeshInstanceType},
    plugin::RenderWorld,
    renderer::{KajiyaRenderers, RenderContext},
    KajiyaSceneDescriptor, KajiyaMesh,
};

#[derive(serde::Deserialize)]
pub struct SceneDesc {
    pub instances: Vec<SceneInstanceDesc>,
}

#[derive(serde::Deserialize)]
pub struct SceneInstanceDesc {
    pub position: [f32; 3],
    pub mesh: String,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct SunState {
    pub(crate) theta: f32,
    pub(crate) phi: f32,
}

impl SunState {
    pub fn direction(&self) -> Vec3 {
        fn spherical_to_cartesian(theta: f32, phi: f32) -> Vec3 {
            let x = phi.sin() * theta.cos();
            let y = phi.cos();
            let z = phi.sin() * theta.sin();
            Vec3::new(x, y, z)
        }

        spherical_to_cartesian(self.theta, self.phi)
    }
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LocalLightsState {
    pub theta: f32,
    pub phi: f32,
    pub count: u32,
    pub distance: f32,
    pub multiplier: f32,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SceneViewState {
    pub camera_position: Vec3,
    pub camera_rotation: Quat,
    pub vertical_fov: f32,
    pub emissive_multiplier: f32,
    pub sun: SunState,
    pub lights: LocalLightsState,
    pub ev_shift: f32,
}

#[derive(Component)]
pub struct MeshInstanceHandles {
    instance_handle: Option<InstanceHandle>,
    mesh_handle: Option<MeshHandle>,
}

#[derive(Component)]
pub struct MeshInstanceTransform {
    pub transform: Option<(Vec3, Quat)>,
}

#[derive(Bundle)]
pub struct SceneMeshInstanceExtracted {
    pub handles: MeshInstanceHandles,
    pub transform: MeshInstanceTransform,
    pub baked: MeshInstanceBaked,
}

#[derive(Component)]
pub struct MeshInstanceBaked {
    file_name: String,
}

pub fn setup_scene_view(
    mut commands: Commands,
    wr_res: NonSendMut<KajiyaRenderers>,
    scene: Res<KajiyaSceneDescriptor>,
    render_context: Res<RenderContext>,
) {
    let scene_file = format!("assets/scenes/{}.ron", scene.scene_name);
    let scene_desc: SceneDesc = ron::de::from_reader(
        File::open(&scene_file).expect("Kajiya error: Could not open scene description file"),
    )
    .expect("Kajiya error: Could not read description file");
    let mut world_renderer = wr_res.world_renderer.lock().unwrap();

    world_renderer.world_gi_scale = scene.gi_volume_scale;

    let mut scene_instances = Vec::new();
    for (indx, instance) in scene_desc.instances.iter().enumerate() {
        let position = instance.position.into();
        let rotation = Quat::IDENTITY;

        scene_instances.push(MeshInstanceExtractedBundle {
            mesh_instance: MeshInstanceExtracted {
                instance_type: MeshInstanceType::SceneInstanced(indx),
                mesh_name: instance.mesh.clone(),
                transform: (position, rotation),
            },
        });
    }

    let extracted_camera = ExtractedCamera {
        camera: KajiyaCamera {
            aspect_ratio: render_context.aspect_ratio(),
            ..Default::default()
        },
        ..Default::default()
    };

    let lens = CameraLens {
        aspect_ratio: extracted_camera.camera.aspect_ratio,
        vertical_fov: extracted_camera.camera.vertical_fov,
        near_plane_distance: extracted_camera.camera.near_plane_distance,
    };
    let frame_desc = WorldFrameDesc {
        camera_matrices: extracted_camera.transform.through(&lens),
        render_extent: render_context.render_extent,
        sun_direction: extracted_camera.environment.sun_theta_phi.direction(),
    };

    let render_instances = RenderInstances {
        user_instances: HashMap::default(),
        scene_instances: HashMap::default(),
    };

    commands.spawn_batch(scene_instances);
    commands.insert_resource(render_instances);
    commands.insert_resource(frame_desc);
    commands.insert_resource(extracted_camera);
}

pub fn update_scene_view(
    wr_res: NonSendMut<KajiyaRenderers>,
    mut frame_desc: ResMut<WorldFrameDesc>,
    extracted_camera: Res<ExtractedCamera>,
    mut render_instances: ResMut<RenderInstances>,
    query_extracted_instances: Query<&MeshInstanceExtracted>,
) {
    let mut world_renderer = wr_res.world_renderer.lock().unwrap();

    for extracted_instance in query_extracted_instances.iter() {
        let (new_pos, new_rot) = extracted_instance.transform;
        match &extracted_instance.instance_type {
            MeshInstanceType::UserInstanced(entity) => {
                if let Some(render_instance) = render_instances.user_instances.get(&entity) {
                    println!(
                        "UPDATE MESH {} TRANSFORM {:?}",
                        extracted_instance.mesh_name,
                        (new_pos, new_rot)
                    );
        
                    world_renderer.set_instance_transform(
                        render_instance.instance_handle,
                        new_pos,
                        new_rot,
                    );
                } else {
                    let mesh = world_renderer
                        .add_baked_mesh(
                            format!("/baked/{}.mesh", extracted_instance.mesh_name),
                            AddMeshOptions::new(),
                        )
                        .expect(&format!(
                            "Kajiya error: could not find baked mesh {}",
                            extracted_instance.mesh_name
                        ));
        
                    render_instances.user_instances.insert(
                        *entity,
                        RenderInstance {
                            instance_handle: world_renderer.add_instance(mesh, new_pos, new_rot),
                            transform: (new_pos, new_rot),
                        },
                    );

                    println!(
                        "ADD MESH {} with trans {:?}",
                        extracted_instance.mesh_name,
                        (new_pos, new_rot)
                    );
                }
            },
            MeshInstanceType::SceneInstanced(mesh_indx) => {
                if let Some(render_instance) = render_instances.scene_instances.get(&mesh_indx) {
                    println!(
                        "UPDATE SCENE MESH {} TRANSFORM {:?}",
                        extracted_instance.mesh_name,
                        (new_pos, new_rot)
                    );
        
                    world_renderer.set_instance_transform(
                        render_instance.instance_handle,
                        new_pos,
                        new_rot,
                    );
                } else {
                    let mesh = world_renderer
                        .add_baked_mesh(
                            format!("/baked/{}.mesh", extracted_instance.mesh_name),
                            AddMeshOptions::new(),
                        )
                        .expect(&format!(
                            "Kajiya error: could not find baked mesh {}",
                            extracted_instance.mesh_name
                        ));
        
                    render_instances.scene_instances.insert(
                        *mesh_indx,
                        RenderInstance {
                            instance_handle: world_renderer.add_instance(mesh, new_pos, new_rot),
                            transform: (new_pos, new_rot),
                        },
                    );

                    println!(
                        "ADD SCENE MESH {} with trans {:?}",
                        extracted_instance.mesh_name,
                        (new_pos, new_rot)
                    );
                }   
            },
        }
    }

    // Update WorldFrameDescription
    let lens = CameraLens {
        aspect_ratio: extracted_camera.camera.aspect_ratio,
        vertical_fov: extracted_camera.camera.vertical_fov,
        near_plane_distance: extracted_camera.camera.near_plane_distance,
    };
    frame_desc.camera_matrices = extracted_camera.transform.through(&lens);
    frame_desc.sun_direction = extracted_camera.environment.sun_theta_phi.direction();
}
