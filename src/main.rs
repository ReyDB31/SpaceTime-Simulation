use macroquad::prelude::*;

const MAX_BODIES: usize = 8;

#[derive(Clone, Copy)]
struct Body {
    pos: Vec2,   // normalized screen coordinates, origin at top-left
    radius: f32, // fraction of screen height
    mass: f32,   // 0.25..2.5
}

const VERTEX_SHADER: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
uniform mat4 Model;
uniform mat4 Projection;
varying vec2 uv;
void main() {
    gl_Position = Projection * Model * vec4(position, 1.0);
    uv = texcoord;
}
"#;

const FRAGMENT_SHADER: &str = r#"#version 100
precision highp float;
varying vec2 uv;

uniform vec2 u_resolution;
uniform float u_time;
uniform int u_body_count;
uniform vec4 u_body0;
uniform vec4 u_body1;
uniform vec4 u_body2;
uniform vec4 u_body3;
uniform vec4 u_body4;
uniform vec4 u_body5;
uniform vec4 u_body6;
uniform vec4 u_body7;

vec4 get_body(int i) {
    if (i == 0) return u_body0;
    if (i == 1) return u_body1;
    if (i == 2) return u_body2;
    if (i == 3) return u_body3;
    if (i == 4) return u_body4;
    if (i == 5) return u_body5;
    if (i == 6) return u_body6;
    return u_body7;
}

vec3 planet_color(int i) {
    if (i == 0) return vec3(0.20, 0.47, 0.98);
    if (i == 1) return vec3(0.86, 0.31, 0.47);
    if (i == 2) return vec3(0.30, 0.77, 0.62);
    if (i == 3) return vec3(0.91, 0.61, 0.27);
    if (i == 4) return vec3(0.55, 0.43, 0.90);
    if (i == 5) return vec3(0.23, 0.69, 0.83);
    if (i == 6) return vec3(0.78, 0.78, 0.84);
    return vec3(0.77, 0.47, 0.25);
}

float hash(vec2 p) {
    return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
}

float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x),
               mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x), f.y);
}

void main() {
    vec2 screen_uv = vec2(gl_FragCoord.x / u_resolution.x,
                          1.0 - gl_FragCoord.y / u_resolution.y);
    float aspect = u_resolution.x / u_resolution.y;

    // Bend the sampling coordinates around every massive body.
    vec2 warped = screen_uv;
    for (int i = 0; i < 8; i++) {
        if (i < u_body_count) {
            vec4 b = get_body(i);
            vec2 delta = warped - b.xy;
            delta.x *= aspect;
            float dist = max(length(delta), 0.025);
            vec2 direction = normalize(vec2(delta.x / aspect, delta.y));
            float influence = (0.0025 + b.w * 0.010) /
                              (dist * dist + b.z * b.z * 0.45);
            warped += direction * influence;
        }
    }

    // Perspective projection of a 2D slice of the spacetime grid.
    float horizon = 0.40;
    float depth = max(warped.y - horizon, 0.018);
    float gx = (warped.x - 0.5) / depth * 0.82;
    float gy = 1.0 / depth * 0.44;

    float line_x = 1.0 - smoothstep(0.0, 0.025, abs(fract(gx) - 0.5));
    float line_y = 1.0 - smoothstep(0.0, 0.025, abs(fract(gy) - 0.5));
    float grid = max(line_x, line_y);
    float fade = smoothstep(horizon, 0.68, screen_uv.y);
    vec3 background = vec3(0.025, 0.030, 0.043);
    vec3 grid_color = vec3(0.54, 0.66, 0.78) * grid * fade * 0.72;
    vec3 color = background + grid_color;

    // Faint stars in the empty space above the grid.
    float star = step(0.9975, hash(floor(screen_uv * u_resolution * 0.32)));
    color += vec3(0.45, 0.56, 0.72) * star * (1.0 - fade) * 0.55;

    // Draw the planets as lit procedural spheres, with atmospheric rims.
    for (int i = 0; i < 8; i++) {
        if (i < u_body_count) {
            vec4 b = get_body(i);
            vec2 p = (screen_uv - b.xy) * vec2(aspect, 1.0);
            float r = b.z;
            float d = length(p);
            float rim = 1.0 - smoothstep(r * 0.91, r * 1.10, d);
            if (d < r * 1.10) {
                vec2 nxy = p / r;
                float nz = sqrt(max(0.0, 1.0 - dot(nxy, nxy)));
                vec3 normal = normalize(vec3(nxy, nz));
                vec3 light_dir = normalize(vec3(-0.48, -0.58, 0.88));
                float diffuse = max(dot(normal, light_dir), 0.0);
                float specular = pow(max(dot(reflect(-light_dir, normal), vec3(0.0, 0.0, 1.0)), 0.0), 28.0);
                float bands = 0.92 + 0.08 * sin((nxy.y * 19.0 + noise(nxy * 8.0) * 2.0) * 3.0);
                float cloud = noise(nxy * 15.0 + vec2(u_time * 0.025, 0.0));
                vec3 base = planet_color(i) * bands * (0.88 + cloud * 0.15);
                vec3 sphere = base * (0.16 + diffuse * 0.95);
                sphere += vec3(0.55, 0.70, 1.0) * specular * 0.48;
                float atmosphere = pow(1.0 - nz, 3.0);
                sphere += vec3(0.24, 0.52, 1.0) * atmosphere * 0.34;
                color = mix(color, sphere, rim);
                float glow = exp(-max(d - r, 0.0) * 90.0) * (1.0 - smoothstep(r, r * 1.12, d));
                color += vec3(0.15, 0.35, 0.85) * glow * 0.18;
            }
        }
    }

    gl_FragColor = vec4(color, 1.0);
}
"#;

