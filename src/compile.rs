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
        //println!("{}", data);
        let list = data
            .split("@article")
            .filter(|s| s.chars().take(1).next() == Some('{'))
            .map(|s| trim_pars(&s.trim().replace("\n", "").replace("\t", "")))
            .collect::<Vec<String>>();

        let bib = get_bibliography(&list);
        let size = utils::thebibliography_size(bib.len());
        let mut formatted = format!("\\begin{{thebibliography}}{{{size}}}\n\n");
        for b in bib.into_iter() {
            let a = &b.params.get("author").unwrap();
            let authors = format_all_author(a);
            let t = format!("{{\\em {}}},",&b.params.get("title").unwrap());
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
            if cli.publisher {elements.push(&p);}
            elements.push(&y);
            formatted.push_str(&format!("{}\n\n", utils::clean_bib_text(&elements.join(" "))));
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
        return authors[0].to_owned();
    }
    let a1 = &authors[0..authors.len() - 1].join(", ");
    let a2 = &authors[authors.len() - 1];
    return format!("{} \\& {}", a1, a2);
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

fn get_bibliography<'a>(list: &'a Vec<String>) -> Vec<Entry<'a>> {
    let mut bib = Vec::<Entry>::new();
    for l in list.iter() {
        let (key, mut rest) = get_key_name(l);
        let mut entry = Entry {
            name: key,
            params: HashMap::new(),
        };
        while rest.len() > 0 {
            let (label, body, rest2) = get_param(rest);
            entry.params.insert(label, body);
            rest = rest2;
        }
        bib.push(entry);
    }
    bib
}

fn trim_pars(mut s: &str) -> String {
    s = s.trim();
    if s.len() > 2 && &s[0..1] == "{" && &s[s.len() - 1..] == "}" {
        s = &s[1..s.len() - 1];
    }
    return s.to_owned();
}

fn get_param(rest: &str) -> (&str, &str, &str) {
    let (label, rest) = get_param_name(&rest);
    let (body, rest) = get_param_body(&rest);
    (label, body, rest)
}

fn get_param_body(rest: &str) -> (&str, &str) {
    let mut pos = 0;
    let mut pars = 0;
    let mut has_pars = false;
    for (i, c) in rest.chars().enumerate() {
        if c == '{' {
            pars += 1;
            has_pars = true;
        }
        if c == '}' {
            pars -= 1;
        }
        if c == ',' && pars == 0 {
            pos = i;
            break;
        }
        if i == rest.len() - 1 {
            pos = i + 1;
            break;
        }
    }
    let mut body = rest[0..pos].trim();
    if has_pars {
        body = &body[1..body.len() - 1]
    }
    let mut rest = rest[pos..].trim();
    if rest.len() > 0 {
        rest = &rest[1..];
    }
    (body, rest)
}

fn get_param_name(mut rest: &str) -> (&str, &str) {
    let mut pos = 0;
    for (i, c) in rest.chars().enumerate() {
        if c == '=' {
            pos = i;
            break;
        }
    }
    let label = &rest[0..pos].trim();
    rest = &rest[pos + 1..].trim();
    (label, rest)
}

fn get_key_name(rem: &str) -> (&str, &str) {
    let mut pos = 0;
    for (i, c) in rem.chars().enumerate() {
        if c == ',' {
            pos = i;
            break;
        }
    }
    let key = &rem[0..pos];
    return (key, &rem[pos + 1..]);
}
