use lazy_static::lazy_static;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl, VideoSubsystem,
};

const WINDOW_SCALE: u32 = 4;
const W: i32 = 160;
const H: i32 = 120;

struct Engine {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    canvas: Canvas<Window>,
    current_color: Option<i32>,
    // texture_creator: TextureCreator<WindowContext>,
}

struct MyCanvas<'a> {
    engine: &'a Engine,
    texture: Texture<'a>,
    pixels: Vec<u32>,
}

impl Engine {
    pub fn new() -> Self {
        let blank = 0x0u32;

        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(
                "rust-sdl2 demo: Video",
                W as u32 * WINDOW_SCALE,
                H as u32 * WINDOW_SCALE,
            )
            .position_centered()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .software()
            // .present_vsync()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas
            .set_scale(WINDOW_SCALE as f32, WINDOW_SCALE as f32)
            .unwrap();

        Self {
            sdl_context,
            video_subsystem,
            canvas,
            current_color: None,
            // texture_creator,
            // texture,
            // pixels,
        }
    }
    pub fn draw_pixel(&mut self, x: i32, y: i32, color_index: i32) {
        if self.current_color != Some(color_index) {
            let color = get_color(color_index);
            self.canvas.set_draw_color(color);
            self.current_color = Some(color_index);
        }
        self.canvas.draw_point((x, y)).unwrap();
    }
}

fn get_color(color_index: i32) -> Color {
    match color_index {
        0 => Color::RGB(255, 255, 0),
        1 => Color::RGB(160, 160, 0),
        2 => Color::RGB(0, 255, 0),
        3 => Color::RGB(0, 160, 0),
        4 => Color::RGB(0, 255, 255),
        5 => Color::RGB(0, 160, 160),
        6 => Color::RGB(160, 100, 0),
        7 => Color::RGB(110, 50, 0),

        _ => Color::RGB(0, 60, 130),
    }
}

struct Player {
    x: i32,
    y: i32,
    z: i32,
    a: i32,
    l: i32,
}

impl Player {
    pub fn new() -> Self {
        Player {
            x: 70,
            y: -110,
            z: 20,
            a: 0,
            l: 0,
        }
    }
}

struct Wall {
    x1: i32,
    x2: i32,
    y1: i32,
    y2: i32,
    c: i32,
}

#[derive(Default)]
struct Sector {
    wall_range: std::ops::Range<usize>,
    z1: i32,
    z2: i32,
    x: i32,
    y: i32,
    d: i32,
}

fn dist(x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    let xd = x2 - x1;
    let yd = y2 - y1;
    ((xd * xd + yd * yd) as f32).sqrt() as i32
}

fn sin_cos_table() -> ([f32; 360], [f32; 360]) {
    let mut s = [0.0; 360];
    let mut c = [0.0; 360];

    for x in 0..360 {
        s[x] = (x as f32 / 180.0 * std::f32::consts::PI).sin();
        c[x] = (x as f32 / 180.0 * std::f32::consts::PI).cos();
    }
    (s, c)
}

