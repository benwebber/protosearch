# Architecture

`protosearch` is a protobuf library and plugin to define document mappings as protobuf message types.

It has three connected components:

1. The [`protosearch`](proto/protosearch/protosearch.proto) library provides the core protobuf field options extension.
2. Vendor- and version-specific libraries such as [`protosearch.es.v8`](proto/protosearch/es/v8/mapping.proto) extend the core field extension with options specific to individual vendors and versions (e.g. Elasticsearch 8).
3. The `protoc-gen-protosearch` `protoc` compiler plugin compiles messages annotated with these options into document mappings.

## Terminology

* **Dialect.** A specific vendor and version combination, such as Elasticsearch 8 (`es.v8`).

## Design

This is a Rust project with two crates:

* [`protosearch-gen`](crates/protosearch-gen/)

  This crate compiles Elasticsearch/OpenSearch OpenAPI specifications into protobuf libraries providing field options specific to that dialect.
  It is only of interest to `protosearch` developers.

* [`protosearch-plugin`](crates/protosearch-plugin/)

  This crate provides the `protoc` compiler plugin `protoc-gen-protosearch` that compiles messages annotated with `protosearch` field options into document mappings.

### `protosearch-gen`

The `protosearch-gen` binary provides three commands:

* `extract`

  Extract an abstract specification of the dialect's supported mapping types.
* `compile`

  Compile the abstract specification into a representation suitable to render as a protobuf file.
  Map OpenAPI types to protobuf types (e.g. `number` to `double`).
* `render`

  Render the compiled specification as a protobuf file.

### `protosearch-plugin`

The `protoc-gen-protosearch` plugin transforms Protobuf descriptors into document mappings.

The plugin iterates over all top-level messages declared in the input files and looks for fields annotated with the `protosearch` dialect options.
If it finds any, it adds them to an internal representation of the document mapping.
Finally, it outputs the document mapping to a file named after the message and dialect (`{message}.{vendor}.{version}.json`).

#### Validation and intermediate representation

The plugin **does not** validate user input.
Internally, it uses a "stringly-typed" intermediate representation to build the mapping.

This means it is possible to declare invalid mappings like so:

```protobuf
string uuid = 1 [(protosearch.mapping) = {
  [protosearch.es.v8.long]: {}
}];
```

Validation is not the plugin's responsibility.
Elasticsearch or OpenSearch would validate the mapping syntax, or reject invalid documents.

This is an area for improvement.
We intend to find a compromise between a strict, completely typed representation of the mapping and the current implementation.

## API

Users use the library by annotating messages with the `protosearch` field options.

First, they must import the `protosearch` library and the appropriate library for their dialect.
It is possible to declare mappings for multiple dialects in the same message, although at present the library only supports Elasticsearch 8.

```protobuf
import "protosearch/protosearch.proto";
import "protosearch/es/v8/mapping.proto";
```

Then they can use the message field options to declare how protobuf field types map to document mapping properties.

The API is designed to balance several concerns:

* Each dialect corresponds to a discrete protobuf package, to facilitate breaking change detection (e.g., with Buf).
* `protosearch` reserves a single, top-level extension for field options (`(protosearch.mapping)`).

  This ensures we do not need to register additional extensions to support new dialects.
* Each dialect package provides discrete field extensions within the top-level `(protosearch.mapping)` extension.

This leads to a verbose, but expressive API.
We intend to simplify the API at some point in the future.

If a user does not annotate a field with a `protosearch` option, `protoc-gen-protosearch` does not include that field in the document mapping.

### Examples

#### Map a protobuf field to a document mapping type

Declare a simple property with no parameters.

```protobuf
string uid = 1 [(protosearch.mapping) = {
  [protosearch.es.v8.keyword]: {}
}];
```

```json
{
  "uid": {
    "type": "keyword"
  }
}
```

#### Rename a field

Change the name of the field in the mapping.

```protobuf
string uid = 1 [(protosearch.mapping) = {
  name: "custom_uid",
  [protosearch.es.v8.keyword]: {}
}];
```

```json
{
  "custom_uid": {
    "type": "keyword"
  }
}
```

#### Set mapping parameters

Set mapping parameters such as analyzers.

```protobuf
string title = 1 [(protosearch.mapping) = {
  [protosearch.es.v8.text]: {
    analyzer: "standard",
    search_analyzer: "simple"
  }
}];
```

```json
{
  "title": {
    "type": "text",
    "analyzer": "standard",
    "search_analyzer": "simple"
  }
}
```

#### Configure dialect-specific mappings

Declare options for different dialects (e.g., if Elasticsearch and OpenSearch provide different field types).

```protobuf
message Drawing {
  Point origin = 1 [(protosearch.mapping) = {
    [protosearch.es.v8.point]: {},
    [protosearch.os.v2.xy_point]: {},
  }]
}

message Point {
  double x = 1;
  double y = 2;
}
```

This produces the following mapping for Elasticsearch 8 (`Drawing.es.v8.json`).

```json
{
  "origin": {
    "type": "point"
  }
}
```

And the following mapping for OpenSearch 2 (`Drawing.os.v2.json`).

```json
{
  "origin": {
    "type": "xy_point"
  }
}
```

#### Declare `object` and `nested` properties

The `object` and `nested` types render nested message types as `object` or `nested` properties, respectively.

```protobuf
message Article {
  Author author = 4 [(protosearch.mapping) = {
    [protosearch.es.v8.object]: {}
  }];
}

message Author {
  string name = 1;
}
```

```json
{
  "author": {
    "type": "object",
    "properties": {
      "name": {
        "type": "text"
      }
    }
  }
}
```

```protobuf
message Article {
  repeated Author authors = 4 [(protosearch.mapping) = {
    [protosearch.es.v8.nested]: {}
  }];
}
```

```json
{
  "authors": {
    "type": "nested",
    "properties": {
      "name": {
        "type": "text"
      }
    }
  }
}
```

#### Declare multi-fields

Declare [multi-fields](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/multi-fields) using an inline [`google.protobuf.Struct`](https://protobuf.dev/reference/protobuf/google.protobuf/#struct).

```protobuf
string title = 1 [(protosearch.mapping) = {
  [protosearch.es.v8.text]: {
    fields: {
      fields: {
        key: "en"
        value: {
          struct_value: {
            fields: {
              key: "type"
              value: { string_value: "text" }
            }
            fields: {
              key: "analyzer"
              value: { string_value: "english" }
            }
          }
        }
      }
      fields: {
        key: "fr"
        value: {
          struct_value: {
            fields: {
              key: "type"
              value: { string_value: "text" }
            }
            fields: {
              key: "analyzer"
              value: { string_value: "french" }
            }
          }
        }
      }
    }
  }
}];
```

```json
{
  "title": {
    "type": "text",
    "fields": {
      "en": {
        "type": "text",
        "analyzer": "english",
      },
      "fr": {
        "type": "text",
        "analyzer": "french",
      }
    }
  }
}
```

## Usage

After configuring their protobufs project, users can invoke `protoc` with the `protoc-gen-protosearch` plugin to generate document mappings.

The plugin will produce one JSON file in the output directory for each top-level message in the file, and for each declared dialect.

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example/example.proto
```
