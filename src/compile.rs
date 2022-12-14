//! # bibcompiler
//! This software compiles BibTeX files to legacy {thebibliography} TeX code.
//!
//! Run it as `bibcompiler -h` for help.
//!
//! Run it as `bibcompiler -f master.bib` for basic functionality.

use crate::utils;
use clap::Args;
use std::collections::HashMap;
pub const DEF_OUTPUT: &str = "print to stdout.";

#[derive(Args)]
pub struct CompileCli {
    #[arg(short, long)]
    /// Master BibTeX file
    file: String,
    #[arg(short, long, default_value = DEF_OUTPUT)]
    /// Output TeX file name, it will contain a `thebibliography` environment.
    output: String,
    #[arg(short, long, default_value_t = false)]
    /// add "Publisher" segment to each `bibitem`.
    publisher: bool,
}

type StrMap<'a> = HashMap<&'a str, &'a str>;
#[derive(Debug)]
struct Entry<'a> {
    name: &'a str,
    params: StrMap<'a>,
}

pub fn run_compile(cli: &CompileCli) {
    if let Some(data) = utils::read_tex_stripped(&cli.file) {
        let bib = parse_bibliography(&data);
        let size = utils::thebibliography_size(bib.len());
        let mut formatted = format!("\\begin{{thebibliography}}{{{size}}}\n\n");
        for b in bib.into_iter() {
            let a = &b.params.get("author").unwrap();
            let authors = format_all_author(a);
            let t = format!("\\textit{{{}}},", &b.params.get("title").unwrap());
            let y = format!("({})", &b.params.get("year").unwrap());

            let p = if let Some(tmp) = &b.params.get("publisher") {
                format!("- {}", tmp)
            } else {
                "".to_owned()
            };

            let j = &b.params.get("journal").unwrap_or(&"");

            let vol_fmt = format_volume(&b);

            let bibkey = format!("\\bibitem{{{}}}", b.name);
            let mut elements: Vec<&str> = vec![];
            elements.push(&bibkey);
            elements.push(&authors);
            elements.push(&t);
            elements.push(&j);
            elements.push(&vol_fmt);
            if cli.publisher {
                elements.push(&p);
            }
            elements.push(&y);
            formatted.push_str(&format!(
                "{}\n\n",
                utils::clean_bib_text(&elements.join(" "))
            ));
        }
        formatted.push_str("\\end{thebibliography}");
        if cli.output != DEF_OUTPUT {
            utils::write_file(cli.output.to_owned(), &formatted);
        } else {
            println!("{}", formatted);
        }
    } else {
        println!("ERROR: Unable to read file \"{}\"", &cli.file);
    }
}

fn format_volume(b: &Entry) -> String {
    let mut vol: Vec<&str> = vec![];
    if let Some(tmp) = b.params.get("pages") {
        vol.push(&tmp);
    }
    if let Some(tmp) = b.params.get("number") {
        vol.push(&tmp);
    }
    if let Some(tmp) = b.params.get("volume") {
        vol.push(&tmp);
    }
    let mut vol_fmt = "".to_owned();
    if vol.len() > 0 {
        let v = vol.pop().unwrap();
        vol_fmt.push_str(&format!("\\textbf{{{v}}}"));
    }
    if vol.len() > 0 {
        let v = vol.pop().unwrap();
        vol_fmt.push_str(&format!("({v})"));
    }
    if vol.len() > 0 {
        let v = vol.pop().unwrap();
        vol_fmt.push_str(&format!(":{v}"));
    }
    return vol_fmt;
}

fn format_all_author(a: &str) -> String {
    let authors = a
        .split(" and ")
        .map(|s| s.split(", ").collect())
        .map(format_author)
        .collect::<Vec<String>>();
    if authors.len() == 0 {
        return "".to_owned();
    } else if authors.len() == 1 {
        return format!("\\textsc{{{}}}",authors[0]);
    }
    let a1 = &authors[0..authors.len() - 1].join(", ");
    let a2 = &authors[authors.len() - 1];
    return format!("\\textsc{{{} \\& {}}}", a1, a2);
}

fn format_author(auth: Vec<&str>) -> String {
    if auth.len() == 0 {
        return "".to_owned();
    }
    let mut fmt_auth = auth[0].to_owned();
    if auth.len() > 1 {
        let proc = auth[1]
            .split(" ")
            .map(|s| {
                return s[0..1].to_ascii_uppercase();
            })
            .collect::<Vec<String>>()
            .join(". ");
        fmt_auth = format!("{}. {}", proc, fmt_auth);
    }
    fmt_auth
}

fn trim_pars(sa: &str) -> &str {
    let mut s = sa.trim();
    if s.len() > 2 && &s[0..1] == "{" && &s[s.len() - 1..] == "}" {
        s = &s[1..s.len() - 1];
    }
    return s;
}

fn find_closing_token(sub: &str, tok: u8, at: isize) -> Option<usize> {
    let mut l: isize = 0;
    for (i, &c) in sub.as_bytes().into_iter().enumerate() {
        if l == at && c == tok {
            return Some(i);
        }
        if c == b'{' {
            l += 1;
        }
        if c == b'}' {
            l -= 1;
        }
    }
    return None;
}

fn get_bibentry_raw(data: &str) -> Option<(&str, &str, &str, &str)> {
    let o = data.find("@");
    if o.is_none() {
        return None;
    }
    let mut rest = &data[o.unwrap() + 1..];

    let o = rest
        .find("{")
        .expect("found \"@\", and expected to find \"{\" after entry type.");
    let etype = &rest[0..o].trim();
    rest = &rest[o + 1..];

    let o = rest
        .find(",")
        .expect("found \"{\", and expected to find \",\" after article keyname.");
    let keyname = &rest[0..o].trim();
    rest = &rest[o + 1..];

    let o = find_closing_token(rest, b'}', 0).expect("expected to find bibentry closing brace.");
    let fields = &rest[0..o].trim();
    rest = &rest[o + 1..];

    return Some((rest, etype, keyname, fields));
}

fn parse_bibliography<'a>(data: &'a str) -> Vec<Entry<'a>> {
    let mut entries = vec![];
    let mut sub = data;

    while let Some((rest, _, keyname, fields_s)) = get_bibentry_raw(sub) {
        let mut entry = Entry {
            name: keyname,
            params: HashMap::new(),
        };
        sub = rest;
        let mut fields = fields_s;
        while let Some(e) = fields.find("=") {
            let fieldname = &fields[0..e].trim();
            fields = &fields[e + 1..];
            let f = find_closing_token(fields, b',', 0).unwrap_or(fields.len());
            let value = trim_pars(&fields[0..f].trim());
            if f < fields.len() {
                fields = &fields[f + 1..];
            }
            let _ = entry.params.insert(fieldname, value);
        }
        entries.push(entry);
    }
    return entries;
}
