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
    D: FnMut(i32, &mut S, &mut S, u32),
{
    let compare = |elem: &P, prev: &P| {
        let (px, py) = get_xy(prev);
        let (cx, cy) = get_xy(elem);

        (py, px).partial_cmp(&(cy, cx))
    };
    let mut first_point = 0;
    let mut last_point = 0;
    for (i, p) in points.iter().enumerate() {
        match compare(&points[first_point], p) {
            Some(std::cmp::Ordering::Less) => first_point = i,
            _ => (),
        }
        match compare(&points[last_point], p) {
            Some(std::cmp::Ordering::Greater) => last_point = i,
            _ => (),
        }
    }

    if first_point == last_point {
        return;
    }

    let mut cur_point_left = first_point;
    let mut cur_point_right = first_point;
    let mut right_side = false;

    let mut cur_y = get_xy(&points[first_point]).1 as i32;
    let mut next_y_left = cur_y;
    let mut next_y_right = cur_y;

    let mut slope_left = S::default();
    let mut slope_right = S::default();

    let forwards = false;
    loop {
        let (cur_point, next_y, slope) = if !right_side {
            (&mut cur_point_left, &mut next_y_left, &mut slope_left)
        } else {
            (&mut cur_point_right, &mut next_y_right, &mut slope_right)
        };
        if *cur_point == last_point {
            break;
        }
        let prev_point = *cur_point;

        if right_side == forwards {
            if prev_point < points.len() - 1 {
                *cur_point = prev_point + 1;
            } else {
                *cur_point = 0;
            }
        } else {
            if prev_point > 0 {
                *cur_point = prev_point - 1;
            } else {
                *cur_point = points.len() - 1;
            }
        }
        *next_y = get_xy(&points[*cur_point]).1 as i32;
        *slope = make_slope(
            &points[prev_point],
            &points[*cur_point],
            (*next_y - cur_y) as f32,
        );
        right_side = next_y_left > next_y_right;

        let limit = if !right_side {
            next_y_left
        } else {
            next_y_right
        };
        let aux = G_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        while cur_y < limit {
            draw_scanline(cur_y, &mut slope_left, &mut slope_right, aux);
            cur_y += 1;
        }
    }
}
