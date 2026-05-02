//! Grid layout engine: tries uniform grids AND split layouts (portrait column
//! + landscape grid) to minimize black bars across all video aspect ratios.

#[derive(Clone)]
pub struct CellRect {
    pub x: i32, pub y: i32, pub w: i32, pub h: i32,
}

pub struct LayoutConfig {
    pub cell_gap: i32,
    pub outer_pad: i32,
}

/// Calculate per-video positions/sizes. Tries:
/// 1. Uniform grids (all row×col combinations)
/// 2. Split layouts: portrait videos in dedicated column, landscapes in grid
/// Picks the one that maximizes total video fill area.
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
    let screen_area = usable_w as f64 * usable_h as f64;

    // Single video: maximize size preserving AR
    if n == 1 {
        return vec![single_video_rect(aspect_ratios[0], usable_w, usable_h, cfg)];
    }

    let mut best_score: f64 = -1.0;
    let mut best_rects = vec![];

    // ── 1. Uniform grid candidates ──────────────────────────
    try_uniform_grids(aspect_ratios, usable_w, usable_h, screen_area, cfg, &mut best_score, &mut best_rects);

    // ── 2. Split layouts (for mixed portrait+landscape) ─────
    let n_portrait = aspect_ratios.iter().filter(|&&a| a < 0.85).count();
    let n_landscape = aspect_ratios.iter().filter(|&&a| a > 1.15).count();

    if n_portrait > 0 && n_landscape > 0 {
        // Partition into portrait and landscape indices
        let portraits: Vec<(usize, f64)> = aspect_ratios.iter().enumerate()
            .filter(|(_, &a)| a < 0.85).map(|(i, &a)| (i, a)).collect();
        let landscapes: Vec<(usize, f64)> = aspect_ratios.iter().enumerate()
            .filter(|(_, &a)| a > 1.15).map(|(i, &a)| (i, a)).collect();
        let squares: Vec<(usize, f64)> = aspect_ratios.iter().enumerate()
            .filter(|(_, &a)| (0.85..=1.15).contains(&a)).map(|(i, &a)| (i, a)).collect();

        // Try portrait-right split
        try_portrait_split(&portraits, &landscapes, &squares, usable_w, usable_h, screen_area, cfg, true, &mut best_score, &mut best_rects);
        // Try portrait-left split
        try_portrait_split(&portraits, &landscapes, &squares, usable_w, usable_h, screen_area, cfg, false, &mut best_score, &mut best_rects);
    }

    // ── Fallback ────────────────────────────────────────────
    if best_rects.is_empty() {
        best_rects = fallback_layout(aspect_ratios, usable_w, usable_h, cfg);
    }

    best_rects
}

// ── Helpers ──────────────────────────────────────────────────

fn single_video_rect(ar: f64, uw: i32, uh: i32, cfg: &LayoutConfig) -> CellRect {
    let ar = ar.max(0.3).min(3.0);
    let screen_ar = uw as f64 / uh as f64;
    let (w, h) = if ar > screen_ar {
        (uw, (uw as f64 / ar) as i32)
    } else {
        ((uh as f64 * ar) as i32, uh)
    };
    CellRect {
        x: cfg.outer_pad + (uw - w) / 2,
        y: cfg.outer_pad + (uh - h) / 2,
        w, h,
    }
}

fn try_uniform_grids(
    ratios: &[f64], uw: i32, uh: i32, screen_area: f64, cfg: &LayoutConfig,
    best_score: &mut f64, best_rects: &mut Vec<CellRect>,
) {
    let n = ratios.len();
    let max_cols = n.min((uw / 120).max(1) as usize);
    let max_rows = n.min((uh / 80).max(1) as usize);

    for cols in 1..=max_cols {
        for rows in 1..=max_rows {
            if rows * cols < n { continue; }
            if rows * cols - n > n { continue; }

            let tgw = (cols as i32 - 1).max(0) * cfg.cell_gap;
            let tgh = (rows as i32 - 1).max(0) * cfg.cell_gap;
            let cw = (uw - tgw) / cols as i32;
            let ch = (uh - tgh) / rows as i32;
            if cw < 40 || ch < 30 { continue; }

            let cell_ar = cw as f64 / ch as f64;
            let mut total_fill = 0.0;
            for &ar in ratios.iter() {
                total_fill += cell_fill_area(ar.max(0.3).min(3.0), cw, ch, cell_ar);
            }
            let score = total_fill / screen_area;

            if score > *best_score {
                *best_score = score;
                *best_rects = build_rects(ratios.len(), cols, cw, ch, tgw, tgh, cfg);
            }
        }
    }
}

