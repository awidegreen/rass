use std::path::PathBuf;

pub fn strip_path(full: &PathBuf, with: &PathBuf) -> PathBuf {
    let mut it_full = full.iter();

    for comp in with.iter() {
        match it_full.next() {
            Some(x)  => {
                if x == comp { continue; } else { break; }
            },
            None => break
        }
    }

    let mut result = PathBuf::new();
    for comp in it_full {
        result.push(comp);
    }
    result
}

#[test]
fn test_strip_path() {
    let full1 = PathBuf::from("/home/foobar/.hiden/file");
    let full2 = PathBuf::from("/home/foobar/.hiden/dir/file");
    let with = PathBuf::from("/home/foobar/.hiden");

    assert_eq!(strip_path(&full1, &with).to_str(), Some("file"));
    assert_eq!(strip_path(&full2, &with).to_str(), Some("dir/file"));
}

