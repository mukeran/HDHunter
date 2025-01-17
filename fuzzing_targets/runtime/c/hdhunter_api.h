#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif

#ifndef HDHUNTER_API
#define HDHUNTER_API

#ifdef __cplusplus
#define EXTERNC extern "C"
#else
#define EXTERNC
#endif

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct http_param {
    // Content-length header value.
    int64_t content_length[10];

    // Chunked: 1. Not chunked: 0.
    int8_t chunked_encoding[10];

    // The number of bytes consumed for one request
    int64_t consumed_length[10];

    // The length of the HTTP body
    int64_t body_length[10];

    // Message count
    int32_t message_count;

    // Message processed flag
    int8_t message_processed;

    // Message status (update by the harness)
    int16_t status[10];

    // Message order (update by the harness)
    int32_t order[10];
} http_param_t;

extern http_param_t *__http_param;
EXTERNC void hdhunter_init();
EXTERNC void hdhunter_set_content_length(int64_t length, int32_t mode);
EXTERNC void hdhunter_set_chunked_encoding(int8_t chunked, int32_t mode);
EXTERNC void hdhunter_inc_consumed_length(int64_t length, int32_t mode);
EXTERNC void hdhunter_inc_body_length(int64_t length, int32_t mode);
EXTERNC void hdhunter_mark_message_processed(int32_t mode);

#define HDHUNTER_MODE_REQUEST  1
#define HDHUNTER_MODE_RESPONSE 2
#define HDHUNTER_MODE_SCGI     4
#define HDHUNTER_MODE_FASTCGI  8
#define HDHUNTER_MODE_AJP      16
#define HDHUNTER_MODE_UWSGI    32

#endif /*HDHUNTER_API */
