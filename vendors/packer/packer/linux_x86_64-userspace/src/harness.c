#include <sys/mman.h>
#include <dlfcn.h>
#include <stdint.h>
#include <sys/types.h>
#include <unistd.h>
#include <string.h>
#include <sys/uio.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <sys/wait.h>
#include <inttypes.h>
#include <sys/syscall.h>
#include <sys/resource.h>
#include <time.h>
#include <link.h>
#include <stdbool.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <sys/shm.h>

#include "nyx.h"
#include "misc/crash_handler.h"
#include "misc/harness_state.h"
#include "netfuzz/syscalls.h"
#include "hdhunter_api.h"

#define MAX_RETRY_TIME 60
#define RECV_BUFFER_SIZE 65536
#define min(a, b) ((a) < (b) ? (a) : (b))

#define HTTP_RESPONSE "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 30\r\n\r\n<html><body>Pong</body></html>"
#define SCGI_RESPONSE "Status: 200 OK\r\nContent-Type: text/html\r\nContent-Length: 30\r\n\r\n<html><body>Pong</body></html>"
#define FASTCGI_RESPONSE "\x01\x06\x00\x01\x00]\x00\x00Status: 200 OK\r\nContent-Type: text/html\r\nContent-Length: 30\r\n\r\n<html><body>Pong</body></html>\x01\x06\x00\x01\x00\x00\x00\x00\x01\x03\x00\x01\x00\x08\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"
#define UWSGI_RESPONSE "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 30\r\n\r\n<html><body>Pong</body></html>"
#define AJP_RESPONSE "\x41\x42\x00\x3b\x04\x00\xc8\x00\x02\x4f\x4b\x00\x00\x02\x00\x0c\x43\x6f\x6e\x74\x65\x6e\x74\x2d\x54\x79\x70\x65\x00\x00\x09\x74\x65\x78\x74\x2f\x68\x74\x6d\x6c\x00\x00\x0e\x43\x6f\x6e\x74\x65\x6e\x74\x2d\x4c\x65\x6e\x67\x74\x68\x00\x00\x02\x33\x30\x00\x41\x42\x00\x22\x03\x00\x1e\x3c\x68\x74\x6d\x6c\x3e\x3c\x62\x6f\x64\x79\x3e\x50\x6f\x6e\x67\x3c\x2f\x62\x6f\x64\x79\x3e\x3c\x2f\x68\x74\x6d\x6c\x3e\x00\x41\x42\x00\x02\x05\x00"

size_t input_buffer_size = 0;
void* trace_buffer = NULL;
int trace_buffer_size = 0;
int port = 0;
struct sockaddr_in servaddr;
struct sockaddr_in upstaddr;
http_param_t* __http_param = NULL;
uint64_t* execution_path = NULL;
struct timeval timeout;
unsigned short upstream_server_port = 59243;
kAFL_payload* payload_buffer = NULL;
char *check_payload = NULL;
size_t check_payload_length = 0;
int mode = HDHUNTER_MODE_REQUEST;

void set_status(short status);
void capabilites_configuration(bool timeout_detection, bool agent_tracing);
void start_upstream_server();
void start_target();
int connect_target();
void get_payload(kAFL_payload* payload_buffer);
void extract_response_info(int sockfd, char *buffer, char *line_buffer);

void set_status(short status) {
    for (int i = 0; i < 10; ++i) {
        if (__http_param->status[i] == 0) {
            __http_param->status[i] = status;
            break;
        }
    }
}

void set_order(int order) {
    for (int i = 0; i < 10; ++i) {
        if (__http_param->order[i] == 0) {
            __http_param->order[i] = order;
            break;
        }
    }
}

