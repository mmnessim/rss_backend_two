pub mod watcher;

pub fn remove_html(raw: String) -> String {
    if ammonia::is_html(&raw) {
        let plain = match html2text::from_read(raw.as_bytes(), raw.len()) {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Error stripping HTML {:?}", e);
                return raw;
            }
        };
        let no_md = plain
            .lines()
            .map(|line| {
                if line.starts_with('#') {
                    line.trim_start_matches('#').trim_start().to_string()
                } else {
                    line.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        return no_md;
    }
    return raw;
}

#[cfg(test)]
mod tests {
    use super::remove_html;

    #[test]
    fn remove_all_tags() {
        let with_html = "<html>Test</html>";
        let expected = "Test";
        assert_eq!(remove_html(with_html.to_string()), expected.to_string());

        let more_html = r#"
        <h1>H1 Text</h1>
        <p>P tag text</p>
        "#;
        println!("{}", remove_html(more_html.to_string()));
    }
}
