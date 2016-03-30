//! Contains structs and traits for a Tree structure. Subtrees are of the same 
//! type `Tree<T>`. A visitor-pattern trait is provided which also internally 
//! used for the `TreePrinter`.
//!
//! `Path` represents a path through the tree to a specific not. Finding an
//! element returns a vector of paths.

use std::fmt;
use std::io::{Write};
use std::cell::RefCell;
use std::string;
use std::clone;
use std::cmp;
use std::str;
use std::vec;


/// Represents a path to a specific element in the tree structure. Not that 
/// parameter `T` shall be of the same type as for the corresponding `Tree<T>`.
#[derive(Debug, Default)]
pub struct Path<T> 
    where T: fmt::Display + clone::Clone 
{
    elements: Vec<T>,
}

impl<'a, T> Path<T>
    where T: fmt::Display + clone::Clone 
{
    /// Creates a new `Path<T>` based on the given elements. Note that the takes
    /// ownership of the provided `elements`.
    pub fn from(elements: vec::Vec<T>) -> Path<T> {
        Path {
            elements: elements,
        }
    }
    
    /// Returns the string representation of a `Path<T>`.
    pub fn to_string(&self) -> String {
        let mut r: Vec<u8> = vec![];
        let len = self.elements.len();
        for (i, e) in self.elements.iter().enumerate() {
            if e.to_string().is_empty() {
                continue
            }
            let _ = write!(r, "{}", e);
            if i+1 < len {
                let _ = write!(r, "/");
            }
        }
        string::String::from_utf8(r).unwrap_or(String::new())
    }
}

/// `std::fmt::Display` trait implementation of a `Path<T>`.
impl<'a, T> fmt::Display for Path<T> 
    where T: fmt::Display + clone::Clone 
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}


/// Internal structure used for building a Vector of `Path<T>`s for a given 
/// Tree<T>. It uses the visitor pattern in order to create each path of the 
/// tree structure. Note: the result of the visitor execution is stored in
/// `self.results` of the `PathBuilder`.
struct PathBuilder<T>
    where T: fmt::Display + clone::Clone 
{
    result: RefCell<Vec<Path<T>>>,
    current: RefCell<Option<T>>,
    trace: RefCell<Vec<T>>,
}

impl<T> PathBuilder<T> 
    where T: fmt::Display + clone::Clone
{
    /// Creates a new PathBuilder.
    fn new() -> PathBuilder<T> {
        PathBuilder {
            result: RefCell::new(vec![]),
            current: RefCell::new(None),
            trace: RefCell::new(vec![]),
        }
    }
}

/// Visitor pattern implementation for the PathBuilder.
/// Each node in the path will be visited. 
impl<'a, T> TreeVisitor<'a, T> for PathBuilder<T> 
    where T: fmt::Display + clone::Clone + cmp::PartialEq 
{
    fn visit(&self, tree: &'a Tree<T>, _: bool) 
    {
        let elements = self.trace.clone();
        elements.borrow_mut().push(tree.name().clone());
        let path = Path {
            elements: elements.into_inner()
        };

        let mut r = self.result.borrow_mut();
        r.push(path);

        let mut c = self.current.borrow_mut();
        *c = Some(tree.name().clone());
    }

    fn step_down(&self, _: bool) {
        // don't add anything to trace if current is empty!
        match *self.current.borrow() {
            Some(ref c) => {
                let mut t = self.trace.borrow_mut();
                t.push(c.clone());
            },
            None => ()
        }
    }

    fn step_up(&self) { 
        let mut t = self.trace.borrow_mut();
        t.pop();
    }
}


/// A Tree structure which contains elements that are also trees?
///
/// Note: the paraemter `T` requires some trait boundaries:
/// * std::fmt::Display
/// * std::clone::Clone
/// * std::cmp::PartialEq
#[derive(Debug, Default)]
pub struct Tree<T> where T: fmt::Display + cmp::PartialEq + clone::Clone
{
    name: T,
    subs: Vec<Tree<T>>,
}

/// Display implementation for the tree
impl<T> fmt::Display for  Tree<T> 
    where T: fmt::Display + cmp::PartialEq + clone::Clone
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = write!(f, "{} [", &self.name);
        for x in &self.subs {
            let _ = write!(f, "{}", x.name);
        }
        write!(f, "]\n")
    }
}

impl<T> Tree<T> where T: fmt::Display + cmp::PartialEq + clone::Clone
{
    /// Creates a new structure where `name` parameter will be the name element 
    /// of the tree. The sub-tree will initial with empty.
    pub fn new(name: T) -> Tree<T> {
        Tree {
            name: name,
            subs: vec![],
        }
    }                                           

