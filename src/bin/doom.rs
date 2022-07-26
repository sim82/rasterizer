use lazy_static::lazy_static;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
    pixels::{Color, PixelFormatEnum},
    render::{Canvas, Texture, TextureCreator},
    video::{Window, WindowContext},
    Sdl, VideoSubsystem,
};

const WINDOW_SCALE: u32 = 2;
const W: u32 = 424;
const H: u32 = 240;

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
            .window("rust-sdl2 demo: Video", W * WINDOW_SCALE, H * WINDOW_SCALE)
            .position_centered()
            .resizable()
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
            player.y += dx;
        }
        if keyboard_state.is_scancode_pressed(Scancode::A) {
            player.x -= dy;
            player.y -= dx;
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

        // engine.canvas.set_draw_color(Color::RGB(255, 128, 0));
        // engine.canvas.draw_point((100, 100)).unwrap();
        engine.canvas.set_draw_color(get_color(8));
        engine.canvas.clear();
        for y in 0..8 {
            engine.draw_pixel(100, y + 100, y);
        }

        // engine.draw_pixel(player.x, player.y, 0);
        draw3d(&player, &mut engine);
        engine.canvas.present();

        std::thread::sleep(std::time::Duration::from_millis(16));
    }
}

fn draw3d(player: &Player, engine: &mut Engine) {
    let cs = M_COS[player.a as usize];
    let sn = M_SIN[player.a as usize];

    println!("{} {}", cs, sn);

    let x1 = 40.0 - player.x as f32;
    let y1 = 10.0 - player.y as f32;

    let x2 = 40.0 - player.x as f32;
    let y2 = 290.0 - player.y as f32;

    // engine.draw_pixel(x1 as i32, y1 as i32, 3);
    // engine.draw_pixel(x2 as i32, y2 as i32, 4);

    let mut wx = [x1 * cs - y1 * sn, x2 * cs - y2 * sn];
    let mut wy = [y1 * cs + x1 * sn, y2 * cs + x2 * sn];
    let mut wz = [0.0 - player.z as f32, 0.0 - player.z as f32];

    const SW2: f32 = W as f32 / 2.0;
    const SH2: f32 = H as f32 / 2.0;

    wx[0] = wx[0] * 200.0 / wy[0] + SW2;
    wy[0] = wz[0] * 200.0 / wy[0] + SW2;

    wx[1] = wx[1] * 200.0 / wy[1] + SH2;
    wy[1] = wz[1] * 200.0 / wy[1] + SH2;

    // let wx = [x1, x2];
    // let wy = [y1, y2];

    println!("{:?} {:?}", wx, wy);

    engine.draw_pixel(wx[0] as i32, wy[0] as i32, 1);
    engine.draw_pixel(wx[1] as i32, wy[1] as i32, 2);
}
