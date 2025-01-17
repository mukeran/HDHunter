.PHONY: all
all: monitor_http_param monitor_edge_map monitor_execution_path send_to_server
monitor_http_param: monitor_http_param.c
	$(CC) -o $@ $^
monitor_edge_map: monitor_edge_map.c
	$(CC) -o $@ $^
monitor_execution_path: monitor_execution_path.c
	$(CC) -o $@ $^
send_to_server: send_to_server.c
	$(CC) -o $@ $^
clean:
	rm -f monitor_http_param monitor_edge_map monitor_execution_path send_to_server
