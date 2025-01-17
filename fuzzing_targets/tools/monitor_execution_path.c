#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
#include <sys/shm.h>

int main(int argc, char *argv[]) {
    if (argc != 2) {
        printf("Usage: %s [create|show|delete|clear] ...\n", argv[0]);
        return 1;
    }
    if (strcmp(argv[1], "create") == 0) {
        // Create shared memory
        key_t key = 0x1441;
        int shmid = shmget(key, sizeof(uint64_t), IPC_CREAT | 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        printf("shmid: %d\n", shmid);
        printf("size: %ld\n", sizeof(uint64_t));
        printf("export __EXECUTION_PATH=%d __EXECUTION_PATH_SIZE=%ld\n", shmid, sizeof(uint64_t));
        return 0;
    } else if (strcmp(argv[1], "show") == 0) {
        // Show shared memory
        key_t key = 0x1441;
        int shmid = shmget(key, sizeof(uint64_t), 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        uint64_t *execution_path = (uint64_t *)shmat(shmid, NULL, 0);
        if (execution_path == (uint64_t *)-1) {
            perror("shmat");
            return 1;
        }
        printf("execution_path: %ld\n", *execution_path);
        if (shmdt(execution_path) < 0) {
            perror("shmdt");
            return 1;
        }
        return 0;
    } else if (strcmp(argv[1], "delete") == 0) {
        // Delete shared memory
        key_t key = 0x1441;
        int shmid = shmget(key, sizeof(uint64_t), 0666);
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
        key_t key = 0x1441;
        int shmid = shmget(key, sizeof(uint64_t), 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        uint64_t *execution_path = (uint64_t *)shmat(shmid, NULL, 0);
        if (execution_path == (uint64_t *)-1) {
            perror("shmat");
            return 1;
        }
        *execution_path = 0;
        if (shmdt(execution_path) < 0) {
            perror("shmdt");
            return 1;
        }
        return 0;
    } else {
        printf("Usage: %s [create|show|delete|clear] ...\n", argv[0]);
        return 1;
    }
}