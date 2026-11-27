use anyhow::{Context, Result, anyhow};
use c2flowch_core::{Language, ParseOptions, Program};
use clap::{Parser, Subcommand, ValueEnum};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "c2flowch", version, author, about)]
#[command(long_about = "c2flowch (C to Flowchart)\n\n\
Converts C source code into flowcharts with ANSI/ISO 5807 symbols, in two steps:\n\n\
  1. parse:  C source → YAML (a structured description you can edit by hand)\n  \
2. render: YAML → text, SVG or PNG")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum Format {
    /// Unicode box-drawing characters
    Ascii,
    /// Vector image
    Svg,
    /// Raster image, rendered from the SVG
    Png,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Lang {
    En,
    Es,
}

impl From<Lang> for Language {
    fn from(l: Lang) -> Self {
        match l {
            Lang::En => Language::En,
            Lang::Es => Language::Es,
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Parse C code into the YAML description of its flowcharts
    ///
    /// The YAML holds one flowchart per function, as a tree of statements
    /// (process, io, call, if, while, for, switch…). Every text is the
    /// label printed in the symbol, so you can edit it before rendering.
    Parse {
        /// The C source file
        input: PathBuf,
        /// The YAML file to write [default: INPUT with the .yaml extension]
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Level of detail
        ///
        /// 1: calls to functions of the file, and the structures around them
        /// 2: every control structure, input/output and calls
        /// 3: level 2 plus assignments and other operations (default)
        /// 4: level 3 plus declarations without a value
        /// 5: every statement in its own symbol (nothing is grouped)
        #[arg(short, long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..=5), verbatim_doc_comment)]
        level: u8,
        /// Language of the labels (Start, End, Print, True…)
        #[arg(long, value_enum, default_value_t = Lang::En)]
        lang: Lang,
    },
    /// Render the YAML description as a flowchart
    Render {
        /// The YAML file written by `parse`
        input: PathBuf,
        /// The file to write [default: standard output]
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Output format
        #[arg(short, long, value_enum, default_value_t = Format::Ascii)]
        format: Format,
        /// Write one file per function: OUTPUT for the first one and
        /// OUTPUT-<function>.<ext> for the others
        #[arg(short, long)]
        split_subroutines: bool,
    },
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Commands::Parse {
            input,
            output,
            level,
            lang,
        } => parse(&input, output, level, lang.into()),
        Commands::Render {
            input,
            output,
            format,
            split_subroutines,
        } => render(&input, output.as_deref(), format, split_subroutines),
    }
}

fn parse(input: &Path, output: Option<PathBuf>, level: u8, lang: Language) -> Result<()> {
    let code = fs::read_to_string(input).with_context(|| format!("reading {}", input.display()))?;
    let title = input
        .file_name()
        .map(|f| f.to_string_lossy().into_owned())
        .unwrap_or_default();
    let parsed = c2flowch_core::parse(&code, &ParseOptions { title, level, lang })?;
    for w in &parsed.warnings {
        eprintln!("warning: {}:{w}", input.display());
    }
    let output = output.unwrap_or_else(|| input.with_extension("yaml"));
    fs::write(&output, parsed.program.to_yaml()?)
        .with_context(|| format!("writing {}", output.display()))?;
    eprintln!("Saved {}", output.display());
    Ok(())
}

fn render(input: &Path, output: Option<&Path>, format: Format, split: bool) -> Result<()> {
    let yaml = fs::read_to_string(input).with_context(|| format!("reading {}", input.display()))?;
    let program =
        Program::from_yaml(&yaml).with_context(|| format!("reading {}", input.display()))?;
    for w in c2flowch_core::validate(&program) {
        eprintln!("warning: {}: {w}", input.display());
    }

    let parts: Vec<(Option<String>, Program)> = if split {
        program
            .functions
            .iter()
            .enumerate()
            .map(|(i, f)| {
                let suffix = (i > 0).then(|| f.name.clone());
                let one = Program {
                    functions: vec![f.clone()],
                    ..program.clone()
                };
                (suffix, one)
            })
            .collect()
    } else {
        vec![(None, program)]
    };

    for (suffix, prog) in parts {
        let bytes = match format {
            Format::Ascii => c2flowch_core::render_ascii(&prog).into_bytes(),
            Format::Svg => c2flowch_core::render_svg(&prog).into_bytes(),
            Format::Png => png(&c2flowch_core::render_svg(&prog))?,
        };
        match output {
            Some(path) => {
                let path = match &suffix {
                    Some(s) => suffixed(path, s),
                    None => path.to_path_buf(),
                };
                fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
                eprintln!("Saved {}", path.display());
            }
            None => std::io::stdout().write_all(&bytes)?,
        }
    }
    Ok(())
}

/// `out/flow.svg` + `sum` → `out/flow-sum.svg`.
fn suffixed(path: &Path, suffix: &str) -> PathBuf {
    let safe: String = suffix
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let name = match path.extension() {
        Some(ext) => format!("{stem}-{safe}.{}", ext.to_string_lossy()),
        None => format!("{stem}-{safe}"),
    };
    path.with_file_name(name)
}

fn png(svg: &str) -> Result<Vec<u8>> {
    use resvg::{tiny_skia, usvg};
    let mut fonts = usvg::fontdb::Database::new();
    fonts.load_system_fonts();
    // Embedded so that the image looks the same on every machine.
    fonts
        .load_font_data(include_bytes!("../../../resources/fonts/IBMPlexMono-Medium.ttf").to_vec());
    fonts.load_font_data(include_bytes!("../../../resources/fonts/IBMPlexMono-Bold.ttf").to_vec());
    fonts
        .load_font_data(include_bytes!("../../../resources/fonts/IBMPlexMono-Italic.ttf").to_vec());
    fonts.set_monospace_family("IBM Plex Mono");
    fonts.set_sans_serif_family("IBM Plex Mono");
    let opt = usvg::Options {
        fontdb: std::sync::Arc::new(fonts),
        ..Default::default()
    };
    let tree = usvg::Tree::from_str(svg, &opt).map_err(|e| anyhow!("SVG parsing failed: {e}"))?;
    let size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| anyhow!("the image is too large"))?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());
    Ok(pixmap.encode_png()?)
}