    /// Return the name of the element
    pub fn name(&self) -> &T {
        &self.name
    }

    /// Adds a node to the tree where the note is of type `Tree<T>`.
    pub fn add(&mut self, sub: Tree<T>) {
        self.subs.push(sub)
    }

    /// Remove an element from the Tree as specified by the `path`. Returns 
    /// `true` if the element has been found and removed.
    pub fn remove(&mut self, path: &Path<T>) -> bool {
        if path.elements.len() == 1 { return false; }

        let e = path.elements[1..]
            .iter()
            .map(|x| x.clone())
            .collect();
        let new_path = Path::from(e);

        if self.name != path.elements[0] {
            return false;
        }

        if new_path.elements.len() == 1 {
            let l_before = self.subs.len();

            self.subs.retain(|ref x| x.name != new_path.elements[0]);

            return l_before != self.subs.len();
        }

        for x in &mut self.subs {
            if x.remove(&new_path) { return true; }
        }

        return false
    }
}

/// Visitor pattern implementation for the `Tree<T>`, the Acceptor part. A tree
/// implements the TreeAcceptor trait. Each node in the tree will yield a call 
/// to the visitor. If the tree is step-up or step-down the corresponding 
/// function is called on the visitor.
impl<'a, T, V> TreeAcceptor<'a,T, V> for Tree<T> 
    where V: TreeVisitor<'a, T>, T: fmt::Display + cmp::PartialEq + clone::Clone
{
    fn accept(&'a self, visitor: &V, is_last: bool) {
        visitor.visit(self, is_last);

        let len = self.subs.len();
        visitor.step_down(is_last);
        for (i, element) in self.subs.iter().enumerate() {
            let is_last = i+1 == len;
            element.accept(visitor, is_last);
        }
        visitor.step_up();
    }
}

/// non-consuming version IntoIterator trait implementation for the `Tree<T>`.
impl<'a, T> IntoIterator for &'a Tree<T> 
    where T: fmt::Display + cmp::PartialEq + clone::Clone
{
    type Item = Path<T>;
    type IntoIter = TreeIterator<T>;

    fn into_iter(self) -> Self::IntoIter {
        let builder = PathBuilder::new();
        self.accept(&builder, false);
        let v = builder.result.into_inner();
        TreeIterator { it: v.into_iter(), }
    }
}

pub struct TreeIterator<T> where 
    T: fmt::Display + cmp::PartialEq + clone::Clone
{
    it: vec::IntoIter<Path<T>>,
}

impl<T> Iterator for TreeIterator<T> 
    where T: fmt::Display + cmp::PartialEq + clone::Clone
{
    type Item = Path<T>;
    fn next(&mut self) -> Option<Self::Item> {
        self.it.next()
    }
}

