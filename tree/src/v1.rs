//! `Node` is shared by `Rc`s

use std::{rc::{Rc, Weak}, fmt, cell::{RefCell, Ref, RefMut}, ptr, mem};


struct TreeNode<T> {
    value: T,
    parent: Weak<RefCell<TreeNode<T>>>,
    children: Vec<Rc<RefCell<TreeNode<T>>>>,
    // ref_self: Weak<Tree<T>>,
}

impl<T: PartialEq> PartialEq for TreeNode<T> {
    fn eq(&self, other: &Self) -> bool {
        if self.value == other.value {
            self.children == other.children
        } else {
            false
        }
    }
}

#[derive(PartialEq)]
pub struct Tree<T>(Rc<RefCell<TreeNode<T>>>);

impl<T> Clone for Tree<T> {
    fn clone(&self) -> Self {
        Tree(Rc::clone(&self.0))
    }
}

impl<T> Tree<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(RefCell::new(TreeNode { value, parent: Weak::new(), children: Vec::new() })))
    }
    fn borrow_node(&self) -> Ref<'_, TreeNode<T>> {
        self.0.as_ref().borrow()
    }
    fn borrow_mut_node(&self) -> RefMut<'_, TreeNode<T>> {
        self.0.as_ref().borrow_mut()
    }
    pub fn is_parent_of(&self, t: &Self) -> bool {
        ptr::eq(Rc::as_ptr(&self.0), t.borrow_node().parent.as_ptr())
    }
    pub fn is_root(&self) -> bool {
        self.borrow_node().parent.upgrade().is_none()
    }
    pub fn is_leaf(&self) -> bool {
        self.borrow_node().children.is_empty()
    }
    pub fn num_children(&self) -> usize {
        self.borrow_node().children.len()
    }
    pub fn depth(&self) -> usize {
        let mut i = 0;
        let mut t = Rc::clone(&self.0);
        loop {
            let Some(u) = t.as_ref().borrow().parent.upgrade()
                else { break };
            i += 1;
            t = u;
        }
        i
    }
    pub fn get_parent(&self) -> Option<Tree<T>> {
        self.borrow_node().parent.upgrade().map(|t| Tree(t))
    }
    pub fn get_child(&self, idx: usize) -> Option<Tree<T>> {
        self.borrow_node().children.get(idx).map(|t| Tree(Rc::clone(t)))
    }
    pub fn get_children(&self) -> IterChild<T> {
        IterChild { parent: Tree::clone(self), next_idx: 0 }
    }
    pub fn get_child_idx(&self) -> Option<usize> {
        let parent = self.get_parent()?;
        let ref_node = parent.borrow_node();
        ref_node.children.iter()
            .position(|t| ptr::eq(Rc::as_ptr(t), Rc::as_ptr(&self.0)))
    }
    pub fn find_child_idx<P>(&self, p: P) -> Option<usize>
    where P: FnMut(&Tree<T>) -> bool {
        let mut p = p;
        self.borrow_node().children.iter()
            .position(|t| p(&Tree(Rc::clone(t))))
    }
    fn get_parent_and_idx(&self) -> Option<(Self, usize)> {
        let parent = self.get_parent()?;
        let idx = parent.borrow_node().children.iter()
            .position(|t| ptr::eq(Rc::as_ptr(t), Rc::as_ptr(&self.0)))
            .unwrap();
        Some((parent, idx))
    }
    pub fn unlink_parent(&mut self) -> Option<Self> {
        let (parent, index) = self.get_parent_and_idx()?;
        parent.borrow_mut_node().children.remove(index);
        self.borrow_mut_node().parent = Weak::new();
        Some(parent)
    }
    pub fn unlink_child(&mut self, idx: usize) -> Option<Self> {
        let child = self.get_child(idx)?;
        child.borrow_mut_node().parent = Weak::new();
        self.borrow_mut_node().children.remove(idx);
        Some(child)
    }
    pub fn link_push(&mut self, t: &Self) -> bool {
        if self.is_parent_of(t) { return false; }
        if let Some((parent, index)) = t.get_parent_and_idx() {
            parent.borrow_mut_node().children.remove(index);
        }
        t.borrow_mut_node().parent = Rc::downgrade(&self.0);
        self.borrow_mut_node().children.push(Rc::clone(&t.0));
        true
    }
    pub fn link_idx(&mut self, idx: usize, t: &Self) -> Option<Self> {
        if self.is_parent_of(t) { return None; }
        let mut ref_mut = self.borrow_mut_node();
        let ch = ref_mut.children.get_mut(idx)?;
        let old = Tree(mem::replace(ch, Rc::clone(&t.0)));
        t.borrow_mut_node().parent = Rc::downgrade(&self.0);
        old.borrow_mut_node().parent = Weak::new();
        Some(old)
    }
    pub fn replace(&self, t: &Self) -> bool {
        if let Some((parent, idx)) = self.get_parent_and_idx() {
            parent.borrow_mut_node().children[idx] = Rc::clone(&t.0);
        }
        if let Some((parent, idx)) = t.get_parent_and_idx() {
            parent.borrow_mut_node().children[idx] = Rc::clone(&self.0);
        }
        let x = &mut self.borrow_mut_node().parent;
        let y = &mut t.borrow_mut_node().parent;
        mem::swap(x, y);
        true
    }
    pub fn traverse(&self) -> Traversal<T> {
        Traversal(Some(TraversalData { current: Tree::clone(self) }))
    }
    pub fn search_path(&self, p: &[usize]) -> Option<Self> {
        let mut t = Rc::clone(&self.0);
        let mut p = p;
        loop {
            if p.is_empty() {
                return Some(Tree(t));
            }
            t = {
                let tmp = t.as_ref().borrow();
                Rc::clone(tmp.children.get(p[0])?)
            };
            p = &p[1..];
        }
    }
    pub fn set(&mut self, value: T) {
        self.borrow_mut_node().value = value;
    }
    pub fn get_ref(&self) -> Ref<'_, T> {
        Ref::map(self.borrow_node(), |t| &t.value)
    }
    pub fn get_mut(&mut self) -> RefMut<'_, T> {
        RefMut::map(self.borrow_mut_node(), |t| &mut t.value)
    }
}
impl<T> Tree<T>
where T: PartialEq {
    pub fn matches(&self, t: &Self) -> impl Iterator<Item = Self> {
        let t = Tree::clone(t);
        self.traverse().filter(move |u| t == *u)
    }
}
impl<T> Tree<T> where T: Clone {
    pub fn clone_src(&self) -> Self {
        let t = self.borrow_node();

        let tn = TreeNode {value: t.value.clone(), parent: Weak::new(), children: Vec::new() };
        let t = Tree(Rc::new(RefCell::new(tn)));
        
        t
    }
}

