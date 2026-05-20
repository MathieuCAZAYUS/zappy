wit_bindgen::generate!({
    path: "../wit",
    world: "plugin-world",
});

struct PluginCube;

impl Guest for PluginCube {
    fn update_cube(state: CubeState) -> RenderCommand {
        let speed = 2.0;
        let angle = state.time * speed;

        RenderCommand {
            x: angle.cos() * 150.0,
            y: angle.sin() * 150.0,
            rotation: angle,
        }
    }
}

export!(PluginCube);
