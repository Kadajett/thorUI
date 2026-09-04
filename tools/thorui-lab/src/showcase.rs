use serde::{Deserialize, Serialize};

const MAX_MARKS: usize = 320;
const CURSOR_STEP: f64 = 0.018;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub const CENTER: Self = Self { x: 0.5, y: 0.5 };

    #[must_use]
    pub fn clamped(self) -> Self {
        Self {
            x: self.x.clamp(0.02, 0.98),
            y: self.y.clamp(0.04, 0.96),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum LumenColor {
    Mint,
    Cyan,
    Violet,
    Coral,
    Gold,
}

impl LumenColor {
    pub const ALL: [Self; 5] = [
        Self::Mint,
        Self::Cyan,
        Self::Violet,
        Self::Coral,
        Self::Gold,
    ];

    #[must_use]
    pub const fn css(self) -> &'static str {
        match self {
            Self::Mint => "#72f4c4",
            Self::Cyan => "#65d8ff",
            Self::Violet => "#ae8cff",
            Self::Coral => "#ff7d8f",
            Self::Gold => "#ffd36e",
        }
    }

    #[must_use]
    pub fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|color| *color == self)
            .unwrap_or_default();
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct Mark {
    pub point: Point,
    pub color: LumenColor,
    pub strength: f64,
    pub sequence: u32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ShowcaseAction {
    Move {
        horizontal: f64,
        vertical: f64,
        paint: bool,
    },
    Paint {
        point: Point,
        strength: f64,
    },
    SelectColor {
        color: LumenColor,
    },
    CycleColor,
    Clear,
    SetPeer {
        connected: bool,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShowcaseModel {
    cursor: Point,
    color: LumenColor,
    marks: Vec<Mark>,
    next_sequence: u32,
    peer_connected: bool,
}

impl Default for ShowcaseModel {
    fn default() -> Self {
        Self {
            cursor: Point::CENTER,
            color: LumenColor::Mint,
            marks: Vec::new(),
            next_sequence: 1,
            peer_connected: false,
        }
    }
}

impl ShowcaseModel {
    #[must_use]
    pub fn apply(mut self, action: ShowcaseAction) -> Self {
        match action {
            ShowcaseAction::Move {
                horizontal,
                vertical,
                paint,
            } => {
                self.cursor = Point {
                    x: self.cursor.x + horizontal * CURSOR_STEP,
                    y: self.cursor.y + vertical * CURSOR_STEP,
                }
                .clamped();
                if paint {
                    self.push_mark(self.cursor, 0.72);
                }
            }
            ShowcaseAction::Paint { point, strength } => {
                self.cursor = point.clamped();
                self.push_mark(self.cursor, strength.clamp(0.15, 1.0));
            }
            ShowcaseAction::SelectColor { color } => self.color = color,
            ShowcaseAction::CycleColor => self.color = self.color.next(),
            ShowcaseAction::Clear => self.marks.clear(),
            ShowcaseAction::SetPeer { connected } => self.peer_connected = connected,
        }
        self
    }

    #[must_use]
    pub const fn cursor(&self) -> Point {
        self.cursor
    }

    #[must_use]
    pub const fn color(&self) -> LumenColor {
        self.color
    }

    #[must_use]
    pub fn marks(&self) -> &[Mark] {
        &self.marks
    }

    #[must_use]
    pub const fn peer_connected(&self) -> bool {
        self.peer_connected
    }

    fn push_mark(&mut self, point: Point, strength: f64) {
        self.marks.push(Mark {
            point,
            color: self.color,
            strength,
            sequence: self.next_sequence,
        });
        self.next_sequence += 1;
        if self.marks.len() > MAX_MARKS {
            self.marks.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LumenColor, Point, ShowcaseAction, ShowcaseModel};

    #[test]
    fn movement_is_clamped_and_can_paint() {
        let model = ShowcaseModel::default().apply(ShowcaseAction::Move {
            horizontal: 100.0,
            vertical: -100.0,
            paint: true,
        });

        assert_eq!(model.cursor(), Point { x: 0.98, y: 0.04 });
        assert_eq!(model.marks().len(), 1);
    }

    #[test]
    fn color_selection_applies_to_the_next_mark() {
        let model = ShowcaseModel::default()
            .apply(ShowcaseAction::SelectColor {
                color: LumenColor::Coral,
            })
            .apply(ShowcaseAction::Paint {
                point: Point::CENTER,
                strength: 1.0,
            });

        assert_eq!(model.color(), LumenColor::Coral);
        assert_eq!(model.marks()[0].color, LumenColor::Coral);
    }

    #[test]
    fn clear_preserves_cursor_and_color() {
        let model = ShowcaseModel::default()
            .apply(ShowcaseAction::CycleColor)
            .apply(ShowcaseAction::Paint {
                point: Point::CENTER,
                strength: 1.0,
            });
        let cleared = model.apply(ShowcaseAction::Clear);

        assert_eq!(cleared.color(), LumenColor::Cyan);
        assert!(cleared.marks().is_empty());
    }
}
