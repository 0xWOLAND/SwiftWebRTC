#ifndef RUST_FIB_H
#define RUST_FIB_H

#include <stdint.h>
#include <stdbool.h>

typedef struct WebRTC WebRTC;

// Fibonacci function
uint32_t fibonacci(uint32_t n);

// WebRTC API
WebRTC* webrtc_new(void);
void webrtc_create_offer(WebRTC* rtc);
void webrtc_create_answer(WebRTC* rtc);
void webrtc_set_remote_description(WebRTC* rtc, const char* sdp);
char* webrtc_get_local_description(WebRTC* rtc);
void webrtc_send_message(WebRTC* rtc, const char* msg);
char** webrtc_get_messages(WebRTC* rtc, size_t* count);
void webrtc_destroy(WebRTC* rtc);
void webrtc_free_messages(char** messages, size_t count);

// String management
void free_string(char* s);

#endif