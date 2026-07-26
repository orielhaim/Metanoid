#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrickGrid {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<bool>,
}

impl BrickGrid {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![false; cols * rows],
        }
    }

    pub fn filled(cols: usize, rows: usize) -> Self {
        Self {
            cols,
            rows,
            cells: vec![true; cols * rows],
        }
    }

    pub fn get(&self, col: usize, row: usize) -> bool {
        if col < self.cols && row < self.rows {
            self.cells[row * self.cols + col]
        } else {
            false
        }
    }

    pub fn set(&mut self, col: usize, row: usize, value: bool) {
        if col < self.cols && row < self.rows {
            self.cells[row * self.cols + col] = value;
        }
    }

    pub fn count_filled(&self) -> usize {
        self.cells.iter().filter(|&&c| c).count()
    }

    pub fn fill_ratio(&self) -> f32 {
        let total = self.cols * self.rows;
        if total == 0 {
            return 0.0;
        }
        self.count_filled() as f32 / total as f32
    }

    pub fn mirror_horizontal(&mut self) {
        let half = self.rows / 2;
        for row in 0..half {
            let mirror_row = self.rows - 1 - row;
            for col in 0..self.cols {
                let val = self.get(col, row);
                self.set(col, mirror_row, val);
            }
        }
    }

    pub fn mirror_vertical(&mut self) {
        let half = self.cols / 2;
        for row in 0..self.rows {
            for col in 0..half {
                let mirror_col = self.cols - 1 - col;
                let val = self.get(col, row);
                self.set(mirror_col, row, val);
            }
        }
    }

    pub fn mirror_both(&mut self) {
        self.mirror_horizontal();
        self.mirror_vertical();
    }

    pub fn apply_threshold(&mut self, threshold: f32, values: &[f32]) {
        for (cell, &val) in self.cells.iter_mut().zip(values.iter()) {
            *cell = val >= threshold;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty() {
        let g = BrickGrid::new(5, 3);
        assert_eq!(g.count_filled(), 0);
        assert_eq!(g.cols, 5);
        assert_eq!(g.rows, 3);
    }

    #[test]
    fn filled_count() {
        let g = BrickGrid::filled(4, 3);
        assert_eq!(g.count_filled(), 12);
    }

    #[test]
    fn set_get() {
        let mut g = BrickGrid::new(3, 3);
        g.set(1, 1, true);
        assert!(g.get(1, 1));
        assert!(!g.get(0, 0));
    }

    #[test]
    fn mirror_horizontal_symmetry() {
        let mut g = BrickGrid::new(4, 3);
        g.set(0, 0, true);
        g.set(1, 0, true);
        g.mirror_horizontal();
        assert!(g.get(0, 2));
        assert!(g.get(1, 2));
    }

    #[test]
    fn mirror_vertical_symmetry() {
        let mut g = BrickGrid::new(4, 3);
        g.set(0, 0, true);
        g.set(0, 1, true);
        g.mirror_vertical();
        assert!(g.get(3, 0));
        assert!(g.get(3, 1));
    }

    #[test]
    fn fill_ratio() {
        let mut g = BrickGrid::new(4, 4);
        for i in 0..8 {
            g.cells[i] = true;
        }
        assert!((g.fill_ratio() - 0.5).abs() < 1e-6);
    }
}
