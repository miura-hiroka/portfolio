use crate::v2::Tree;
use std::ptr;

fn absorb(v: &mut Vec<Tree<String>>, indices: &mut Vec<usize>) -> Result<(), String> {
    let Some(idx) = indices.pop() else {
        return Err("missing parenL".to_string());
    };
    let v_buf = v.as_ptr();
    let v_len_old = v.len();
    unsafe {
        v.set_len(idx + 1);
    }
    let parent = &mut v[idx];
    for i in idx + 1 .. v_len_old {
        unsafe {
            let child = ptr::read(v_buf.add(i));
            parent.append_tree(child);
        }
    }
    Ok(())
}

pub fn parse_tree(mut s: &str) -> Result<Vec<Tree<String>>, String> {
    let mut v = Vec::new();
    let mut v_paren_l_idx = Vec::new();
    let mut j = 0;
    'g: loop {
        s = s[j..].trim_start();
        if s.is_empty() {
            return Ok(v);
        }
        for (i, c) in s.char_indices() {
            if c.is_whitespace() {
                v.push(Tree::new(s[..i].to_string()));
            } else if c == '(' {
                if i != 0 {
                    v.push(Tree::new(s[..i].to_string()));
                }
                if v.is_empty() ||
                    !v_paren_l_idx.last().map_or(true, |&k| k < v.len() - 1)
                {
                    return Err("missing parent node".to_string());
                }
                v_paren_l_idx.push(v.len() - 1);
            } else if c == ')' {
                if i != 0 {
                    v.push(Tree::new(s[..i].to_string()));
                }
                absorb(&mut v, &mut v_paren_l_idx)?;
            } else {
                continue;
            }
            j = i + c.len_utf8();
            continue 'g;
        }
        v.push(Tree::new(s.to_string()));
        break;
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_tree() {
        let trees = parse_tree("a(b (0) cc( 0 1 ) d()) f(x) () (y z) g ").unwrap();
        for (i, tree) in trees.iter().enumerate() {
            println!("{}: {}", i, tree);
        }
    }
}
