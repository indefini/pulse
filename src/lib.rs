#![feature(borrow_state)]
#![feature(box_syntax)]
#![feature(rc_counts)]

extern crate dormin;
extern crate uuid;
extern crate libc;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

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
