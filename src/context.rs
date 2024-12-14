use crate::{
    chapters::{Children, Config as Chapters},
    sections::Config as Sections,
};
use std::{error::Error, fmt::Display};
use toml::value::{Table, Value};

const DEFAULT_REMOVE: bool = false;
pub(crate) const DEFAULT_SECTIONS_COMMENT: &str = "internal";
pub(crate) const DEFAULT_SECTIONS_WRAP: &str = "blockquote";
pub(crate) const DEFAULT_SECTIONS_CLASS: &str = "mdbook-internal";
pub(crate) const DEFAULT_SECTIONS_LABEL: &str = "Internal";
pub(crate) const DEFAULT_SECTIONS_STYLE_WRAP: &str = "position: relative; padding: 20px 20px;";
pub(crate) const DEFAULT_SECTIONS_STYLE_LABEL: &str =
    "position: absolute; top: 0; right: 5px; font-size: 80%; opacity: 0.4;";
pub(crate) const DEFAULT_CHAPTERS_PREFIX: &str = "_";
pub(crate) const DEFAULT_CHAPTERS_STRIP: bool = false;
pub(crate) const DEFAULT_CHAPTERS_RECALCULATE: bool = true;
pub(crate) const DEFAULT_CHAPTERS_CHILDREN: Children = Children::Keep;

#[derive(Debug)]
pub struct ConfigError {
    message: &'static str,
}
impl ConfigError {
    pub(crate) fn new(message: &'static str) -> Self {
        Self { message }
    }
}
impl Error for ConfigError {}
impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
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
            None => Ok(DEFAULT_REMOVE),
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