/// Portrait videos in a dedicated column (left or right),
/// landscapes + squares in a grid filling the rest.
///
/// Key insight: sweep across different portrait column widths to find
/// the optimal balance between portrait fill and landscape grid fill.
fn try_portrait_split(
    portraits: &[(usize, f64)], landscapes: &[(usize, f64)], squares: &[(usize, f64)],
    uw: i32, uh: i32, screen_area: f64, cfg: &LayoutConfig,
    portrait_right: bool,
    best_score: &mut f64, best_rects: &mut Vec<CellRect>,
) {
    let gap = cfg.cell_gap;

    let total_portraits = portraits.len() + squares.len();
    if total_portraits == 0 { return; }

    // Collect all "portrait-style" videos for the column
    let mut col_videos: Vec<(usize, f64)> = portraits.to_vec();
    col_videos.extend(squares.iter().copied());
    col_videos.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    let n_col = col_videos.len();
    let n_land = landscapes.len();
    if n_land == 0 { return; }

    // "Natural" column width from portrait ARs — upper bound
    let inv_sum: f64 = col_videos.iter().map(|(_, a)| 1.0 / a.max(0.3)).sum();
    let natural_w = (uh as f64 / inv_sum) as i32;

    // Sweep column widths: from minimum reasonable to natural width
    // Try ~20 steps across the range
    let min_w = (uw / 10).max(120); // minimum viewable portrait width
    let max_w = natural_w.max(min_w).min(uw * 2 / 3);
    let steps = 20;
    let step_size = ((max_w - min_w) as f64 / steps as f64).max(1.0) as i32;

    for step in 0..=steps {
        let col_w = (min_w + step * step_size).min(max_w);
        let land_w = uw - col_w - gap;
        if land_w < 120 { continue; }

        // Calculate portrait fill for this column width
        // Each portrait: ideal height = col_w / ar (to fill width perfectly)
        let total_ideal_h: f64 = col_videos.iter()
            .map(|(_, a)| col_w as f64 / a.max(0.3).max(0.3))
            .sum();
        let total_gap_h = (n_col as i32 - 1).max(0) * gap;
        // Scale to fit available height
        let scale = if total_ideal_h + total_gap_h as f64 > uh as f64 {
            (uh - total_gap_h) as f64 / total_ideal_h
        } else {
            1.0
        };
        let scale = scale.max(0.1);

        let scaled_w = (col_w as f64 * scale) as i32;
        let mut p_fill = 0.0;
        for &(_, ar) in &col_videos {
            let ar = ar.max(0.3).min(3.0);
            let vid_h = ((col_w as f64 / ar) * scale) as i32;
            let vid_h = vid_h.max(30);
            let cell_ar = if vid_h > 0 { scaled_w as f64 / vid_h as f64 } else { 1.0 };
            p_fill += cell_fill_area(ar, scaled_w, vid_h, cell_ar);
        }

        // Try landscape grid layouts with the remaining width
        let max_lcols = n_land.min((land_w / 120).max(1) as usize);
        for lcols in 1..=max_lcols {
            for lrows in 1..=n_land {
                if lrows * lcols < n_land { continue; }
                if lrows * lcols - n_land > n_land { continue; }

                let ltgw = (lcols as i32 - 1).max(0) * gap;
                let ltgh = (lrows as i32 - 1).max(0) * gap;
                let lcw = (land_w - ltgw) / lcols as i32;
                let lch = (uh - ltgh) / lrows as i32;
                if lcw < 40 || lch < 30 { continue; }

                let cell_ar = lcw as f64 / lch as f64;
                let mut l_fill = 0.0;
                for &(_, ar) in landscapes {
                    l_fill += cell_fill_area(ar.max(0.3).min(3.0), lcw, lch, cell_ar);
                }

                let score = (p_fill + l_fill) / screen_area;
                if score > *best_score {
                    *best_score = score;

                    let col_x = if portrait_right { cfg.outer_pad + land_w + gap } else { cfg.outer_pad };
                    let land_x = if portrait_right { cfg.outer_pad } else { cfg.outer_pad + col_w + gap };

                    let mut rects = vec![CellRect{x:0,y:0,w:0,h:0}; portraits.len() + landscapes.len() + squares.len()];

                    // Portrait column rects
                    let mut y = cfg.outer_pad;
                    for &(orig_idx, ar) in &col_videos {
                        let ar = ar.max(0.3).min(3.0);
                        let vid_h = ((col_w as f64 / ar) * scale) as i32;
                        let vid_h = vid_h.max(30);
                        rects[orig_idx] = CellRect { x: col_x, y, w: scaled_w, h: vid_h };
                        y += vid_h + gap;
                    }

                    // Landscape grid rects
                    for (j, &(orig_idx, _)) in landscapes.iter().enumerate() {
                        let col = (j % lcols) as i32;
                        let row = (j / lcols) as i32;
                        rects[orig_idx] = CellRect {
                            x: land_x + col * (lcw + gap),
                            y: cfg.outer_pad + row * (lch + gap),
                            w: lcw, h: lch,
                        };
                    }

                    *best_rects = rects;
                }
            }
        }
    }
}

fn cell_fill_area(ar: f64, cw: i32, ch: i32, cell_ar: f64) -> f64 {
    if cell_ar > ar {
        (ch as f64 * ar) * ch as f64  // fw = ch*ar, fh = ch
    } else {
        (cw as f64) * (cw as f64 / ar) // fw = cw, fh = cw/ar
    }
}

fn build_rects(n: usize, cols: usize, cw: i32, ch: i32, _tgw: i32, _tgh: i32, cfg: &LayoutConfig) -> Vec<CellRect> {
    let mut rects = Vec::with_capacity(n);
    for i in 0..n {
        let col = (i % cols) as i32;
        let row = (i / cols) as i32;
        rects.push(CellRect {
            x: cfg.outer_pad + col * (cw + cfg.cell_gap),
            y: cfg.outer_pad + row * (ch + cfg.cell_gap),
            w: cw, h: ch,
        });
    }
    rects
}

fn fallback_layout(ratios: &[f64], uw: i32, uh: i32, cfg: &LayoutConfig) -> Vec<CellRect> {
    let n = ratios.len();
    let ch = uh / n as i32;
    (0..n).map(|i| CellRect {
        x: cfg.outer_pad,
        y: cfg.outer_pad + i as i32 * (ch + cfg.cell_gap),
        w: uw, h: ch,
    }).collect()
}
