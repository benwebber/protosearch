# protosearch

Compile protobuf messages to Elasticsearch document mappings.

## Example

Imagine you have a protobuf message representing a search document.

```protobuf
message Article {
  message Author {
    optional string uid = 1;
    optional string name = 2;
  }

  optional string uid = 1;
  optional string title = 2;
  repeated Author authors = 3;
}
```

Annotate the message with `protosearch.field` options to map its fields to [mapping field types](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/field-data-types).


```protobuf
import "protosearch/protosearch.proto";

message Article {
  message Author {
    string uid = 1 [(protosearch.field) = {}];
    string name = 2 [(protosearch.field).mapping.type = "text"];
  }

  string uid = 1 [(protosearch.field) = {}];
  string title = 2 [(protosearch.field).mapping = {
    type: "text"
    fields: {
      key: "en"
      value: {
        type: "text"
        analyzer: "english"
      }
    }
  }];
  repeated Author authors = 3 [
    (protosearch.field).name = "author",
    (protosearch.field).mapping.type = "nested",
  ];
}
```

Then use `protoc-gen-protosearch` to compile this to a document mapping:

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/article.proto
```

```json
{
  "properties": {
    "uid": {
      "type": "keyword"
    },
    "title": {
      "type": "text",
      "fields": {
        "en": {
          "type": "text",
          "analyzer": "english"
        }
      }
    },
    "author": {
      "type": "nested",
      "properties": {
        "uid": {
          "type": "keyword"
        },
        "name": {
          "type": "text"
        }
      }
    }
  }
}
```

## Install

1. Download [latest release](https://github.com/benwebber/protosearch/releases) of the plugin for your system, or build from source.
2. Install `protoc-gen-protosearch` to your `$PATH`.
2. Copy [`protosearch/protosearch.proto`](proto/protosearch/protosearch.proto) to your Protobuf path.

## Usage

1. Annotate your messages. (See [reference](doc/reference.md).)
2. Compile a Protobuf file to mappings. The plugin will produce one JSON file for each message type.

    ```
    protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example.proto
    ```
