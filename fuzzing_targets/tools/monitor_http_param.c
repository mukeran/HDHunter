#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/shm.h>
#include "../runtime/c/hdhunter_api.h"

void clear(http_param_t *http_param) {
    memset(http_param, 0, sizeof(http_param_t));
    for (int i = 0; i < 10; i++) {
        http_param->content_length[i] = -1;
        http_param->chunked_encoding[i] = 0;
        http_param->consumed_length[i] = -1;
        http_param->body_length[i] = -1;
        http_param->status[i] = 0;
    }
}

int main(int argc, char *argv[]) {
    if (argc != 2) {
        printf("Usage: %s [create|show|delete|clear] ...\n", argv[0]);
        return 1;
    }
    if (strcmp(argv[1], "create") == 0) {
        // Create shared memory
        key_t key = 0x1337;
        int shmid = shmget(key, sizeof(http_param_t), IPC_CREAT | 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        printf("shmid: %d\n", shmid);
        printf("size: %ld\n", sizeof(http_param_t));
        printf("export __HTTP_PARAM=%d __HTTP_PARAM_SIZE=%ld\n", shmid, sizeof(http_param_t));
        return 0;
    } else if (strcmp(argv[1], "show") == 0) {
        // Show shared memory
        key_t key = 0x1337;
        int shmid = shmget(key, sizeof(http_param_t), 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        http_param_t *http_param = (http_param_t *)shmat(shmid, NULL, 0);
        if (http_param == (http_param_t *)-1) {
            perror("shmat");
            return 1;
        }
        printf("http_param->content_length: ");
        for (int i = 0; i < 10; i++) {
            printf("%ld ", http_param->content_length[i]);
        }
        printf("\n");
        printf("http_param->chunked_encoding: ");
        for (int i = 0; i < 10; i++) {
            printf("%d ", http_param->chunked_encoding[i]);
        }
        printf("\n");
        printf("http_param->consumed_length: ");
        for (int i = 0; i < 10; i++) {
            printf("%ld ", http_param->consumed_length[i]);
        }
        printf("\n");
        printf("http_param->body_length: ");
        for (int i = 0; i < 10; i++) {
            printf("%ld ", http_param->body_length[i]);
        }
        printf("\n");
        printf("http_param->message_count: %d\n", http_param->message_count);
        printf("http_param->message_processed: %d\n", http_param->message_processed);
        printf("http_param->status: ");
        for (int i = 0; i < 10; i++) {
            printf("%hd ", http_param->status[i]);
        }
        if (shmdt(http_param) < 0) {
            perror("shmdt");
            return 1;
        }
        return 0;
    } else if (strcmp(argv[1], "delete") == 0) {
        // Delete shared memory
        key_t key = 0x1337;
        int shmid = shmget(key, sizeof(http_param_t), 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        if (shmctl(shmid, IPC_RMID, NULL) < 0) {
            perror("shmctl");
            return 1;
        }
        return 0;
    } else if (strcmp(argv[1], "clear") == 0) {
        // Clear shared memory
        key_t key = 0x1337;
        int shmid = shmget(key, sizeof(http_param_t), 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        http_param_t *http_param = (http_param_t *)shmat(shmid, NULL, 0);
        if (http_param == (http_param_t *)-1) {
            perror("shmat");
            return 1;
        }
        clear(http_param);
        if (shmdt(http_param) < 0) {
            perror("shmdt");
            return 1;
        }
        return 0;
    } else {
        printf("Usage: %s [create|show|delete|clear] ...\n", argv[0]);
        return 1;
    }
}