fn window_conf() -> Conf {
    Conf {
        window_title: "Spacetime Fabric — Rust".to_owned(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

fn body_uniform(bodies: &[Body], index: usize) -> Vec4 {
    if let Some(b) = bodies.get(index) {
        Vec4::new(b.pos.x, b.pos.y, b.radius, b.mass)
    } else {
        Vec4::ZERO
    }
}

fn nearest_body(bodies: &[Body], point: Vec2, aspect: f32) -> Option<usize> {
    let mut best = None;
    let mut best_dist = f32::MAX;
    for (i, b) in bodies.iter().enumerate() {
        let delta = (point - b.pos) * vec2(aspect, 1.0);
        let d = delta.length();
        if d < b.radius * 1.35 && d < best_dist {
            best = Some(i);
            best_dist = d;
        }
    }
    best
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut material = load_material(
        ShaderSource::Glsl {
            vertex: VERTEX_SHADER,
            fragment: FRAGMENT_SHADER,
        },
        MaterialParams {
            uniforms: vec![
                UniformDesc::new("u_resolution", UniformType::Float2),
                UniformDesc::new("u_time", UniformType::Float1),
                UniformDesc::new("u_body_count", UniformType::Int1),
                UniformDesc::new("u_body0", UniformType::Float4),
                UniformDesc::new("u_body1", UniformType::Float4),
                UniformDesc::new("u_body2", UniformType::Float4),
                UniformDesc::new("u_body3", UniformType::Float4),
                UniformDesc::new("u_body4", UniformType::Float4),
                UniformDesc::new("u_body5", UniformType::Float4),
                UniformDesc::new("u_body6", UniformType::Float4),
                UniformDesc::new("u_body7", UniformType::Float4),
            ],
            ..Default::default()
        },
    )
    .expect("Could not compile the spacetime shader");

    let mut bodies = vec![
        Body { pos: vec2(0.52, 0.43), radius: 0.105, mass: 2.2 },
        Body { pos: vec2(0.18, 0.37), radius: 0.036, mass: 0.55 },
    ];
    let mut selected: Option<usize> = Some(0);
    let mut show_help = true;

    loop {
        let sw = screen_width().max(1.0);
        let sh = screen_height().max(1.0);
        let aspect = sw / sh;

        // Click a planet to select it; click empty space to create a new one.
        if is_mouse_button_pressed(MouseButton::Left) {
            let mouse = mouse_position();
            let p = vec2(mouse.0 / sw, mouse.1 / sh);
            if let Some(index) = nearest_body(&bodies, p, aspect) {
                selected = Some(index);
            } else if bodies.len() < MAX_BODIES {
                let mass = rand::gen_range(0.55, 1.55);
                bodies.push(Body {
                    pos: p,
                    radius: 0.035 + mass * 0.018,
                    mass,
                });
                selected = Some(bodies.len() - 1);
            }
        }

        // Right-click removes the nearest planet.
        if is_mouse_button_pressed(MouseButton::Right) {
            let mouse = mouse_position();
            let p = vec2(mouse.0 / sw, mouse.1 / sh);
            if let Some(index) = nearest_body(&bodies, p, aspect) {
                bodies.remove(index);
                selected = if bodies.is_empty() { None } else { Some(index.min(bodies.len() - 1)) };
            }
        }

        if is_key_pressed(KeyCode::N) && bodies.len() < MAX_BODIES {
            let p = vec2(rand::gen_range(0.12, 0.88), rand::gen_range(0.18, 0.70));
            let mass = rand::gen_range(0.55, 1.8);
            bodies.push(Body { pos: p, radius: 0.035 + mass * 0.018, mass });
            selected = Some(bodies.len() - 1);
        }
        if is_key_pressed(KeyCode::Delete) {
            if let Some(i) = selected {
                if i < bodies.len() {
                    bodies.remove(i);
                    selected = if bodies.is_empty() { None } else { Some(i.min(bodies.len() - 1)) };
                }
            }
        }
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            if let Some(i) = selected {
                if let Some(b) = bodies.get_mut(i) {
                    b.mass = (b.mass + 0.15).min(2.5);
                    b.radius = 0.035 + b.mass * 0.018;
                }
            }
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            if let Some(i) = selected {
                if let Some(b) = bodies.get_mut(i) {
                    b.mass = (b.mass - 0.15).max(0.25);
                    b.radius = 0.035 + b.mass * 0.018;
                }
            }
        }
        if is_key_pressed(KeyCode::H) {
            show_help = !show_help;
        }
        if is_key_pressed(KeyCode::R) {
            bodies.clear();
            bodies.push(Body { pos: vec2(0.52, 0.43), radius: 0.105, mass: 2.2 });
            bodies.push(Body { pos: vec2(0.18, 0.37), radius: 0.036, mass: 0.55 });
            selected = Some(0);
        }

        material.set_uniform("u_resolution", vec2(sw, sh));
        material.set_uniform("u_time", get_time() as f32);
        material.set_uniform("u_body_count", bodies.len() as i32);
        for i in 0..MAX_BODIES {
            let name = format!("u_body{i}");
            material.set_uniform(name.as_str(), body_uniform(&bodies, i));
        }

        clear_background(Color::from_rgba(6, 8, 13, 255));
        gl_use_material(&material);
        draw_rectangle(0.0, 0.0, sw, sh, WHITE);
        gl_use_default_material();

        // Selected body gets a small unobtrusive selection ring.
        if let Some(i) = selected {
            if let Some(b) = bodies.get(i) {
                let x = b.pos.x * sw;
                let y = b.pos.y * sh;
                draw_circle_lines(x, y, b.radius * sh * 1.22, 1.2, Color::new(0.55, 0.75, 1.0, 0.65));
            }
        }

        if show_help {
            draw_rectangle(16.0, 16.0, 300.0, 124.0, Color::new(0.015, 0.02, 0.035, 0.82));
            draw_text("SPACETIME FABRIC", 28.0, 40.0, 19.0, Color::new(0.72, 0.84, 1.0, 1.0));
            draw_text("Left-click empty space: add planet", 28.0, 64.0, 15.0, LIGHTGRAY);
            draw_text("Click planet: select  |  Right-click: remove", 28.0, 85.0, 14.0, LIGHTGRAY);
            draw_text("+ / -: change mass  |  N: add  |  R: reset", 28.0, 106.0, 14.0, LIGHTGRAY);
            draw_text("H: hide controls  |  Max 8 planets", 28.0, 127.0, 14.0, LIGHTGRAY);
        }

        if let Some(i) = selected {
            if let Some(b) = bodies.get(i) {
                let label = format!("Selected mass: {:.2}   Bodies: {}/{}", b.mass, bodies.len(), MAX_BODIES);
                let tw = measure_text(&label, None, 16, 1.0).width;
                draw_text(&label, sw - tw - 20.0, 28.0, 16.0, Color::new(0.72, 0.82, 0.95, 0.9));
            }
        }

        next_frame().await;
    }
}
