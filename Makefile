SPEC  := spec/$(VENDOR).$(VERSION).json

PROTOSEARCH_GEN = cargo run --bin protosearch-gen

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
	$(PROTOSEARCH_GEN) -- extract $< | $(PROTOSEARCH_GEN) -- compile --number-offset 100 protosearch.es.v8 | $(PROTOSEARCH_GEN) -- render > $@
