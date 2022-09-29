pub mod eventmgr;
pub use crate::eventmgr::*;

#[derive(Debug)]
pub enum Event {
    One(String),
    Two(&'static [u8]),
    Three
}
