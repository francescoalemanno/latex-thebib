use clap::Parser;
use regex::{Regex, RegexBuilder};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::Hash;
use std::path::PathBuf;
const STR_PROTECT: &str = "@#[[3G3H498FG297EGF928HF2HRG82RHFOWKDNVKJSX]]@#*";

#[derive(Parser, Debug)]
#[command(author, version, about = "Biblio cleaner", long_about = None)]
struct Cli {
    #[arg(short, long)]
    file: String,
    #[arg(short, long, default_value_t = 0.3)]
    threshold: f64,
    #[arg(short, long, default_value = "cleaned")]
    subdir: String,
}

#[derive(Debug)]
struct Cite {
    list: Vec<String>,
    kind: String,
    raw: String,
}

impl fmt::Display for Cite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\\{}{{{}}}", self.kind, self.list.join(","))
    }
}

#[derive(Debug, Hash, Clone)]
struct BibEntry {
    key: String,
    text: String,
}

impl fmt::Display for BibEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\\bibitem{{{}}} {}", self.key, self.text)
    }
}

fn vec_dedup<T: Eq + Hash + Copy>(v: &mut Vec<T>) {
    let mut uniques = HashSet::new();
    v.retain(|e| uniques.insert(*e));
}

fn main() {
    let cli = Cli::parse();
    let (cites, bib) = parse_citations_and_biblio(&cli.file);
    let (clean_cites, used_bib) = take_used(&bib, &cites, cli.threshold);
    apply_changes(&cli.file, &used_bib, &clean_cites, &cli);
}

fn parse_citations_and_biblio(fname: &str) -> (Vec<Cite>, Vec<BibEntry>) {
    let contents = read_tex_stripped(fname);
    let mut cite_list: Vec<Cite> = vec![];
    let mut bib_list: Vec<BibEntry> = vec![];
    bib_list.append(&mut parse_bibliography(&contents));
    let re_cite_input = Regex::new(r"\\cite\{[^}]+\}|\\citet\{[^}]+\}|\\citep\{[^}]+\}|\\input\{[^}]+\}|\\include\{[^}]+\}|\\includeonly\{[^}]+\}").unwrap();
    let re_parse_cmd = Regex::new(r"\\(?P<type>[a-zA-Z]*)\{(?P<content>.*)\}").unwrap();
    for cap in re_cite_input.captures_iter(&contents) {
        let tok = &cap[0];
        let match_tok = re_parse_cmd.captures(&tok).unwrap();
        let t = &match_tok["type"];
        let c = &match_tok["content"];
        if t == "cite" || t == "citet" || t == "citep" {
            let mut entry = Cite {
                list: vec![],
                kind: t.trim().to_owned(),
                raw: match_tok[0].to_owned(),
            };
            for cite in c.split(',') {
                let cite = cite.trim();
                entry.list.push(cite.to_owned());
            }
            cite_list.push(entry);
        } else {
            let name =
                file_from_file(fname, c).expect(&("Should have found file: ".to_owned() + c));
            let (mut parsed, mut parsed_bib) = parse_citations_and_biblio(&name);
            cite_list.append(&mut parsed);
            bib_list.append(&mut parsed_bib);
        }
    }
    return (cite_list, bib_list);
}


