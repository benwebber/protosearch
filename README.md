# protosearch

Compile Protobuf messages to Elasticsearch/OpenSearch document mappings.

`protosearch` provides field options to map message fields to [mapping field types](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/field-data-types).

Annotate your Protobuf messages like this:

```protobuf
import "protosearch/protosearch.proto";

message Article {
  string uid = 1 [(protosearch.field).type = "keyword"];
  string title = 2 [(protosearch.field) = {
    type: "text"
    fields: {
      key: "en"
      value: {
        type: "text"
        analyzer: "english"
      }
    }
  }];
  repeated Author authors = 3 [(protosearch.field) = {
    output: {name: "author"}
    type: "nested"
  }];
}

message Author {
  string uid = 1 [(protosearch.field).type = "keyword"];
  string name = 2 [(protosearch.field).type = "text"];
}
```

You can then compile a document mapping using `protoc-gen-protosearch`:

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example/article.proto
```


```javascript
// example.Article.json
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

## Usage

1. Install `protoc-gen-protosearch` to your `$PATH`.
2. Copy [`protosearch/protosearch.proto`](proto/protosearch/protosearch.proto) to your Protobuf path.
3. Annotate your messages. (Refer to examples.)
4. Compile a Protobuf file to mappings. The plugin will produce one JSON file for each message type.

    ```
    protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example/example.proto
    ```

## Type inference

If `type` is not specified, `protoc-gen-protosearch` will infer a field type from the protobufs type.

|Protobuf|Elasticsearch|
|---|---|
|`string`|`keyword`|
|`bool`|`boolean`|
|`int32`, `sint32`, `sfixed32`|`integer`|
|`uint32`,`fixed32`|`long`|
|`int64`, `sint64`, `sfixed64`|`long`|
|`uint64`,`fixed64`|`unsigned_long`|
|`float`|`float`|
|`double`|`double`|
|`bytes`|`binary`|
|message|`object`|