void capabilites_configuration(bool timeout_detection, bool agent_tracing) {
    static bool done = false;

    if(!done){
        init_syscall_fptr();

        hprintf("[capablities] agent_tracing: %d\n", agent_tracing);

        host_config_t host_config;
        kAFL_hypercall(HYPERCALL_KAFL_GET_HOST_CONFIG, (uintptr_t)&host_config);

        if(host_config.host_magic != NYX_HOST_MAGIC){
            habort("Error: NYX_HOST_MAGIC not found in host configuration - You are probably using an outdated version of QEMU-Nyx...");
        }

        if(host_config.host_version != NYX_HOST_VERSION){ 
            habort("Error: NYX_HOST_VERSION not found in host configuration - You are probably using an outdated version of QEMU-Nyx...");
        }

        hprintf("[capablities] host_config.bitmap_size: 0x%"PRIx64"\n", host_config.bitmap_size);
        hprintf("[capablities] host_config.payload_buffer_size: 0x%"PRIx64"\n", host_config.payload_buffer_size);

        input_buffer_size = host_config.payload_buffer_size;

        agent_config_t agent_config = {0};

        agent_config.agent_magic = NYX_AGENT_MAGIC;
        agent_config.agent_version = NYX_AGENT_VERSION;
        agent_config.agent_timeout_detection = (uint8_t)timeout_detection;
        agent_config.agent_tracing = (uint8_t)agent_tracing;

        agent_config.coverage_bitmap_size = host_config.bitmap_size;
        trace_buffer_size = host_config.bitmap_size;

        /* Create trace_buffer with shared memory */
        int shmid = shmget(0x1337, host_config.bitmap_size, IPC_CREAT | 0666);
        if (shmid == -1) {
            habort("Error: Failed to create shared memory segment...");
        }
        hprintf("[capablities] trace_buffer shmid: %d\n", shmid);
        char buffer[20] = {0};
        sprintf(buffer, "%d", shmid);
        setenv("__AFL_SHM", buffer, 1);
        sprintf(buffer, "%d", host_config.bitmap_size);
        setenv("__AFL_SHM_SIZE", buffer, 1);
        trace_buffer = shmat(shmid, NULL, 0);
        hprintf("[capablities] trace_buffer: %p\n", trace_buffer);
        memset(trace_buffer, 0xff, agent_config.coverage_bitmap_size);

        agent_config.trace_buffer_vaddr = (uintptr_t)trace_buffer;
        
        /* Create http_param with shared memory */
        shmid = shmget(0x1338, 0x1000, IPC_CREAT | 0666);
        if (shmid == -1) {
            habort("Error: Failed to create shared memory segment...");
        }
        hprintf("[capablities] http_param shmid: %d\n", shmid);
        sprintf(buffer, "%d", shmid);
        setenv("__HTTP_PARAM", buffer, 1);
        sprintf(buffer, "%d", 0x1000);
        setenv("__HTTP_PARAM_SIZE", buffer, 1);
        __http_param = shmat(shmid, NULL, 0);
        hprintf("[capablities] __http_param: %p\n", __http_param);
        memset(__http_param, 0xff, 0x1000);
        
        agent_config.http_param_vaddr = (uintptr_t)__http_param;

        /* Create execution_path with shared memory */
        shmid = shmget(0x1339, 0x1000, IPC_CREAT | 0666);
        if (shmid == -1) {
            habort("Error: Failed to create shared memory segment...");
        }
        hprintf("[capablities] execution_path shmid: %d\n", shmid);
        sprintf(buffer, "%d", shmid);
        setenv("__EXECUTION_PATH", buffer, 1);
        sprintf(buffer, "%d", 0x1000);
        setenv("__EXECUTION_PATH_SIZE", buffer, 1);
        execution_path = shmat(shmid, NULL, 0);
        hprintf("[capablities] execution_path: %p\n", execution_path);
        memset(execution_path, 0xff, 0x1000);
        
        agent_config.execution_path_vaddr = (uintptr_t)execution_path;

        agent_config.agent_non_reload_mode = get_harness_state()->fast_exit_mode;

        kAFL_hypercall(HYPERCALL_KAFL_SET_AGENT_CONFIG, (uintptr_t)&agent_config);
        
        done = true;
    }
}

int extract_desync_id(char *message) {
    char *pos = strstr(message, "X-Desync-Id:");
    if (pos == NULL) {
        pos = strstr(message, "x-desync-id:");
        if (pos == NULL) {
            return -1;
        }
    }
    pos += 12;
    int id;
    sscanf(pos, "%d", &id);
    return id;
}

