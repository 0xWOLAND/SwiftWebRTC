import Foundation

// Declare C functions from Rust FFI
@_silgen_name("webrtc_new")
func webrtc_new() -> OpaquePointer

@_silgen_name("webrtc_destroy") 
func webrtc_destroy(_ rtc: OpaquePointer)

@_silgen_name("webrtc_create_offer")
func webrtc_create_offer(_ rtc: OpaquePointer)

@_silgen_name("webrtc_create_answer")
func webrtc_create_answer(_ rtc: OpaquePointer)

@_silgen_name("webrtc_set_remote_description")
func webrtc_set_remote_description(_ rtc: OpaquePointer, _ sdp: UnsafePointer<CChar>)

@_silgen_name("webrtc_get_local_description")
func webrtc_get_local_description(_ rtc: OpaquePointer) -> UnsafeMutablePointer<CChar>?

@_silgen_name("webrtc_send_message")
func webrtc_send_message(_ rtc: OpaquePointer, _ message: UnsafePointer<CChar>)

@_silgen_name("webrtc_get_messages")
func webrtc_get_messages(_ rtc: OpaquePointer, _ count: UnsafeMutablePointer<Int>) -> UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>?

@_silgen_name("webrtc_free_messages")
func webrtc_free_messages(_ messages: UnsafeMutablePointer<UnsafeMutablePointer<CChar>?>, _ count: Int)

@_silgen_name("free_string")
func free_string(_ string: UnsafeMutablePointer<CChar>)

public class WebRTCClient: ObservableObject {
    private let rtc: OpaquePointer
    @Published public var messages: [String] = []
    @Published public var isConnected = false
    @Published public var status = ""
    
    public init() {
        self.rtc = webrtc_new()
    }
    
    public func createOffer() -> String? {
        webrtc_create_offer(rtc)
        
        if let sdpPtr = webrtc_get_local_description(rtc),
           let sdp = String(validatingUTF8: sdpPtr) {
            free_string(sdpPtr)
            return sdp
        }
        return nil
    }
    
    public func setRemoteDescription(_ description: String) {
        webrtc_set_remote_description(rtc, description)
    }
    
    public func createAnswer() -> String? {
        webrtc_create_answer(rtc)
        
        if let sdpPtr = webrtc_get_local_description(rtc),
           let answer = String(validatingUTF8: sdpPtr) {
            free_string(sdpPtr)
            return answer
        }
        return nil
    }
    
    public func sendMessage(_ message: String) {
        webrtc_send_message(rtc, message)
    }
    
    public func getMessages() -> [String] {
        var count: size_t = 0
        guard let messagesPtr = webrtc_get_messages(rtc, &count), count > 0 else { 
            return []
        }
        
        var newMessages: [String] = []
        for i in 0..<count {
            if let msgPtr = messagesPtr.advanced(by: i).pointee {
                newMessages.append(String(cString: msgPtr))
            }
        }
        
        webrtc_free_messages(messagesPtr, count)
        return newMessages
    }
    
    deinit {
        webrtc_destroy(rtc)
    }
}