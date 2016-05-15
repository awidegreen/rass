extern crate rasslib;
use rasslib::tree;

pub type MyTree     = tree::Tree<String>;
pub type MyTreePath = tree::Path<String>;

fn main() {
    let mut root = MyTree::new("root".to_string());
    let mut s1 = MyTree::new("s1".to_string());
    let mut s1_s1 = MyTree::new("s1_s1".to_string());
    let s1_s1_s1 = MyTree::new("s1_s1_s1".to_string());
    let s1_s1_s2 = MyTree::new("s1_s1_s2".to_string());
    let s1_s2 = MyTree::new("s1_s2".to_string());
    let s1_s3 = MyTree::new("s1_s3".to_string());
    s1_s1.add(s1_s1_s1);
    s1_s1.add(s1_s1_s2);
    s1.add(s1_s1);
    s1.add(s1_s2);
    s1.add(s1_s3);

    let s2 = MyTree::new("s2".to_string());
    root.add(s1);
    root.add(s2);

    let printer = tree::TreePrinter::new();
    printer.print(&root);

    for e in &root {
        println!("{}", e);
    }


    let e = ["root", "s1", "s1_s1"].iter().map(|x| x.to_string()).collect();
    let p = tree::Path::from(e);

    let r = root.remove(&p);

    if r {
        println!("It has been removed!");
    } else {
        println!("Nothing has been removed!");
    }

    printer.print(&root);
}