fn take_used(raw_bib: &Vec<BibEntry>, cites: &Vec<Cite>, th: f64) -> (Vec<Cite>, Vec<BibEntry>) {
    let (replacements, bib) = reduce_bib(&raw_bib, th);

    let clean_cites = cites
        .iter()
        .map(|c| {
            let ll = &c.list;
            let mut l = ll
                .into_iter()
                .map(|e| {
                    if replacements.contains_key(e) {
                        replacements.get(e).unwrap()
                    } else {
                        &e
                    }
                })
                .collect::<Vec<&String>>();
            vec_dedup(&mut l);
            let k = &c.kind;
            let raw = &c.raw;
            Cite {
                list: l.into_iter().map(|v| v.to_owned()).collect(),
                kind: k.to_owned(),
                raw: raw.to_owned(),
            }
        })
        .collect::<Vec<Cite>>();

    let mut set_cites = HashSet::<&String>::new();
    let mut ord_cites = Vec::<&String>::new();
    for s in &clean_cites {
        for c in &s.list {
            if set_cites.insert(&c) {
                ord_cites.push(&c);
            }
        }
    }

    let mut hash_bib = HashMap::<&String, &String>::new();

    for b in &bib {
        hash_bib.insert(&b.key, &b.text);
    }

    let mut minimal_bib = Vec::<BibEntry>::new();
    for c in &ord_cites {
        if hash_bib.contains_key(c) {
            minimal_bib.push(BibEntry {
                key: c.to_string(),
                text: hash_bib.get(c).unwrap().to_string(),
            });
        } else {
            minimal_bib.push(BibEntry {
                key: c.to_string(),
                text: "ERROR, BIBENTRY NOT FOUND.".to_owned(),
            });
        }
    }

    (clean_cites, minimal_bib)
}

fn apply_changes(fname: &str, bib: &Vec<BibEntry>, cite: &Vec<Cite>, options: &Cli) {
    let n_fname = change_path(&fname, &options.subdir).unwrap();

    let mut contents = read_tex_stripped(&fname);
    for v in cite {
        contents = contents.replace(&v.raw, &format!("{}", v));
    }

    let bre = thebibliography_regex();
    let mut n = 0;
    while n<bib.len() {
        n=n*10+9;
    }

    let bibstr = format!("\\begin{{thebibliography}}{{{}}}\n{}\n\\end{{thebibliography}}",n,bib.into_iter().map(|v| format!["{}",v]).collect::<Vec<String>>().join("\n\n"));

    contents = bre.replace(&contents, STR_PROTECT).to_string().replace(STR_PROTECT, &bibstr);

    write_file(n_fname, &contents);

    let re = Regex::new(r"\\input\{[^}]+\}|\\include\{[^}]+\}|\\includeonly\{[^}]+\}").unwrap();
    let re2 = Regex::new(r"\\[a-zA-Z]*\{(?P<content>.*)\}").unwrap();
    for cap in re.captures_iter(&contents) {
        let tok = &cap[0];
        let match_tok = re2.captures(&tok).unwrap();
        let c = &match_tok["content"];
        let name = file_from_file(fname, c).expect(&("Should have found file: ".to_owned() + c));
        apply_changes(&name, &bib, &cite, &options);
    }
}


fn reduce_bib(bib: &Vec<BibEntry>, th: f64) -> (HashMap<String, String>, Vec<BibEntry>) {
    let components = find_connected_components(&bib, th);
    let mut reps: HashMap<String, String> = HashMap::new();
    let mut red_bib: Vec<BibEntry> = Vec::new();
    for c in components.into_iter() {
        let mut first: Option<usize> = None;
        for i in c.into_iter() {
            if let Some(idx) = first {
                if bib[idx].key == bib[i].key {
                    continue;
                }
                reps.insert(bib[i].key.to_owned(), bib[idx].key.to_owned());
            } else {
                first = Some(i);
                red_bib.push(bib[i].clone());
            }
        }
    }
    red_bib.sort_by(|a, b| a.key.partial_cmp(&b.key).unwrap());
    (reps, red_bib)
}

