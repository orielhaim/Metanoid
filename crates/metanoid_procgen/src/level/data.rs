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

#[derive(Debug, Clone)]
pub struct LevelDefinition {
    pub cols: usize,
    pub rows: usize,
    pub bricks: Vec<BrickData>,
}

impl LevelDefinition {
    pub fn destructible_count(&self) -> usize {
        self.bricks.iter().filter(|b| b.is_destructible()).count()
    }
}
