<div align="center">

# 🕊️💡 bevy-kajiya 
**A plugin that enables bevy to use the kajiya renderer**
</div>


**WARNING**: This plugin is barebones and supports a limited set of features. Please go [read more about kajiya](https://github.com/EmbarkStudios/kajiya) for context.  It is an *experiment* in Rust rendering, not intended to be a fully featured renderer.  

Yes, you can visualize some bevy entities in ray-traced glory, but don't expect much else for now; there is a finite number of meshes that can be instanced and meshes cannot be uninstanced yet.  Expect some bugs and crashes!

**NOTE**: Only tested on Windows, using the Bevy on latest `main`.


# Example

See [bevy-kajiya-playground](https://github.com/Seabass247/bevy-kajiya-playground) for basic usage of `bevy` and `bevy-kajiya` in a bevy app.  You can fly around the scene in first person and manipulate the sun.

# Usage

You must disable the default bevy renderer.  Additionally, a patch is required for ray-tracing extensions. Put the following in your top-level `Cargo.toml`:

```
[dependencies]
bevy-kajiya = { git = "https://github.com/Seabass247/bevy-kajiya" }
bevy = { git = "https://github.com/bevyengine/bevy", default-features = false, features = ["bevy_winit"] }

[patch.crates-io]
# Official ray-tracing extensions
rspirv = { git = "https://github.com/gfx-rs/rspirv.git", rev = "dae552c" }
spirv_headers = { git = "https://github.com/gfx-rs/rspirv.git", rev = "dae552c" }

```

`kajiya` does not support resizable windows yet.  Make sure to use these window settings (changing window width/height is fine):
```
    .insert_resource(WindowDescriptor {
        title: "Bevy Kajiya Playground".to_string(),
        width: 1920.,
        height: 1080.,
        vsync: false,
        resizable: false,
        scale_factor_override: Some(1.0),
        ..Default::default()
    })
```

## Scenes
You specify the scene to be loaded on startup with the `KajiyaSceneDescriptor` resource inserted in `App::new()`.  The scene as specified by `"my-scene"` should be located in `assets/scenes/my-scene.ron`

```
    .insert_resource(KajiyaSceneDescriptor {
        scene_name: "my_scene".to_string(),
        ..Default::default()
    })
```

### Scene Format

```
(
    instances: [
        (
            position: (0, -0.001, 0),
            mesh: "336_lrm",
        ),
        (
            position: (0, 0, 0),
            mesh: "floor",
        ),
    ]
)
```

## Meshes

You must run `bake.cmd` any time mesh assets have been modified (or if first time building).  Adding new meshes requires adding a line in `bake.cmd`:

```
%BAKE% --scene "assets/meshes/my_new_mesh/scene.gltf" --scale 1.0 -o my_new_mesh
```
