# protosearch

Compile Protobuf messages to Elasticsearch document mappings.

`protosearch` provides field options to map message fields to [mapping field types](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/field-data-types).

Annotate your Protobuf messages like this:

```protobuf
syntax = "proto3";

package example.v1;

import "protosearch/protosearch.proto";
import "protosearch/es/v8/mapping.proto";

message Article {
  string uid = 1 [(protosearch.mapping) = {
    [protosearch.es.v8.keyword]: {},
  }];
  string url = 2 [(protosearch.mapping) = {
    [protosearch.es.v8.text]: {},
  }];
  string title = 3 [(protosearch.mapping) = {
    [protosearch.es.v8.text]: {},
  }];
  repeated Author authors = 4 [(protosearch.mapping) = {
    [protosearch.es.v8.nested]: {},
  }];
  string text = 5 [(protosearch.mapping) = {
    [protosearch.es.v8.text]: {},
  }];
}

message Author {
  string uid = 1 [(protosearch.mapping) = {
    [protosearch.es.v8.keyword]: {},
  }];
  string name = 2 [(protosearch.mapping) = {
    [protosearch.es.v8.text]: {},
  }];
}
```

You can then compile a document mapping using `protoc-gen-protosearch`:

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example/article.proto
```


```javascript
// Article.es.v8.json
{
  "properties": {
    "uid": {
      "type": "keyword"
    },
    "url": {
      "type": "text"
    },
    "title": {
      "type": "text"
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
    },
    "text": {
      "type": "text"
    }
  }
}
```

## Usage

1. Install `protoc-gen-protosearch` to your `$PATH`.
2. Copy [`protosearch/protosearch.proto`](proto/protosearch/protosearch.proto) to your Protobuf path.
3. Copy the appropriate vendor- and version-specific mapping options for to your Protobuf path. You can declare mappings for multiple vendors and versions.

    |Vendor|Version|File|
    |---|---|---|
    |Elasticsearch|8|[`protosearch/es/v8/mapping.proto`](proto/protosearch/es/v8/mapping.proto)|
4. Annotate your messages. (Refer to examples.)
5. Compile a Protobuf file to mappings. The plugin will produce one JSON file for each message type.

    ```
    protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example/example.proto
    ```
