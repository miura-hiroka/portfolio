use std::{collections::BTreeMap, fmt, str::FromStr};

#[derive(Clone)]
pub struct Tree<T> {
    nodes: BTreeMap<usize, Node<T>>,
    root_id: usize,
}

#[derive(Debug, Clone)]
pub struct Node<T> {
    parent: Option<usize>,
    children: Vec<usize>,
    value: T,
}

pub struct Subtree<'a, T> {
    nodes: &'a BTreeMap<usize, Node<T>>,
    root: &'a Node<T>,
    root_id: usize,
}

impl<T> Node<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
    pub fn children(&self) -> Children<'_> {
        Children { children: &self.children, next_idx: 0 }
    }
    pub fn children_len(&self) -> usize {
        self.children.len()
    }
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

impl<T> Tree<T> {
    /// Creates an `Tree` with a root `value`.
    pub fn new(value: T) -> Self {
        let node = Node { parent: None, children: Vec::new(), value };
        Self { nodes: [(0, node)].into(), root_id: 0 }
    }
    pub fn contains_id(&self, id: usize) -> bool {
        self.nodes.contains_key(&id)
    }
    #[inline]
    pub fn root_id(&self) -> usize {
        self.root_id
    }
    pub fn get(&self, id: usize) -> Option<&T> {
        self.nodes.get(&id).map(|node| &node.value)
    }
    pub fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        self.nodes.get_mut(&id).map(|node| &mut node.value)
    }
    pub fn get_root(&self) -> Option<&T> {
        self.nodes.get(&self.root_id).map(|root| &root.value)
    }
    pub fn get_mut_root(&mut self) -> Option<&mut T> {
        self.nodes.get_mut(&self.root_id).map(|root| &mut root.value)
    }
    pub fn get_node(&self, id: usize) -> Option<&Node<T>> {
        self.nodes.get(&id)
    }
    pub fn get_node_mut(&mut self, id: usize) -> Option<&mut Node<T>> {
        self.nodes.get_mut(&id)
    }
    pub fn children(&self, id: usize) -> Children<'_> {
        Children { children: self.nodes.get(&id).unwrap().children.as_slice(), next_idx: 0 }
    }
    pub fn children_root(&self) -> Children<'_> {
        Children { children: self.nodes.get(&self.root_id).unwrap().children.as_slice(), next_idx: 0 }
    }
    /// Pre-order traversal
    pub fn ids(&self) -> Ids<'_, T> {
        Ids { nodes: &self.nodes, stack: vec![self.root_id] }
    }
    /// Pre-order traversal
    pub fn ids_sub(&self, id: usize) -> Ids<'_, T> {
        Ids { nodes: &self.nodes, stack: vec![id] }
    }
    pub fn subtree(&self, id: usize) -> Subtree<T> {
        Subtree { nodes: &self.nodes, root: self.nodes.get(&id).unwrap(), root_id: id }
    }
    pub fn subtree_root(&self) -> Subtree<T> {
        Subtree { nodes: &self.nodes, root: self.nodes.get(&self.root_id).unwrap(), root_id: self.root_id }
    }
    fn child_idx(&self, id: usize) -> Option<usize> {
        let child = self.nodes.get(&id).unwrap();
        let parent = self.nodes.get(&child.parent?).unwrap();
        parent.children.iter().position(|&child_id| child_id == id)
    }
    fn new_id(&self) -> usize {
        let &last = self.nodes.last_key_value().unwrap().0;
        if last == usize::MAX {
            assert!(self.nodes.len() != usize::MAX);
            let mut id = 0;
            while self.nodes.contains_key(&id) { id += 1; }
            id
        } else {
            last + 1
        }
    }
    pub fn push(&mut self, parent_id: usize, value: T) -> usize {
        let id = self.new_id();
        let parent = self.nodes.get_mut(&parent_id).unwrap();
        parent.children.push(id);
        let new = Node { parent: Some(parent_id), children: Vec::new(), value };
        self.nodes.insert(id, new);
        id
    }

    pub fn insert(&mut self, parent_id: usize, idx: usize, value: T) -> usize {
        let id = self.new_id();
        let parent = self.nodes.get_mut(&parent_id).unwrap();
        parent.children.insert(idx, id);
        let new = Node { parent: Some(parent_id), children: Vec::new(), value };
        self.nodes.insert(id, new);
        id
    }

    pub fn push_tree(&mut self, id: usize, mut tree: Self) {
        let mut stack = vec![(id, tree.root_id)];
        while let Some((parent_id, src_id)) = stack.pop() {
            let src = tree.nodes.remove(&src_id).unwrap();
            let new_id = self.push(parent_id, src.value);
            stack.extend(src.children.iter().map(|&child| (new_id, child)).rev());
        }
    }

    pub fn remove(&mut self, id: usize) {
        let node = self.nodes.get(&id).unwrap();
        if let Some(parent_id) = node.parent {
            let parent = self.nodes.get_mut(&parent_id).unwrap();
            let pos = parent.children.iter().position(|&child| child == id).unwrap();
            parent.children.remove(pos);
        }
        let ids: Vec<usize> = self.ids_sub(id).collect();
        for id in ids {
            self.nodes.remove(&id);
        }
    }

    pub fn cut_off(&mut self, id: usize) -> Self {
        let node = self.nodes.get_mut(&id).unwrap();
        if let Some(parent_id) = node.parent.take() {
            let parent = self.nodes.get_mut(&parent_id).unwrap();
            let pos = parent.children.iter().position(|&child| child == id).unwrap();
            parent.children.remove(pos);
        }
        let mut sub = BTreeMap::new();
        let ids: Vec<usize> = self.ids_sub(id).collect();
        for id in ids {
            let value = self.nodes.remove(&id).unwrap();
            sub.insert(id, value);
        }
        Self { nodes: sub, root_id: id }
    }
}