void start_upstream_server() {
    int pid = fork();
    if (pid == 0) {
        char buffer[RECV_BUFFER_SIZE];
        /* Start HTTP server */
        int sockfd = socket(AF_INET, SOCK_STREAM, 0);
        if (sockfd == -1) {
            habort("[harness][upstream] Error: Failed to create socket");
        }
        if (setsockopt(sockfd, SOL_SOCKET, SO_REUSEADDR, &(int){1}, sizeof(int)) < 0) {
            habort("[harness][upstream] Error: Failed to set socket options");
        }
        memset(&upstaddr, 0, sizeof(upstaddr));
        upstaddr.sin_family = AF_INET;
        upstaddr.sin_addr.s_addr = INADDR_ANY;
        upstaddr.sin_port = htons(upstream_server_port);
        if (bind(sockfd, (struct sockaddr *)&upstaddr, sizeof(upstaddr)) != 0) {
            habort("[harness][upstream] Error: Failed to bind socket");
        }
        if (listen(sockfd, 10) != 0) {
            habort("[harness][upstream] Error: Failed to listen on socket");
        }
        hprintf("[harness][upstream] Upstream server started on port %hu\n", upstream_server_port);
        while (1) {
            int connfd = accept(sockfd, (struct sockaddr *)NULL, NULL);
            if (connfd == -1) {
                habort("[harness][upstream] Error: Failed to accept connection");
            }
            int ret = read(connfd, buffer, RECV_BUFFER_SIZE - 1);
            if (ret == -1) {
                habort("[harness][upstream] Error: Failed to read from socket");
            }
            buffer[ret] = '\0';
            // hprintf("[harness][upstream] DEBUG Received: %s\n", buffer);
            if (mode != HDHUNTER_MODE_REQUEST) {
                if (payload_buffer->size > 0) {
                    write(connfd, payload_buffer->data, payload_buffer->size);
                } else {
                    switch (mode) {
                    case HDHUNTER_MODE_RESPONSE:
                        write(connfd, HTTP_RESPONSE, sizeof(HTTP_RESPONSE) - 1);
                        break;
                    case HDHUNTER_MODE_SCGI:
                        write(connfd, SCGI_RESPONSE, sizeof(SCGI_RESPONSE) - 1);
                        break;
                    case HDHUNTER_MODE_FASTCGI:
                        write(connfd, FASTCGI_RESPONSE, sizeof(FASTCGI_RESPONSE) - 1);
                        break;
                    case HDHUNTER_MODE_UWSGI:
                        write(connfd, UWSGI_RESPONSE, sizeof(UWSGI_RESPONSE) - 1);
                        break;
                    case HDHUNTER_MODE_AJP:
                        write(connfd, AJP_RESPONSE, sizeof(AJP_RESPONSE) - 1);
                        break;
                    }
                }
            } else {
                int id = extract_desync_id(buffer);
                char response[100];
                if (id != -1) {
                    sprintf(response, "HTTP/1.1 200 OK\nX-Desync-Id: %d\nContent-Type: text/html\nContent-Length: 0\n\n", id);
                } else {
                    sprintf(response, "HTTP/1.1 200 OK\nContent-Type: text/html\nContent-Length: 0\n\n");
                }
                write(connfd, response, strlen(response));
            }
            close(connfd);
        }
        exit(-1);
    }
}

