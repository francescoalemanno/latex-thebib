use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
pub const STR_PROTECT: &str = "@#[[3G3H498FG297EGF928HF2HRG82RHFOWKDNVKJSX]]@#*";

pub fn read_tex_stripped(fname: &str) -> Option<String> {
    if let Ok(data) = std::fs::read_to_string(&fname) {
        let data = data.replace("\\%", STR_PROTECT);
        let lines = data
            .lines()
            .into_iter()
            .map(|s| s.trim())
            .filter(|x| x.len() == 0 || x.chars().nth(0).unwrap() != '%')
            .map(|s| s.split('%').collect::<Vec<&str>>()[0].trim())
            .collect::<Vec<&str>>();
        let data = dedup_token(&lines.join("\n"), "\n", 3);
        return Some(data.replace(STR_PROTECT, "\\%"));
    }
    return None;
}

pub fn vec_dedup<T: Eq + Hash + Copy>(v: &mut Vec<T>) {
    let mut uniques = HashSet::new();
    v.retain(|e| uniques.insert(*e));
}

pub fn edit_distance(a: &str, b: &str) -> usize {
    let mut result = 0;

    /* Shortcut optimizations / degenerate cases. */
    if a == b {
        return result;
    }

    let length_a = a.chars().count();
    let length_b = b.chars().count();

    if length_a == 0 {
        return length_b;
    }

    if length_b == 0 {
        return length_a;
    }

    /* Initialize the vector.
     *
     * This is why it’s fast, normally a matrix is used,
     * here we use a single vector. */
    let mut cache: Vec<usize> = (1..).take(length_a).collect();
    let mut distance_a;
    let mut distance_b;

    /* Loop. */
    for (index_b, code_b) in b.chars().enumerate() {
        result = index_b;
        distance_a = index_b;

        for (index_a, code_a) in a.chars().enumerate() {
            distance_b = if code_a == code_b {
                distance_a
            } else {
                distance_a + 1
            };

            distance_a = cache[index_a];

            result = if distance_a > result {
                if distance_b > result {
                    result + 1
                } else {
                    distance_b
                }
            } else if distance_b > distance_a {
                distance_a + 1
            } else {
                distance_b
            };

            cache[index_a] = result;
        }
    }

    result
}

pub fn write_file(n_fname: String, contents: &String) {
    let path = std::path::Path::new(&n_fname);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
    let mut file = File::create(&n_fname).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

pub fn clean_bib_text(s: &str) -> String {
    let mut ns = s.to_owned();
    loop {
        let ps = ns
            .replace("  ", " ")
            .replace("\\it ", "\\em ")
            .replace('\n', " ")
            .replace('\r', " ")
            .replace('\t', " ");
        if ps == ns {
            break;
        }
        ns = ps;
    }
    return ns;
}

pub fn dedup_token(s: &str, token: &str, reps: usize) -> String {
    let mut ns = s.to_owned();
    let mut ds = token.to_owned();
    let mut ts = "".to_owned();
    for _ in 1..reps {
        ds.push_str(token);
        ts.push_str(token);
    }
    loop {
        let ps = ns.replace(&ds, &ts);
        if ps == ns {
            break;
        }
        ns = ps;
    }
    return ns;
}

pub fn thebibliography_size(biblen: usize) -> usize {
    let mut size = 0;
    while size < biblen {
        size = size * 10 + 9;
    }
    return size;
}
