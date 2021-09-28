use std::{fmt::Debug, iter::Step, ops::AddAssign};

use num_traits::One;

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
    T: num_traits::Float + AddAssign + One,
    G: Fn(&P) -> (T, T),
    S: Debug,
    M: Fn(&P, &P, T) -> S,
    D: FnMut(T, &mut S, &mut S, u32),
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
        let mut left = &mut short_side;
        let mut right = &mut long_side;
        if shortside_right {
            // important: this swapps the references in left/right, *not* the actual slopes!
            std::mem::swap(&mut left, &mut right);
        }
        let mut y = y0;
        while y < y1 {
            draw_scanline(y, left, right, 0);
            y += T::one();
        }
    }
    if y1 < y2 {
        let mut short_side = make_slope(&p1, &p2, y2 - y1);
        let mut left = &mut short_side;
        let mut right = &mut long_side;
        if shortside_right {
            // important: this swapps the references in left/right, *not* the actual slopes!
            std::mem::swap(&mut left, &mut right);
        }
        let mut y = y1;
        while y < y2 {
            draw_scanline(y, left, right, 1);
            y += T::one();
        }
    }
}

pub fn rasterize_polygon<T, P, G, S, M, D>(
    points: &[P],
    get_xy: G,
    make_slope: M,
    mut draw_scanline: D,
) where
    P: Eq,
    T: num_traits::Float + AddAssign + One + Ord,
    G: Fn(&P) -> (T, T),
    S: Debug + Default,
    M: Fn(&P, &P, T) -> S,
    D: FnMut(T, &mut S, &mut S, u32),
{
    let mut forward = 0;
    let compare = |elem: &&P, prev: &&P| {
        let (px, py) = get_xy(*prev);
        let (cx, cy) = get_xy(*elem);

        (py, px).cmp(&(cy, cx))
    };
    let first = points.iter().min_by(compare).expect("min failed");
    let last = points.iter().max_by(compare).expect("min failed");
    let mut cur = [first, first];
    let y = |side: usize| get_xy(cur[side]).1;
    let mut sides = [S::default(), S::default()];
    let mut side = 0;
    let mut cury = y(side);

    while cur[side] != last {}
}
