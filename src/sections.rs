use crate::context::*;
use mdbook::{book::Book, BookItem};
use regex::{Captures, Regex};
use toml::value::{Table, Value};

pub(crate) struct Config {
    pub(crate) comment: Option<String>,
    pub(crate) wrap: Option<String>,
    pub(crate) class: Option<String>,
    pub(crate) label: Option<String>,
    pub(crate) styles: Option<Inline>,
}

fn regex(comment: &str) -> Regex {
    Regex::new(&format!(r#"\s*<!--\s*\[{0}\]\s*((?s).*?)\s*\[\/{0}\]\s*-->\s*"#, regex::escape(comment))).unwrap()
}

pub(crate) fn internalise_sections(book: Book, remove: bool, config: &Config) -> Book {
    match (&config.comment, remove) {
        (None, _) => book,
        (Some(comment), true) => remove_internal_sections(book, regex(comment)),
        (Some(comment), false) => stylise_internal_sections(book, regex(comment), config),
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

fn stylise_internal_sections(mut book: Book, re: Regex, config: &Config) -> Book {
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

impl Default for Config {
    fn default() -> Self {
        Self {
            comment: Some(DEFAULT_SECTIONS_COMMENT.to_string()),
            wrap: Some(DEFAULT_SECTIONS_WRAP.to_string()),
            class: Some(DEFAULT_SECTIONS_CLASS.to_string()),
            label: Some(DEFAULT_SECTIONS_LABEL.to_string()),
            styles: Some(Inline::default()),
        }
    }
}
impl Config {
    pub(crate) fn disabled() -> Self {
        Self { comment: None, ..Default::default() }
    }

    pub(crate) fn from_context(config: &Table) -> Result<Self, ConfigError> {
        let comment = match config.get("comment") {
            Some(Value::Boolean(true)) | None => Ok(Some(DEFAULT_SECTIONS_COMMENT.to_string())),
            Some(Value::String(comment)) => Ok(Some(comment.to_owned())),
            Some(Value::Boolean(false)) => Ok(None),
            _ => Err(ConfigError::new("`sections.comment` must be string or boolean")),
        }?;
        let wrap = match config.get("wrap") {
            Some(Value::Boolean(true)) | None => Ok(Some(DEFAULT_SECTIONS_WRAP.to_string())),
            Some(Value::String(element)) => Ok(Some(element.to_owned())),
            Some(Value::Boolean(false)) => Ok(None),
            _ => Err(ConfigError::new("`sections.wrap` must be string or boolean")),
        }?;
        let class = match config.get("class") {
            Some(Value::Boolean(true)) | None => Ok(Some(DEFAULT_SECTIONS_CLASS.to_string())),
            Some(Value::String(class)) => Ok(Some(class.to_owned())),
            Some(Value::Boolean(false)) => Ok(None),
            _ => Err(ConfigError::new("`sections.class` must be string or boolean")),
        }?;
        let label = match config.get("label") {
            Some(Value::Boolean(true)) | None => Ok(Some(DEFAULT_SECTIONS_LABEL.to_string())),
            Some(Value::String(label)) => Ok(Some(label.to_owned())),
            Some(Value::Boolean(false)) => Ok(None),
            _ => Err(ConfigError::new("`sections.label` must be string or boolean")),
        }?;
        let styles = match config.get("styles") {
            Some(Value::Boolean(true)) | None => Ok(Some(Inline::default())),
            Some(Value::Table(table)) => Ok(Some(Inline::from_context(table)?)),
            Some(Value::Boolean(false)) => Ok(None),
            _ => Err(ConfigError::new("`sections.styles` must be an object/map/table, or a boolean")),
        }?;
        Ok(Self { comment, wrap, class, label, styles })
    }

    pub(crate) fn get_wrap_opening_tag(&self) -> String {
        if let Some(wrap) = &self.wrap {
            let label = match (&self.label, &self.class, &self.styles) {
                (Some(label), None, None) => format!("<span>{label}</span>"),
                (Some(label), Some(class), None)
                | (Some(label), Some(class), Some(Inline { label: None, wrap: _ })) => {
                    format!("<span class='{class}'>{label}</span>")
                },
                (Some(label), Some(class), Some(Inline { label: Some(style), wrap: _ })) => {
                    format!("<span class='{class}' style='{style}'>{label}</span>")
                },
                (Some(label), None, Some(Inline { label: Some(style), wrap: _ })) => {
                    format!("<span style='{style}'>{label}</span>")
                },
                _ => "".to_string(),
            };
            let opening_tag = match (&self.class, &self.styles) {
                (Some(class), None) | (Some(class), Some(Inline { wrap: None, label: _ })) => {
                    format!("<{wrap} class='{class}'>")
                },
                (Some(class), Some(Inline { wrap: Some(style), label: _ })) => {
                    format!("<{wrap} class='{class}' style='{style}'>")
                },
                (None, Some(Inline { wrap: Some(style), label: _ })) => format!("<{wrap} style='{style}'>"),
                _ => format!("<{wrap}>"),
            };

            format!("\n{opening_tag}\n{label}\n")
        } else {
            "".to_string()
        }
    }

    pub(crate) fn get_wrap_closing_tag(&self) -> String {
        if let Some(wrap) = &self.wrap {
            format!("\n</{wrap}>\n")
        } else {
            "".to_string()
        }
    }
}

pub(crate) struct Inline {
    pub(crate) wrap: Option<String>,
    pub(crate) label: Option<String>,
}
impl Default for Inline {
    fn default() -> Self {
        Self {
            wrap: Some(DEFAULT_SECTIONS_STYLE_WRAP.to_string()),
            label: Some(DEFAULT_SECTIONS_STYLE_LABEL.to_string()),
        }
    }
}
impl Inline {
    fn from_context(config: &Table) -> Result<Self, ConfigError> {
        let wrap = match config.get("wrap") {
            Some(Value::Boolean(true)) | None => Ok(Some(DEFAULT_SECTIONS_STYLE_WRAP.to_string())),
            Some(Value::String(style)) => Ok(Some(style.to_owned())),
            Some(Value::Boolean(false)) => Ok(None),
            _ => Err(ConfigError::new("`sections.style.wrap` must be string or boolean")),
        }?;

        let label = match config.get("label") {
            Some(Value::Boolean(true)) | None => Ok(Some(DEFAULT_SECTIONS_STYLE_WRAP.to_string())),
            Some(Value::String(style)) => Ok(Some(style.to_owned())),
            Some(Value::Boolean(false)) => Ok(None),
            _ => Err(ConfigError::new("`sections.style.label` must be string or boolean")),
        }?;

        Ok(Self { wrap, label })
    }
}
