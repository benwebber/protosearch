# protosearch example

This is an example project demonstrating an assortment of `protosearch` features.

The project features two targets to represent compiling mappings for different vendors.
The `pub.v1.Article` and `pub.v1.Author` messages each have a `repeated float` field representing a vector embedding.

The default mapping does not specify a field type, so the plugin will infer the mapping type as `float`.
To generate the default mapping with `protoc`, run:

```
mkdir -p gen/default/
protoc -I proto --plugin=protoc-gen-protosearch --protosearch_opt=target=elasticsearch --protosearch_out=gen/default proto/pub/v1/*.proto
```

The `elasticsearch` target maps this field to a `dense_vector` field.
To generate the Elasticsearch mapping with `protoc`, run:

```
mkdir -p gen/elasticsearch/
protoc -I proto --plugin=protoc-gen-protosearch --protosearch_opt=target=opensearch --protosearch_out=gen/elasticsearch proto/pub/v1/*.proto
```

The `opensearch` target maps this field to a `knn_vector` field.
To generate the OpenSearch mapping with `protoc`, run:

```
mkdir -p gen/opensearch/
protoc -I proto --plugin=protoc-gen-protosearch --protosearch_out=gen/opensearch proto/pub/v1/*.proto
```

This is a [Buf](https://buf.build/) project, so you can also use Buf to generate the same targets:

```
buf generate
```
