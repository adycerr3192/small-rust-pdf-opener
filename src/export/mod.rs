//! Export PDF page text to Markdown or Word (.docx).

use std::path::Path;

use crate::error::Result;
use crate::pdf::DocumentSession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageText {
    /// 1-based page number for display.
    pub page_1based: usize,
    pub text: String,
}

/// Collect embedded text for 0-based page indices via MuPDF.
pub fn collect_page_texts(session: &DocumentSession, pages: &[usize]) -> Result<Vec<PageText>> {
    let mut out = Vec::with_capacity(pages.len());
    for &page in pages {
        let text = session.extract_text(page)?;
        out.push(PageText {
            page_1based: page + 1,
            text,
        });
    }
    Ok(out)
}

/// Format page texts as agent-friendly Markdown with page headings.
pub fn format_markdown(title: &str, pages: &[PageText]) -> String {
    let mut s = String::new();
    s.push_str("# ");
    s.push_str(title);
    s.push_str("\n\n");
    for (i, page) in pages.iter().enumerate() {
        if i > 0 {
            s.push('\n');
        }
        s.push_str(&format!("## Page {}\n\n", page.page_1based));
        let body = page.text.trim_end();
        if !body.is_empty() {
            s.push_str(body);
            s.push('\n');
        }
    }
    s
}

pub fn write_markdown(title: &str, pages: &[PageText], path: &Path) -> Result<()> {
    let content = format_markdown(title, pages);
    std::fs::write(path, content)?;
    Ok(())
}

pub fn write_docx(title: &str, pages: &[PageText], path: &Path) -> Result<()> {
    use docx_rs::*;

    let mut children: Vec<Paragraph> = Vec::new();
    children.push(
        Paragraph::new().add_run(Run::new().add_text(title).bold().size(32)),
    );

    for (i, page) in pages.iter().enumerate() {
        if i > 0 {
            children.push(Paragraph::new().add_run(Run::new().add_break(BreakType::Page)));
        }
        children.push(
            Paragraph::new().add_run(
                Run::new()
                    .add_text(format!("Page {}", page.page_1based))
                    .bold()
                    .size(28),
            ),
        );
        let body = page.text.trim_end();
        if body.is_empty() {
            children.push(Paragraph::new());
        } else {
            for line in body.split('\n') {
                children.push(Paragraph::new().add_run(Run::new().add_text(line)));
            }
        }
    }

    let mut doc = Docx::new();
    for p in children {
        doc = doc.add_paragraph(p);
    }
    let file = std::fs::File::create(path)?;
    doc.build().pack(file).map_err(|e| {
        crate::error::AppError::msg(format!("failed to write docx: {e}"))
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    #[test]
    fn format_markdown_includes_title_and_page_headings() {
        let pages = vec![
            PageText {
                page_1based: 1,
                text: "Hello\n".into(),
            },
            PageText {
                page_1based: 3,
                text: String::new(),
            },
        ];
        let md = format_markdown("invoice.pdf", &pages);
        assert!(md.starts_with("# invoice.pdf\n\n"));
        assert!(md.contains("## Page 1\n\nHello\n"));
        assert!(md.contains("## Page 3\n\n"));
        // Empty page still has heading; no body after heading before next section or EOF
        let page3 = md.find("## Page 3\n\n").unwrap();
        assert_eq!(&md[page3..], "## Page 3\n\n");
    }

    #[test]
    fn write_docx_produces_valid_zip_with_document_xml() {
        let dir = std::env::temp_dir().join(format!(
            "pdf-opener-export-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.docx");
        let pages = vec![
            PageText {
                page_1based: 1,
                text: "Line one".into(),
            },
            PageText {
                page_1based: 2,
                text: String::new(),
            },
        ];
        write_docx("demo.pdf", &pages, &path).unwrap();
        assert!(path.metadata().unwrap().len() > 0);

        let file = std::fs::File::open(&path).unwrap();
        let mut archive = zip::ZipArchive::new(file).unwrap();
        let mut document = archive.by_name("word/document.xml").unwrap();
        let mut xml = String::new();
        document.read_to_string(&mut xml).unwrap();
        assert!(xml.contains("demo.pdf"));
        assert!(xml.contains("Page 1"));
        assert!(xml.contains("Line one"));
        assert!(xml.contains("Page 2"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn collect_and_write_markdown_from_hello_pdf() {
        let pdf = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/hello.pdf");
        if !pdf.exists() {
            return;
        }
        let session = DocumentSession::open(&pdf).unwrap();
        let pages = collect_page_texts(&session, &[0]).unwrap();
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].page_1based, 1);
        let dir = std::env::temp_dir().join(format!(
            "pdf-opener-md-smoke-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let out = dir.join("hello.md");
        write_markdown("hello.pdf", &pages, &out).unwrap();
        let md = std::fs::read_to_string(&out).unwrap();
        assert!(md.starts_with("# hello.pdf\n\n## Page 1\n\n"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
