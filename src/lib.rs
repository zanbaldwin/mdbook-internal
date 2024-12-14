use context::{Chapters as ChapterConfig, Children, Config, Sections};
use mdbook::{
    book::{Book, Chapter, SectionNumber},
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};
use regex::{Captures, Regex};
use std::path::PathBuf;

mod context;

const PREPROCESSOR_NAME: &str = "internal";

pub struct Internal;

impl Preprocessor for Internal {
    fn name(&self) -> &'static str {
        PREPROCESSOR_NAME
    }

    fn run(&self, ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
        let config = if let Some(preprocessor_config) = ctx.config.get_preprocessor(self.name()) {
            Config::from_context(preprocessor_config)
        } else {
            Ok(Config::default())
        }
        .map_err(Error::from)?;

        let book = Self::internalise_sections(book, &config);
        let mut book = Self::internalise_chapters(book, &config);
        if config.chapters.recalculate {
            book.sections = Self::recalculate_chapter_numbers(book.sections, &mut Vec::new());
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}

impl Internal {
    fn regex(comment: &str) -> Regex {
        Regex::new(&format!(r#"\s*<!--\s*\[{0}\]\s*((?s).*?)\s*\[\/{0}\]\s*-->\s*"#, regex::escape(comment))).unwrap()
    }

    fn internalise_sections(book: Book, config: &Config) -> Book {
        match (&config.sections.comment, config.remove) {
            (None, _) => book,
            (Some(comment), true) => Self::remove_internal_sections(book, Self::regex(comment)),
            (Some(comment), false) => Self::stylise_internal_sections(book, Self::regex(comment), &config.sections),
        }
    }

    fn remove_internal_sections(mut book: Book, re: Regex) -> Book {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                // The regular expression also matches whitespace before and after the tags, make sure to add a couple
                // extra new lines to prevent accidentally combining the Markdown sections before and after it.
                let modified = re.replace_all(chapter.content.as_str(), "\n\n".to_string());
                chapter.content = modified.to_string();
            }
        });
        book
    }

    fn stylise_internal_sections(mut book: Book, re: Regex, config: &Sections) -> Book {
        let opening = config.get_wrap_opening_tag();
        let closing = config.get_wrap_closing_tag();
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                let modified = re.replace_all(chapter.content.as_str(), |captures: &Captures| {
                    format!("\n{}\n{}\n{}\n", &opening, &captures[1], &closing)
                });
                chapter.content = modified.to_string();
            }
        });
        book
    }

    fn internalise_chapters(mut book: Book, config: &Config) -> Book {
        if let Some(ref prefix) = config.chapters.prefix {
            if config.remove {
                book.sections = Self::remove_internal_chapters(book.sections, prefix, &config.chapters);
            } else if config.chapters.strip {
                book.sections = Self::strip_prefix_from_chapter_path(book.sections, prefix);
            }
        }
        book
    }

    fn has_prefix(chapter: &Chapter, prefix: &str) -> bool {
        chapter
            .source_path
            .as_ref()
            .is_some_and(|s| s.file_name().is_some_and(|f| f.to_str().is_some_and(|f| f.starts_with(prefix))))
    }

    fn remove_prefix(path: PathBuf, prefix: &str) -> PathBuf {
        path.file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_prefix(prefix))
            .map(|filename| {
                let mut stripped = path.clone();
                stripped.set_file_name(filename);
                stripped
            })
            .unwrap_or(path)
    }

    fn remove_internal_chapters(items: Vec<BookItem>, prefix: &str, config: &ChapterConfig) -> Vec<BookItem> {
        items
            .into_iter()
            .filter_map(|item| match item {
                BookItem::Chapter(mut chapter) => match (Self::has_prefix(&chapter, prefix), &config.children) {
                    (true, Children::Remove) => None,
                    (false, _) => {
                        chapter.sub_items = Self::remove_internal_chapters(chapter.sub_items, prefix, config);
                        Some(BookItem::Chapter(chapter))
                    },
                    (_, Children::Keep) => {
                        chapter.sub_items = Self::remove_internal_chapters(chapter.sub_items, prefix, config);
                        match chapter.sub_items.len() {
                            0 => None,
                            _ => {
                                chapter.content = "".to_string();
                                chapter.path = None;
                                chapter.source_path = None;
                                Some(BookItem::Chapter(chapter))
                            },
                        }
                    },
                },
                _ => Some(item),
            })
            .collect()
    }

    fn strip_prefix_from_chapter_path(items: Vec<BookItem>, prefix: &str) -> Vec<BookItem> {
        items
            .into_iter()
            .map(|item| match item {
                BookItem::Chapter(mut chapter) => {
                    if Self::has_prefix(&chapter, prefix) {
                        chapter.path = chapter.path.map(|path| Self::remove_prefix(path, prefix));
                    }
                    chapter.sub_items = Self::strip_prefix_from_chapter_path(chapter.sub_items, prefix);
                    BookItem::Chapter(chapter)
                },
                _ => item,
            })
            .collect()
    }

    fn recalculate_chapter_numbers(items: Vec<BookItem>, nested_count: &mut Vec<u32>) -> Vec<BookItem> {
        let mut current = 0;
        items
            .into_iter()
            .map(|item| match item {
                BookItem::Chapter(mut chapter) if chapter.number.is_some() => {
                    current += 1;
                    nested_count.push(current);
                    chapter.number = Some(SectionNumber(nested_count.clone()));
                    chapter.sub_items = Self::recalculate_chapter_numbers(chapter.sub_items, nested_count);
                    nested_count.pop();
                    BookItem::Chapter(chapter)
                },
                _ => item,
            })
            .collect()
    }
}