/// The `TreeVisitor` trait is the visitor part of the visitor pattern. 
/// Objects which on to get notified about a visited node while a tree structure
/// is traversed this trait should be implemented. 
pub trait TreeVisitor<'a, T> 
    where T: fmt::Display + cmp::PartialEq + clone::Clone
{
    fn visit(&self, tree: &'a Tree<T>, is_last: bool);
    fn step_down(&self, is_last: bool);
    fn step_up(&self);
}

/// The `TreeAcceptor` trait is the acceptor part of the visitor pattern. 
/// It should be implemented for structures which shall be traversed, in this 
/// case the `Tree<T>`.
trait TreeAcceptor<'a, T, V: TreeVisitor<'a, T>> 
    where T: fmt::Display + cmp::PartialEq + clone::Clone
{
    fn accept(&'a self, visitor: &V, is_last: bool);
}

// printer
struct Parts {
    entry:  &'static str,
    last:   &'static str,
    empty:  &'static str,
    cont:   &'static str,
}

static PARTS: Parts = Parts {
    entry: "├── ",
    last:  "└── ",
    empty: "    ",
    cont:  "│   ",
};

#[derive(Debug)]
pub struct TreePrinter {
    trace: RefCell<Vec<&'static str>>,
    out:   RefCell<Vec<u8>>,
    depth: RefCell<u8>,
    root:  String,
}

impl TreePrinter {
    pub fn new(root_node: &str) -> TreePrinter {
        TreePrinter { 
            trace: RefCell::new(vec![]), 
            out:   RefCell::new(vec![]),
            depth: RefCell::new(0),
            root:  root_node.to_string(),
        }
    }

    pub fn print<T>(&self, tree: &Tree<T>) 
        where T: fmt::Display + cmp::PartialEq + clone::Clone
    {
        self.reset();
        tree.accept(self, false);
        print!("{}", str::from_utf8(&*self.out.borrow()).unwrap());
    }

    fn reset(&self) {
        (*self.trace.borrow_mut()).clear();
        (*self.out.borrow_mut()).clear();
        (*self.depth.borrow_mut()) = 0;
    }   
}                                 


impl<'a, T> TreeVisitor<'a, T> for TreePrinter  
    where T: fmt::Display + cmp::PartialEq + clone::Clone
{
    fn visit(&self, tree: &'a Tree<T>, is_last: bool) {
        if *self.depth.borrow() == 0 {
            let _ = write!(*self.out.borrow_mut(), "{}\n", tree.name());
            return;
        }

        let trace = self.trace.borrow();
        for s in &*trace {
            let _ = write!(*self.out.borrow_mut(), "{}", s);
        }
        let _ = write!(*self.out.borrow_mut(), "{}{}\n",
            if is_last { PARTS.last } else { PARTS.entry }, tree.name());
    }

    fn step_down(&self, is_last: bool) {
        let mut depth = self.depth.borrow_mut();
        *depth = *depth + 1;
        if *depth == 1 { return; }

        let mut trace = self.trace.borrow_mut();
        if is_last {
            trace.push(PARTS.empty);
        } else {
            trace.push(PARTS.cont);
        }
    }

    fn step_up(&self) {
        let mut depth = self.depth.borrow_mut();
        *depth = *depth - 1;

        let mut trace = self.trace.borrow_mut();
        trace.pop();
    }
}


#[cfg(test)]
mod test {
    #[test]
    fn tree_add() {
        type Tree = super::Tree<String>;
        type Path = super::Path<String>;
        let mut root = Tree::new("root".to_string());
        let mut s1 = Tree::new("s1".to_string());
        let s1_s1 = Tree::new("s1_s1".to_string());
        let s1_s2 = Tree::new("s1_s2".to_string());
        let s1_s3 = Tree::new("s1_s3".to_string());
        s1.add(s1_s1);
        s1.add(s1_s2);
        s1.add(s1_s3);
        root.add(s1);

        let paths: Vec<Path> = root.into_iter().collect();
        assert_eq!(paths.len(), 5);
        assert_eq!(paths[0].to_string(), "root");
        assert_eq!(paths[1].to_string(), "root/s1");
        assert_eq!(paths[2].to_string(), "root/s1/s1_s1");
        assert_eq!(paths[3].to_string(), "root/s1/s1_s2");
        assert_eq!(paths[4].to_string(), "root/s1/s1_s3");
    }

    #[test]
    fn tree_remove() {
        type Tree = super::Tree<String>;
        type Path = super::Path<String>;
        let mut root = Tree::new("root".to_string());
        let mut s1 = Tree::new("s1".to_string());
        let s1_s1 = Tree::new("s1_s1".to_string());
        let s1_s2 = Tree::new("s1_s2".to_string());
        s1.add(s1_s1);
        s1.add(s1_s2);
        root.add(s1);

        let e = ["root", "s1", "s1_s1"].iter().map(|x| x.to_string()).collect();
        let p = Path::from(e);
        // WORKING
        let result = root.remove(&p);
        assert_eq!(result, true);

        let paths: Vec<Path> = root.into_iter().collect();
        assert_eq!(paths.len(), 3);
        assert_eq!(paths[0].to_string(), "root");
        assert_eq!(paths[1].to_string(), "root/s1");
        assert_eq!(paths[2].to_string(), "root/s1/s1_s2");

        let e = ["root", "s2"].iter().map(|x| x.to_string()).collect();
        let p = Path::from(e);
        // NOT WORKING
        let result = root.remove(&p);
        assert_eq!(result, false);
        assert_eq!(paths.len(), 3);

        // remove root shall not work, rather remove the root element as such
        let e = ["root"].iter().map(|x| x.to_string()).collect();
        let p = Path::from(e);
        let result = root.remove(&p);
        assert_eq!(result, false);
        assert_eq!(paths.len(), 3);

        let e = ["root", "s1"].iter().map(|x| x.to_string()).collect();
        let p = Path::from(e);
        // WORKING
        let result = root.remove(&p);
        assert_eq!(result, true);

        let paths: Vec<Path> = root.into_iter().collect();
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].to_string(), "root");
    }
}
