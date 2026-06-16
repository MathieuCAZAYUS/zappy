use crate::{
    log_all,
    manager::ModuleManager,
    model_loader::TextureRegistry,
    module::ui_plugin::local::zappy::graphic::{Color, RenderCommand, TextAlign},
};
use colored::Colorize;
use macroquad::prelude::{Color as MQColor, *};

const NORMAL_W: f32 = 0.0;

pub fn to_mq_color(c: &Color) -> MQColor {
    MQColor::from_rgba(c.r, c.g, c.b, c.a)
}

pub fn extract_camera(cmds: &[RenderCommand]) -> Option<Camera3D> {
    for cmd in cmds {
        if let RenderCommand::Camera(c) = cmd {
            return Some(Camera3D {
                position: vec3(c.position.x, c.position.y, c.position.z),
                target: vec3(c.target.x, c.target.y, c.target.z),
                up: vec3(c.up.x, c.up.y, c.up.z),
                fovy: c.fovy,
                aspect: None,
                projection: Projection::Perspective,
                render_target: None,
                viewport: None,
                z_near: 0.1,
                z_far: 10000.0,
            });
        }
    }
    None
}

fn convert_vertex(
    v: &crate::module::ui_plugin::local::zappy::graphic::Vertex,
) -> macroquad::models::Vertex {
    macroquad::models::Vertex {
        position: vec3(v.position.x, v.position.y, v.position.z),
        normal: macroquad::math::Vec4::new(v.normal.x, v.normal.y, v.normal.z, NORMAL_W),
        uv: vec2(v.uv.x, v.uv.y),
        color: [v.color.r, v.color.g, v.color.b, v.color.a],
    }
}

pub fn render_3d_command(manager: &mut ModuleManager, cmd: &RenderCommand, reg: &TextureRegistry) {
    match cmd {
        RenderCommand::Cube(c) => {
            let pos = vec3(c.position.x, c.position.y, c.position.z);
            let size = vec3(c.size.x, c.size.y, c.size.z);
            draw_cube(pos, size, None, to_mq_color(&c.color));
            /*log_all!(
                manager,
                "{} {} {} {} {} {} {} {}",
                "[DRAW]".bright_green().bold(),
                "cube".bright_blue().bold().italic(),
                "at position:".bright_black(),
                format!("{:?}", pos).bright_black().underline(),
                "with size:".bright_black(),
                format!("{:?}", size).bright_black().underline(),
                "and color:".bright_black(),
                format!("{:?}", c.color).bright_black().underline(),
            );*/
        }
        RenderCommand::InstancedCubes(ic) => {
            for c in &ic.cubes {
                let pos = vec3(c.position.x, c.position.y, c.position.z);
                let size = vec3(c.size.x, c.size.y, c.size.z);
                draw_cube(pos, size, None, to_mq_color(&c.color));
                /*log_all!(
                    manager,
                    "{} {} {} {} {} {} {} {}",
                    "[DRAW]".bright_green().bold(),
                    "cube".bright_blue().bold().italic(),
                    "position =".bright_black(),
                    format!("{:?}", pos).bright_black().underline(),
                    "size =".bright_black(),
                    format!("{:?}", size).bright_black().underline(),
                    "color =".bright_black(),
                    format!("{:?}", c.color).bright_black().underline(),
                );*/
            }
        }
        RenderCommand::Grid3d(g) => {
            draw_grid(
                g.slices,
                g.spacing,
                to_mq_color(&g.color2),
                to_mq_color(&g.color1),
            );
            /*log_all!(
                manager,
                "{} {} {} {} {} {} {} {}",
                "[DRAW]".bright_green().bold(),
                "grid3d".bright_blue().bold().italic(),
                "slices =".bright_black(),
                format!("{}", g.slices).bright_black().underline(),
                "spacing =".bright_black(),
                format!("{}", g.spacing).bright_black().underline(),
                "axes_color & other_color =".bright_black(),
                format!("{:?} {:?}", g.color2, g.color1)
                    .bright_black()
                    .underline(),
            );*/
        }
        RenderCommand::Line3d(l) => {
            draw_line_3d(
                vec3(l.start.x, l.start.y, l.start.z),
                vec3(l.end.x, l.end.y, l.end.z),
                to_mq_color(&l.color),
            );
            /*log_all!(
                manager,
                "{} {} {} {} {} {} {} {}",
                "[DRAW]".bright_green().bold(),
                "line3d".bright_blue().bold().italic(),
                "start =".bright_black(),
                format!("{:?}", l.start).bright_black().underline(),
                "end =".bright_black(),
                format!("{:?}", l.end).bright_black().underline(),
                "color =".bright_black(),
                format!("{:?}", l.color).bright_black().underline(),
            );*/
        }
        RenderCommand::Mesh3d(m) => {
            let vertices: Vec<macroquad::models::Vertex> =
                m.vertices.iter().map(convert_vertex).collect();

            /*log_all!(
                manager,
                "{} {} {} {} {} {} {} {}",
                "[DRAW]".bright_green().bold(),
                "mesh".bright_blue().bold().italic(),
                "vertices =".bright_black(),
                format!("{:?}", &vertices).bright_black().underline(),
                "indices =".bright_black(),
                format!("{:?}", m.indices).bright_black().underline(),
                "texture =".bright_black(),
                format!("{:?}", None::<Texture2D>)
                    .bright_black()
                    .underline(),
            );*/

            if vertices.len() >= u16::MAX as usize {
                log_all!(
                    manager,
                    "{} {} {} {}",
                    "[WARN]".bright_yellow().bold(),
                    "vertices exceeded max drawcall size, clamping (".bright_black(),
                    format!("{}", vertices.len()).bright_black().underline(),
                    ").".bright_black(),
                );
            }

            if m.indices.len() >= u16::MAX as usize {
                log_all!(
                    manager,
                    "{} {} {} {}",
                    "[WARN]".bright_yellow().bold(),
                    "indices exceeded max drawcall size, clamping (".bright_black(),
                    format!("{}", m.indices.len()).bright_black().underline(),
                    ").".bright_black(),
                );
            }

            let mesh = macroquad::models::Mesh {
                vertices,
                indices: m.indices.clone(),
                texture: reg.get(m.texture_id).cloned(),
            };
            draw_mesh(&mesh);
        }
        _ => {}
    }
}

pub fn render_2d_command(cmd: &RenderCommand) {
    match cmd {
        RenderCommand::Rect(r) => {
            draw_rectangle_ex(
                r.x,
                r.y,
                r.w,
                r.h,
                DrawRectangleParams {
                    rotation: r.rotation,
                    color: to_mq_color(&r.color),
                    ..Default::default()
                },
            );
        }
        RenderCommand::Text(t) => {
            let dimensions = measure_text(&t.text, None, t.size as u16, 1.0);

            let final_x = match t.align {
                TextAlign::Left => t.x,
                TextAlign::Right => t.x - dimensions.width,
                TextAlign::Center => t.x - (dimensions.width / 2.0),
            };
            draw_text(&t.text, final_x, t.y, t.size, to_mq_color(&t.color));
        }
        _ => {}
    }
}