void start_target() {
    FILE *fp;
    /* Read testing mode (request/response) */
    fp = fopen("/tmp/target/mode", "r");
    if (fp != NULL) {
        char mode_str[10];
        fscanf(fp, "%s", mode_str);
        if (strcmp(mode_str, "request") != 0) {
            if (strcmp(mode_str, "response") == 0) {
                mode = HDHUNTER_MODE_RESPONSE;
            } else if (strcmp(mode_str, "scgi") == 0) {
                mode = HDHUNTER_MODE_SCGI;
            } else if (strcmp(mode_str, "fastcgi") == 0) {
                mode = HDHUNTER_MODE_FASTCGI;
            } else if (strcmp(mode_str, "uwsgi") == 0) {
                mode = HDHUNTER_MODE_UWSGI;
            } else if (strcmp(mode_str, "ajp") == 0) {
                mode = HDHUNTER_MODE_AJP;
            } else {
                habort("[harness] Error: Invalid mode");
            }
        }
        setenv("HDHUNTER_MODE", mode_str, 1);
        fclose(fp);
    }

    /* Start upstream server */
    start_upstream_server();

    /* Run start script */
    int ret = system("/tmp/target/start.sh");
    if (ret != 0) {
        habort("[harness] Error: Failed to run target startup script");
    }

    /* Read check alive payload */
    check_payload = (char*)malloc(input_buffer_size);
    fp = fopen("/tmp/target/check_payload", "r");
    if (fp == NULL) {
        habort("[harness] Error: Failed to open payload file");
    }
    check_payload_length = fread(check_payload, 1, input_buffer_size, fp);
    fclose(fp);

    /* Read listen port */
    fp = fopen("/tmp/target/port", "r");
    if (fp == NULL) {
        habort("[harness] Error: Failed to open port file");
    }
    fscanf(fp, "%d", &port);
    fclose(fp);

    /* Read timeout */
    fp = fopen("/tmp/target/timeout", "r");
    if (fp == NULL) {
        timeout.tv_sec = 0;
        timeout.tv_usec = 200000;
    } else {
        fscanf(fp, "%ld %ld", &timeout.tv_sec, &timeout.tv_usec);
        fclose(fp);
    }

    /* Check alive */
    bzero(&servaddr, sizeof(servaddr));
    servaddr.sin_family = AF_INET;
    servaddr.sin_addr.s_addr = inet_addr("127.0.0.1");
    servaddr.sin_port = htons(port);

    /* Healthcheck */
    hprintf("[harness] Start healthchecking...\n");
    time_t start = time(NULL);
    char *buffer = (char*)malloc(1024);
    bool success = false;
    while (time(NULL) - start < MAX_RETRY_TIME) {
        int sockfd = socket(AF_INET, SOCK_STREAM, 0);
        if (sockfd == -1) {
            habort("[harness] Error: Failed to create socket");
        }

        if (setsockopt(sockfd, SOL_SOCKET, SO_SNDTIMEO, &timeout, sizeof timeout) < 0) {
            habort("[harness] Error: Failed to set send timeout");
        }

        if (setsockopt(sockfd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof timeout) < 0) {
            habort("[harness] Error: Failed to set recv timeout");
        }

        int ret = connect(sockfd, (struct sockaddr *)&servaddr, sizeof(servaddr));
        if (ret != 0) {
            hprintf("[harness] Healthcheck: Failed to connect to target\n");
            goto cleanup;
        }
        
        ret = write(sockfd, check_payload, input_buffer_size);
        if (ret == -1) {
            hprintf("[harness] Healthcheck: Failed to send payload\n");
            goto cleanup;
        }

        ret = read(sockfd, buffer, 1024);
        if (ret == -1) {
            hprintf("[harness] Healthcheck: Failed to receive response\n");
            goto cleanup;
        }

        if (strstr(buffer, "200") != NULL) {
            success = true;
            close(sockfd);
            break;
        } else {
            hprintf("[harness] DEBUG: %s\n", buffer);
            hprintf("[harness] Healthcheck: Not 200\n");
            goto cleanup;
        }

        cleanup:
        close(sockfd);
        sleep(1);
    }
    
    if (!success) {
        habort("[harness] Error: Failed to receive correct response");
    } else {
        hprintf("[harness] Healthcheck: Done!\n");
        hprintf("[harness] Healthcheck: Sleep for 5s to avoid buffered requests\n");
        sleep(5);
    }
    free(buffer);
}

int connect_target() {
    int sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd == -1) {
        habort("[harness] Error: Failed to create socket");
    }

    if (setsockopt(sockfd, SOL_SOCKET, SO_RCVTIMEO, &timeout, sizeof timeout) < 0) {
        habort("[harness] Error: Failed to set recv timeout");
    }

    int ret = connect(sockfd, (struct sockaddr *)&servaddr, sizeof(servaddr));
    if (ret != 0) {
        habort("[harness] Error: Failed to connect to target");
    }
    return sockfd;
}

void get_payload(kAFL_payload* payload_buffer) {
    if(input_buffer_size == 0) {
        habort("[harness] Error: The size of the input buffer has not been specified by the host...");
    }

    mlock(payload_buffer, (size_t)input_buffer_size);
    memset(payload_buffer, 0, input_buffer_size);

    hprintf("[harness] payload buffer is mapped at %p (size: 0x%lx)\n", payload_buffer, input_buffer_size);

    kAFL_hypercall(HYPERCALL_KAFL_GET_PAYLOAD, (uintptr_t)payload_buffer);
}

