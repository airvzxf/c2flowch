use anyhow::Result;
use c2flowch_core::i18n::Language;
use c2flowch_core::ir::Flowchart;
use c2flowch_core::render::render_flowchart;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "c2flowch")]
#[command(about = "Convert C source code into ANSI flowcharts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum RenderFormat {
    Ascii,
    Svg,
    Png,
}

impl std::fmt::Display for RenderFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderFormat::Ascii => write!(f, "ascii"),
            RenderFormat::Svg => write!(f, "svg"),
            RenderFormat::Png => write!(f, "png"),
        }
    }
}

#[derive(Subcommand)]
enum Commands {
    /// Parse C code and generate an intermediate YAML representation
    Parse {
        /// The C source file to parse
        input: PathBuf,
        /// The output YAML file (defaults to input.yaml)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Level of detail (1-5)
        #[arg(short, long, default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..=5))]
        level: u8,
        /// Language (en, es)
        #[arg(long, default_value = "en")]
        lang: String,
    },
    /// Render a flowchart from an intermediate YAML representation
    Render {
        /// The YAML file to render
        input: PathBuf,
        /// The output file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Format (ascii, svg, png)
        #[arg(short, long, default_value = "ascii")]
        format: RenderFormat,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            input,
            output,
            level,
            lang,
        } => {
            let parsed_lang = lang.parse::<Language>().unwrap_or(Language::En);
            println!(
                "Parsing {} with detail level {level} in language {lang}...",
                input.display()
            );
            let code = fs::read_to_string(&input)?;
            let file_name = input.file_name().unwrap_or_default().to_string_lossy();
            let flowchart =
                c2flowch_core::parser::parse_source(&code, &file_name, level, parsed_lang)?;
            let yaml = flowchart.to_yaml()?;

            let output_path = output.unwrap_or_else(|| input.with_extension("yaml"));

            fs::write(&output_path, yaml)?;
            println!(
                "Success! Intermediate representation saved to {}",
                output_path.display()
            );
        }
        Commands::Render {
            input,
            output,
            format,
        } => {
            println!("Rendering {} as {format}...", input.display());
            let yaml = fs::read_to_string(&input)?;
            let flowchart = Flowchart::from_yaml(&yaml)?;

            match format {
                RenderFormat::Ascii => {
                    let result = render_flowchart(&flowchart)?;
                    if let Some(out_path) = output {
                        fs::write(&out_path, &result)?;
                        println!("Diagram saved to {}", out_path.display());
                    } else {
                        println!("{result}");
                    }
                }
                RenderFormat::Svg => {
                    let result = c2flowch_core::render::render_svg(&flowchart)?;
                    if let Some(out_path) = output {
                        fs::write(&out_path, &result)?;
                        println!("Diagram saved to {}", out_path.display());
                    } else {
                        println!("{result}");
                    }
                }
                RenderFormat::Png => {
                    let svg_string = c2flowch_core::render::render_svg(&flowchart)?;

                    let mut fontdb = resvg::usvg::fontdb::Database::new();
                    fontdb.load_system_fonts();

                    // Embed a guaranteed high-quality, readable font into the binary.
                    // IBM Plex Mono Medium is slightly thicker than Regular, solving the "thin/gray" issue in rasterized SVGs.
                    let font_data_medium =
                        include_bytes!("../../../resources/fonts/IBMPlexMono-Medium.ttf");
                    fontdb.load_font_data(font_data_medium.to_vec());

                    let font_data_bold =
                        include_bytes!("../../../resources/fonts/IBMPlexMono-Bold.ttf");
                    fontdb.load_font_data(font_data_bold.to_vec());

                    let font_data_italic =
                        include_bytes!("../../../resources/fonts/IBMPlexMono-Italic.ttf");
                    fontdb.load_font_data(font_data_italic.to_vec());

                    // Force our embedded font as the primary monospace fallback
                    fontdb.set_monospace_family("IBM Plex Mono");

                    let opt = resvg::usvg::Options {
                        fontdb: std::sync::Arc::new(fontdb),
                        ..Default::default()
                    };

                    let tree = resvg::usvg::Tree::from_str(&svg_string, &opt)
                        .map_err(|e| anyhow::anyhow!("SVG parsing failed: {}", e))?;

                    let pixmap_size = tree.size().to_int_size();
                    let mut pixmap =
                        resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
                            .ok_or_else(|| anyhow::anyhow!("Failed to create pixmap"))?;

                    resvg::render(
                        &tree,
                        resvg::tiny_skia::Transform::default(),
                        &mut pixmap.as_mut(),
                    );

                    if let Some(out_path) = output {
                        pixmap.save_png(&out_path)?;
                        println!("Diagram saved to {}", out_path.display());
                    } else {
                        let png_data = pixmap.encode_png()?;
                        use std::io::Write;
                        std::io::stdout().write_all(&png_data)?;
                    }
                }
            }
        }
    }

    Ok(())
}
