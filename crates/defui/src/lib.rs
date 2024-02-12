// #![feature(type_alias_impl_trait)]

#[cfg(feature = "winit")]
#[macro_use]
extern crate lazy_static;

pub use kurbo::*;
pub use log::debug;
pub use trist::*;

mod view;
pub use view::*;

mod views;
pub use views::*;

mod events;
pub use events::*;

mod view_id;
pub use view_id::*;

mod view_tuple;
pub use view_tuple::*;

mod lens;
pub use lens::*;

mod binding;
pub use binding::*;

mod colors;
pub use colors::*;

mod align;
pub use align::*;

#[cfg(feature = "winit")]
mod winit_event_loop;

#[cfg(feature = "winit")]
pub use winit_event_loop::*;

// See https://rust-lang.github.io/api-guidelines/future-proofing.html
pub(crate) mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {

    // use super::*;

    #[test]
    fn test_button() {
        // let _ = button("click me", |_cx| {
        //     println!("clicked!");
        // });
    }
}
