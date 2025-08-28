#ifndef RUST_FIB_H
#define RUST_FIB_H

#include <stdint.h>
#include <stdbool.h>

typedef struct PeerConnection PeerConnection;

uint32_t fibonacci(uint32_t n);

PeerConnection* create_peer_connection(void);
char* create_offer(PeerConnection* pc);
bool set_remote_description(PeerConnection* pc, const char* sdp, bool is_offer);
void destroy_peer_connection(PeerConnection* pc);
void free_string(char* s);

#endif