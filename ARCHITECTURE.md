# Architecture

`protosearch` is a protobuf library and plugin to define document mappings as protobuf message types.

It has two connected components:

1. The [`protosearch`](proto/protosearch/protosearch.proto) library provides the core protobuf field options extension.
2. The `protoc-gen-protosearch` `protoc` compiler plugin compiles messages annotated with these options into document mappings.

## Design

This is a Rust project with two crates:

* [`protosearch-vendor`](crates/protosearch-vendor/)

  This crate compiles Elasticsearch/OpenSearch OpenAPI specifications into protobuf libraries providing field options specific to that vendor.
  It is only of interest to `protosearch` developers.

* [`protosearch-plugin`](crates/protosearch-plugin/)

  This crate provides the `protoc` compiler plugin `protoc-gen-protosearch` that compiles messages annotated with `protosearch` field options into document mappings.

### `protosearch-vendor`

The `protosearch-vendor` binary provides three commands:

* `extract`

  Extract an abstract specification of the vendor's supported mapping types.
* `compile`

  Compile the abstract specification into a representation suitable to render as a protobuf file.
  Map OpenAPI types to protobuf types (e.g. `number` to `double`).
* `render`

  Render the compiled specification as a protobuf file.

### `protosearch-plugin`

The `protoc-gen-protosearch` plugin transforms Protobuf descriptors into document mappings.

The plugin iterates over all top-level messages declared in the input files and looks for fields annotated with `protosearch` options.
If it finds any, it adds them to an internal representation of the document mapping.
Finally, it outputs the document mapping to a file named `{package}.{message}.json`.

#### Validation and intermediate representation

The plugin passes most field values through to the mapping without validation.
It does validate certain fields.
Refer to the [reference documentation](doc/reference.md) for details.

This means it is possible to declare invalid mappings:

```protobuf
string uuid = 1 [(protosearch.mapping).field.type = "long"];
```

## API

Refer to the [reference documentation](doc/reference.md).
