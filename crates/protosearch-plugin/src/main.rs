use std::io::{Read, Write};

use protobuf::Message;

fn main() -> protosearch_plugin::Result<()> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;
    let req = protobuf::plugin::CodeGeneratorRequest::parse_from_bytes(&buf)?;
    let (mut resp, diagnostics) = protosearch_plugin::process(req)?;
    let (errors, warnings): (Vec<_>, Vec<_>) = diagnostics.iter().partition(|d| d.is_error());
    if !errors.is_empty() {
        resp.set_error(
            diagnostics
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    for w in &warnings {
        eprintln!("{w}");
    }
    let out = resp.write_to_bytes()?;
    std::io::stdout().write_all(&out)?;
    Ok(())
}