lazy_static! {
    static ref M_SIN: [f32; 360] = sin_cos_table().0;
    static ref M_COS: [f32; 360] = sin_cos_table().1;
}
fn main() {
    let mut engine = Engine::new();

    let mut player = Player::new();

    let walls = [
        // sector 0
        Wall {
            x1: 0,
            y1: 0,
            x2: 32,
            y2: 0,
            c: 0,
        },
        Wall {
            x1: 32,
            y1: 0,
            x2: 32,
            y2: 32,
            c: 1,
        },
        Wall {
            x1: 32,
            y1: 32,
            x2: 0,
            y2: 32,
            c: 0,
        },
        Wall {
            x1: 0,
            y1: 32,
            x2: 0,
            y2: 0,
            c: 1,
        },
        // sector 1
        Wall {
            x1: 0 + 64,
            y1: 0,
            x2: 32 + 64,
            y2: 0,
            c: 2,
        },
        Wall {
            x1: 32 + 64,
            y1: 0,
            x2: 32 + 64,
            y2: 32,
            c: 3,
        },
        Wall {
            x1: 32 + 64,
            y1: 32,
            x2: 0 + 64,
            y2: 32,
            c: 2,
        },
        Wall {
            x1: 0 + 64,
            y1: 32,
            x2: 0 + 64,
            y2: 0,
            c: 3,
        },
        // sector 2
        Wall {
            x1: 0 + 64,
            y1: 0 + 64,
            x2: 32 + 64,
            y2: 0 + 64,
            c: 4,
        },
        Wall {
            x1: 32 + 64,
            y1: 0 + 64,
            x2: 32 + 64,
            y2: 32 + 64,
            c: 5,
        },
        Wall {
            x1: 32 + 64,
            y1: 32 + 64,
            x2: 0 + 64,
            y2: 32 + 64,
            c: 4,
        },
        Wall {
            x1: 0 + 64,
            y1: 32 + 64,
            x2: 0 + 64,
            y2: 0 + 64,
            c: 5,
        },
        // sector 3
        Wall {
            x1: 0,
            y1: 0 + 64,
            x2: 32,
            y2: 0 + 64,
            c: 6,
        },
        Wall {
            x1: 32,
            y1: 0 + 64,
            x2: 32,
            y2: 32 + 64,
            c: 7,
        },
        Wall {
            x1: 32,
            y1: 32 + 64,
            x2: 0,
            y2: 32 + 64,
            c: 6,
        },
        Wall {
            x1: 0,
            y1: 32 + 64,
            x2: 0,
            y2: 0 + 64,
            c: 7,
        },
    ];
    let mut sectors = [
        Sector {
            wall_range: 0..4,
            z1: 0,
            z2: 40,
            d: 0,
            ..Default::default()
        },
        Sector {
            wall_range: 4..8,
            z1: 0,
            z2: 40,
            d: 0,
            ..Default::default()
        },
        Sector {
            wall_range: 8..12,
            z1: 0,
            z2: 40,
            d: 0,
            ..Default::default()
        },
        Sector {
            wall_range: 12..16,
            z1: 0,
            z2: 40,
            d: 0,
            ..Default::default()
        },
    ];

    'mainloop: loop {
        let mut event_pump = engine.sdl_context.event_pump().unwrap();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,
                _ => {}
            }
        }

        let keyboard_state = event_pump.keyboard_state();

        let dx = (M_SIN[player.a as usize] * 10.0) as i32;
        let dy = (M_COS[player.a as usize] * 10.0) as i32;

        let keyboard_state = event_pump.keyboard_state();
        if keyboard_state.is_scancode_pressed(Scancode::W) {
            player.x += dx;
            player.y += dy;
        }
        if keyboard_state.is_scancode_pressed(Scancode::S) {
            player.x -= dx;
            player.y -= dy;
        }
        if keyboard_state.is_scancode_pressed(Scancode::D) {
            player.x += dy;
            player.y -= dx;
        }
        if keyboard_state.is_scancode_pressed(Scancode::A) {
            player.x -= dy;
            player.y += dx;
        }
        if keyboard_state.is_scancode_pressed(Scancode::Left) {
            player.a -= 4;
            if player.a < 0 {
                player.a += 360
            }
        }
        if keyboard_state.is_scancode_pressed(Scancode::Right) {
            player.a += 4;
            if player.a >= 360 {
                player.a -= 360
            }
        }

        if keyboard_state.is_scancode_pressed(Scancode::R) {
            player.z += 10;
        }

        if keyboard_state.is_scancode_pressed(Scancode::F) {
            player.z -= 10;
        }
        // engine.canvas.set_draw_color(Color::RGB(255, 128, 0));
        // engine.canvas.draw_point((100, 100)).unwrap();
        engine.canvas.set_draw_color(get_color(8));
        engine.canvas.clear();
        for y in 0..8 {
            engine.draw_pixel(100, y + 100, y);
        }

        engine.draw_pixel(W as i32 - 1, H as i32 - 1, 0);
        // engine.draw_pixel(player.x, player.y, 0);

        draw3d(&player, &mut sectors, &walls, &mut engine);
        engine.canvas.present();

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn draw_wall(x1: i32, x2: i32, b1: i32, b2: i32, t1: i32, t2: i32, c: i32, engine: &mut Engine) {
    // println!("{} {} {} {} {} {}", x1, x2, b1, b2, t1, t2);

    let dyb = b2 - b1;
    let dyt = t2 - t1;
    let dx = x2 - x1;
    let dx = if dx != 0 { dx } else { 1 };
    let xs = x1;

    let x1 = x1.max(1).min(W - 1);
    let x2 = x2.max(1).min(W - 1);

    for x in x1..x2 {
        // let y1 = dyb * (((x - xs) as f32 + 0.5) / dx as f32 + b1 as f32) as i32;
        let y1 = dyb * (x - xs) / dx + b1;
        let y2 = dyt * (x - xs) / dx + t1;

        let y1 = y1.max(1).min(H - 1);
        let y2 = y2.max(1).min(H - 1);

        for y in y1..y2 {
            engine.draw_pixel(x, y, c);
        }
    }
}

