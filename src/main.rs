mod ir;
mod parser;
mod render;

use crate::ir::Flowchart;
use crate::render::render_flowchart;
use anyhow::Result;
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
}

impl std::fmt::Display for RenderFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderFormat::Ascii => write!(f, "ascii"),
            RenderFormat::Svg => write!(f, "svg"),
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
        #[arg(short, long, default_value_t = 3)]
        level: u8,
    },
    /// Render a flowchart from an intermediate YAML representation
    Render {
        /// The YAML file to render
        input: PathBuf,
        /// The output file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Format (ascii, svg)
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
        } => {
            println!("Parsing {:?} with detail level {}...", input, level);
            let flowchart = crate::parser::parse_file(&input, level)?;
            let yaml = flowchart.to_yaml()?;

            let output_path = output.unwrap_or_else(|| input.with_extension("yaml"));

            fs::write(&output_path, yaml)?;
            println!(
                "Success! Intermediate representation saved to {:?}",
                output_path
            );
        }
        Commands::Render {
            input,
            output,
            format,
        } => {
            println!("Rendering {:?} as {}...", input, format);
            let yaml = fs::read_to_string(&input)?;
            let flowchart = Flowchart::from_yaml(&yaml)?;

            match format {
                RenderFormat::Ascii => {
                    let result = render_flowchart(&flowchart)?;
                    if let Some(out_path) = output {
                        fs::write(&out_path, &result)?;
                        println!("Diagram saved to {:?}", out_path);
                    } else {
                        println!("{}", result);
                    }
                }
                RenderFormat::Svg => {
                    anyhow::bail!("Format svg not supported yet");
                }
            }
        }
    }

    Ok(())
}
