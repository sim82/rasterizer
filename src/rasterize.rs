use std::{fmt::Debug, ops::AddAssign};

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

pub static G_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub fn rasterize_polygon<P, G, S, M, D>(
    points: &[P],
    get_xy: G,
    make_slope: M,
    mut draw_scanline: D,
) where
    G: Fn(&P) -> (f32, f32),
    S: Debug + Default,
    M: Fn(&P, &P, f32) -> S,
    D: FnMut(f32, &mut S, &mut S, u32),
{
    let compare = |elem: &P, prev: &P| {
        let (px, py) = get_xy(prev);
        let (cx, cy) = get_xy(elem);

        (py, px).partial_cmp(&(cy, cx))
    };
    let mut first = 0;
    let mut last = 0;
    for (i, p) in points.iter().enumerate() {
        match compare(&points[first], p) {
            Some(std::cmp::Ordering::Less) => first = i,
            // Some(std::cmp::Ordering::Greater) => last = i,
            _ => (),
        }
        match compare(&points[last], p) {
            // Some(std::cmp::Ordering::Less) => first = i,
            Some(std::cmp::Ordering::Greater) => last = i,
            _ => (),
        }
    }

    if first == last {
        return;
    }

    let mut cur_left = first;
    let mut cur_right = first;
    let mut right_side = false;

    // let yi = |side: bool| {
    //     (if !side {
    //         get_xy(&points[cur_left]).1
    //     } else {
    //         get_xy(&points[cur_right]).1
    //     }) as usize
    // };
    let mut cury = get_xy(&points[first]).1;
    let mut next_left = cury;
    let mut next_right = cury;

    let mut slope_left = S::default();
    let mut slope_right = S::default();

    let forwards = false;
    loop {
        let (cur, next, slope) = if !right_side {
            (&mut cur_left, &mut next_left, &mut slope_left)
        } else {
            (&mut cur_right, &mut next_right, &mut slope_right)
        };
        if *cur == last {
            break;
        }
        let prev = *cur;

        if right_side == forwards {
            if prev < points.len() - 1 {
                *cur = prev + 1;
            } else {
                *cur = 0;
            }
        } else {
            if prev > 0 {
                *cur = prev - 1;
            } else {
                *cur = points.len() - 1;
            }
        }
        *next = get_xy(&points[*cur]).1;
        *slope = make_slope(&points[prev], &points[*cur], (*next - cury) as f32);
        right_side = next_left > next_right;

        let limit = if !right_side { next_left } else { next_right };
        let aux = G_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        while cury < limit {
            draw_scanline(cury as f32, &mut slope_left, &mut slope_right, aux);
            cury += 1.0;
        }
    }
    // let shortside_right = (y1 - y0) * (x2 - x0) < (x1 - x0) * (y2 - y0);
    // let mut long_side = make_slope(&p0, &p2, y2 - y0);
}
