# protosearch reference

This document describes the complete `protosearch` API.

## API

`protosearch` exposes a single field extension, `protosearch.field`.

This extension is a protobuf message (`protosearch.FieldMapping`).
The `protoc-gen-protosearch` plugin compiles these message options to a JSON file containing the document mapping.

### Basic mappings

In most cases, you can use the top-level extension fields to define fields.

`FieldMapping` supports the [most common mapping parameters](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/mapping-parameters) with three important differences:

* It does not support `index_phrases` and `index_prefixes` because those are specific to the `text` field type.
* It does not support `properties`, because the plugin supports defining `object` and `nested` fields as protobuf message fields.
* It includes a special `output` field that controls how the plugin renders the mapping.

If you do not annotate a protobuf field with `(protosearch.field)` options, it will be excluded from the mapping.

|Field|Type|Description|
|---|---|---|
|[`type`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/type)|`string`|The field type. If omitted, the plugin infers the type from the protobuf field type.|
|[`analyzer`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/analyzer)|`string`|Analyzer used at index time. Applies to `text` fields.|
|[`coerce`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/coerce)|`bool`|Whether to coerce values to the declared mapping type. Applies to numeric and date fields.|
|[`copy_to`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/copy-to)|`string`|Copy this field's value to the named field.|
|[`doc_values`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/doc-values)|`bool`|Whether to store doc values for sorting and aggregation.|
|[`dynamic`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/dynamic)|`string`|How to handle unknown subfields. Applies to `object` fields.|
|[`eager_global_ordinals`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/eager-global-ordinals)|`bool`|Whether to load global ordinals at refresh time.|
|[`enabled`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/enabled)|`bool`|Whether to parse and index the field.|
|[`fielddata`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/fielddata)|[`google.protobuf.Value`](https://protobuf.dev/reference/protobuf/google.protobuf/#value)|Fielddata configuration for in-memory aggregations. Applies to `text` fields.|
|[`fields`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/fields)|`map<string, FieldMapping>`|A multi-field mapping.|
|[`format`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/format)|`string`|The date format. Applies to `date` and `date_nanos` fields.|
|[`ignore_above`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/ignore-above)|`int32`|Do not index strings longer than this length. Applies to `keyword` fields.|
|[`ignore_malformed`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/ignore-malformed)|`bool`|Ignore invalid values instead of rejecting the document.|
|[`index_options`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/index-options)|`string`|Which information to store in the index. Applies to `text` fields.|
|[`index`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/index)|`bool`|Whether to index the field.|
|[`meta`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/meta)|`map<string, string>`|Metadata about the field.|
|[`normalizer`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/normalizer)|`string`|Normalize `keyword` fields with this normalizer.|
|[`norms`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/norms)|`bool`|Whether to store field length norms for scoring.|
|[`null_value`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/null-value)|[`google.protobuf.Value`](https://protobuf.dev/reference/protobuf/google.protobuf/#value)|Replace explicit `null` values with this value at index time.|
|[`position_increment_gap`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/position-increment-gap)|`int32`|A gap inserted between elements in an array to prevent spurious matches. Applies to `text` fields.|
|[`search_analyzer`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/search-analyzer)|`string`|Analyzer used at search time.|
|[`similarity`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/similarity)|`string`|The scoring algorithm.|
|[`store`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/store)|`bool`|Whether to store this field separately from `_source`.|
|[`subobjects`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/subobjects)|`bool`|Whether dotted field names are interpreted as nested subobjects.|
|[`term_vector`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/term-vector)|`string`|Whether to store term vectors.|

### Advanced mappings

The special `output` field gives you complete control over how a protobuf field compiles to a mapping property.

It is a message with the following fields:

|Field|Type|Description|
|---|---|---|
|`name`|`string`|Rename this protobuf field in the mapping.|
|`target`|`repeated protosearch.OutputTarget`|Configure a literal mapping for a specific target.|

#### `target`

If you need to define a more complex mapping type, you can use `output.target` to define the mapping as a JSON string.

`output.target` is a repeated message with the following fields.

|Field|Type|Description|
|---|---|---|
|`label`|`string`|A human-readable label used to target that particular mapping with `--protosearch_opt=target=<label>`.|
|`json`|`string`|A literal JSON string containing the mapping.|

You can also use this to define mappings for different clusters or vendors.
You can specify this field more than once.

For example, you might want to represent a `Point` object as a `geo_point` in Elasticsearch and an `xy_point` in OpenSearch.
You can create targets for both mappings:

```protobuf
Point origin = 1 [(protosearch.field) = {
  output: {
    target: {
      label: "elasticsearch"
      json: '{"type": "point"}'
    }
    target: {
      label: "opensearch"
      json: '{"type": "xy_point"}'
    }
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

## Type inference

If `type` is not specified, `protoc-gen-protosearch` will infer a field type from the protobuf type.

|Protobuf|Elasticsearch|
|---|---|
|`string`|`keyword`|
|`bool`|`boolean`|
|`int32`, `sint32`, `sfixed32`|`integer`|
|`uint32`, `fixed32`|`long`|
|`int64`, `sint64`, `sfixed64`|`long`|
|`uint64`, `fixed64`|`unsigned_long`|
|`float`|`float`|
|`double`|`double`|
|`bytes`|`binary`|
|message|`object`|

## `protoc-gen-protosearch`

With `protoc-gen-protosearch` installed on your `$PATH`, you can compile mappings like so:

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example/article.proto
```

Specify `--protosearch_opt=target=<label>` to compile the mapping for a specific target.

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. --protosearch_opt=target=<label> proto/example/article.proto
```
