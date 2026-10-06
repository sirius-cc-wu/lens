use crate::markdown::escape_html;

const APP_SCRIPT: &str = include_str!("assets/app.js");
const APP_STYLESHEET: &str = include_str!("assets/app.css");
// Bundled Mermaid 11.17.2 from https://cdn.jsdelivr.net/npm/mermaid@11.17.2/dist/mermaid.min.js
const MERMAID_SCRIPT: &str = include_str!("assets/mermaid.min.js");

pub(super) fn app_script() -> &'static str {
    embedded_asset(APP_SCRIPT)
}

pub(super) fn app_stylesheet() -> &'static str {
    embedded_asset(APP_STYLESHEET)
}

pub(super) fn mermaid_script() -> &'static str {
    embedded_asset(MERMAID_SCRIPT)
}

fn embedded_asset(asset: &'static str) -> &'static str {
    asset
        .strip_suffix("\r\n")
        .or_else(|| asset.strip_suffix('\n'))
        .unwrap_or(asset)
}

pub(super) fn page(
    title: &str,
    document_html: String,
    document_revision: Option<(&str, u64)>,
    session_token: &str,
) -> String {
    let refresh_attributes = document_revision
        .map(|(document_id, revision)| {
            format!(
                r#" data-document-id="{}" data-document-revision="{revision}""#,
                escape_html(document_id),
            )
        })
        .unwrap_or_default();
    let document_html = inject_capability(&document_html, session_token);
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <meta name="referrer" content="no-referrer">
  <title>Lens: {}</title>
  <link rel="stylesheet" href="/app.css?token={}">
</head>
<body>
  <main{refresh_attributes} data-session-token="{}">
    <section class="document-content">
      <header class="document-header"><p class="eyebrow">Lens</p><h1>{}</h1></header>
      <article>{document_html}</article>
    </section>
  </main>
  <script src="/mermaid.js?token={}"></script>
  <script src="/app.js?token={}"></script>
</body>
</html>"#,
        escape_html(title),
        session_token,
        session_token,
        escape_html(title),
        session_token,
        session_token,
    )
}

pub(super) fn document_unavailable_page(session_token: &str) -> String {
    page(
        "Document unavailable",
        format!(
            "<p>Lens can display the selected document, but the requested document is not part of this viewing session.</p><p><a href=\"/?token={session_token}\">Return to the initial document</a></p>"
        ),
        None,
        session_token,
    )
}

struct CapabilityAttribute {
    tag: &'static str,
    attr: &'static str,
    url_prefix: &'static str,
}

const CAPABILITY_ATTRIBUTES: &[CapabilityAttribute] = &[
    CapabilityAttribute {
        tag: "a",
        attr: "href",
        url_prefix: "/documents/",
    },
    CapabilityAttribute {
        tag: "img",
        attr: "src",
        url_prefix: "/diagrams/",
    },
];

pub(super) fn inject_capability(html: &str, token: &str) -> String {
    scan(html, token)
}

fn is_capability_attribute(tag: &str, attr: &str, url: &str) -> bool {
    CAPABILITY_ATTRIBUTES.iter().any(|cap| {
        tag.eq_ignore_ascii_case(cap.tag)
            && attr.eq_ignore_ascii_case(cap.attr)
            && url.starts_with(cap.url_prefix)
    })
}

fn find_fragment_delimiter(url: &str) -> Option<usize> {
    url.match_indices('#').find_map(|(idx, _)| {
        if idx > 0 && url.as_bytes()[idx - 1] == b'&' {
            None
        } else {
            Some(idx)
        }
    })
}

fn push_capability_url(result: &mut String, url: &str, token: &str) {
    let fragment_start = find_fragment_delimiter(url);
    let (base, fragment) = match fragment_start {
        Some(pos) => (&url[..pos], Some(&url[pos + 1..])),
        None => (url, None),
    };
    result.push_str(base);
    if base.contains('?') {
        result.push_str("&token=");
    } else {
        result.push_str("?token=");
    }
    result.push_str(token);
    if let Some(f) = fragment {
        result.push('#');
        result.push_str(f);
    }
}

