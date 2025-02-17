use std::path::{Path, PathBuf};

use anstream::ColorChoice;
use anyhow::{bail, Context as _, Result};
use args::{CliArgs, Command, GlobalArgs};
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use config::Config;
use cooklang::{convert::ConverterBuilder, Converter, CooklangParser};
use cooklang_fs::{resolve_recipe, FsIndex};
use once_cell::sync::OnceCell;

// commands
mod config;
mod convert;
mod generate_completions;
mod list;
mod recipe;
#[cfg(feature = "serve")]
mod serve;
mod shopping_list;
mod units;

// other modules
mod args;
mod util;

const COOK_DIR: &str = ".cooklang";
const APP_NAME: &str = "cooklang-chef";

pub fn main() -> Result<()> {
    let args = CliArgs::parse();

    let color_ctx = init_color(args.global_args.color);
    if args.global_args.debug_trace {
        tracing_subscriber::FmtSubscriber::builder()
            .compact()
            .with_max_level(tracing::Level::TRACE)
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::CLOSE
                    | tracing_subscriber::fmt::format::FmtSpan::NEW,
            )
            .with_ansi(color_ctx.color_stderr)
            .init();
    } else {
        tracing_subscriber::FmtSubscriber::builder()
            .compact()
            .with_target(false)
            .with_ansi(color_ctx.color_stderr)
            .init();
    }

    let ctx = configure_context(args.global_args, color_ctx)?;

    match args.command {
        Command::Recipe(args) => recipe::run(&ctx, args),
        Command::List(args) => list::run(&ctx, args),
        #[cfg(feature = "serve")]
        Command::Serve(args) => serve::run(ctx, args),
        Command::ShoppingList(args) => shopping_list::run(&ctx, args),
        Command::Units(args) => units::run(ctx.parser()?.converter(), args),
        Command::Convert(args) => convert::run(ctx.parser()?.converter(), args),
        Command::Config => config::run(&ctx),
        Command::GenerateCompletions(args) => generate_completions::run(args),
    }
}

struct ColorContext {
    color_stderr: bool,
}

fn init_color(color: colorchoice_clap::Color) -> ColorContext {
    color.write_global();
    let color_stderr = anstream::AutoStream::choice(&std::io::stderr()) != ColorChoice::Never;

    ColorContext { color_stderr }
}

pub struct Context {
    parser: OnceCell<CooklangParser>,
    recipe_index: FsIndex,
    global_args: GlobalArgs,
    base_path: Utf8PathBuf,
    config: config::Config,
    config_path: PathBuf,
    color: ColorContext,
}

#[tracing::instrument(level = "debug", skip_all)]
fn configure_context(args: GlobalArgs, color_ctx: ColorContext) -> Result<Context> {
    let base_path = args
        .path
        .as_deref()
        .unwrap_or(Utf8Path::new("."))
        .to_path_buf();
    let (mut config, config_path) = Config::read(&base_path)?;
    config.override_with_args(&args);
    if !base_path.is_dir() {
        bail!("Base path '{base_path}' is not a directory");
    }

    let mut index = FsIndex::new(&base_path, config.max_depth)?;
    index.set_config_dir(COOK_DIR.to_string());

    Ok(Context {
        parser: OnceCell::new(),
        recipe_index: index,
        config,
        config_path,
        global_args: args,
        base_path,
        color: color_ctx,
    })
}

impl Context {
    fn parser(&self) -> Result<&CooklangParser> {
        self.parser.get_or_try_init(|| {
            configure_parser(
                &self.config,
                self.base_path.as_std_path(),
                &self.config_path,
            )
        })
    }

    fn checker(&self, relative_to: Option<&Utf8Path>) -> Option<cooklang::RecipeRefChecker> {
        if self.global_args.no_recipe_ref_check {
            None
        } else {
            let relative_to = relative_to.map(|r| r.to_path_buf());
            Some(Box::new(move |name: &str| {
                resolve_recipe(name, &self.recipe_index, relative_to.as_deref()).is_ok()
            }) as cooklang::RecipeRefChecker)
        }
    }

    fn parse_content(
        &self,
        content: &cooklang_fs::RecipeContent,
    ) -> Result<cooklang::RecipeResult> {
        Ok(self.parser()?.parse_with_recipe_ref_checker(
            content.text(),
            content.name(),
            self.checker(Some(content.path())),
        ))
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn configure_parser(
    config: &Config,
    base_path: &Path,
    config_path: &Path,
) -> Result<CooklangParser> {
    let units = config.units(config_path, base_path);
    let converter = if config.default_units || !units.is_empty() {
        let mut builder = ConverterBuilder::new();
        if config.default_units {
            builder
                .add_bundled_units()
                .expect("Failed to add bundled units");
        }
        for file in units {
            tracing::debug!("Loading units {}", file.display());
            let text = std::fs::read_to_string(&file)
                .with_context(|| format!("Cannot find units file: {}", file.display()))?;
            let units = toml::from_str(&text)?;
            builder.add_units_file(units)?;
        }
        builder.finish()?
    } else {
        Converter::default()
    };
    Ok(CooklangParser::new(config.extensions, converter))
}
