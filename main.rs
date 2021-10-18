use anyhow::Result;
use std::collections;
use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path;
use yaml_rust::parser::Parser;
use yaml_rust::YamlLoader;

macro_rules! gen_cs_line {
    ($x:expr) => {
        format!("colors: *{}", $x)
    };
}

const CS_LINE_PATTERN: &str = r"colors: \*(\S+)";

#[derive(Debug)]
enum Error {
    NotFoundConfigFile,
    NotFoundColorScheme,
    ColorSchemeNotInList(String),
}

fn get_config_file_path() -> Result<String> {
    let mut possible_paths = Vec::<path::PathBuf>::new();
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME")
        .as_ref()
        .map(|x| path::Path::new(x))
    {
        possible_paths.push(xdg_config_home.join("alacritty/alacritty.yml"));
        possible_paths.push(xdg_config_home.join("alacritty.yml"));
    }
    if let Ok(home) = env::var("HOME").as_ref().map(|x| path::Path::new(x)) {
        possible_paths.push(home.join(".config/alacritty/alacritty.yml"));
        possible_paths.push(home.join(".alacritty.yml"));
    }
    let path = possible_paths
        .iter()
        .find(|x| x.exists())
        .map(|x| x.to_str().unwrap().to_owned())
        .ok_or(Error::NotFoundConfigFile)?;
    Ok(path)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::NotFoundConfigFile => write!(f, "Could not find config file"),
            Self::NotFoundColorScheme => write!(f, "Could not find set colorscheme inside config"),
            Self::ColorSchemeNotInList(ref anchor) => write!(f, "Anchor `{}` not found", anchor),
        }
    }
}
impl std::error::Error for Error {}

fn get_cs_anchors(config_text: &str) -> Result<Vec<String>> {
    let mut loader = YamlLoader {
        docs: Vec::new(),
        doc_stack: Vec::new(),
        key_stack: Vec::new(),
        anchor_map: collections::BTreeMap::new(),
    };
    let mut parser = Parser::new(config_text.chars());
    parser.load(&mut loader, true)?;
    let mut anchors_with_index: Vec<_> = parser.anchors.iter().collect();
    anchors_with_index.sort_by(|x, y| x.1.cmp(y.1));
    Ok(anchors_with_index.iter().map(|x| x.0).cloned().collect())
}

fn main() -> Result<()> {
    let config_file_path: &str = &get_config_file_path()?;
    let f = fs::File::open(config_file_path)?;
    let config_text = fs::read_to_string(config_file_path)?;
    let anchors = get_cs_anchors(&config_text)?;
    let config_buf = io::BufReader::new(f);
    let cs_line_regex = regex::Regex::new(CS_LINE_PATTERN)?;
    let mut cur_cs_line_opt: Option<&mut String> = None;
    let mut cur_cs_anchor_opt = None;
    let mut config_lines = config_buf
        .lines()
        .map(|x| x.unwrap())
        .collect::<Vec<String>>();
    for (i, line) in config_lines.clone().iter().enumerate() {
        for caps in cs_line_regex.captures_iter(line) {
            cur_cs_anchor_opt = Some(caps[1].to_owned());
            cur_cs_line_opt = Some(&mut config_lines[i]);
        }
    }
    let cur_cs_line = cur_cs_line_opt.ok_or(Error::NotFoundColorScheme)?;
    let cur_cs_anchor = cur_cs_anchor_opt.ok_or(Error::NotFoundColorScheme)?;
    let mut cs_anchors_iter = anchors.iter().chain(&anchors);
    cs_anchors_iter
        .position(|x| *x == cur_cs_anchor)
        .ok_or_else(|| Error::ColorSchemeNotInList(cur_cs_line.to_owned()))?;
    let new_cs_anchor = cs_anchors_iter.next().unwrap();
    let new_cs_line = gen_cs_line!(new_cs_anchor);
    *cur_cs_line = new_cs_line;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(config_file_path)?;
    writeln!(f, "{}", config_lines.join("\n"))?;
    Ok(())
}
