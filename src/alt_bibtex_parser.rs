use std::collections::HashMap;
use std::{str, vec};

pub type StrMap = HashMap<String, String>;
#[derive(Debug)]
pub struct Entry {
    pub name: String,
    pub kind: String,
    pub params: StrMap,
}
fn find(s: &[u8],bt: u8) -> Option<usize> {
    for (i, b) in s.into_iter().enumerate() {
        if b == &bt {
            return Some(i);
        }
    }
    return None;
}
pub fn parse(data: &str) -> Vec<Entry> {
    let mut entries = vec![];
    let mut b = data.as_bytes();
    while let Some(s) = find(&b, b'@') {
        b=&b[s+1..];
        let x = find(b, b'{').unwrap();
        let entry_type = str::from_utf8(&b[0..x]).unwrap();
        b=&b[x..];


        let mut brace_level = 0;
        let mut fields_parts: Vec<String> = vec![];
        let mut accum: Vec<u8>= vec![];
        for (i,&v) in b.into_iter().enumerate() {
            if brace_level == 1 && v == b'}' {
                b = &b[i+1..];
                break;
            } else if brace_level == 1 && v == b',' {
                fields_parts.push(str::from_utf8(&accum).unwrap().trim().to_owned());
                accum = vec![];
            } else {
                accum.push(v);
            }
            if v==b'{' {brace_level += 1;}
            if v==b'}' {brace_level -= 1;}
        }
        if accum.len()!=0 {
            fields_parts.push(str::from_utf8(&accum).unwrap().trim().to_owned());
        }

        let mut entry = Entry {
            name: fields_parts[0][1..].to_owned(),
            kind: entry_type.to_owned(),
            params: StrMap::new(),
        };

        fields_parts = fields_parts[1..].to_vec();

        for part in fields_parts.into_iter() {
            let et = part.find('=').unwrap();
            let key = part[0..et].trim();
            let value = utils::trim_braces(&part[et+1..]);
            entry.params.insert(key.to_owned(), value.to_owned());
        }

        entries.push(entry);
    }
    return entries;
}
