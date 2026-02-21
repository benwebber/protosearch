use std::collections::{HashMap, HashSet};
use std::io::Write;

pub mod cli;
pub mod error;
pub mod openapi;
pub mod proto;
pub mod spec;

pub use error::{Error, Result};

/// Extract a [`Spec`](spec::Spec) from an OpenAPI specification.
pub fn extract(openapi: &openapiv3::OpenAPI) -> Result<spec::MappingSpec> {
    let components = openapi
        .components
        .as_ref()
        .ok_or(Error::InvalidSpec("missing components".into()))?;
    let property_schema = components
        .schemas
        .get("_types.mapping.Property")
        .ok_or(Error::InvalidSpec(
            "missing _types.mapping.Property schema".into(),
        ))?
        .as_item()
        .ok_or(Error::InvalidSpec(
            "_types.mapping.Property is not an item".into(),
        ))?;
    let discriminator = &property_schema
        .schema_data
        .discriminator
        .as_ref()
        .ok_or(Error::InvalidSpec("missing discriminator".into()))?
        .mapping;
    let mut types: HashMap<String, spec::PropertyType> = HashMap::new();
    for (schema_name, typ) in openapi::iter_discriminator_types(discriminator) {
        let mut parameters = HashMap::new();
        let schema = openapi::resolve(components, schema_name)?;
        openapi::collect_parameters_into(components, schema, &mut parameters)?;
        parameters.remove("type");
        types.insert(
            schema_name.to_string(),
            spec::PropertyType {
                name: typ.to_string(),
                parameters,
            },
        );
    }

    let mut shared_types: HashMap<String, spec::SharedType> = HashMap::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut queue: Vec<String> = Vec::new();

    for prop in types.values() {
        for param in prop.parameters.values() {
            if let Some(name) = param.definition_name()
                && seen.insert(name.to_string())
            {
                queue.push(name.to_string());
            }
        }
    }

    while let Some(type_ref) = queue.pop() {
        if let Some(schema_ref) = components.schemas.get(&type_ref)
            && let Some(schema) = schema_ref.as_item()
        {
            let mut parameters = HashMap::new();
            openapi::collect_parameters_into(components, schema, &mut parameters)?;
            if !parameters.is_empty() {
                for param in parameters.values() {
                    if let Some(name) = param.definition_name()
                        && seen.insert(name.to_string())
                    {
                        queue.push(name.to_string());
                    }
                }
                shared_types.insert(type_ref, spec::SharedType { parameters });
            }
        }
    }

    let spec = spec::MappingSpec {
        types,
        shared_types,
    };
    Ok(spec)
}

/// Compile a [`Spec`](spec::Spec) into an existing [`File`](proto::File).
pub fn compile_into(
    spec: &spec::MappingSpec,
    file: Option<&mut proto::File>,
    number_offset: u32,
) -> Result<()> {
    let mut new = proto::File::new("");
    let file = file.unwrap_or(&mut new);
    let mut fields: Vec<proto::Field> = spec
        .types
        .iter()
        .map(|(ref_name, property)| proto::Field {
            name: property.name.clone(),
            typ: proto::FieldType::Optional(proto::ValueType::Message(
                proto::message_name(ref_name).into(),
            )),
            number: 0,
        })
        .collect();
    fields.sort_by(|a, b| a.name.cmp(&b.name));
    let new_ext = proto::ExtendBlock {
        name: "protosearch.FieldMappingOptions".into(),
        fields,
        reserved: Vec::new(),
    };
    if !file.extensions.iter().any(|e| e.name == new_ext.name) {
        file.extensions.push(proto::ExtendBlock {
            name: new_ext.name.clone(),
            fields: Vec::new(),
            reserved: Vec::new(),
        });
    }
    let ext = file
        .extensions
        .iter_mut()
        .find(|e| e.name == new_ext.name)
        .unwrap();
    ext.merge(new_ext, number_offset)?;

    let iter = spec
        .types
        .iter()
        .map(|(k, v)| (k, &v.parameters))
        .chain(spec.shared_types.iter().map(|(k, v)| (k, &v.parameters)));
    for (ref_name, parameters) in iter {
        let name = proto::message_name(ref_name).to_string();
        let mut params: Vec<_> = parameters.iter().collect();
        params.sort_by_key(|(name, _)| name.as_str());
        let fields: Vec<_> = params
            .into_iter()
            .map(|(k, v)| proto::Field {
                name: k.into(),
                typ: v.clone().into(),
                number: 0,
            })
            .collect();
        let new_message = proto::Message {
            name: name.clone(),
            fields,
            reserved: Vec::new(),
        };
        if !file.messages.iter().any(|m| m.name == name) {
            file.messages.push(proto::Message {
                name: name.clone(),
                fields: Vec::new(),
                reserved: Vec::new(),
            });
        }
        let message = file.messages.iter_mut().find(|m| m.name == name).unwrap();
        message.merge(new_message, 1)?;
    }

    file.messages.sort_by(|a, b| a.name.cmp(&b.name));
    file.extensions.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(())
}

/// Write a [`File`](proto::File) to a writer.
pub fn render(w: &mut impl Write, file: &proto::File) -> Result<()> {
    Ok(write!(w, "{}", file)?)
}

#[cfg(test)]
mod tests {
    macro_rules! snapshot_tests {
        ($mod:ident, $spec_path:expr, $package:expr, $number_offset:expr) => {
            mod $mod {
                fn load_openapi() -> openapiv3::OpenAPI {
                    let content = std::fs::read_to_string($spec_path).unwrap();
                    serde_json::from_str(&content).unwrap()
                }

                #[test]
                fn extract() {
                    let openapi = load_openapi();
                    let spec = crate::extract(&openapi).unwrap();
                    insta::with_settings!({ sort_maps => true }, {
                        insta::assert_json_snapshot!(spec);
                    });
                }

                #[test]
                fn compile_into() {
                    let openapi = load_openapi();
                    let spec = crate::extract(&openapi).unwrap();
                    let mut file = crate::proto::File::new($package);
                    crate::compile_into(&spec, Some(&mut file), $number_offset).unwrap();
                    insta::with_settings!({ sort_maps => true }, {
                        insta::assert_json_snapshot!(file);
                    });
                }

                #[test]
                fn render() {
                    let openapi = load_openapi();
                    let spec = crate::extract(&openapi).unwrap();
                    let mut file = crate::proto::File::new($package);
                    crate::compile_into(&spec, Some(&mut file), $number_offset).unwrap();
                    let mut buf = Vec::new();
                    crate::render(&mut buf, &file).unwrap();
                    insta::assert_snapshot!(String::from_utf8(buf).unwrap());
                }
            }
        };
    }

    snapshot_tests!(
        elasticsearch_v8,
        "../../spec/elasticsearch.v8.json",
        "protosearch.es.v8",
        100
    );
}
