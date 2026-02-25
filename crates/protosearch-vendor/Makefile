SPEC  := spec/$(VENDOR).$(VERSION).json

PROTOSEARCH_GEN = cargo run --bin protosearch-gen

proto/protosearch/elasticsearch/elasticsearch.proto: spec/elasticsearch.v8.json
	mkdir -p $(dir $@)
	$(PROTOSEARCH_GEN) -- extract $< | $(PROTOSEARCH_GEN) -- compile --number-offset 1000 protosearch.elasticsearch | $(PROTOSEARCH_GEN) -- render > $@
