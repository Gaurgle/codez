use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Category {
    #[default]
    Http,
    Exit,
    Curl,
    Git,
}

impl Category {
    pub const ALL: [Category; 4] = [Category::Http, Category::Exit, Category::Curl, Category::Git];

    pub fn key(self) -> &'static str {
        match self {
            Category::Http => "http",
            Category::Exit => "exit",
            Category::Curl => "curl",
            Category::Git => "git",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Entry {
    pub code: String,
    pub name: String,
    pub group: String,
    pub summary: String,
    #[serde(default)]
    pub detail: Option<String>,
    #[serde(default)]
    pub fix: Option<String>,
    #[serde(default, rename = "ref")]
    pub reference: Option<String>,
    #[serde(skip)]
    pub category: Category,
}

#[derive(Deserialize)]
struct DataFile {
    entry: Vec<Entry>,
}

fn parse(src: &str, category: Category, name: &str) -> Vec<Entry> {
    let file: DataFile =
        toml::from_str(src).unwrap_or_else(|e| panic!("codez: failed to parse {name}: {e}"));
    file.entry
        .into_iter()
        .map(|mut e| {
            e.category = category;
            e
        })
        .collect()
}

/// Load every embedded category data file into a single flat list.
pub fn load_all() -> Vec<Entry> {
    let mut all = Vec::new();
    all.extend(parse(include_str!("../data/http.toml"), Category::Http, "http.toml"));
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_loads_with_404() {
        let all = load_all();
        let e = all
            .iter()
            .find(|e| e.code == "404" && e.category == Category::Http)
            .expect("404 present");
        assert_eq!(e.name, "Not Found");
        assert!(e.detail.is_some());
    }

    #[test]
    fn http_has_all_group_bands() {
        let all = load_all();
        for band in ["2xx", "3xx", "4xx", "5xx"] {
            assert!(all.iter().any(|e| e.group.starts_with(band)), "missing {band}");
        }
    }

    #[test]
    fn every_entry_is_tagged_http() {
        let all = load_all();
        assert!(all.iter().all(|e| e.category == Category::Http));
        assert!(all.len() >= 55);
    }
}
