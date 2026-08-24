#include <stdio.h>
#include <string.h>

#ifdef _WIN32
  #include <winsock2.h>
  #include <ws2tcpip.h>
  typedef SOCKET socket_t;
  #define CLOSE_SOCKET(s) closesocket(s)
#else
  #include <sys/socket.h>
  #include <arpa/inet.h>
  #include <unistd.h>
  typedef int socket_t;
  #define CLOSE_SOCKET(s) close(s)
#endif

typedef struct {
    socket_t sock_fd;
    char host[128];
    int port;
} NetClient;

void client_init(NetClient *client, const char *host, int port) {
    if (!client) return;
    client->sock_fd = -1;
    strncpy(client->host, host ? host : "127.0.0.1", sizeof(client->host) - 1);
    client->port = port;
}

void client_close(NetClient *client) {
    if (client && client->sock_fd != -1) {
        CLOSE_SOCKET(client->sock_fd);
        client->sock_fd = -1;
    }
}
