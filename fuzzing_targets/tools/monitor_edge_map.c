#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <sys/shm.h>

#define EDGE_MAP_SIZE 65536

int main(int argc, char *argv[]) {
    if (argc != 2) {
        printf("Usage: %s [create|show|delete|clear] ...\n", argv[0]);
        return 1;
    }
    if (strcmp(argv[1], "create") == 0) {
        // Create shared memory
        key_t key = 0x1338;
        int shmid = shmget(key, EDGE_MAP_SIZE, IPC_CREAT | 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        printf("shmid: %d\n", shmid);
        printf("size: %d\n", EDGE_MAP_SIZE);
        printf("export __AFL_SHM=%d __AFL_SHM_SIZE=%d\n", shmid, EDGE_MAP_SIZE);
        return 0;
    } else if (strcmp(argv[1], "show") == 0) {
        // Show shared memory
        key_t key = 0x1338;
        int shmid = shmget(key, EDGE_MAP_SIZE, 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        char *edge_map = (char *)shmat(shmid, NULL, 0);
        if (edge_map == (char *)-1) {
            perror("shmat");
            return 1;
        }
        for (int i = 0; i < EDGE_MAP_SIZE; i++) {
            if (edge_map[i] != 0) {
                printf("%d = %d\n", i, edge_map[i]);
            }
        }
        if (shmdt(edge_map) < 0) {
            perror("shmdt");
            return 1;
        }
        return 0;
    } else if (strcmp(argv[1], "delete") == 0) {
        // Delete shared memory
        key_t key = 0x1338;
        int shmid = shmget(key, EDGE_MAP_SIZE, 0666);
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
        key_t key = 0x1338;
        int shmid = shmget(key, EDGE_MAP_SIZE, 0666);
        if (shmid < 0) {
            perror("shmget");
            return 1;
        }
        char *edge_map = (char *)shmat(shmid, NULL, 0);
        if (edge_map == (char *)-1) {
            perror("shmat");
            return 1;
        }
        memset(edge_map, 0, EDGE_MAP_SIZE);
        if (shmdt(edge_map) < 0) {
            perror("shmdt");
            return 1;
        }
        return 0;
    } else {
        printf("Usage: %s [create|show|delete|clear] ...\n", argv[0]);
        return 1;
    }
}