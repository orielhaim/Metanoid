#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BrickKind {
    #[default]
    Normal,
    MultiHit,
    Invincible,
    Explosive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpecialType {
    #[default]
    None,
    Moving,
    Regenerating,
    Teleport,
}

#[derive(Debug, Clone)]
pub struct BrickData {
    pub col: usize,
    pub row: usize,
    pub kind: BrickKind,
    pub health: u32,
    pub max_health: u32,
    pub special: SpecialType,
    pub powerup_chance: f32,
}

impl BrickData {
    pub fn normal(col: usize, row: usize) -> Self {
        Self {
            col,
            row,
            kind: BrickKind::Normal,
            health: 1,
            max_health: 1,
            special: SpecialType::None,
            powerup_chance: 0.0,
        }
    }

    pub fn is_destructible(&self) -> bool {
        self.kind != BrickKind::Invincible
    }

    pub fn is_explosive(&self) -> bool {
        self.kind == BrickKind::Explosive
    }
}

/// Visual / physical size of bricks for this level (varies with difficulty).
#[derive(Debug, Clone, Copy)]
pub struct BrickMetrics {
    pub cols: usize,
    pub rows: usize,
    pub brick_w: f32,
    pub brick_h: f32,
    pub gap: f32,
}

impl Default for BrickMetrics {
    fn default() -> Self {
        Self {
            cols: 14,
            rows: 8,
            brick_w: 80.0,
            brick_h: 30.0,
            gap: 4.0,
        }
    }
}

impl BrickMetrics {
    pub fn cell_w(self) -> f32 {
        self.brick_w + self.gap
    }

    pub fn cell_h(self) -> f32 {
        self.brick_h + self.gap
    }
}

#[derive(Debug, Clone)]
pub struct LevelDefinition {
    pub cols: usize,
    pub rows: usize,
    pub bricks: Vec<BrickData>,
    /// Per-level brick sizing (easy = fewer larger bricks).
    pub metrics: BrickMetrics,
}

impl LevelDefinition {
    pub fn destructible_count(&self) -> usize {
        self.bricks.iter().filter(|b| b.is_destructible()).count()
    }

    pub fn with_metrics(mut self, metrics: BrickMetrics) -> Self {
        self.cols = metrics.cols;
        self.rows = metrics.rows;
        self.metrics = metrics;
        self
    }
}