fn draw3d(player: &Player, sectors: &mut [Sector], walls: &[Wall], engine: &mut Engine) {
    for s in sectors.iter_mut() {
        s.d = 0;
        for w in walls[s.wall_range.clone()].iter() {
            let cs = M_COS[player.a as usize];
            let sn = M_SIN[player.a as usize];

            // println!("{} {}", cs, sn);

            let x1 = (w.x1 - player.x) as f32;
            let y1 = (w.y1 - player.y) as f32;
            let x2 = (w.x2 - player.x) as f32;
            let y2 = (w.y2 - player.y) as f32;

            //let dist = dist(x1 as i32, y1 as i32, x2 as i32, y2 as i32);
            // let dist = (x1 * x1 + y1 * y1) + (x2 * x2 + y2 * y2);
            s.d += dist(player.x, player.y, (w.x1 + w.x2) / 2, (w.y1 + w.y2) / 2);
            // println!("dist: {} {} {} {} {}", dist, x1, y1, x2, y2);
            // engine.draw_pixel(x1 as i32, y1 as i32, 3);
            // engine.draw_pixel(x2 as i32, y2 as i32, 4);

            // let wx = [
            let mut wx0 = (x1 * cs - y1 * sn) as i32;
            let mut wx1 = (x2 * cs - y2 * sn) as i32;
            let mut wx2 = (x1 * cs - y1 * sn) as i32;
            let mut wx3 = (x2 * cs - y2 * sn) as i32;
            // ];
            // let wy = [
            let mut wy0 = (y1 * cs + x1 * sn) as i32;
            let mut wy1 = (y2 * cs + x2 * sn) as i32;
            let mut wy2 = (y1 * cs + x1 * sn) as i32;
            let mut wy3 = (y2 * cs + x2 * sn) as i32;
            // ];
            let mut wz0 = s.z1 - player.z;
            let mut wz1 = s.z1 - player.z;
            let mut wz2 = s.z2 - player.z;
            let mut wz3 = s.z2 - player.z;

            if wy0 < 1 && wy1 < 1 {
                continue;
            }

            if wy0 < 1 {
                clip_behind_player(&mut wx0, &mut wy0, &mut wz0, wx1, wy1, wz1);
                clip_behind_player(&mut wx2, &mut wy2, &mut wz2, wx3, wy3, wz3);
            }

            if wy1 < 1 {
                clip_behind_player(&mut wx1, &mut wy1, &mut wz1, wx0, wy0, wz0);
                clip_behind_player(&mut wx3, &mut wy3, &mut wz3, wx2, wy2, wz2);
            }

            const SW2: i32 = W / 2;
            const SH2: i32 = H / 2;

            // screen pos
            let sx = [
                wx0 * 200 / wy0 + SW2,
                wx1 * 200 / wy1 + SW2,
                wx2 * 200 / wy2 + SW2,
                wx3 * 200 / wy3 + SW2,
            ];
            let sy = [
                wz0 * 200 / wy0 + SH2,
                wz1 * 200 / wy1 + SH2,
                wz2 * 200 / wy2 + SH2,
                wz3 * 200 / wy3 + SH2,
            ];

            // let wx = [x1, x2];
            // let wy = [y1, y2];

            // println!("{:?} {:?} {:?} {:?}", wx, wy, sx, sy);

            // engine.draw_pixel(sx[0] as i32, sy[0] as i32, 1);
            // engine.draw_pixel(sx[1] as i32, sy[1] as i32, 2);
            draw_wall(
                sx[0] as i32,
                sx[1] as i32,
                sy[0] as i32,
                sy[1] as i32,
                sy[2] as i32,
                sy[3] as i32,
                w.c,
                engine,
            );
        }
        // println!("d: {}", s.d);

        s.d /= s.wall_range.len() as i32;
    }

    sectors.sort_by(|s1, s2| s2.d.cmp(&s1.d));
    // println!("sector order: ");
    for s in sectors.iter() {
        println!("{:?}", s.wall_range);
    }
}

