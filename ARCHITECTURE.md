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

The plugin **does not** validate user input.
Internally, it uses a "stringly-typed" intermediate representation to build the mapping.

This means it is possible to declare invalid mappings:

```protobuf
string uuid = 1 [(protosearch.field).type = "long"];
```

Validation is not the plugin's responsibility.
Elasticsearch or OpenSearch will validate the mapping syntax and reject invalid documents.

## API

Users annotate messages with the `protosearch` field options.

If a user does not annotate a field with a `protosearch` option, `protoc-gen-protosearch` does not include that field in the document mapping.

### Basic mappings

Import the field options:

```
import "protosearch/protosearch.proto";
```

The `protosearch.field` extension provides a set of common mapping parameters.

```protobuf
string uid = 1 [(protosearch.field).type = "keyword"];
```

Here is a fully annotated field (note that the resulting mapping is non-functional):

```protobuf
string test_basic_field = 1 [(protosearch.field) = {
  type: "text"
  analyzer: "english"
  coerce: true
  copy_to: "copy_field"
  doc_values: true
  dynamic: "strict"
  eager_global_ordinals: true
  enabled: true
  fielddata: { bool_value: true }
  fields: { key: "raw" value: { type: "keyword" } }
  format: "yyyy-MM-dd"
  ignore_above: 256
  ignore_malformed: false
  index_options: "positions"
  index: true
  meta: { key: "unit" value: "ms" }
  normalizer: "lowercase"
  norms: true
  null_value: { string_value: "NULL" }
  position_increment_gap: 100
  properties: { key: "sub" value: { type: "keyword" } }
  search_analyzer: "english"
  similarity: "BM25"
  subobjects: "false"
  store: true
  term_vector: "with_positions"
}];
```

### Output options

The `output` sub-message controls how the plugin emits a field. 
The mapping never includes the contents of `output` itself.

It is part of the same message for convenience and ergonomics.
This also avoids registering more than one extension number.
The name `output` was specifically chosen to read fluently:

```
string name = 1 [(protosearch.field).output.name = "author_name"];
```


#### Rename a field

Use `output.name` to rename the field in the output mapping.

```protobuf
repeated Author authors = 3 [(protosearch.field) = {
  output: {name: "author"}
  type: "nested"
}];
```

```json
{
  "author": {
    "type": "nested",
    "properties": { ... }
  }
}
```

#### Override a field representation

Use `output.target` to provide raw JSON for a specific target label.
Labels are arbitrary strings.
Pass `--protosearch_opt=target=<label>` to select a target at compile time.

Consider the example below:

```protobuf
Point origin = 1 [(protosearch.field) = {
  output: {
    target: {label: "elasticsearch" json: '{"type": "point"}'}
    target: {label: "opensearch" json: '{"type": "xy_point"}'}
  }
}];
```

With `--protosearch_opt=target=elasticsearch`:

```json
{
  "origin": {
    "type": "point"
  }
}
```

With `--protosearch_opt=target=opensearch`:

```json
{
  "origin": {
    "type": "xy_point"
  }
}
```

If `target` does not match an existing label, the plugin falls back on the common mapping parameters.

### `object` and `nested` properties

The plugin automatically compiles message fields to as `object` fields when that field's message type has annotated fields of its own.
Users can set `type` to change the output to a `nested` field.

```protobuf
message Article {
  repeated Author authors = 3 [(protosearch.field) = {
    output: {name: "author"}
    type: "nested"
  }];
}

message Author {
  string name = 1 [(protosearch.field).type = "text"];
}
```

```json
{
  "author": {
    "type": "nested",
    "properties": {
      "name": {
        "type": "text"
      }
    }
  }
}
```

## Usage

After configuring their protobufs project, users can invoke `protoc` with the `protoc-gen-protosearch` plugin to generate document mappings.

The plugin produces one JSON file in the output directory for each top-level message in the file.

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example/example.proto
```

To select a target label:

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. --protosearch_opt=target=es proto/example/example.proto
```
