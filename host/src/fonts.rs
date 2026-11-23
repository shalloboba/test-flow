use fontdue::{Font, FontSettings};
use tiny_skia::Pixmap;

pub struct Fonts {
    all: Vec<Font>,
    pub text: Vec<usize>,
    pub semi: Vec<usize>,
    pub icon: Vec<usize>,
}

fn fonts_dir() -> std::path::PathBuf {
    let root = std::env::var("windir").unwrap_or_else(|_| "C:\\Windows".into());
    std::path::Path::new(&root).join("Fonts")
}

fn load_file(name: &str) -> Option<Font> {
    let bytes = std::fs::read(fonts_dir().join(name)).ok()?;
    Font::from_bytes(
        bytes,
        FontSettings {
            collection_index: 0,
            ..FontSettings::default()
        },
    )
    .ok()
}

impl Fonts {
    pub fn load() -> Fonts {
        let mut all = Vec::new();
        let mut push = |name: &str| -> Option<usize> {
            let f = load_file(name)?;
            all.push(f);
            Some(all.len() - 1)
        };
        let regular = push("segoeui.ttf");
        let semibold = push("seguisb.ttf");
        let icon_a = push("SegoeIcons.ttf");
        let icon_b = push("segmdl2.ttf");
        let mut fallback = Vec::new();
        for name in ["YuGothM.ttc", "YuGothR.ttc", "meiryo.ttc", "msgothic.ttc", "msyh.ttc", "malgun.ttf", "seguisym.ttf"] {
            if let Some(i) = push(name) {
                fallback.push(i);
            }
        }
        let mut text: Vec<usize> = regular.into_iter().collect();
        text.extend(&fallback);
        let mut semi: Vec<usize> = semibold.into_iter().collect();
        semi.extend(&text);
        let icon: Vec<usize> = icon_a.into_iter().chain(icon_b).collect();
        Fonts { all, text, semi, icon }
    }

    fn pick(&self, chain: &[usize], ch: char) -> Option<&Font> {
        chain
            .iter()
            .map(|&i| &self.all[i])
            .find(|f| f.lookup_glyph_index(ch) != 0)
            .or_else(|| chain.first().map(|&i| &self.all[i]))
    }

    pub fn width(&self, chain: &[usize], px: f32, s: &str) -> f32 {
        s.chars()
            .map(|c| self.pick(chain, c).map(|f| f.metrics(c, px).advance_width).unwrap_or(0.0))
            .sum()
    }

    pub fn wrap(&self, chain: &[usize], px: f32, s: &str, max_w: f32, max_lines: usize) -> Vec<String> {
        let chars: Vec<char> = s.chars().collect();
        let mut lines: Vec<String> = Vec::new();
        let mut cur = String::new();
        let mut cur_w = 0.0;
        let mut i = 0;
        while i < chars.len() {
            let ch = chars[i];
            let w = self.width(chain, px, &ch.to_string());
            if cur_w + w > max_w && !cur.is_empty() {
                if lines.len() + 1 == max_lines {
                    let ell = self.width(chain, px, "…");
                    while !cur.is_empty() && cur_w + ell > max_w {
                        let last = cur.pop().unwrap();
                        cur_w -= self.width(chain, px, &last.to_string());
                    }
                    cur.push('…');
                    lines.push(cur);
                    return lines;
                }
                lines.push(std::mem::take(&mut cur));
                cur_w = 0.0;
            }
            cur.push(ch);
            cur_w += w;
            i += 1;
        }
        if !cur.is_empty() {
            lines.push(cur);
        }
        lines
    }

    pub fn draw(&self, pm: &mut Pixmap, chain: &[usize], px: f32, s: &str, x: f32, baseline: f32, color: [u8; 4]) {
        let mut pen = x;
        for ch in s.chars() {
            let Some(font) = self.pick(chain, ch) else { continue };
            let (m, cov) = font.rasterize(ch, px);
            blit(pm, &cov, m.width, m.height, (pen + m.xmin as f32).round() as i32, (baseline - m.ymin as f32 - m.height as f32).round() as i32, color);
            pen += m.advance_width;
        }
    }

    pub fn draw_centered(&self, pm: &mut Pixmap, chain: &[usize], px: f32, s: &str, cx: f32, baseline: f32, color: [u8; 4]) {
        let w = self.width(chain, px, s);
        self.draw(pm, chain, px, s, cx - w / 2.0, baseline, color);
    }

    pub fn draw_glyph_box_centered(&self, pm: &mut Pixmap, chain: &[usize], px: f32, s: &str, cx: f32, cy: f32, color: [u8; 4]) {
        let Some(ch) = s.chars().next() else { return };
        let Some(font) = self.pick(chain, ch) else { return };
        let (m, cov) = font.rasterize(ch, px);
        let gx = (cx - m.width as f32 / 2.0).round() as i32;
        let gy = (cy - m.height as f32 / 2.0).round() as i32;
        blit(pm, &cov, m.width, m.height, gx, gy, color);
    }
}

fn blit(pm: &mut Pixmap, cov: &[u8], w: usize, h: usize, x0: i32, y0: i32, color: [u8; 4]) {
    let pw = pm.width() as i32;
    let ph = pm.height() as i32;
    let data = pm.data_mut();
    for row in 0..h as i32 {
        let y = y0 + row;
        if y < 0 || y >= ph {
            continue;
        }
        for col in 0..w as i32 {
            let x = x0 + col;
            if x < 0 || x >= pw {
                continue;
            }
            let c = cov[(row as usize) * w + col as usize] as u32;
            if c == 0 {
                continue;
            }
            let a = c * color[3] as u32 / 255;
            let idx = ((y * pw + x) * 4) as usize;
            let inv = 255 - a;
            for k in 0..3 {
                let src = color[k] as u32 * a / 255;
                data[idx + k] = (src + data[idx + k] as u32 * inv / 255).min(255) as u8;
            }
            data[idx + 3] = (a + data[idx + 3] as u32 * inv / 255).min(255) as u8;
        }
    }
}
