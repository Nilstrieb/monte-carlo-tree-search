#![allow(dead_code)]

use rand::Rng;

mod basic_search;

pub trait GameState {
    fn points(&self) -> i32;

    fn next(&self) -> Box<dyn Iterator<Item = Self>>;
}

impl GameState for i32 {
    fn points(&self) -> i32 {
        *self
    }

    fn next(&self) -> Box<dyn Iterator<Item = Self>> {
        let child_amount = rand::thread_rng().gen_range(0..10);
        let mut i = 0;
        Box::new(std::iter::from_fn(move || {
            if i < child_amount {
                Some(rand::random())
            } else {
                None
            }
        }))
    }
}

#[macro_export]
macro_rules! tree {
    ($first:expr $(, $($rest:expr),*)?) => {
        $crate::Node {
            value: $first,
            children: vec![$($($rest),*)?],
        }
    };
}

mod mcts {}

#[cfg(test)]
mod test {
    #[test]
    fn t() {}
}
