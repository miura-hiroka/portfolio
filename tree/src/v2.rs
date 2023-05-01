//! Owned Tree

use std::{ptr, fmt};
use crate::util;

#[derive(Debug, PartialEq)]
pub struct Tree<T> {
    root: Box<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    value: T,
    children: Vec<Box<Self>>,
    parent: *mut Self,
}

#[allow(unused)]
impl<T> Node<T> {
    fn new(value: T) -> Self {
        Self { value, children: Vec::new(), parent: ptr::null_mut() }
    }
    fn get_parent(&self) -> Option<&Self> {
        if self.parent.is_null() { None }
        else { unsafe { Some(&*self.parent) } }
    }
    fn get_index(&self) -> Option<usize> {
        if self.parent.is_null() { None }
        else { unsafe {
            (*self.parent).children.iter()
                .position(|elem| ptr::eq(&**elem, self))
        } }
    }
    fn get_next_sibling(&self) -> Option<&Self> {
        let index = self.get_index()?;
        unsafe {
            (*self.parent).children.get(index + 1)
                .map(|boxed| &**boxed)
        }
    }
    fn depth(&self) -> usize {
        if self.parent.is_null() { 0 }
        else { unsafe { (*self.parent).depth() + 1 } }
    }
}

impl<T: PartialEq> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value &&
        self.children == other.children
    }
}

impl<T: Clone> Clone for Box<Node<T>> {
    fn clone(&self) -> Box<Node<T>> {
        let mut current = Box::new(Node {
            value: self.value.clone(),
            children: Vec::with_capacity(self.children.len()),
            parent: ptr::null_mut(),
        });
        let ptr: *mut Node<T> = &mut *current;
        for child in &self.children {
            let mut cloned = child.clone();
            cloned.parent = ptr;
            current.children.push(cloned);
        }
        current
    }
}

impl<T: fmt::Display> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        let mut it = self.children.iter();
        if let Some(first) = it.next() {
            s.push_str((**first).to_string().as_str());
        } else {
            return write!(f, "{}", self.value);
        }
        for child in it {
            s.push_str(format!(" {}", **child).as_str());
        }
        write!(f, "{}({})", self.value, s)
    }
}

impl<T> Tree<T> {
    pub fn new(value: T) -> Self {
        let node = Node { value, children: Vec::new(), parent: ptr::null_mut() };
        let root = Box::new(node);
        Self { root }
    }

    pub fn get_value_root(&self) -> &T {
        &self.root.value
    }

    pub fn append_value(&mut self, value: T) {
        let node = Node { value, children: Vec::new(), parent: &mut *self.root };
        self.root.children.push(Box::new(node));
    }

    pub fn append_tree(&mut self, mut t: Self) {
        t.root.parent = &mut *self.root;
        self.root.children.push(t.root);
    }

    pub fn traverse(&self) -> Traversal<'_, T> {
        Traversal::from(self)
    }

    pub fn cursor_at(&self, path: &[usize]) -> Option<Cursor<'_, T>> {
        let mut current = &*self.root;
        for &index in path {
            current = current.children.get(index)?;
        }
        Some(Cursor { target: current })
    }

    pub fn cursor(&self) -> Cursor<'_, T> {
        Cursor { target: &self.root }
    }
    
    pub fn cursor_mut_at(&mut self, path: &[usize]) -> Option<CursorMut<'_, T>> {
        let mut current = &mut *self.root;
        for &index in path {
            current = current.children.get_mut(index)?;
        }
        Some(CursorMut { target: current })
    }

    pub fn cursor_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut { target: &mut self.root }
    }
}

impl<T: Clone> Tree<T> {
    
    pub fn clone_child(&self, index: usize) -> Option<Self> {
        let child = self.root.children.get(index)?;
        Some(Self { root: child.clone() })
    }
}

impl<T: Clone> Clone for Tree<T> {
    fn clone(&self) -> Self {
        let mut stack_src = vec![&*self.root];
        let mut stack_new = Vec::new();
        loop {
            let ref_current = stack_src.pop().unwrap();
            stack_src.extend(
                ref_current.children.iter().rev()
                    .map(|node| &**node)
            );
            let len = ref_current.children.len();
            let new_node = Box::new(Node {
                value: ref_current.value.clone(),
                children: Vec::with_capacity(len),
                parent: ptr::null_mut(),
            });
            if 0 < len {
                stack_new.push((new_node, len));
                continue;
            }
            let mut last = new_node;
            loop {
                if stack_new.len() == 0 { return Self { root: last }; }
                let mut penultimate = stack_new.last_mut().unwrap();
                last.parent = &mut *penultimate.0;
                penultimate.0.children.push(last);
                penultimate.1 -= 1;
                if penultimate.1 > 0 { break; }
                last = stack_new.pop().unwrap().0;
            }
        }
    }
}

impl std::str::FromStr for Tree<String> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut v = util::parse_tree(s)?;
        v.truncate(1);
        v.pop().ok_or_else(|| "empty input".to_string())
    }
}