fn draw3d_test(player: &Player, engine: &mut Engine) {
    let cs = M_COS[player.a as usize];
    let sn = M_SIN[player.a as usize];

    // println!("{} {}", cs, sn);

    let x1 = 40.0 - player.x as f32;
    let y1 = 10.0 - player.y as f32;

    let x2 = 40.0 - player.x as f32;
    let y2 = 290.0 - player.y as f32;

    // engine.draw_pixel(x1 as i32, y1 as i32, 3);
    // engine.draw_pixel(x2 as i32, y2 as i32, 4);

    // let wx = [
    let mut wx0 = (x1 * cs - y1 * sn) as i32;
    let mut wx1 = (x2 * cs - y2 * sn) as i32;
    let mut wx2 = (x1 * cs - y1 * sn) as i32;
    let mut wx3 = (x2 * cs - y2 * sn) as i32;
    // ];
    // let wy = [
    let mut wy0 = (y1 * cs + x1 * sn) as i32;
    let mut wy1 = (y2 * cs + x2 * sn) as i32;
    let mut wy2 = (y1 * cs + x1 * sn) as i32;
    let mut wy3 = (y2 * cs + x2 * sn) as i32;
    // ];
    let mut wz0 = 0 - player.z;
    let mut wz1 = 0 - player.z;
    let mut wz2 = 40 - player.z;
    let mut wz3 = 40 - player.z;

    if wy0 < 1 && wy1 < 1 {
        return;
    }

    if wy0 < 1 {
        clip_behind_player(&mut wx0, &mut wy0, &mut wz0, wx1, wy1, wz1);
        clip_behind_player(&mut wx2, &mut wy2, &mut wz2, wx3, wy3, wz3);
    }

    if wy1 < 1 {
        clip_behind_player(&mut wx1, &mut wy1, &mut wz1, wx0, wy0, wz0);
        clip_behind_player(&mut wx3, &mut wy3, &mut wz3, wx2, wy2, wz2);
    }

    const SW2: i32 = W / 2;
    const SH2: i32 = H / 2;

    // screen pos
    let sx = [
        wx0 * 200 / wy0 + SW2,
        wx1 * 200 / wy1 + SW2,
        wx2 * 200 / wy2 + SW2,
        wx3 * 200 / wy3 + SW2,
    ];
    let sy = [
        wz0 * 200 / wy0 + SH2,
        wz1 * 200 / wy1 + SH2,
        wz2 * 200 / wy2 + SH2,
        wz3 * 200 / wy3 + SH2,
    ];

    // let wx = [x1, x2];
    // let wy = [y1, y2];

    // println!("{:?} {:?} {:?} {:?}", wx, wy, sx, sy);

    // engine.draw_pixel(sx[0] as i32, sy[0] as i32, 1);
    // engine.draw_pixel(sx[1] as i32, sy[1] as i32, 2);
    draw_wall(
        sx[0] as i32,
        sx[1] as i32,
        sy[0] as i32,
        sy[1] as i32,
        sy[2] as i32,
        sy[3] as i32,
        0,
        engine,
    );
}

fn clip_behind_player(x1: &mut i32, y1: &mut i32, z1: &mut i32, x2: i32, y2: i32, z2: i32) {
    let da = *y1 as f32;
    let db = y2 as f32;
    let d = da - db;
    let d = if d != 0.0 { d } else { 1.0 };
    let s = da / d;
    *x1 = *x1 + (s * (x2 - *x1) as f32) as i32;
    *y1 = *y1 + (s * (y2 - *y1) as f32) as i32;
    if *y1 == 0 {
        *y1 = 1
    }
    *z1 = *z1 + (s * (z2 - *z1) as f32) as i32;
}
