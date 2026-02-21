use std::io::{Read, Write};

use protobuf::Message;

fn main() -> protosearch_plugin::Result<()> {
    let mut buf = Vec::new();
    std::io::stdin().read_to_end(&mut buf)?;
    let req = protobuf::plugin::CodeGeneratorRequest::parse_from_bytes(&buf)?;
    let resp = protosearch_plugin::process(&req)?;
    let out = resp.write_to_bytes()?;
    std::io::stdout().write_all(&out)?;
    Ok(())
}
