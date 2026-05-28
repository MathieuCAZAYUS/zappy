use crate::module::ui_plugin::local::zappy::graphic::{Color, RenderCommand, TextAlign};
use macroquad::prelude::{Color as MQColor, *};

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
                z_far: 100.0,
            });
        }
    }
    None
}

pub fn render_3d_command(cmd: &RenderCommand) {
    match cmd {
        RenderCommand::Cube(c) => {
            let pos = vec3(c.position.x, c.position.y, c.position.z);
            let size = vec3(c.size.x, c.size.y, c.size.z);
            draw_cube(pos, size, None, to_mq_color(&c.color));
        }
        RenderCommand::Grid3d(g) => {
            draw_grid(
                g.slices,
                g.spacing,
                to_mq_color(&g.color2),
                to_mq_color(&g.color1),
            );
        }
        RenderCommand::Line3d(l) => {
            draw_line_3d(
                vec3(l.start.x, l.start.y, l.start.z),
                vec3(l.end.x, l.end.y, l.end.z),
                to_mq_color(&l.color),
            );
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
