//! Grid layout: aspect-ratio-aware layout that minimizes black bars.

/// Per-cell layout info
pub struct CellRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Layout padding configuration
pub struct LayoutPadding {
    /// Gap between cells
    pub cell_gap: i32,
    /// Outer padding around the entire grid
    pub outer_pad: i32,
}

/// Calculate per-cell positions and sizes based on video aspect ratios.
///
/// Strategy:
/// - All videos in a row share the same height
/// - Cell width is proportional to the video's aspect ratio
/// - Rows fill the available width
/// - Mixed orientation (portrait + landscape) videos go in separate rows
pub fn calculate_layout(
    aspect_ratios: &[f64],
    available_w: i32,
    available_h: i32,
    pad: &LayoutPadding,
) -> Vec<CellRect> {
    let n = aspect_ratios.len();
    if n == 0 {
        return vec![];
    }

    let usable_w = available_w - 2 * pad.outer_pad;
    let usable_h = available_h - 2 * pad.outer_pad;

    // Single video: use full area, preserve aspect ratio
    if n == 1 {
        let ar = aspect_ratios[0].max(0.5).min(2.5);
        let screen_ar = usable_w as f64 / usable_h as f64;
        let (w, h) = if ar > screen_ar {
            (usable_w, (usable_w as f64 / ar) as i32)
        } else {
            (((usable_h as f64) * ar) as i32, usable_h)
        };
        let x = pad.outer_pad + (usable_w - w) / 2;
        let y = pad.outer_pad + (usable_h - h) / 2;
        return vec![CellRect { x, y, w, h }];
    }

    // For multiple videos: try to find optimal row arrangement
    let best_rows = find_best_row_split(aspect_ratios, usable_w as f64, usable_h as f64);

    // Lay out rows
    let total_ideal_height: f64 = best_rows.iter().map(|r| r.ideal_height).sum();
    let total_gap = (best_rows.len() as i32 - 1).max(0) * pad.cell_gap;
    let scale = (usable_h - total_gap) as f64 / total_ideal_height;

    let mut rects = Vec::with_capacity(n);
    let mut y_offset = pad.outer_pad;

    for row in &best_rows {
        let row_height = (row.ideal_height * scale) as i32;
        let row_count = row.end - row.start;

        // Calculate cell widths to fill the row
        let total_gap_w = (row_count as i32 - 1).max(0) * pad.cell_gap;
        let available_row_w = usable_w - total_gap_w;

        // Sum of aspect ratios in this row (width / height = ar, so width = ar * height)
        let ar_sum: f64 = (row.start..row.end).map(|i| aspect_ratios[i]).sum();
        // cell_w[i] = aspect_ratios[i] * row_height
        // Sum of widths = row_height * ar_sum
        // Scale so sum fits: scale_factor = available_row_w / (row_height * ar_sum)
        let scale_factor = if ar_sum > 0.0 {
            available_row_w as f64 / (row_height as f64 * ar_sum)
        } else {
            1.0
        };

        let mut x_offset = pad.outer_pad;
        for i in row.start..row.end {
            let ar = aspect_ratios[i].max(0.3).min(3.0);
            let cell_w = ((ar * row_height as f64) * scale_factor) as i32;
            rects.push(CellRect {
                x: x_offset,
                y: y_offset,
                w: cell_w.max(40),
                h: row_height.max(40),
            });
            x_offset += cell_w + pad.cell_gap;
        }
        y_offset += row_height + pad.cell_gap;
    }

    rects
}

struct RowSplit {
    start: usize,
    end: usize,
    /// Ideal height before scaling (based on tallest video in row)
    ideal_height: f64,
}

/// Split videos into rows that fill width evenly.
/// Uses a greedy algorithm: add videos to current row until width exceeds target,
/// then start new row. Target width is the available screen width.
fn find_best_row_split(ratios: &[f64], available_w: f64, _available_h: f64) -> Vec<RowSplit> {
    let n = ratios.len();

    // Simple cases: 1-4 videos in a single row each
    if n <= 3 {
        // All in one row
        let max_ar = ratios.iter().cloned().fold(0.0f64, f64::max);
        let ideal_h = available_w / (ratios.iter().sum::<f64>() * 1.05); // 5% padding
        return vec![RowSplit {
            start: 0,
            end: n,
            ideal_height: ideal_h.max(available_w / (max_ar * n as f64)),
        }];
    }

    // For 4+ videos, try to split into balanced rows
    // Greedy: fill each row to ~available_w
    let mut rows = Vec::new();
    let mut start = 0usize;

    while start < n {
        let mut sum_ar = 0.0;
        let mut end = start;
        let mut max_ar: f64 = 0.0;

        while end < n {
            let ar = ratios[end].max(0.3).min(3.0);
            // Height if we add this video: h = available_w / (sum_ar + ar)
            let new_sum = sum_ar + ar;
            let h = available_w / new_sum;
            // Try to keep cell height reasonable (not too tall, not too short)
            if end > start && h < available_w / (ratios.iter().skip(start).take(end - start + 1).sum::<f64>().max(0.1) * 3.0) {
                break;
            }
            max_ar = max_ar.max(ar);
            sum_ar = new_sum;
            end += 1;

            // Don't put too many videos in one row
            if end - start >= 5 {
                break;
            }
        }

        if end == start {
            end = start + 1; // At least one video per row
            sum_ar = ratios[start].max(0.3).min(3.0);
        }

        let ideal_h = available_w / sum_ar.max(0.01);
        rows.push(RowSplit {
            start,
            end,
            ideal_height: ideal_h,
        });
        start = end;
    }

    rows
}
