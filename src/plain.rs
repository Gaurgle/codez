use crate::app::App;
use crate::model::Entry;
use crate::theme;

fn line(buf: &mut String, e: &Entry) {
    let gc = theme::ansi(theme::group_color(&e.group));
    let sapphire = theme::ansi(theme::SAPPHIRE);
    let overlay = theme::ansi(theme::OVERLAY);
    let r = theme::RESET;
    let b = theme::BOLD;

    buf.push_str(&format!(
        "\n  {gc}{b}{}{r}  {sapphire}{}{r} {overlay}- {}{r}\n",
        e.code, e.name, e.group
    ));
    buf.push_str(&format!("  {}\n", e.summary));
    if let Some(d) = &e.detail {
        buf.push_str(&format!("  {overlay}{d}{r}\n"));
    }
    if let Some(f) = &e.fix {
        buf.push_str(&format!("  {}fix:{} {f}\n", theme::ansi(theme::GREEN), r));
    }
    if let Some(rf) = &e.reference {
        buf.push_str(&format!("  {overlay}ref: {rf}{r}\n"));
    }
}

/// Render the app's current filtered view for non-interactive output.
/// Returns the body and an exit code (0 = at least one match, 1 = none).
pub fn render(app: &App) -> (String, i32) {
    let hits = app.filtered();
    if hits.is_empty() {
        let q = &app.query;
        return (format!("  no match for \"{q}\"\n"), 1);
    }
    let mut buf = String::new();
    for e in hits {
        line(&mut buf, e);
    }
    buf.push('\n');
    (buf, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{load_all, Category};

    #[test]
    fn lookup_404_includes_name_and_detail() {
        let mut a = App::new(load_all());
        a.query = "404".to_string();
        let (out, code) = render(&a);
        assert_eq!(code, 0);
        assert!(out.contains("404"));
        assert!(out.contains("Not Found"));
        assert!(out.contains("no representation")); // from detail
    }

    #[test]
    fn miss_returns_exit_1() {
        let mut a = App::new(load_all());
        a.query = "zzzzz-nope".to_string();
        let (out, code) = render(&a);
        assert_eq!(code, 1);
        assert!(out.to_lowercase().contains("no match"));
    }

    #[test]
    fn category_restricts_lookup() {
        let mut a = App::new(load_all());
        a.filter = Some(Category::Git);
        a.query = "rejected".to_string();
        let (out, code) = render(&a);
        assert_eq!(code, 0);
        assert!(out.contains("Updates were rejected"));
        assert!(!out.contains("Too Many Requests")); // http 429 excluded
    }
}