fn find_connected_components(bib: &Vec<BibEntry>, th: f64) -> Vec<Vec<usize>> {
    // Build graph of duplicates
    let mut g = HashMap::<usize, HashSet<usize>>::new();
    for (i,bi) in bib.into_iter().enumerate() {
        for (j, bj) in bib.into_iter().enumerate() {
            if j<=i {continue;}
            let mut ed = 2.0 * (edit_distance(&bi.text, &bj.text) as f64)
                / ((&bi.text.len() + &bj.text.len()) as f64);
            if bi.key == bj.key {
                ed = th;
            }
            if ed <= th {
                if g.contains_key(&i) {
                    g.get_mut(&i).unwrap().insert(j);
                } else {
                    g.insert(i, HashSet::from([j]));
                }
                if g.contains_key(&j) {
                    g.get_mut(&j).unwrap().insert(i);
                } else {
                    g.insert(j, HashSet::from([i]));
                }
            }
        }
    }
    // Perform BFS to find connected components
    let mut res: Vec<Vec<usize>> = vec![];
    let mut n = HashSet::<usize>::new();

    for (k, _) in g.iter() {
        if n.contains(k) {
            continue;
        }
        let mut q: Vec<usize> = Vec::from([k.to_owned()]);
        let mut c_comp: Vec<usize> = vec![];
        while q.len() > 0 {
            let e = q.pop().unwrap();
            if n.contains(&e) {
                continue;
            }
            c_comp.push(e);
            n.insert(e);
            let v = g.get(&e).unwrap();
            for ne in v.into_iter() {
                if !n.contains(ne) {
                    q.push(*ne);
                }
            }
        }
        res.push(c_comp);
    }
    for i in 0..bib.len() {
        if n.contains(&i) {
            continue;
        }
        res.push(vec![i]);
    }
    // return conn. comp.
    return res;
}

fn file_from_file<'a>(path: &'a str, fname: &'a str) -> Option<String> {
    let mut wkdir = PathBuf::from(path);
    let _ = wkdir.pop();
    for ext in ["", ".tex", ".latex", ".bib", ".bbl"] {
        let mut np = wkdir.clone();
        let name = fname.to_owned() + ext;
        np.push(name);
        if np.exists() {
            return Some(np.to_str().unwrap().to_owned());
        }
    }
    return None;
}

fn change_path<'a>(path: &'a str, add: &'a str) -> Option<String> {
    let mut wkdir = PathBuf::from(path);
    let name = &wkdir.file_name().unwrap().to_owned();
    let _ = wkdir.pop();
    wkdir.push(add);
    wkdir.push(name);
    return Some(wkdir.to_str().unwrap().to_owned());
}

fn read_tex_stripped(fname: &str) -> String {
    let data = std::fs::read_to_string(&fname).unwrap_or("%".to_owned());
    
    let data = data.replace("\\%", STR_PROTECT);
    let lines = data
        .lines()
        .into_iter()
        .map(|s| s.split('%').collect::<Vec<&str>>()[0])
        .filter(|s| s.trim() != "")
        .collect::<Vec<&str>>();
    let data = lines.join("\n");
    return data.replace(STR_PROTECT, "\\%");
}

fn parse_bibliography(contents: &str) -> Vec<BibEntry> {
    let re = thebibliography_regex();
    let re2 = Regex::new(r"\{(.*?)\}(.*)").unwrap();
    let bib = re.captures(&contents);
    let mut res: Vec<BibEntry> = vec![];
    if let Some(bib) = bib {
        let bib = bib["bib"].to_owned();
        for s in bib.split("\\bibitem").into_iter() {
            let st = s.trim().replace("\n", "");
            let cp = re2.captures(&st);
            if let Some(captured) = cp {
                res.push(BibEntry {
                    key: captured[1].trim().to_owned(),
                    text: clean_bib_text(captured[2].trim()),
                });
            }
        }
    }
    return res;
}

fn thebibliography_regex() -> Regex {
    return RegexBuilder::new(
        r"(?s)\\begin\{thebibliography\}(\{\d+\}|)(?P<bib>.*?)\\end\{thebibliography\}",
    )
    .build()
    .unwrap();
}

fn clean_bib_text(s: &str) -> String {
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

use std::fs::File;
use std::io::prelude::*;

fn write_file(n_fname: String, contents: &String) {
    let path = std::path::Path::new(&n_fname);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).unwrap();
    let mut file = File::create(&n_fname).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}
