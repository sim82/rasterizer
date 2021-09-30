use std::{fs, path::Path};

use sdl2::{rect::Rect, render::Texture};

const NUM_COLORS: usize = 256;
const NUM_ROWS: usize = 320;

pub fn read_pcx<P>(path: P) -> Vec<u8>
where
    P: AsRef<Path>,
{
    let mut pcx_reader = pcx::Reader::new(fs::File::open(path).unwrap()).unwrap();

    let width = pcx_reader.width() as usize;
    let height = pcx_reader.height() as usize;

    let mut buf = vec![0u8; width * height];
    for line in buf.chunks_mut(width) {
        pcx_reader.next_row_paletted(line);
    }
    buf
}

pub fn read_colormap() -> ([u32; NUM_COLORS], [[u8; NUM_COLORS]; NUM_ROWS]) {
    let mut pcx_reader = pcx::Reader::new(fs::File::open("assets/COLORMAP.PCX").unwrap()).unwrap();
    assert!(pcx_reader.palette_length() == Some(256));

    assert!(pcx_reader.width() as usize == NUM_COLORS);
    assert!(pcx_reader.height() as usize == NUM_ROWS);

    let mut mapping_table = [[0u8; NUM_COLORS]; NUM_ROWS];

    for row in mapping_table.iter_mut() {
        pcx_reader.next_row_paletted(row);
    }

    let mut palette_raw = [0u8; 3 * 256];
    pcx_reader.read_palette(&mut palette_raw);
    let mut palette = [0u32; 256];
    for (raw, c) in palette_raw.chunks(3).zip(palette.iter_mut()) {
        *c = raw[0] as u32 | (raw[1] as u32) << 8 | (raw[2] as u32) << 16;
    }
    (palette, mapping_table)
}

pub struct Framebuffer {
    width: u32,
    height: u32,
    palette: [u32; NUM_COLORS],
    pub framebuffer: Vec<u8>,
    framebuffer_rgb: Vec<u32>,
}

impl Framebuffer {
    pub fn new(width: u32, height: u32, palette: &[u32; NUM_COLORS]) -> Self {
        Framebuffer {
            width,
            height,
            palette: *palette,
            framebuffer: vec![0u8; (width * height) as usize],
            framebuffer_rgb: vec![0u32; (width * height) as usize],
        }
    }
    pub fn upload(&mut self, texture: &mut Texture) {
        for (p, p_rgb) in self.framebuffer.iter().zip(self.framebuffer_rgb.iter_mut()) {
            *p_rgb = self.palette[*p as usize];
        }
        texture
            .update(
                Some(Rect::new(0, 0, self.width, self.height)),
                unsafe { std::mem::transmute::<&[u32], &[u8]>(&self.framebuffer_rgb) },
                self.width as usize * 4,
            )
            .unwrap();
    }
}

pub fn quantize(palette: &[u32; NUM_COLORS], rgb: &[u32]) -> Vec<u8> {
    let msd = |a, b| {
        let r = (a & 0xff) as i32 - (b & 0xff) as i32;
        let g = ((a >> 8) & 0xffu32) as i32 - ((b >> 8) & 0xffu32) as i32;
        let b = ((a >> 16) & 0xffu32) as i32 - ((b >> 16) & 0xffu32) as i32;
        (r * r + g * g + b * b) as u32
    };
    rgb.iter()
        .map(|rgb| {
            let mut out = 0;
            let mut min = u32::MAX;
            for (i, p) in palette.iter().enumerate() {
                let err = msd(rgb, p);
                if err < min {
                    min = err;
                    out = i;
                }
            }
            out as u8
        })
        .collect()
}
