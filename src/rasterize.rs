use std::{fmt::Debug, iter::Step};

pub trait Point2d<T> {
    fn get_xy(&self) -> (T, T);
    fn get_x(&self) -> T;
    fn get_y(&self) -> T;
}

pub trait Slope {
    fn get(&self) -> f32;
    fn advance(&mut self);
}

pub fn rasterize_triangle<T, P, G, S, M, D>(
    mut p0: P,
    mut p1: P,
    mut p2: P,
    get_xy: G,
    make_slope: M,
    mut draw_scanline: D,
) where
    T: num_traits::PrimInt + Step,
    G: Fn(&P) -> (T, T),
    S: Slope + Debug,
    M: Fn(&P, &P, T) -> S,
    D: FnMut(T, &mut [&mut S; 2]),
{
    let (mut x0, mut y0) = get_xy(&p0);
    let (mut x1, mut y1) = get_xy(&p1);
    let (mut x2, mut y2) = get_xy(&p2);

    if (y1, x1) < (y0, x0) {
        std::mem::swap(&mut p1, &mut p0);
        std::mem::swap(&mut x1, &mut x0);
        std::mem::swap(&mut y1, &mut y0);
    }
    if (y2, x2) < (y0, x0) {
        std::mem::swap(&mut p2, &mut p0);
        std::mem::swap(&mut x2, &mut x0);
        std::mem::swap(&mut y2, &mut y0);
    }
    if (y2, x2) < (y1, x1) {
        std::mem::swap(&mut p2, &mut p1);
        std::mem::swap(&mut x2, &mut x1);
        std::mem::swap(&mut y2, &mut y1);
    }

    if y0 == y2 {
        return;
    }
    let shortside_right = (y1 - y0) * (x2 - x0) < (x1 - x0) * (y2 - y0);
    let mut long_side = make_slope(&p0, &p2, y2 - y0);

    if y0 < y1 {
        let mut short_side = make_slope(&p0, &p1, y1 - y0);
        let mut sides = if shortside_right {
            [&mut long_side, &mut short_side]
        } else {
            [&mut short_side, &mut long_side]
        };
        // println!("1: {:?} {:?}", sides[0], sides[1]);
        for y in y0..y1 {
            draw_scanline(y, &mut sides);
        }
    }
    if y1 < y2 {
        let mut short_side = make_slope(&p1, &p2, y2 - y1);

        let mut sides = if shortside_right {
            [&mut long_side, &mut short_side]
        } else {
            [&mut short_side, &mut long_side]
        };
        // println!("2: {:?} {:?}", sides[0], sides[1]);

        for y in y1..y2 {
            draw_scanline(y, &mut sides);
        }
    }
}