fn find_tag_end(html: &str, start: usize) -> Option<usize> {
    let bytes = html.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'>' => return Some(i + 1),
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            }
            b'\'' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
                if i < bytes.len() {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }
    None
}

fn scan(html: &str, token: &str) -> String {
    let mut result = String::with_capacity(html.len() + 128);
    let bytes = html.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        let Some(rel_lt) = bytes[cursor..].iter().position(|&b| b == b'<') else {
            result.push_str(&html[cursor..]);
            break;
        };

        let tag_start = cursor + rel_lt;
        result.push_str(&html[cursor..tag_start]);

        let after_lt = tag_start + 1;
        if after_lt >= bytes.len() {
            result.push('<');
            break;
        }

        if html[after_lt..].starts_with("!--") {
            if let Some(end_offset) = html[after_lt + 3..].find("-->") {
                let end = after_lt + 3 + end_offset + 3;
                result.push_str(&html[tag_start..end]);
                cursor = end;
            } else {
                result.push_str(&html[tag_start..]);
                break;
            }
            continue;
        }

        if bytes[after_lt] == b'/' || bytes[after_lt] == b'!' || bytes[after_lt] == b'?' {
            if let Some(gt_offset) = bytes[after_lt..].iter().position(|&b| b == b'>') {
                let end = after_lt + gt_offset + 1;
                result.push_str(&html[tag_start..end]);
                cursor = end;
            } else {
                result.push_str(&html[tag_start..]);
                break;
            }
            continue;
        }

        if !bytes[after_lt].is_ascii_alphabetic() {
            result.push('<');
            cursor = after_lt;
            continue;
        }

        let mut name_end = after_lt + 1;
        while name_end < bytes.len()
            && !bytes[name_end].is_ascii_whitespace()
            && bytes[name_end] != b'>'
            && bytes[name_end] != b'/'
        {
            name_end += 1;
        }
        let tag_name = &html[after_lt..name_end];

        let Some(tag_end) = find_tag_end(html, name_end) else {
            result.push_str(&html[tag_start..]);
            break;
        };

        let matches_capability_tag = CAPABILITY_ATTRIBUTES
            .iter()
            .any(|cap| tag_name.eq_ignore_ascii_case(cap.tag));

        if !matches_capability_tag {
            result.push_str(&html[tag_start..tag_end]);
            cursor = tag_end;
            continue;
        }

        result.push_str(&html[tag_start..name_end]);
        let mut idx = name_end;

        while idx < tag_end {
            let ws_start = idx;
            while idx < tag_end && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }
            result.push_str(&html[ws_start..idx]);
            if idx >= tag_end {
                break;
            }

            if bytes[idx] == b'>' {
                result.push_str(&html[idx..tag_end]);
                break;
            }
            if bytes[idx] == b'/' {
                if idx + 1 < tag_end && bytes[idx + 1] == b'>' {
                    result.push_str("/>");
                    break;
                } else {
                    result.push('/');
                    idx += 1;
                    continue;
                }
            }

            let attr_name_start = idx;
            while idx < tag_end
                && !bytes[idx].is_ascii_whitespace()
                && bytes[idx] != b'='
                && bytes[idx] != b'>'
                && bytes[idx] != b'/'
                && bytes[idx] != b'"'
                && bytes[idx] != b'\''
            {
                idx += 1;
            }
            if idx == attr_name_start {
                result.push(bytes[idx] as char);
                idx += 1;
                continue;
            }
            let attr_name = &html[attr_name_start..idx];
            result.push_str(attr_name);

            let ws_eq_start = idx;
            while idx < tag_end && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }

            if idx >= tag_end || bytes[idx] != b'=' {
                result.push_str(&html[ws_eq_start..idx]);
                continue;
            }

            result.push_str(&html[ws_eq_start..idx + 1]);
            idx += 1;

            let ws_val_start = idx;
            while idx < tag_end && bytes[idx].is_ascii_whitespace() {
                idx += 1;
            }
            result.push_str(&html[ws_val_start..idx]);
            if idx >= tag_end {
                break;
            }

            if bytes[idx] == b'"' {
                result.push('"');
                let val_start = idx + 1;
                let mut val_end = val_start;
                while val_end < tag_end && bytes[val_end] != b'"' {
                    val_end += 1;
                }
                let val = &html[val_start..val_end];
                if is_capability_attribute(tag_name, attr_name, val) {
                    push_capability_url(&mut result, val, token);
                } else {
                    result.push_str(val);
                }
                if val_end < tag_end && bytes[val_end] == b'"' {
                    result.push('"');
                    idx = val_end + 1;
                } else {
                    idx = val_end;
                }
            } else if bytes[idx] == b'\'' {
                result.push('\'');
                let val_start = idx + 1;
                let mut val_end = val_start;
                while val_end < tag_end && bytes[val_end] != b'\'' {
                    val_end += 1;
                }
                let val = &html[val_start..val_end];
                if is_capability_attribute(tag_name, attr_name, val) {
                    push_capability_url(&mut result, val, token);
                } else {
                    result.push_str(val);
                }
                if val_end < tag_end && bytes[val_end] == b'\'' {
                    result.push('\'');
                    idx = val_end + 1;
                } else {
                    idx = val_end;
                }
            } else {
                let val_start = idx;
                while idx < tag_end && !bytes[idx].is_ascii_whitespace() && bytes[idx] != b'>' {
                    idx += 1;
                }
                let val = &html[val_start..idx];
                if is_capability_attribute(tag_name, attr_name, val) {
                    push_capability_url(&mut result, val, token);
                } else {
                    result.push_str(val);
                }
            }
        }
        cursor = tag_end;
    }

    result
}