void extract_response_info(int sockfd, char *buffer, char *line_buffer) {
    int total = 0;
    int remaining = 0;
    while (1) {
        int bytes_read = recv(sockfd, buffer, RECV_BUFFER_SIZE - 1, 0);
        if (bytes_read <= 0) {
            break;
        }
        total += bytes_read;
        buffer[bytes_read] = '\0';
        hprintf("DEBUG: %s", buffer);
        // hprintf("[harness] DEBUG Response received: %s\n", buffer);
        
        char *pos = buffer;
        // Line remaining from previous buffer
        if (remaining) {
            remaining = 0;
            int length = min(11, bytes_read); // strlen("HTTP/1.1 200") == 12, if we have 11 bytes, we can't have a full status line
            strncat(line_buffer, buffer, length);
            pos += length;
            if (memcmp(line_buffer, "HTTP/1.1 ", 9) == 0) {
                short status;
                sscanf(line_buffer + 9, "%hd", &status);
                set_status(status);
            }
            if (memcmp(line_buffer, "X-Desync-Id:", 12) == 0 || memcmp(line_buffer, "x-desync-id:", 12) == 0) {
                int id;
                sscanf(line_buffer + 12, "%d", &id);
                set_order(id);
            }
        }

        // Line remaining
        if (buffer[bytes_read - 1] != '\n') {
            for (int i = bytes_read - 2; i >= 0 && bytes_read - i <= 12; --i) {
                if (buffer[i] == '\n') {
                    remaining = 1;
                    memcpy(line_buffer, buffer + i + 1, bytes_read - i - 1);
                    line_buffer[bytes_read - i - 1] = '\0';
                    break;
                }
            }
        }

        // Search for status line
        while ((pos = strstr(pos, "HTTP/1.1 ")) != NULL) {
            pos += 9; // strlen("HTTP/1.1 ") == 9
            if (strlen(pos) < 3) { // HTTP status code is at least 3 digits
                break;
            }
            short status;
            sscanf(pos, "%hd", &status);
            set_status(status);
        }

        pos = buffer;
        // Search for X-Desync-Id
        while (true) {
            char *tmp = pos;
            if ((pos = strstr(pos, "X-Desync-Id:")) == NULL) {
                pos = tmp;
                if ((pos = strstr(pos, "x-desync-id:")) == NULL) {
                    break;
                }
            }
            pos += 12; // strlen("X-Desync-Id:") == 12
            if (strlen(pos) < 1) {
                break;
            }
            int id;
            sscanf(pos, "%d", &id);
            set_order(id);
        }
    }
}

int main() {
    hprintf("[harness] Harness started!\n");
    capabilites_configuration(false, true);

    payload_buffer = mmap(NULL, input_buffer_size, PROT_READ | PROT_WRITE, MAP_SHARED | MAP_ANONYMOUS, -1, 0);
    if (payload_buffer == MAP_FAILED) {
        habort("[harness] Error: Failed to create payload buffer mmap");
    }
    memset(payload_buffer, 0, input_buffer_size);

    hprintf("[harness] Starting target...\n");
    start_target();

    get_payload(payload_buffer);
    int sock = connect_target();
    hprintf("[harness] Connected to target\n");
    char *recv_buffer = (char*)malloc(RECV_BUFFER_SIZE);
    memset(recv_buffer, 0, RECV_BUFFER_SIZE);
    char *line_buffer = (char*)malloc(RECV_BUFFER_SIZE);
    memset(line_buffer, 0, RECV_BUFFER_SIZE);

    kAFL_hypercall(HYPERCALL_KAFL_USER_FAST_ACQUIRE, 0);

    // hprintf("[harness] DEBUG Received payload: %s\n", payload_buffer->data);

    if (mode != HDHUNTER_MODE_REQUEST) {
        write(sock, check_payload, check_payload_length);
    } else {
        write(sock, payload_buffer->data, payload_buffer->size);
    }
    extract_response_info(sock, recv_buffer, line_buffer);

    kAFL_hypercall(HYPERCALL_KAFL_RELEASE, 0);
}
