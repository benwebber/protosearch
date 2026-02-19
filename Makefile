SPEC  := spec/$(VENDOR).$(VERSION).json

.PHONY: fmt
fmt:
	cargo fmt
	buf format -w

.PHONY: lint
lint:
	cargo clippy
	buf lint

proto/protosearch/es/v8/mapping.proto: spec/elasticsearch.v8.json
	mkdir -p $(dir $@)
	cargo run -- extract $< | cargo run -- compile --tag-offset 100 protosearch.es.v8 | cargo run -- render > $@
