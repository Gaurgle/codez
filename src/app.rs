use crate::model::{Category, Entry};

pub struct App {
    pub entries: Vec<Entry>,
    pub query: String,
    pub filter: Option<Category>,
    pub selected: usize,
}

impl App {
    pub fn new(entries: Vec<Entry>) -> Self {
        App {
            entries,
            query: String::new(),
            filter: None,
            selected: 0,
        }
    }

    pub fn filtered(&self) -> Vec<&Entry> {
        let q = self.query.to_lowercase();
        self.entries
            .iter()
            // No filter = the "all" view, which excludes hidden categories
            // (e.g. errno). An explicit filter shows that category regardless.
            .filter(|e| match self.filter {
                Some(c) => e.category == c,
                None => e.category.default_visible(),
            })
            .filter(|e| {
                q.is_empty()
                    || e.code.to_lowercase().contains(&q)
                    || e.name.to_lowercase().contains(&q)
                    || e.summary.to_lowercase().contains(&q)
                    || e.group.to_lowercase().contains(&q)
                    || e.category.key().contains(q.as_str())
            })
            .collect()
    }

    pub fn apply_char(&mut self, c: char) {
        self.query.push(c);
        self.selected = 0;
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.selected = 0;
    }

    pub fn move_selection(&mut self, delta: isize) {
        let len = self.filtered().len();
        if len == 0 {
            self.selected = 0;
            return;
        }
        let max = (len - 1) as isize;
        self.selected = (self.selected as isize + delta).clamp(0, max) as usize;
    }

    pub fn cycle_category(&mut self, forward: bool) {
        // "all" (None) followed by the visible categories, in canonical order.
        let mut order: Vec<Option<Category>> = vec![None];
        order.extend(Category::visible().into_iter().map(Some));
        let idx = order.iter().position(|c| *c == self.filter).unwrap_or(0);
        let n = order.len() as isize;
        let next = if forward {
            (idx as isize + 1) % n
        } else {
            (idx as isize - 1 + n) % n
        };
        self.filter = order[next as usize];
        self.selected = 0;
    }

    pub fn selected_entry(&self) -> Option<Entry> {
        self.filtered().get(self.selected).map(|e| (*e).clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::load_all;

    fn app() -> App {
        App::new(load_all())
    }

    #[test]
    fn new_app_defaults() {
        let a = app();
        assert!(a.query.is_empty());
        assert_eq!(a.filter, None);
        assert_eq!(a.selected, 0);
        assert!(!a.entries.is_empty());
    }

    #[test]
    fn query_filters_by_substring() {
        let mut a = app();
        a.apply_char('4');
        a.apply_char('0');
        a.apply_char('4');
        let hits = a.filtered();
        assert!(hits.iter().any(|e| e.code == "404"));
        assert!(hits.iter().all(|e| {
            e.code.contains("404")
                || e.name.to_lowercase().contains("404")
                || e.summary.to_lowercase().contains("404")
                || e.group.to_lowercase().contains("404")
        }));
    }

    #[test]
    fn backspace_widens_results() {
        let mut a = app();
        a.apply_char('z');
        let narrow = a.filtered().len();
        a.backspace();
        let wide = a.filtered().len();
        assert!(wide >= narrow);
        assert!(a.query.is_empty());
    }

    #[test]
    fn category_filter_restricts() {
        let mut a = app();
        a.filter = Some(Category::Git);
        assert!(a.filtered().iter().all(|e| e.category == Category::Git));
        assert!(!a.filtered().is_empty());
    }

    #[test]
    fn move_selection_clamps_both_ends() {
        let mut a = app();
        a.move_selection(-1);
        assert_eq!(a.selected, 0);
        a.move_selection(100_000);
        assert_eq!(a.selected, a.filtered().len() - 1);
    }

    #[test]
    fn cycle_category_walks_all_then_wraps() {
        let mut a = app();
        // Visible categories in canonical order, then wrap to None. errno is
        // hidden, so it is not in the cycle.
        let seq = [
            Some(Category::Http),
            Some(Category::Exit),
            Some(Category::Curl),
            Some(Category::Git),
            Some(Category::Rust),
            Some(Category::Docker),
            Some(Category::Podman),
            Some(Category::Ble),
            Some(Category::LeAudio),
            None,
        ];
        for expected in seq {
            a.cycle_category(true);
            assert_eq!(a.filter, expected);
        }
        assert!(!seq.contains(&Some(Category::Errno)));
    }

    #[test]
    fn errno_is_hidden_from_all_but_reachable() {
        let mut a = app();
        // "all" view excludes errno.
        assert!(a.filtered().iter().all(|e| e.category != Category::Errno));
        // Explicit filter shows it.
        a.filter = Some(Category::Errno);
        assert!(a.filtered().iter().all(|e| e.category == Category::Errno));
        assert!(!a.filtered().is_empty());
    }

    #[test]
    fn category_name_acts_as_search_tag() {
        let mut a = app();
        for c in "git".chars() {
            a.apply_char(c);
        }
        let hits = a.filtered();
        assert!(!hits.is_empty());
        assert!(hits.iter().any(|e| e.category == Category::Git));
    }

    #[test]
    fn selected_entry_tracks_selection() {
        let mut a = app();
        a.filter = Some(Category::Http);
        let first = a.selected_entry().unwrap();
        assert_eq!(first.category, Category::Http);
    }
}
