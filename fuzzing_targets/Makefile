export TARGETS_ROOT=$(CURDIR)

define build_pack
	$(eval $@_TARGET = $(1))
	$(eval $@_PACK_ONLY = $(2))
	if [ "${$@_PACK_ONLY}" != "1" ]; then \
		cd targets/${$@_TARGET} && ./build.sh; \
	fi
	cd targets/${$@_TARGET} && ./pack.sh
endef

.PHONY: all
all: tools targets
targets: apache apache-resp apache-cgi tomcat
tools: tools/monitor_http_param tools/monitor_edge_map tools/monitor_execution_path tools/send_to_server
	$(MAKE) -C tools
apache-resp apache-cgi: apache
	@$(call build_pack, $@, $(PACK_ONLY))
apache tomcat:
	@$(call build_pack, $@, $(PACK_ONLY))
clean:
	rm -rf dist/*
	cd tools && $(MAKE) clean
