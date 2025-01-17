#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <netdb.h>

int main(int argc, char *argv[]) {
    if (argc != 4) {
        printf("Usage: %s <ip/domain> <port> <file>\n", argv[0]);
        return 1;
    }
    int sockfd;
    struct sockaddr_in server_addr;
    char buffer[1024];
    struct hostent *server;
    sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd < 0) {
        perror("socket");
        return 1;
    }
    server = gethostbyname(argv[1]);
    if (server == NULL) {
        perror("gethostbyname");
        return 1;
    }
    bzero((char *) &server_addr, sizeof(server_addr));
    server_addr.sin_family = AF_INET;
    bcopy((char *)server->h_addr, (char *)&server_addr.sin_addr.s_addr, server->h_length);
    server_addr.sin_port = htons(atoi(argv[2]));
    if (connect(sockfd, (struct sockaddr *)&server_addr, sizeof(server_addr)) < 0) {
        perror("connect");
        return 1;
    }

    FILE *fp = fopen(argv[3], "rb");
    if (fp == NULL) {
        perror("fopen");
        return 1;
    }
    
    while (1) {
        size_t bytes_read = fread(buffer, 1, sizeof(buffer), fp);
        if (bytes_read == 0) {
            break;
        }
        if (send(sockfd, buffer, bytes_read, 0) < 0) {
            perror("send");
            return 1;
        }
    }

    fclose(fp);

    while (1) {
        int bytes_read = recv(sockfd, buffer, sizeof(buffer), 0);
        if (bytes_read < 0) {
            perror("recv");
            return 1;
        }
        if (bytes_read == 0) {
            break;
        }
        printf("%.*s", bytes_read, buffer);
    }

    close(sockfd);
    return 0;
}