impl<T: Clone> Tree<T> {
    pub fn clone_sub(&self, id: usize) -> Self {
        self.subtree(id).to_owned()
    }

    pub fn push_sub(&mut self, id: usize, subtree: Subtree<'_, T>) {
        let value = subtree.get_root().clone();
        let new_id = self.push(id, value);
        for child in subtree.children_root() {
            self.push_sub(new_id, child);
        }
    }
    
    pub fn insert_sub(&mut self, id: usize, idx: usize, subtree: Subtree<'_, T>) {
        let value = subtree.get_root().clone();
        let new_id = self.insert(id, idx, value);
        for child in subtree.children_root() {
            self.push_sub(new_id, child);
        }
    }

    pub fn paste(&mut self, id: usize, src: Subtree<'_, T>) {
        if let Some(parent_id) = self.nodes.get(&id).unwrap().parent {
            let child_idx = self.child_idx(id).unwrap();
            self.remove(id);
            self.insert_sub(parent_id, child_idx, src);
            return;
        }
        drop(std::mem::replace(self, src.to_owned()));
    }
}

impl<T: Clone + PartialEq> Tree<T> {

    pub fn replace_value(&mut self, from: &T, to: &T) {
        let mut stack = vec![self.root_id];
        while let Some(id) = stack.pop() {
            let node = self.nodes.get_mut(&id).unwrap();
            if node.value == *from {
                node.value = to.clone();
            }
            stack.extend(node.children.iter().map(|&child| child).rev());
        }
    }

    pub fn replace(&mut self, from: Subtree<T>, to: Subtree<T>) {
        let mut stack = vec![self.root_id];
        while let Some(id) = stack.pop() {
            if self.subtree(id) == from {
                self.paste(id, to);
            } else {
                let node = self.nodes.get_mut(&id).unwrap();
                stack.extend(node.children.iter().map(|&child| child).rev());
            }
        }
    }
}

impl<T: PartialEq> PartialEq for Tree<T> {
    fn eq(&self, other: &Self) -> bool {
        self.subtree_root() == other.subtree_root()
    }
}

impl<T: fmt::Display> fmt::Display for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.subtree_root())
    }
}

impl<T: fmt::Debug> fmt::Debug for Tree<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.subtree_root())
    }
}

fn absorb(stack: &mut Vec<Tree<String>>, idx: usize) {
    let stack_buf = stack.as_ptr();
    let len_old = stack.len();
    unsafe {
        stack.set_len(idx);
    }
    let parent = &mut stack[idx - 1];
    for i in idx .. len_old {
        unsafe {
            let child = std::ptr::read(stack_buf.add(i));
            parent.push_tree(parent.root_id, child);
        }
    }
}

pub fn parse_trees(mut s: &str) -> Result<Vec<Tree<String>>, String> {
    let mut stack = Vec::new();
    let mut v_prth_l_idx = Vec::new();
    s = s.trim_start();
    'g: loop {
        if s.is_empty() {
            return Ok(stack);
        }
        for (i, c) in s.char_indices() {
            if c.is_whitespace() {
                stack.push(Tree::new(s[..i].to_string()));
            } else if c == '(' {
                if i != 0 {
                    stack.push(Tree::new(s[..i].to_string()));
                }
                if stack.is_empty() ||
                v_prth_l_idx.last().map_or(false, |&k| k == stack.len()) {
                    return Err("missing parent node".to_string());
                }
                v_prth_l_idx.push(stack.len());
            } else if c == ')' {
                if i != 0 {
                    stack.push(Tree::new(s[..i].to_string()));
                }
                let Some(idx) = v_prth_l_idx.pop() else {
                    return Err("missing parenL".to_string());
                };            
                absorb(&mut stack, idx);
            } else {
                continue;
            }
            s = s[i + c.len_utf8() ..].trim_start();
            continue 'g;
        }
        stack.push(Tree::new(s.to_string()));
        break;
    }
    Ok(stack)
}

