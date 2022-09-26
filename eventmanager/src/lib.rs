
pub mod eventmgr;
pub use crate::eventmgr::*;

pub enum Event {
    One(String),
    Two(&'static [u8]),
    Three
}
