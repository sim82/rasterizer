use std::fs;

use rasterize::{
    palette::{self, quantize},
    test_texture,
};
use sdl2::{event::Event, keyboard::Keycode, pixels::PixelFormatEnum, render::TextureAccess};

fn main() {
    const ZOOM: u32 = 4;

    const WINDOW_SCALE: u32 = ZOOM;
    const W: u32 = 256 * 4 / ZOOM;
    const H: u32 = 240 * 4 / ZOOM;

    let blank = 0x0u32;
    // let mut pixels = [blank; (W * H) as usize];

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

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture(
            PixelFormatEnum::ABGR8888,
            TextureAccess::Streaming,
            256,
            256,
        )
        .unwrap();

    let (palette, mapping_table) = palette::read_colormap();
    // let mut pixels = [0u32; 16 * 16];

    let mut fb = palette::Framebuffer::new(256, 256, &palette);

    let test_texture = quantize(&palette, &test_texture::create());
    // fb.framebuffer.copy_from_slice(&test_texture[..]);
    let mut i = 0;
    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Option::Some(Keycode::Escape),
                    ..
                } => break 'mainloop,

                Event::MouseWheel { y, .. } => {
                    if i >= y && y < 0 {
                        i += y;
                    } else if y > 0 {
                        i += y;
                    }
                }
                _ => (),
            }
        }

        // for y in 0..16 {
        //     for x in 0..16 {
        //         fb.framebuffer[x + 16 * y] =
        //             mapping_table[i as usize % mapping_table.len()][y * 16 + x];
        //         // pixels[y * 16 + x] =
        //         //     palette[mapping_table[i as usize % mapping_table.len()][y * 16 + x] as usize];
        //     }
        // }
        for (d, s) in fb.framebuffer.iter_mut().zip(test_texture.iter()) {
            *d = mapping_table[i as usize % mapping_table.len()][*s as usize];
        }
        fb.upload(&mut texture);
        canvas.copy(&texture, None, None).unwrap();
        canvas.present();
    }
}