impl FromStr for Tree<String> {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut trees = parse_trees(s)?;
        match trees.len() {
            0 => Err("empty tree".to_string()),
            1 => Ok(trees.pop().unwrap()),
            _ => Err("multiple trees".to_string()),
        }
    }
}


impl<T> Subtree<'_, T> {
    pub fn get_root(&self) -> &T {
        &self.root.value
    }
    pub fn children_len(&self) -> usize {
        self.root.children.len()
    }
    pub fn is_leaf(&self) -> bool {
        self.root.children.is_empty()
    }
    pub fn children_root(&self) -> SubtreeChildren<'_, T> {
        SubtreeChildren { nodes: self.nodes, children: &self.root.children, next_idx: 0 }
    }
    pub fn ids(&self) -> Ids<'_, T> {
        Ids { nodes: self.nodes, stack: vec![self.root_id] }
    }
}

impl<T> Clone for Subtree<'_, T> {
    fn clone(&self) -> Self {
        Subtree { nodes: self.nodes, root: self.root, root_id: self.root_id }
    }
}

impl<T> Copy for Subtree<'_, T> {}

impl<T: Clone> Subtree<'_, T> {
    pub fn to_owned(&self) -> Tree<T> {
        let mut nodes = BTreeMap::new();
        for id in self.ids() {
            let node = self.nodes.get(&id).unwrap().clone();
            nodes.insert(id, node);
        }
        Tree { nodes, root_id: self.root_id }
    }
}

impl<T: PartialEq> PartialEq for Subtree<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        if self.root.value != other.root.value {
            return false;
        }
        if self.root.children.len() != other.root.children.len() {
            return false;
        }
        std::iter::zip(self.children_root(), other.children_root())
            .all(|(ch0, ch1)| ch0 == ch1)
    }
}

impl<T: fmt::Display> fmt::Display for Subtree<'_, T> {
    /// Formats a subtree rooted at `id`.
    /// 
    /// # Panics
    /// 
    /// Panics if any id in children is invalid.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.root.value)?;
        let mut children = self.children_root();
        let Some(first) = children.next() else {
            return Ok(());
        };
        write!(f, "({}", first)?;
        for child in children {
            write!(f, " {}", child)?;
        }
        write!(f, ")")
    }
}

impl<T: fmt::Debug> fmt::Debug for Subtree<'_, T> {
    /// Formats a subtree rooted at `id`.
    /// 
    /// # Panics
    /// 
    /// Panics if any id in children is invalid.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.root.value)?;
        let mut children = self.children_root();
        let Some(first) = children.next() else {
            return Ok(());
        };
        write!(f, "({:?}", first)?;
        for child in children {
            write!(f, " {:?}", child)?;
        }
        write!(f, ")")
    }
}


pub struct Ids<'a, T> {
    nodes: &'a BTreeMap<usize, Node<T>>,
    stack: Vec<usize>,
}

impl<'a, T> Iterator for Ids<'a, T> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop().map(|id| {
            let node = self.nodes.get(&id).unwrap();
            self.stack.extend(node.children.iter().map(|&child| child).rev());
            id
        })
    }
}

pub struct Children<'a> {
    children: &'a [usize],
    next_idx: usize,
}

impl<'a> Iterator for Children<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        let &id = self.children.get(self.next_idx)?;
        self.next_idx += 1;
        Some(id)
    }
}

pub struct SubtreeChildren<'a, T> {
    nodes: &'a BTreeMap<usize, Node<T>>,
    children: &'a [usize],
    next_idx: usize,
}

impl<'a, T> Iterator for SubtreeChildren<'a, T> {
    type Item = Subtree<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let id = self.children.get(self.next_idx)?;
        self.next_idx += 1;
        Some(Subtree { nodes: self.nodes, root: self.nodes.get(id).unwrap(), root_id: *id })
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut tree = Tree::new(0);
        let two = tree.push(tree.root_id(), 2);
        tree.push(tree.root_id(), 4);
        tree.push(two, 6);
        assert_eq!(tree.to_string(), "0(2(6) 4)");
        let tree = Tree::<String>::from_str("a (b(01 02())c d(e))").unwrap();
        assert_eq!(tree.to_string(), "a(b(01 02) c d(e))");
        let tree: Tree<String> = "->(a ->(->(b a) a))".parse().unwrap();
        assert_eq!(tree.subtree_root(), tree.subtree_root());
    }
}
