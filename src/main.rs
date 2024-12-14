use clap::{Arg, Command};
use mdbook::errors::Error;
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_internal::Internal;
use semver::{Version, VersionReq};
use std::io;
use std::process;

pub fn make_app() -> Command {
    Command::new("mdbook-internal")
        .about("A mdbook preprocessor which parses, and optionally removes, internal sections and/or chapters")
        .subcommand(
            Command::new("supports")
                .arg(Arg::new("renderer").required(true))
                .about("Check whether a renderer is supported by this preprocessor"),
        )
}

fn main() {
    let matches = make_app().get_matches();
    let preprocessor = Internal {};
    if let Some(args) = matches.subcommand_matches("supports") {
        let support =
            preprocessor.supports_renderer(args.get_one::<String>("renderer").expect("Missing \"renderer\" argument"));
        process::exit(if support { 0 } else { 1 });
    } else if let Err(e) = handle_preprocessing(&preprocessor) {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn handle_preprocessing(preprocessor: &dyn Preprocessor) -> Result<(), Error> {
    let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;
    let book_version = Version::parse(&ctx.mdbook_version)?;
    let bundled_version = VersionReq::parse(mdbook::MDBOOK_VERSION)?;
    if !bundled_version.matches(&book_version) {
        eprintln!(
            "Warning: The {} plugin was built against version {} of mdbook, but we're being called from version {}",
            preprocessor.name(),
            mdbook::MDBOOK_VERSION,
            ctx.mdbook_version
        );
    }
    serde_json::to_writer(io::stdout(), &preprocessor.run(&ctx, book)?)?;
    Ok(())
}