pub struct Traversal<'a, T> {
    stack: Vec<&'a Node<T>>,
}
impl<'a, T> From<&'a Tree<T>> for Traversal<'a, T> {
    fn from(t: &'a Tree<T>) -> Self {
        Self { stack: vec![&*t.root] }
    }
}
impl<'a, T> Iterator for Traversal<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.stack.pop()?;
        self.stack.extend(ret.children.iter().rev().map(|child| &**child));
        Some(&ret.value)
    }
}

pub struct TraversalDepth<'a, T> {
    stack: Vec<(&'a Node<T>, usize)>,
}
impl<'a, T> From<&'a Tree<T>> for TraversalDepth<'a, T> {
    fn from(t: &'a Tree<T>) -> Self {
        Self { stack: vec![(&*t.root, 0)] }
    }
}
impl<'a, T> Iterator for TraversalDepth<'a, T> {
    type Item = (&'a T, usize);
    fn next(&mut self) -> Option<Self::Item> {
        let (ret, depth) = self.stack.pop()?;
        let next_depth = depth + 1;
        self.stack.extend(ret.children.iter().rev().map(|child| (&**child, next_depth)));
        Some((&ret.value, depth))
    }
}

impl<T> fmt::Display for Tree<T>
where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let it = TraversalDepth::from(self);
        if f.alternate() {
            for (t, depth) in it {
                let indent = "  ".repeat(depth);
                write!(f, "{}{}\n", indent, t)?;
            }
        } else {
            write!(f, "{}", *self.root)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Cursor<'a, T> {
    target: &'a Node<T>,
}

impl<'a, T> Cursor<'a, T> {
    pub fn get(&self) -> &T {
        &self.target.value
    }
    pub fn move_to_parent(&mut self) -> Result<(), ()> {
        if self.target.parent.is_null() {
            Err(())
        } else {
            self.target = unsafe { &*self.target.parent };
            Ok(())
        }
    }
    pub fn move_to_child(&mut self, idx: usize) -> Result<(), ()> {
        if let Some(child) = self.target.children.get(idx) {
            self.target = child;
            Ok(())
        } else {
            Err(())
        }
    }
    pub fn clone_child(&self, idx: usize) -> Result<Self, ()> {
        if let Some(child) = self.target.children.get(idx) {
            Ok(Self { target: child })
        } else {
            Err(())
        }
    }
    pub fn children(&self) -> IterChildren<'a, T> {
        IterChildren { children: &self.target.children, next_idx: 0 }
    }
}

pub struct IterChildren<'a, T> {
    children: &'a Vec<Box<Node<T>>>,
    next_idx: usize,
}

impl<'a, T> Iterator for IterChildren<'a, T> {
    type Item = Cursor<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.children.get(self.next_idx)
            .map(|node| {
                self.next_idx += 1;
                Cursor { target: &**node }
            })
    }
}

pub struct CursorMut<'a, T> {
    target: &'a mut Node<T>,
}

impl<'a, T> CursorMut<'a, T> {
    pub fn get(&mut self) -> &mut T {
        &mut self.target.value
    }
    pub fn append_value(&mut self, value: T) {
        let node = Box::new(Node {
            value,
            children: Vec::new(),
            parent: self.target,
        });
        self.target.children.push(node);
    }
    pub fn append_tree(&mut self, tree: Tree<T>) {
        let mut node = tree.root;
        node.parent = self.target;
        self.target.children.push(node);
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut t0 = Tree::new(0);
        t0.append_value(1);
        let mut t1 = Tree::new(2);
        t1.append_value(3);
        t1.append_value(4);
        t0.append_tree(t1);
        
        let mut t2 = t0.clone();
        assert_eq!(t0, t2);
        t2.append_value(5);
        assert_ne!(t0, t2);

        let mut tr = t0.traverse();
        assert_eq!(tr.next(), Some(&0));
        assert_eq!(tr.next(), Some(&1));
        assert_eq!(tr.next(), Some(&2));
        assert_eq!(tr.next(), Some(&3));
        assert_eq!(tr.next(), Some(&4));

        let t9 = Tree::new("a".to_string());
        assert_eq!(format!("{}", t9), "a");
    }
    #[test]
    fn cursor() {
        let mut t0 = "a(b c(d e) f(g))".parse::<Tree<_>>().unwrap();
        let b_path = [0];
        let d_path = [1, 0];
        let f_path = [2];
        let cursor = t0.cursor_at(&d_path).unwrap();
        assert_eq!(cursor.get(), "d");
        let mut cursor_mut = t0.cursor_mut_at(&b_path).unwrap();
        assert_eq!(cursor_mut.get(), "b");
        cursor_mut.append_value("0".to_string());
        cursor_mut = t0.cursor_mut_at(&f_path).unwrap();
        cursor_mut.append_value("h".to_string());
        let t1 = "F(x y z)".parse::<Tree<_>>().unwrap();
        cursor_mut = t0.cursor_mut_at(&d_path).unwrap();
        cursor_mut.append_tree(t1);
        let t2 = "a(b(0) c(d(F(x y z)) e) f(g h))".parse::<Tree<_>>().unwrap();
        assert_eq!(t0, t2);
    }
}
