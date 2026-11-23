use crate::fonts::Fonts;
use radial_core::{system, Config, Item};
use std::f32::consts::{FRAC_PI_2, TAU};
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

pub struct Theme {
    pub tile: [u8; 4],
    pub hover: [u8; 4],
    pub center: [u8; 4],
    pub border: [u8; 4],
    pub text: [u8; 4],
    pub sub: [u8; 4],
    pub accent: [u8; 3],
}

impl Theme {
    pub fn resolve(cfg: &Config) -> Theme {
        let light = match cfg.theme.as_str() {
            "light" => true,
            "dark" => false,
            _ => system::system_is_light(),
        };
        let accent = if cfg.accent == "auto" {
            system::system_accent()
        } else {
            system::parse_hex(&cfg.accent).unwrap_or_else(system::system_accent)
        };
        if light {
            Theme {
                tile: [0xf9, 0xf9, 0xf9, 0xff],
                hover: [0xe9, 0xe9, 0xe9, 0xff],
                center: [0xf3, 0xf3, 0xf3, 0xff],
                border: [0, 0, 0, 36],
                text: [0x1a, 0x1a, 0x1a, 0xff],
                sub: [0x5f, 0x5f, 0x5f, 0xff],
                accent,
            }
        } else {
            Theme {
                tile: [0x2b, 0x2b, 0x2b, 0xff],
                hover: [0x3d, 0x3d, 0x3d, 0xff],
                center: [0x20, 0x20, 0x20, 0xff],
                border: [255, 255, 255, 28],
                text: [0xff, 0xff, 0xff, 0xff],
                sub: [0xc5, 0xc5, 0xc5, 0xff],
                accent,
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Geometry {
    pub s: f32,
    pub rc: f32,
    pub r1: f32,
    pub r2: f32,
    pub gap: f32,
    pub corner: f32,
    pub size: i32,
    pub c: f32,
}

impl Geometry {
    pub fn new(s: f32) -> Geometry {
        let r2 = 170.0 * s;
        let margin = 14.0 * s;
        let size = (2.0 * (r2 + margin)).ceil() as i32;
        Geometry {
            s,
            rc: 66.0 * s,
            r1: 66.0 * s,
            r2,
            gap: 0.0,
            corner: 0.0,
            size,
            c: size as f32 / 2.0,
        }
    }

    pub fn icon() -> Geometry {
        Geometry {
            s: 1.0,
            rc: 0.0,
            r1: 5.0,
            r2: 14.5,
            gap: 1.4,
            corner: 1.0,
            size: 32,
            c: 16.0,
        }
    }
}

fn arc(pb: &mut PathBuilder, c: f32, r: f32, a0: f32, a1: f32) {
    let n = ((a1 - a0).abs() / FRAC_PI_2).ceil().max(1.0) as i32;
    let da = (a1 - a0) / n as f32;
    let k = 4.0 / 3.0 * (da / 4.0).tan();
    for i in 0..n {
        let s = a0 + da * i as f32;
        let e = s + da;
        let (sc, ss) = (s.cos(), s.sin());
        let (ec, es) = (e.cos(), e.sin());
        pb.cubic_to(
            c + r * (sc - k * ss),
            c + r * (ss + k * sc),
            c + r * (ec + k * es),
            c + r * (es - k * ec),
            c + r * ec,
            c + r * es,
        );
    }
}

fn sector_path(g: &Geometry, tc: f32, half: f32) -> Option<Path> {
    let d = g.gap / 2.0 + g.corner;
    let ro = g.r2 - g.corner;
    let ri = g.r1 + g.corner;
    let a = tc - half;
    let b = tc + half;
    let ao = a + (d / ro).asin();
    let bo = b - (d / ro).asin();
    let ai = a + (d / ri).asin();
    let bi = b - (d / ri).asin();
    let mut pb = PathBuilder::new();
    pb.move_to(g.c + ro * ao.cos(), g.c + ro * ao.sin());
    arc(&mut pb, g.c, ro, ao, bo);
    pb.line_to(g.c + ri * bi.cos(), g.c + ri * bi.sin());
    arc(&mut pb, g.c, ri, bi, ai);
    pb.close();
    pb.finish()
}

fn paint(c: [u8; 4]) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(c[0], c[1], c[2], c[3]);
    p.anti_alias = true;
    p
}

fn round_stroke(w: f32) -> Stroke {
    Stroke {
        width: w,
        line_join: LineJoin::Round,
        line_cap: LineCap::Round,
        ..Stroke::default()
    }
}

pub fn hit(g: &Geometry, dx: f32, dy: f32, n: usize) -> Option<usize> {
    let d = (dx * dx + dy * dy).sqrt();
    if n == 0 || d < (g.rc + g.r1) / 2.0 {
        return None;
    }
    let step = TAU / n as f32;
    let a = (dy.atan2(dx) + FRAC_PI_2 + step / 2.0).rem_euclid(TAU);
    Some(((a / step) as usize).min(n - 1))
}

pub struct LayerInfo<'a> {
    pub name: &'a str,
    pub index: usize,
    pub count: usize,
}

pub fn render(items: &[Item], info: &LayerInfo, th: &Theme, g: &Geometry, fonts: &Fonts, hover: Option<usize>) -> Pixmap {
    let mut pm = Pixmap::new(g.size as u32, g.size as u32).unwrap();
    let n = items.len();
    let step = TAU / n as f32;
    let s = g.s;
    let bw = s.max(1.0);
    let id = Transform::identity();

    // 一つの輪（ドーナツ）として下地を描く
    let mut ring = PathBuilder::new();
    ring.push_circle(g.c, g.c, g.r2);
    ring.push_circle(g.c, g.c, g.r1);
    if let Some(ring) = ring.finish() {
        pm.fill_path(&ring, &paint(th.tile), FillRule::EvenOdd, id, None);
    }
    // ホバー中のピースだけ色を変える
    if let Some(i) = hover {
        let tc = -FRAC_PI_2 + step * i as f32;
        if let Some(path) = sector_path(g, tc, step / 2.0) {
            pm.fill_path(&path, &paint(th.hover), FillRule::Winding, id, None);
        }
    }
    // 外周・内周の縁取り
    for r in [g.r2, g.r1] {
        if let Some(c) = PathBuilder::from_circle(g.c, g.c, r) {
            pm.stroke_path(&c, &paint(th.border), &round_stroke(bw), id, None);
        }
    }
    // ピースの境目は細い区切り線だけにする
    if n > 1 {
        for i in 0..n {
            let a = -FRAC_PI_2 + step * i as f32 - step / 2.0;
            let mut pb = PathBuilder::new();
            pb.move_to(g.c + g.r1 * a.cos(), g.c + g.r1 * a.sin());
            pb.line_to(g.c + g.r2 * a.cos(), g.c + g.r2 * a.sin());
            if let Some(p) = pb.finish() {
                pm.stroke_path(&p, &paint(th.border), &round_stroke(bw), id, None);
            }
        }
    }

    if let Some(i) = hover {
        let tc = -FRAC_PI_2 + step * i as f32;
        let inset = (g.gap / 2.0 + g.corner) / g.r2;
        let mut pb = PathBuilder::new();
        let r = g.r2 + 8.0 * s;
        let a0 = tc - step / 2.0 + inset + 0.02;
        let a1 = tc + step / 2.0 - inset - 0.02;
        pb.move_to(g.c + r * a0.cos(), g.c + r * a0.sin());
        arc(&mut pb, g.c, r, a0, a1);
        if let Some(p) = pb.finish() {
            let a = th.accent;
            pm.stroke_path(&p, &paint([a[0], a[1], a[2], 255]), &round_stroke(4.0 * s), id, None);
        }
    }

    if let Some(c) = PathBuilder::from_circle(g.c, g.c, g.rc + bw) {
        pm.fill_path(&c, &paint(th.border), FillRule::Winding, id, None);
    }
    if let Some(c) = PathBuilder::from_circle(g.c, g.c, g.rc) {
        pm.fill_path(&c, &paint(th.center), FillRule::Winding, id, None);
    }

    let rm = (g.r1 + g.r2) / 2.0;
    let chord = 2.0 * rm * (step / 2.0).sin() - g.gap - 12.0 * s;
    let wrap_w = chord.min(104.0 * s).max(44.0 * s);
    for (i, it) in items.iter().enumerate() {
        let tc = -FRAC_PI_2 + step * i as f32;
        let px = g.c + rm * tc.cos();
        let py = g.c + rm * tc.sin();
        fonts.draw_glyph_box_centered(&mut pm, &fonts.icon, 26.0 * s, &it.icon, px, py - 18.0 * s, th.text);
        let lines = fonts.wrap(&fonts.text, 12.0 * s, &it.label, wrap_w, 2);
        let total = lines.len() as f32;
        for (k, line) in lines.iter().enumerate() {
            let base = py + 10.0 * s + 15.0 * s * k as f32 + 4.0 * s - (total - 1.0) * 0.0;
            fonts.draw_centered(&mut pm, &fonts.text, 12.0 * s, line, px, base, th.text);
        }
    }

    match hover.and_then(|i| items.get(i)) {
        Some(it) => {
            let w = 2.0 * g.rc - 18.0 * s;
            let title = fonts.wrap(&fonts.semi, 14.0 * s, &it.label, w, 1);
            if let Some(t) = title.first() {
                fonts.draw_centered(&mut pm, &fonts.semi, 14.0 * s, t, g.c, g.c - 2.0 * s, th.text);
            }
            let sub = fonts.wrap(&fonts.text, 11.5 * s, &it.subtitle(), w, 1);
            if let Some(t) = sub.first() {
                fonts.draw_centered(&mut pm, &fonts.text, 11.5 * s, t, g.c, g.c + 17.0 * s, th.sub);
            }
        }
        None => {
            if info.count > 1 {
                let w = 2.0 * g.rc - 18.0 * s;
                let title = fonts.wrap(&fonts.semi, 14.0 * s, info.name, w, 1);
                if let Some(t) = title.first() {
                    fonts.draw_centered(&mut pm, &fonts.semi, 14.0 * s, t, g.c, g.c - 2.0 * s, th.text);
                }
                let pos = format!("{} / {}", info.index + 1, info.count);
                fonts.draw_centered(&mut pm, &fonts.text, 11.5 * s, &pos, g.c, g.c + 17.0 * s, th.sub);
            } else {
                fonts.draw_glyph_box_centered(&mut pm, &fonts.icon, 20.0 * s, "\u{e711}", g.c, g.c, th.sub);
            }
        }
    }

    if info.count > 1 {
        let spacing = 11.0 * s;
        let total = spacing * (info.count as f32 - 1.0);
        for k in 0..info.count {
            let x = g.c - total / 2.0 + spacing * k as f32;
            let y = g.c + 40.0 * s;
            if let Some(dot) = PathBuilder::from_circle(x, y, 2.6 * s) {
                let col = if k == info.index {
                    [th.accent[0], th.accent[1], th.accent[2], 255]
                } else {
                    [th.sub[0], th.sub[1], th.sub[2], 110]
                };
                pm.fill_path(&dot, &paint(col), FillRule::Winding, id, None);
            }
        }
    }
    pm
}

pub fn render_icon(accent: [u8; 3]) -> Pixmap {
    let g = Geometry::icon();
    let mut pm = Pixmap::new(32, 32).unwrap();
    let n = 6;
    let step = TAU / n as f32;
    let col = [accent[0], accent[1], accent[2], 255];
    for i in 0..n {
        let tc = -FRAC_PI_2 + step * i as f32;
        if let Some(path) = sector_path(&g, tc, step / 2.0) {
            pm.fill_path(&path, &paint(col), FillRule::Winding, Transform::identity(), None);
            pm.stroke_path(&path, &paint(col), &round_stroke(2.0 * g.corner), Transform::identity(), None);
        }
    }
    pm
}
