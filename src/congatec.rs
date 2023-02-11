use crate::{
    bindings::{CgosLibInitialize, CgosLibUninitialize},
    board::{Board, BoardClass},
};

pub struct Congatec {}

impl Congatec {
    pub fn new() -> Self {
        assert_ne!(unsafe { CgosLibInitialize() }, 0);
        Self {}
    }

    pub fn get_number_of_boards(&self, class: BoardClass) -> usize {
        Board::amount(class)
    }

    pub fn get_board<'library>(&'library self, class: BoardClass, index: usize) -> Board<'library> {
        Board::new(class, index)
    }

    pub fn get_board_from_name<'library>(&'library self, name: &str) -> Board<'library> {
        Board::from_name(name)
    }
}

impl Drop for Congatec {
    fn drop(&mut self) {
        assert_ne!(unsafe { CgosLibUninitialize() }, 0);
    }
}
