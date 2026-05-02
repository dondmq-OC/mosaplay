//! Grid layout: tries multiple grid patterns, picks the one with minimal black bars.

pub struct CellRect {
    pub x: i32, pub y: i32, pub w: i32, pub h: i32,
}

pub struct LayoutConfig {
    pub cell_gap: i32,
    pub outer_pad: i32,
}

/// Calculate positions/sizes for n videos with given aspect ratios.
/// Tries multiple grid layouts and picks the one that minimizes wasted (black bar) area.
pub fn calculate_layout(
    aspect_ratios: &[f64],
    available_w: i32,
    available_h: i32,
    cfg: &LayoutConfig,
) -> Vec<CellRect> {
    let n = aspect_ratios.len();
    if n == 0 { return vec![]; }

    let usable_w = available_w - 2 * cfg.outer_pad;
    let usable_h = available_h - 2 * cfg.outer_pad;

    // Single video: maximize size while preserving AR
    if n == 1 {
        let ar = aspect_ratios[0].max(0.3).min(3.0);
        let screen_ar = usable_w as f64 / usable_h as f64;
        let (w, h) = if ar > screen_ar {
            (usable_w, (usable_w as f64 / ar) as i32)
        } else {
            ((usable_h as f64 * ar) as i32, usable_h)
        };
        let x = cfg.outer_pad + (usable_w - w) / 2;
        let y = cfg.outer_pad + (usable_h - h) / 2;
        return vec![CellRect { x, y, w, h }];
    }

    // ── Try multiple grid candidates ──────────────────────
    // Score = total video fill area / total usable screen area.
    // Maximize this. Empty cells contribute 0 fill, so they naturally
    // penalize grids with unused slots — but only when a more-packed
    // grid would give larger total video area.
    let screen_area = usable_w as f64 * usable_h as f64;
    let mut best_score: f64 = -1.0;
    let mut best_rects = vec![];

    let max_cols = n.min((usable_w / 120).max(1) as usize);
    let max_rows = n.min((usable_h / 80).max(1) as usize);

    for cols in 1..=max_cols {
        for rows in 1..=max_rows {
            if rows * cols < n { continue; }
            let empty = rows * cols - n;
            if empty > n { continue; }

            let total_gap_w = (cols as i32 - 1).max(0) * cfg.cell_gap;
            let total_gap_h = (rows as i32 - 1).max(0) * cfg.cell_gap;
            let cell_w = (usable_w - total_gap_w) / cols as i32;
            let cell_h = (usable_h - total_gap_h) / rows as i32;
            if cell_w < 40 || cell_h < 30 { continue; }

            let cell_ar = cell_w as f64 / cell_h as f64;

            // Total fill area: sum over all videos of their rendered area
            let mut total_fill = 0.0;
            for &ar in aspect_ratios.iter() {
                let ar = ar.max(0.3).min(3.0);
                let (fw, fh) = if cell_ar > ar {
                    (cell_h as f64 * ar, cell_h as f64)
                } else {
                    (cell_w as f64, cell_w as f64 / ar)
                };
                total_fill += fw * fh;
            }
            // Score = fraction of usable screen filled by videos
            let score = total_fill / screen_area;

            if score > best_score {
                best_score = score;
                let mut rects = Vec::with_capacity(n);
                for i in 0..n {
                    let col = (i % cols) as i32;
                    let row = (i / cols) as i32;
                    let x = cfg.outer_pad + col * (cell_w + cfg.cell_gap);
                    let y = cfg.outer_pad + row * (cell_h + cfg.cell_gap);
                    rects.push(CellRect { x, y, w: cell_w, h: cell_h });
                }
                best_rects = rects;
            }
        }
    }

    // Fallback: single row
    if best_rects.is_empty() {
        let cell_h = usable_h / n as i32;
        for i in 0..n {
            best_rects.push(CellRect {
                x: cfg.outer_pad,
                y: cfg.outer_pad + i as i32 * (cell_h + cfg.cell_gap),
                w: usable_w,
                h: cell_h,
            });
        }
    }

    best_rects
}
