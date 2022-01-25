#![allow(dead_code)]

use std::collections::VecDeque;

struct Node<T> {
    value: T,
    children: Vec<Node<T>>,
}

#[macro_export]
macro_rules! tree {
    ($first:expr $(, $($rest:expr),*)?) => {
        $crate::basic_search::Node {
            value: $first,
            children: vec![$($($rest),*)?],
        }
    };
}

fn breadth_first_search<T: Eq>(tree: &Node<T>, searched: &T) -> bool {
    let mut candidates = VecDeque::new();
    candidates.push_back(tree);

    while let Some(candidate) = candidates.pop_front() {
        if candidate.value == *searched {
            return true;
        }

        candidates.extend(candidate.children.iter());
    }

    false
}

fn depth_first_search<T: Eq>(tree: &Node<T>, searched: &T) -> bool {
    if tree.value == *searched {
        return true;
    }

    for child in &tree.children {
        if depth_first_search(child, searched) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::basic_search;
    use crate::tree;

    #[test]
    fn dfs() {
        let tree = tree!(1, tree!(6, tree!(5), tree!(4)), tree!(6));

        let has_seven = basic_search::depth_first_search(&tree, &7);
        let has_five = basic_search::depth_first_search(&tree, &5);

        assert!(!has_seven);
        assert!(has_five);
    }

    #[test]
    fn bfs() {
        let tree = tree!(1, tree!(6, tree!(5), tree!(4)), tree!(6));

        let has_seven = basic_search::breadth_first_search(&tree, &7);
        let has_five = basic_search::breadth_first_search(&tree, &5);

        assert!(!has_seven);
        assert!(has_five);
    }
}
