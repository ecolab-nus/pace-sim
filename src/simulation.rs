use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::machine::pe::PEState;

pub struct PE{
    pub state: PEState,
    pub north: Option<Weak<RefCell<PE>>>,
    pub south: Option<Weak<RefCell<PE>>>,
    pub east: Option<Weak<RefCell<PE>>>,
    pub west: Option<Weak<RefCell<PE>>>,
}

/// A convenient alias for a reference-counted, mutable PE
pub type PENode = Rc<RefCell<PE>>;

pub type PEGrid = Vec<Vec<PENode>>;

/// Build a grid of PEs of size `width x height`, initializing each with `init(x, y)`
pub fn create_pe_grid<F>(
    width: usize,
    height: usize,
    mut init: F,
) -> Vec<Vec<PENode>>
where
    F: FnMut(usize, usize) -> PEState,
{
    // 1) Allocate all nodes with no links
    let mut grid: Vec<Vec<PENode>> = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| {
                    Rc::new(RefCell::new(PE {
                        state: init(x, y),
                        north: None,
                        south: None,
                        west: None,
                        east: None,
                    }))
                })
                .collect()
        })
        .collect();

    // 2) Link neighbors using Weak references to avoid reference cycles
    for y in 0..height {
        for x in 0..width {
            let node = &grid[y][x];
            let mut pe = node.borrow_mut();

            // North neighbor
            if y > 0 {
                pe.north = Some(Rc::downgrade(&grid[y - 1][x]));
            }
            // South neighbor
            if y + 1 < height {
                pe.south = Some(Rc::downgrade(&grid[y + 1][x]));
            }
            // West neighbor
            if x > 0 {
                pe.west = Some(Rc::downgrade(&grid[y][x - 1]));
            }
            // East neighbor
            if x + 1 < width {
                pe.east = Some(Rc::downgrade(&grid[y][x + 1]));
            }
        }
    }

    grid
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_pe_grid() {
        let grid: PEGrid = create_pe_grid(4, 4, |x, y| PEState::default());
        assert_eq!(grid.len(), 4);
        assert_eq!(grid[0].len(), 4);
    }
}
