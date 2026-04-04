mod cli;
mod commands;
mod document;
mod error;
mod util;

use clap::Parser;

use cli::{Cli, Command, ConvertTarget, ExtractWhat, FormAction, PipelineAction};
use error::CliError;

fn main() {
    let cli = Cli::parse();

    let result = dispatch(&cli);

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(e.exit_code());
    }
}

fn dispatch(cli: &Cli) -> Result<(), CliError> {
    match &cli.command {
        Command::Info(args) => commands::info::run(args, cli),
        Command::Extract(args) => match &args.what {
            ExtractWhat::Text(a) => commands::extract::run_text(a, cli),
            ExtractWhat::Tables(a) => commands::extract::run_tables(a, cli),
            ExtractWhat::Images(a) => commands::extract::run_images(a, cli),
            ExtractWhat::Structure(a) => commands::extract::run_structure(a, cli),
            ExtractWhat::Bookmarks(a) => commands::extract::run_bookmarks(a, cli),
            ExtractWhat::Annotations(a) => commands::extract::run_annotations(a, cli),
            ExtractWhat::Metadata(a) => commands::extract::run_metadata(a, cli),
        },
        Command::Convert(args) => match &args.target {
            ConvertTarget::Markdown(a) => commands::convert::run_markdown(a, cli),
            ConvertTarget::PdfA(a) => commands::convert::run_pdf_a(a, cli),
            ConvertTarget::ToPdf(a) => commands::convert::run_to_pdf(a, cli),
            ConvertTarget::ToDocx(a) => commands::convert::run_to_docx(a, cli),
            ConvertTarget::ToXlsx(a) => commands::convert::run_to_xlsx(a, cli),
            ConvertTarget::ToHtml(a) => commands::convert::run_to_html(a, cli),
            ConvertTarget::ToEpub(a) => commands::convert::run_to_epub(a, cli),
            ConvertTarget::Auto(a) => commands::convert::run_auto(a, cli),
        },
        Command::Merge(args) => commands::merge::run(args, cli),
        Command::Split(args) => commands::split::run(args, cli),
        Command::Rotate(args) => commands::rotate::run(args, cli),
        Command::Reorder(args) => commands::reorder::run(args, cli),
        Command::Delete(args) => commands::delete::run(args, cli),
        Command::Watermark(args) => commands::watermark::run(args, cli),
        Command::Stamp(args) => commands::stamp::run(args, cli),
        Command::Redact(args) => commands::redact::run(args, cli),
        Command::Sanitize(args) => commands::sanitize::run(args, cli),
        Command::Optimize(args) => commands::optimize::run(args, cli),
        Command::Encrypt(args) => commands::encrypt::run(args, cli),
        Command::Sign(args) => commands::sign::run(args, cli),
        Command::Verify(args) => commands::verify::run(args, cli),
        Command::Validate(args) => commands::validate::run(args, cli),
        Command::Diff(args) => commands::diff::run(args, cli),
        Command::Form(args) => match &args.action {
            FormAction::Inspect(a) => commands::form::run_inspect(a, cli),
            FormAction::Fill(a) => commands::form::run_fill(a, cli),
        },
        Command::Toc(args) => commands::toc::run(args, cli),
        #[cfg(feature = "render")]
        Command::Render(args) => commands::render::run(args, cli),
        Command::Pipeline(args) => match &args.action {
            PipelineAction::Run(a) => commands::pipeline::run(a, cli),
            PipelineAction::Validate(a) => commands::pipeline::validate(a, cli),
        },
    }
}
