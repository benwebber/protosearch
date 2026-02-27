use std::io::{Read, Write};

use protobuf::Message;

fn main() -> protosearch_plugin::Result<()> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;
    let req = protobuf::plugin::CodeGeneratorRequest::parse_from_bytes(&buf)?;
    let (mut resp, diagnostics) = protosearch_plugin::process(req)?;
    if !diagnostics.is_empty() {
        resp.set_error(
            diagnostics
                .iter()
                .map(|d| d.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    let out = resp.write_to_bytes()?;
    std::io::stdout().write_all(&out)?;
    Ok(())
}
