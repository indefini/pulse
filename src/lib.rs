#![feature(box_syntax)]
#![allow(unused_variables)]

extern crate dormin;
extern crate uuid;
extern crate libc;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate serde;

pub mod data;
pub mod state;
pub mod context;
pub mod operation;
pub mod ui;
pub mod dragger;
pub mod control;
pub mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
