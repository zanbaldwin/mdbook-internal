use crate::{
    chapters::{internalise_chapters, recalculate_chapter_numbers_recursive},
    context::Config,
    sections::internalise_sections,
};
use mdbook::{
    book::Book,
    errors::Error,
    preprocess::{Preprocessor, PreprocessorContext},
};

mod chapters;
mod context;
mod sections;

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

        let book = internalise_sections(book, config.remove, &config.sections);
        let mut book = internalise_chapters(book, config.remove, &config.chapters);
        if config.chapters.recalculate {
            book.sections = recalculate_chapter_numbers_recursive(book.sections, &mut Vec::new());
        }

        Ok(book)
    }

    fn supports_renderer(&self, renderer: &str) -> bool {
        renderer != "not-supported"
    }
}
