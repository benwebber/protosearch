use std::fs;

use clap::Parser;
use openapiv3::OpenAPI;

use protosearch::cli;
use protosearch::proto;
use protosearch::spec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = cli::Args::parse();
    match &args.command {
        cli::Command::Compile {
            package,
            input,
            output,
            proto,
            tag_offset,
        } => {
            let reader = input.clone().into_reader()?;
            let spec: spec::Spec = serde_json::from_reader(reader)?;
            let file = match proto {
                Some(path) => {
                    let mut file: proto::File = serde_json::from_reader(fs::File::open(path)?)?;
                    file.package = package.to_string();
                    protosearch::compile_into(&spec, &mut file, *tag_offset)?;
                    file
                }
                None => protosearch::compile(package, &spec, *tag_offset)?,
            };
            let mut writer = output.clone().into_writer()?;
            serde_json::to_writer_pretty(&mut writer, &file)?;
        }
        cli::Command::Extract { input, output } => {
            let reader = input.clone().into_reader()?;
            let openapi: OpenAPI = serde_json::from_reader(reader)?;
            let spec = protosearch::extract(&openapi)?;
            let mut writer = output.clone().into_writer()?;
            serde_json::to_writer_pretty(&mut writer, &spec)?;
        }
        cli::Command::Render { input, output } => {
            let reader = input.clone().into_reader()?;
            let file: proto::File = serde_json::from_reader(reader)?;
            let mut writer = output.clone().into_writer()?;
            protosearch::render(&mut writer, &file)?;
        }
    }
    Ok(())
}
