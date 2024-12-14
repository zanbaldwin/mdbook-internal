use std::path::PathBuf;

use crate::context::*;
use mdbook::{
    book::{Book, Chapter, SectionNumber},
    BookItem,
};
use toml::value::{Table, Value};

pub(crate) enum Children {
    Keep,
    Remove,
}
pub(crate) struct Config {
    pub(crate) prefix: Option<String>,
    pub(crate) strip: bool,
    pub(crate) recalculate: bool,
    pub(crate) children: Children,
}

pub(crate) fn internalise_chapters(mut book: Book, remove: bool, config: &Config) -> Book {
    if let Some(ref prefix) = config.prefix {
        if remove {
            book.sections = remove_internal_chapters(book.sections, prefix, config);
        } else if config.strip {
            book.sections = strip_prefix_from_chapter_path(book.sections, prefix);
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

fn remove_internal_chapters(items: Vec<BookItem>, prefix: &str, config: &Config) -> Vec<BookItem> {
    items
        .into_iter()
        .filter_map(|item| match item {
            BookItem::Chapter(mut chapter) => match (has_prefix(&chapter, prefix), &config.children) {
                (true, Children::Remove) => None,
                (false, _) => {
                    chapter.sub_items = remove_internal_chapters(chapter.sub_items, prefix, config);
                    Some(BookItem::Chapter(chapter))
                },
                (_, Children::Keep) => {
                    chapter.sub_items = remove_internal_chapters(chapter.sub_items, prefix, config);
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
                if has_prefix(&chapter, prefix) {
                    chapter.path = chapter.path.map(|path| remove_prefix(path, prefix));
                }
                chapter.sub_items = strip_prefix_from_chapter_path(chapter.sub_items, prefix);
                BookItem::Chapter(chapter)
            },
            _ => item,
        })
        .collect()
}

pub(crate) fn recalculate_chapter_numbers_recursive(
    items: Vec<BookItem>,
    nested_count: &mut Vec<u32>,
) -> Vec<BookItem> {
    let mut current = 0;
    items
        .into_iter()
        .map(|item| match item {
            BookItem::Chapter(mut chapter) if chapter.number.is_some() => {
                current += 1;
                nested_count.push(current);
                chapter.number = Some(SectionNumber(nested_count.clone()));
                chapter.sub_items = recalculate_chapter_numbers_recursive(chapter.sub_items, nested_count);
                nested_count.pop();
                BookItem::Chapter(chapter)
            },
            _ => item,
        })
        .collect()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: Some(DEFAULT_CHAPTERS_PREFIX.to_string()),
            strip: DEFAULT_CHAPTERS_STRIP,
            recalculate: DEFAULT_CHAPTERS_RECALCULATE,
            children: Children::Remove,
        }
    }
}
impl Config {
    pub(crate) fn disabled() -> Self {
        Self { prefix: None, ..Default::default() }
    }

    pub(crate) fn from_context(config: &Table) -> Result<Self, ConfigError> {
        let prefix = match config.get("prefix") {
            Some(Value::Boolean(true)) | None => Ok(Some(DEFAULT_CHAPTERS_PREFIX.to_string())),
            Some(Value::Boolean(false)) => Ok(None),
            Some(Value::String(prefix)) => Ok(Some(prefix.to_string())),
            _ => Err(ConfigError::new("`chapters.prefix` must be string or boolean")),
        }?;
        let strip = match config.get("strip") {
            Some(Value::Boolean(toggle)) => Ok(*toggle),
            None => Ok(DEFAULT_CHAPTERS_STRIP),
            _ => Err(ConfigError::new("`chapters.strip` must be a boolean")),
        }?;
        let recalculate = match config.get("recalculate") {
            Some(Value::Boolean(toggle)) => Ok(*toggle),
            None => Ok(DEFAULT_CHAPTERS_RECALCULATE),
            _ => Err(ConfigError::new("`chapters.recalculate` must be a boolean")),
        }?;
        let children = match config.get("children") {
            None => Ok(DEFAULT_CHAPTERS_CHILDREN),
            Some(Value::String(value)) if value == "remove" => Ok(Children::Remove),
            Some(Value::String(value)) if value == "keep" => Ok(Children::Keep),
            _ => Err(ConfigError::new("`chapters.children` must be \"keep\" or \"remove\"")),
        }?;
        Ok(Self { prefix, strip, recalculate, children })
    }
}
