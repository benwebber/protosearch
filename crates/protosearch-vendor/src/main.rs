use std::fs;

use clap::Parser;
use openapiv3::OpenAPI;

use protosearch_vendor::{cli, proto, spec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();
    match &args.command {
        cli::Command::Compile {
            package,
            input,
            output,
            existing,
            number_offset,
        } => {
            let reader = input.clone().into_reader()?;
            let spec: spec::MappingSpec = serde_json::from_reader(reader)?;
            let mut file = match existing {
                Some(path) => {
                    let mut file: proto::File = serde_json::from_reader(fs::File::open(path)?)?;
                    file.package = package.to_string();
                    file
                }
                None => proto::File::new(package),
            };
            protosearch_vendor::compile_into(&spec, Some(&mut file), *number_offset)?;
            let mut writer = output.clone().into_writer()?;
            serde_json::to_writer_pretty(&mut writer, &file)?;
        }
        cli::Command::Extract { input, output } => {
            let reader = input.clone().into_reader()?;
            let openapi: OpenAPI = serde_json::from_reader(reader)?;
            let spec = protosearch_vendor::extract(&openapi)?;
            let mut writer = output.clone().into_writer()?;
            serde_json::to_writer_pretty(&mut writer, &spec)?;
        }
        cli::Command::Render { input, output } => {
            let reader = input.clone().into_reader()?;
            let file: proto::File = serde_json::from_reader(reader)?;
            let mut writer = output.clone().into_writer()?;
            protosearch_vendor::render(&mut writer, &file)?;
        }
    }
    Ok(())
}