pub(super) fn content_security_policy() -> &'static str {
    "default-src 'self'; base-uri 'none'; img-src 'self' data:; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'"
}

#[cfg(test)]
mod tests {
    use super::{document_unavailable_page, inject_capability, page};

    const TEST_TOKEN: &str = "test-token";

    #[test]
    fn embedded_asset_with_repository_newline_then_omits_only_final_line_ending() {
        // Arrange
        let assets = ["content\n", "content\r\n", "content"];

        // Act
        let bodies = assets.map(super::embedded_asset);

        // Assert
        assert_eq!(bodies, ["content", "content", "content"]);
    }

    #[test]
    fn document_page_then_omits_document_navigation_controls() {
        // Arrange
        let document_content = "<p>Document content</p>";

        // Act
        let document_page = page("README.md", document_content.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(document_page.contains(document_content));
        assert!(!document_page.contains("Discovered documents"));
        assert!(!document_page.contains("document-catalog"));
        assert!(!document_page.contains("document-search"));
        assert!(!document_page.contains("data-document-navigation-control"));
        assert!(!document_page.contains("data-document-navigation-toggle"));
        assert!(!document_page.contains("<nav id=\"document-navigation\""));
    }

    #[test]
    fn document_page_then_omits_rendering_status_and_disable_control() {
        // Arrange
        let expected_content = "<p>Document content</p>";

        // Act
        let document_page = page("README.md", expected_content.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(document_page.contains(expected_content));
        assert!(!document_page.contains("PlantUML server rendering"));
        assert!(!document_page.contains("rendering-status"));
        assert!(!document_page.contains("data-disable-renderer"));
    }

    #[test]
    fn document_page_then_includes_session_token_and_no_referrer_policy() {
        // Arrange
        let content = "<p>Content</p>";

        // Act
        let rendered = page("Test", content.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(rendered.contains(r#"<meta name="referrer" content="no-referrer">"#));
        assert!(rendered.contains(r#"<link rel="stylesheet" href="/app.css?token=test-token">"#));
        assert!(rendered.contains(r#"<script src="/mermaid.js?token=test-token"></script>"#));
        assert!(rendered.contains(r#"<script src="/app.js?token=test-token"></script>"#));
        assert!(rendered.contains(r#"data-session-token="test-token""#));
    }

    #[test]
    fn inject_capability_then_attaches_token_to_document_and_diagram_urls() {
        // Arrange
        let html = r#"<p><a href="/documents/README.md#install">Install</a></p><img src="/diagrams/0/0" alt="Diagram"><a href="https://example.com">External</a>"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert!(injected.contains(r#"href="/documents/README.md?token=test-token#install""#));
        assert!(injected.contains(r#"src="/diagrams/0/0?token=test-token""#));
        assert!(injected.contains(r#"href="https://example.com""#));
    }

    #[test]
    fn unavailable_document_then_explains_how_to_return() {
        // Arrange
        let expected_message = "requested document is not part of this viewing session";

        // Act
        let page = document_unavailable_page(TEST_TOKEN);

        // Assert
        assert!(page.contains("<title>Lens: Document unavailable</title>"));
        assert!(page.contains(expected_message));
        assert!(page.contains(r#"href="/?token=test-token""#));
    }

    #[test]
    fn document_page_then_injects_script_and_stylesheet_tokens_only_in_head_and_body_tags() {
        // Arrange
        let content = "<p>Standard article content.</p>";

        // Act
        let rendered = page("Doc", content.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(rendered.contains(r#"<link rel="stylesheet" href="/app.css?token=test-token">"#));
        assert!(rendered.contains(r#"<script src="/mermaid.js?token=test-token"></script>"#));
        assert!(rendered.contains(r#"<script src="/app.js?token=test-token"></script>"#));
        assert!(rendered.contains("<article><p>Standard article content.</p></article>"));
    }

    #[test]
    fn document_page_with_head_token_in_prose_then_does_not_inject_script() {
        // Arrange
        let html = r#"<p>head token in prose: <script src="/diagrams/app.js"></script></p>"#;

        // Act
        let rendered = page("Head Token", html.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(rendered.contains(r#"<script src="/diagrams/app.js"></script>"#));
        assert!(!rendered.contains(r#"/diagrams/app.js?token="#));
    }

    #[test]
    fn document_page_with_link_tag_in_code_block_then_does_not_inject_stylesheet() {
        // Arrange
        let html = r#"<pre><code><link rel="stylesheet" href="/documents/style.css"></code></pre>"#;

        // Act
        let rendered = page("Code Block", html.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(rendered.contains(
            r#"<pre><code><link rel="stylesheet" href="/documents/style.css"></code></pre>"#
        ));
        assert!(!rendered.contains("/documents/style.css?token="));
    }

    #[test]
    fn document_page_with_external_links_then_does_not_inject_tokens() {
        // Arrange
        let content = r#"<p><a href="https://example.com/documents/doc.md">External</a></p>"#;

        // Act
        let rendered = page("External", content.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(rendered.contains(r#"<a href="https://example.com/documents/doc.md">External</a>"#));
        assert!(!rendered.contains("https://example.com/documents/doc.md?token="));
    }

    #[test]
    fn document_page_with_query_and_fragment_in_local_link_then_preserves_both() {
        // Arrange
        let content = r#"<p><a href="/documents/guide.md?view=full#chapter-1">Guide</a></p>"#;

        // Act
        let rendered = page("Query & Fragment", content.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(
            rendered.contains(r#"href="/documents/guide.md?view=full&token=test-token#chapter-1""#)
        );
    }

    #[test]
    fn document_page_with_apostrophe_entity_in_local_link_then_preserves_entity_and_fragment() {
        // Arrange
        let html = r#"<p><a href="/documents/O&#x27;Reilly.md#intro">Book</a></p>"#;

        // Act
        let rendered = page("Entity Test", html.to_owned(), None, TEST_TOKEN);

        // Assert
        assert!(rendered.contains(r#"href="/documents/O&#x27;Reilly.md?token=test-token#intro""#));
        assert!(!rendered.contains(r#"href="/documents/O&?token="#));
    }

    #[test]
    fn body_text_resembling_document_attribute_then_remains_byte_identical() {
        // Arrange
        let html = r#"<p>href="/documents/a.md"</p>"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(injected, html);
    }

    #[test]
    fn code_text_resembling_anchor_markup_then_remains_byte_identical() {
        // Arrange
        let html = r#"<pre><code>&lt;a href="/documents/a.md"&gt;</code></pre>"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(injected, html);
    }

    #[test]
    fn attribute_name_ending_in_href_then_is_not_rewritten() {
        // Arrange
        let html = r#"<a title="x" data-href="/documents/a.md" href="https://example.com">"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(injected, html);
    }

    #[test]
    fn capability_attribute_on_other_element_then_is_not_rewritten() {
        // Arrange
        let html = r#"<link href="/documents/a.md"><h1 src="/diagrams/0/0">Title</h1>"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(injected, html);
    }

    #[test]
    fn document_link_with_apostrophe_then_preserves_path_and_fragment() {
        // Arrange
        let html = r#"<a href="/documents/O&#x27;Reilly.md#intro">"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<a href="/documents/O&#x27;Reilly.md?token=test-token#intro">"#
        );
    }

    #[test]
    fn document_link_with_fragment_then_inserts_token_before_fragment() {
        // Arrange
        let html = r#"<a href="/documents/guide.md#install">"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<a href="/documents/guide.md?token=test-token#install">"#
        );
    }

    #[test]
    fn document_link_with_existing_query_then_appends_token_with_ampersand() {
        // Arrange
        let html = r#"<a href="/documents/guide.md?preview=true">"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<a href="/documents/guide.md?preview=true&token=test-token">"#
        );
    }

    #[test]
    fn document_link_with_query_and_fragment_then_preserves_query_and_fragment() {
        // Arrange
        let html = r#"<a href="/documents/guide.md?preview=true#install">"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<a href="/documents/guide.md?preview=true&token=test-token#install">"#
        );
    }

    #[test]
    fn diagram_image_with_trailing_boolean_attribute_then_appends_token() {
        // Arrange
        let html = r#"<img src="/diagrams/0/0" alt="d" data-diagram>"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<img src="/diagrams/0/0?token=test-token" alt="d" data-diagram>"#
        );
    }

    #[test]
    fn truncated_tag_then_copies_remainder_unchanged() {
        // Arrange
        let html = r#"<a href="/documents/a.md""#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(injected, html);
    }

    #[test]
    fn non_ascii_text_around_tags_then_is_preserved() {
        // Arrange
        let html = r#"<p>你好 <a href="/documents/指南.md#安装">安装</a> 世界 🚀</p>"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<p>你好 <a href="/documents/指南.md?token=test-token#安装">安装</a> 世界 🚀</p>"#
        );
    }

    #[test]
    fn document_link_with_decimal_apostrophe_entity_then_preserves_path_and_fragment() {
        // Arrange
        let html = r#"<a href="/documents/O&#39;Reilly.md#intro">"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<a href="/documents/O&#39;Reilly.md?token=test-token#intro">"#
        );
    }

    #[test]
    fn document_link_with_entity_and_no_fragment_then_appends_token() {
        // Arrange
        let html = r#"<a href="/documents/O&#x27;Reilly.md">"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<a href="/documents/O&#x27;Reilly.md?token=test-token">"#
        );
    }

    #[test]
    fn document_link_with_encoded_ampersand_before_fragment_then_splits_at_fragment() {
        // Arrange
        let html = r#"<a href="/documents/guide.md?foo=1&amp;#intro">"#;

        // Act
        let injected = inject_capability(html, TEST_TOKEN);

        // Assert
        assert_eq!(
            injected,
            r#"<a href="/documents/guide.md?foo=1&amp;&token=test-token#intro">"#
        );
    }
}
