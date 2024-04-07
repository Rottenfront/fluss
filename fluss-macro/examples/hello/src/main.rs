#![allow(non_snake_case)]

use fluss_macro::view;

trait View {}

impl View for () {}

fn Text(text: &str) -> impl View {
    println!("Text: {}", text);
}

enum Direction {
    Horizontal, Vertical
}

fn StackImpl(childs: Vec<Box<dyn View>>) -> impl View {

}

fn Button<F: FnMut()>(on_click: F, childs: Vec<Box<dyn View>>) -> impl View {

}

fn Stack(direction: Direction, childs: Vec<Box<dyn View>>) -> impl View {
    view! {
        StackImpl {
            | childs,
            Text("world")
        }
    }
}

fn main() {
    let mut text = "Hello";
    let view = view! {
        Stack(Direction::Horizontal) {
            {
                text = "world"
            },
            Text({text})
        }
    };
}
