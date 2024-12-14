use std::{error::Error, fmt::Display};
use toml::value::{Table, Value};

const DEFAULT_REMOVE: bool = false;
const DEFAULT_SECTIONS_COMMENT: &str = "internal";
const DEFAULT_SECTIONS_WRAP: &str = "blockquote";
const DEFAULT_SECTIONS_CLASS: &str = "mdbook-internal";
const DEFAULT_SECTIONS_LABEL: &str = "Internal";
const DEFAULT_SECTIONS_STYLE_WRAP: &str = "position: relative; padding: 20px 20px;";
const DEFAULT_SECTIONS_STYLE_LABEL: &str = "position: absolute; top: 0; right: 5px; font-size: 80%; opacity: 0.4;";
const DEFAULT_CHAPTERS_PREFIX: &str = "_";
const DEFAULT_CHAPTERS_STRIP: bool = false;
const DEFAULT_CHAPTERS_RECALCULATE: bool = true;
const DEFAULT_CHAPTERS_CHILDREN: Children = Children::Keep;

#[derive(Debug)]
pub struct ConfigError {
    message: &'static str,
}
impl ConfigError {
    fn new(message: &'static str) -> Self {
        Self { message }
    }
}
impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error for ConfigError {}

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

pub(crate) struct Sections {
    pub(crate) comment: Option<String>,
    pub(crate) wrap: Option<String>,
    pub(crate) class: Option<String>,
    pub(crate) label: Option<String>,
    pub(crate) styles: Option<Inline>,
}
impl Default for Sections {
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
impl Sections {
    fn disabled() -> Self {
        Self { comment: None, ..Default::default() }
    }

    fn from_context(config: &Table) -> Result<Self, ConfigError> {
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

pub(crate) enum Children {
    Keep,
    Remove,
}
pub(crate) struct Chapters {
    pub(crate) prefix: Option<String>,
    pub(crate) strip: bool,
    pub(crate) recalculate: bool,
    pub(crate) children: Children,
}
impl Default for Chapters {
    fn default() -> Self {
        Self {
            prefix: Some(DEFAULT_CHAPTERS_PREFIX.to_string()),
            strip: DEFAULT_CHAPTERS_STRIP,
            recalculate: DEFAULT_CHAPTERS_RECALCULATE,
            children: Children::Remove,
        }
    }
}
impl Chapters {
    fn disabled() -> Self {
        Self { prefix: None, ..Default::default() }
    }

    fn from_context(config: &Table) -> Result<Self, ConfigError> {
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

pub(crate) struct Config {
    pub(crate) remove: bool,
    pub(crate) sections: Sections,
    pub(crate) chapters: Chapters,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            remove: DEFAULT_REMOVE,
            sections: Sections::default(),
            chapters: Chapters::default(),
        }
    }
}
impl Config {
    pub(crate) fn from_context(config: &Table) -> Result<Self, ConfigError> {
        let remove = match config.get("remove") {
            Some(Value::Boolean(toggle)) => Ok(*toggle),
            _ => Err(ConfigError::new("`remove` must be a boolean")),
        }?;
        let sections = match config.get("sections") {
            Some(Value::Boolean(true)) | None => Ok(Sections::default()),
            Some(Value::Boolean(false)) => Ok(Sections::disabled()),
            Some(Value::Table(table)) => Sections::from_context(table),
            _ => Err(ConfigError::new("`sections` must be an object/map/table, or a boolean")),
        }?;
        let chapters = match config.get("chapters") {
            Some(Value::Boolean(true)) | None => Ok(Chapters::default()),
            Some(Value::Boolean(false)) => Ok(Chapters::disabled()),
            Some(Value::Table(table)) => Chapters::from_context(table),
            _ => Err(ConfigError::new("`chapters` must be an object/map/table, or a boolean")),
        }?;
        Ok(Self { remove, sections, chapters })
    }
}
