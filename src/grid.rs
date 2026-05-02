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
    let mut best_score = f64::MAX;
    let mut best_rects = vec![];

    let max_cols = n.min((usable_w / 120).max(1) as usize);
    let max_rows = n.min((usable_h / 80).max(1) as usize);

    for cols in 1..=max_cols {
        for rows in 1..=max_rows {
            if rows * cols < n { continue; }
            // Don't try grids with way too many empty cells
            let empty = rows * cols - n;
            if empty > n { continue; }

            let total_gap_w = (cols as i32 - 1).max(0) * cfg.cell_gap;
            let total_gap_h = (rows as i32 - 1).max(0) * cfg.cell_gap;
            let cell_w = (usable_w - total_gap_w) / cols as i32;
            let cell_h = (usable_h - total_gap_h) / rows as i32;
            if cell_w < 40 || cell_h < 30 { continue; }

            let cell_ar = cell_w as f64 / cell_h as f64;

            // Score: average "wasted area" per filled cell
            // wasted = 1 - (video_fill_area / cell_area)
            let mut waste_sum = 0.0;
            for &ar in aspect_ratios.iter() {
                let ar = ar.max(0.3).min(3.0);
                let fill_w = if cell_ar > ar { cell_h as f64 * ar } else { cell_w as f64 };
                let fill_h = if cell_ar > ar { cell_h as f64 } else { cell_w as f64 / ar };
                let fill_area = fill_w * fill_h;
                let cell_area = cell_w as f64 * cell_h as f64;
                let waste = 1.0 - fill_area / cell_area;
                waste_sum += waste;
            }
            let avg_waste = waste_sum / n as f64;

            // Prefer grids that use more of the screen
            let used_frac = n as f64 / (rows * cols) as f64;

            // Score = avg_waste + penalty for empty cells (slight)
            let score = avg_waste + (1.0 - used_frac) * 0.05;

            if score < best_score {
                best_score = score;
                // Generate rects for this grid
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
