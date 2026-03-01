# protosearch reference

This document describes the complete `protosearch` API.

## API

`protosearch` exposes a single field extension, `protosearch.field`.

This extension is a protobuf message (`protosearch.Field`) that wraps the extension options.

|Field|Type|Description|
|---|---|---|
|`name`|`string`|Rename a field in the mapping.|
|`mapping`|`protosearch.FieldMapping`|Define mapping field parameters.|
|`target`|`repeated protosearch.Target`|Configure a literal mapping for a specific target.|

The `protoc-gen-protosearch` plugin compiles these message options to a JSON file containing the document mapping.

The simplest way to annotate a field is:

```protobuf
string uid = 1 [(protosearch.field) = {}];
```

This will generate a basic field mapping with no parameters except for `type`. See [type inference](#type-inference) below.

If you do not annotate a protobuf field with `(protosearch.field)` options, it will be excluded from the mapping.

### `name`

The `name` field lets you rename a protobuf field in the compiled mapping.

```
string uid = 1 [(protosearch.field).name = "user_uid"];
```

```json
{
  "properties": {
    "user_uid": {
      "type": "keyword"
    }
  }
}
```

### `mapping`

In most cases, you will need to use `mapping` to define field parameters.
`FieldMapping` supports the [most common mapping parameters](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/mapping-parameters) with one important difference:

* It does not support `properties`, because the plugin supports defining `object` and `nested` fields as protobuf message fields.

Certain fields, namely `dynamic`, `index_options`, and `term_vector`, are enums.
All provide a default `UNSPECIFIED` value.
The plugin will not output an enum parameter if it has the default `UNSPECIFIED` value.

If you need to generate a parameter that is not in this list, see [`target`](#target) below.

|Field|Type|Description|
|---|---|---|
|[`type`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/type)|`string`|The field type. If omitted, the plugin infers the type from the protobuf field type.|
|[`analyzer`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/analyzer)|`string`|Analyzer used at index time. Applies to `text` fields.|
|[`boost`](https://docs.opensearch.org/latest/mappings/mapping-parameters/boost/)|`double`|Boost a field's score at index time.|
|[`coerce`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/coerce)|`bool`|Whether to coerce values to the declared mapping type. Applies to numeric and date fields.|
|[`copy_to`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/copy-to)|`repeated string`|Copy this field's value to the named field.|
|[`doc_values`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/doc-values)|`bool`|Whether to store doc values for sorting and aggregation.|
|[`dynamic`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/dynamic)|`protosearch.Dynamic`|How to handle unknown subfields. Applies to `object` fields.|
|[`eager_global_ordinals`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/eager-global-ordinals)|`bool`|Whether to load global ordinals at refresh time.|
|[`enabled`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/enabled)|`bool`|Whether to parse and index the field.|
|[`fielddata`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/fielddata)|`bool`|Whether to use in-memory fielddata for sorting and aggregations. Applies to `text` fields.|
|[`fields`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/fields)|`map<string, FieldMapping>`|A multi-field mapping.|
|[`format`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/format)|`string`|The date format. Applies to `date` and `date_nanos` fields.|
|[`ignore_above`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/ignore-above)|`int32`|Do not index strings longer than this length. Applies to `keyword` fields.|
|[`ignore_malformed`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/ignore-malformed)|`bool`|Ignore invalid values instead of rejecting the document.|
|[`index_options`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/index-options)|`protosearch.IndexOptions`|Which information to store in the index. Applies to `text` fields.|
|[`index_phrases`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/index-phrases)|`bool`|Whether to index bigrams separately. Applies to `text` fields.|
|[`index_prefixes`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/index-prefixes)|`protosearch.IndexPrefixes`|Index term prefixes to speed up prefix queries. Applies to `text` fields.|
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
|[`term_vector`](https://www.elastic.co/docs/reference/elasticsearch/mapping-reference/term-vector)|`protosearch.TermVector`|Whether to store term vectors.|

#### `dynamic`

`protosearch.Dynamic` is an enum with the following values:

* `DYNAMIC_TRUE`
* `DYNAMIC_FALSE`
* `DYNAMIC_STRICT`
* `DYNAMIC_RUNTIME`

#### `index_options`

`protosearch.IndexOptions` is an enum with the following values:

* `INDEX_OPTIONS_DOCS`
* `INDEX_OPTIONS_FREQS`
* `INDEX_OPTIONS_POSITIONS`
* `INDEX_OPTIONS_OFFSETS`

#### `index_prefixes`

`protosearch.IndexPrefixes` is a message with the following fields:

|Field|Type|Description|
|---|---|---|
|`min_chars`|`int32`|Minimum prefix length to index.|
|`max_chars`|`int32`|Maximum prefix length to index.|

#### `term_vector`

`protosearch.TermVector` is an enum with the following values:

* `TERM_VECTOR_NO`
* `TERM_VECTOR_YES`
* `TERM_VECTOR_WITH_POSITIONS`
* `TERM_VECTOR_WITH_OFFSETS`
* `TERM_VECTOR_WITH_POSITIONS_OFFSETS`
* `TERM_VECTOR_WITH_POSITIONS_PAYLOADS`
* `TERM_VECTOR_WITH_POSITIONS_OFFSETS_PAYLOADS`

### `target`

The `target` field gives you complete control over how a protobuf field compiles to a mapping property.

It is a message with the following fields:

|Field|Type|Description|
|---|---|---|
|`label`|`string`|A human-readable label used to target that particular mapping with `--protosearch_opt=target=<label>`.|
|`json`|`string`|A literal JSON string containing the mapping.|

Use this to define more complex mapping types, or specify parameters that are not supported in `FieldMapping`.
You can also use this to define mappings for different clusters or vendors.
You can specify this field more than once.

For example, you might want to represent a `Point` object as a `geo_point` in Elasticsearch and an `xy_point` in OpenSearch.
You can create targets for both mappings:

```protobuf
Point origin = 1 [(protosearch.field) = {
  target: {
    label: "elasticsearch"
    json: '{"type": "point"}'
  }
  target: {
    label: "opensearch"
    json: '{"type": "xy_point"}'
  }
}];
```

With `--protosearch_opt=target=elasticsearch`:

```json
{
  "properties": {
    "origin": {
      "type": "point"
    }
  }
}
```

With `--protosearch_opt=target=opensearch`:

```json
{
  "properties": {
    "origin": {
      "type": "xy_point"
    }
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
|enum|`keyword`|

## Diagnostics

The plugin validates some field options and collects diagnostics during compilation.
Errors (`EXXX`) are fatal; `protoc` will exit with an error code and will not produce any output.
The plugin prints warnings (`WXXX`) to standard output.

### Errors

#### E001

The specified value is invalid for this parameter. The plugin will report the reason.

#### E002

`target.json` is not valid JSON.

#### E003

`target.json` is not a JSON object.

### Warnings

#### W001

`name` is invalid.

Names must match the pattern `[@a-z][a-z0-9_]*(\.[a-z0-9_]+)*`.
These are all allowed names:

```
@timestamp
foo
foo_bar
foo.bar.baz
foo_123
```

#### W002

The target `label` does not correspond to a known target.

## `protoc-gen-protosearch`

With `protoc-gen-protosearch` installed on your `$PATH`, you can compile mappings like so:

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. proto/example/article.proto
```

Specify `--protosearch_opt=target=<label>` to compile the mapping for a specific target.

```
protoc -I proto/ --plugin=protoc-gen-protosearch --protosearch_out=. --protosearch_opt=target=<label> proto/example/article.proto
```