pub struct IterChild<T> {
    parent: Tree<T>,
    next_idx: usize,
}

impl<T> Iterator for IterChild<T> {
    type Item = Tree<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = self.next_idx;
        self.next_idx += 1;
        self.parent.get_child(idx)
    }
}


struct TraversalData<T> {
    current: Tree<T>,
}

pub struct Traversal<T>(Option<TraversalData<T>>);

impl<T> Iterator for Traversal<T> {
    type Item = Tree<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let Some(state) = &mut self.0 else { return None };
        if let Some(next) = state.current.get_child(0) {
            let old = mem::replace(&mut state.current, next);
            return Some(old);
        }
        let mut t = Tree::clone(&state.current);
        loop {
            if let Some((parent, idx)) = t.get_parent_and_idx() {
                if let Some(next) = parent.get_child(idx + 1) {
                    let old = mem::replace(&mut state.current, next);
                    return Some(old);
                }
                t = parent;
            } else {
                let Some(old) = self.0.take() else { panic!() };
                return Some(old.current);
            }
        }
    }
}


impl<T> fmt::Display for Tree<T>
where T: fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let it = self.traverse();
        for t in it {
            let indent = "  ".repeat(t.depth());
            write!(f, "{}{}\n", indent, t.borrow_node().value)?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut t = Tree::new(0);
        let mut u = Tree::new(1);
        let mut v = Tree::clone(&u);
        assert!(u.link_push(&Tree::new(2)));
        assert!(u.link_push(&Tree::new(3)));
        assert!(t.link_push(&u));
        assert!(t.link_push(&Tree::new(4)));
        assert!(v.link_push(&Tree::new(5)));
        assert_eq!(format!("{}", t), "0\n  1\n    2\n    3\n    5\n  4\n");
        assert_eq!(*v.unlink_child(2).unwrap().get_ref(), 5);
        let it = t.traverse();
        for (i, node) in it.enumerate() {
            assert_eq!(i, *node.get_ref());
        }
    }
}
