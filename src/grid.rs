//! Grid layout calculations for multi-video display on 16:9 screens.

/// Represents a grid layout configuration.
pub struct GridLayout {
    pub cols: u32,
    pub rows: u32,
    /// Each cell's width (fraction of screen width)
    pub cell_w: u32,
    /// Each cell's height (fraction of screen height)
    pub cell_h: u32,
}

/// Calculate the optimal grid layout for `n` videos on a 16:9 screen.
///
/// Returns (cols, rows) optimized to:
/// - Fill the screen as much as possible
/// - Preserve ~16:9 aspect ratio per cell
/// - Avoid extreme aspect ratios
pub fn calculate_grid(n: u32, screen_w: u32, screen_h: u32) -> GridLayout {
    let screen_ratio = screen_w as f64 / screen_h as f64;

    // Try common layouts in order of preference
    let candidates: &[(u32, u32)] = match n {
        1 => &[(1, 1)],
        2 => &[(2, 1), (1, 2)],
        3 => &[(3, 1), (1, 3)], // user likely wants 3x1 or press key to switch
        4 => &[(2, 2)],
        5 => &[(3, 2), (2, 3)],
        6 => &[(3, 2), (2, 3)],
        7 => &[(4, 2), (3, 3)],
        8 => &[(4, 2), (3, 3)],
        9 => &[(3, 3)],
        10 => &[(4, 3), (5, 2)],
        11 => &[(4, 3), (3, 4)],
        12 => &[(4, 3), (3, 4), (6, 2)],
        13 => &[(4, 4), (5, 3)],
        14 => &[(4, 4), (5, 3)],
        15 => &[(4, 4), (5, 3)],
        16 => &[(4, 4)],
        _ => {
            // For >16, find the best fit
            let cols = (n as f64 * screen_ratio).sqrt().ceil() as u32;
            let rows = (n as f64 / cols as f64).ceil() as u32;
            // Fallback: just use cols x rows
            &[(cols, rows)]
        }
    };

    // Pick the first layout that fits
    let (cols, rows) = candidates
        .iter()
        .find(|(c, r)| c * r >= n)
        .copied()
        .unwrap_or_else(|| {
            // Last resort: compute dimensions
            let cols = (n as f64).sqrt().ceil() as u32;
            let rows = (n as f64 / cols as f64).ceil() as u32;
            (cols, rows)
        });

    let cell_w = screen_w / cols;
    let cell_h = screen_h / rows;

    GridLayout {
        cols,
        rows,
        cell_w,
        cell_h,
    }
}

/// Get cell position (x, y) for a given cell index.
pub fn cell_position(index: u32, layout: &GridLayout, margin: u32) -> (i32, i32) {
    let col = index % layout.cols;
    let row = index / layout.cols;
    (
        (col * (layout.cell_w)) as i32 + margin as i32,
        (row * (layout.cell_h)) as i32 + margin as i32,
    )
}

/// Get cell size with margins applied.
pub fn cell_size(layout: &GridLayout, margin: u32) -> (i32, i32) {
    (
        (layout.cell_w - 2 * margin) as i32,
        (layout.cell_h - 2 * margin) as i32,
    )
}
