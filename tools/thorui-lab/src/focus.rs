#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

pub fn choose(current: Rect, candidates: &[Rect], direction: Direction) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            score(current, *candidate, direction).map(|score| (index, score))
        })
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(index, _)| index)
}

fn score(current: Rect, candidate: Rect, direction: Direction) -> Option<f64> {
    let (x, y) = delta(current, candidate);
    let (forward, cross) = match direction {
        Direction::Up => (-y, x.abs()),
        Direction::Right => (x, y.abs()),
        Direction::Down => (y, x.abs()),
        Direction::Left => (-x, y.abs()),
    };
    (forward > 1.0).then_some(forward + cross * 2.0)
}

fn delta(current: Rect, candidate: Rect) -> (f64, f64) {
    let current_x = current.left + current.width / 2.0;
    let current_y = current.top + current.height / 2.0;
    let candidate_x = candidate.left + candidate.width / 2.0;
    let candidate_y = candidate.top + candidate.height / 2.0;
    (candidate_x - current_x, candidate_y - current_y)
}

#[cfg(test)]
mod tests {
    use super::{Direction, Rect, choose};

    fn rect(left: f64, top: f64) -> Rect {
        Rect {
            left,
            top,
            width: 40.0,
            height: 40.0,
        }
    }

    #[test]
    fn chooses_the_nearest_control_in_the_requested_direction() {
        let candidates = [rect(100.0, 0.0), rect(0.0, 100.0), rect(300.0, 20.0)];
        assert_eq!(
            choose(rect(0.0, 0.0), &candidates, Direction::Right),
            Some(0)
        );
        assert_eq!(
            choose(rect(0.0, 0.0), &candidates, Direction::Down),
            Some(1)
        );
    }

    #[test]
    fn rejects_controls_behind_the_requested_direction() {
        let candidates = [rect(0.0, 100.0)];
        assert_eq!(choose(rect(0.0, 0.0), &candidates, Direction::Up), None);
    }
}
