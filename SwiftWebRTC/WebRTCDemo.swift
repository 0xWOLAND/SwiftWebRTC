import Foundation
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

class WebRTCDemo: ObservableObject {
    private let rtc: OpaquePointer
    @Published var messages: [String] = []
    @Published var isConnected = false
    @Published var status = ""
    private var timer: Timer?
    
    @Published var copiedOffer = false
    @Published var copiedAnswer = false
    
    init() {
        self.rtc = webrtc_new()
    }
    
    func createOffer() {
        status = "Creating offer..."
        webrtc_create_offer(rtc)
        
        if let sdpPtr = webrtc_get_local_description(rtc), 
           let sdp = String(validatingUTF8: sdpPtr) {
            free_string(sdpPtr)
            #if canImport(UIKit)
            UIPasteboard.general.string = "OFFER:\n\(sdp)"
            #elseif canImport(AppKit)
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString("OFFER:\n\(sdp)", forType: .string)
            #endif
            copiedOffer = true
            status = "Offer copied to clipboard!"
        }
    }
    
    func processClipboardOffer() {
        let clipboard: String?
        #if canImport(UIKit)
        clipboard = UIPasteboard.general.string
        #elseif canImport(AppKit)
        clipboard = NSPasteboard.general.string(forType: .string)
        #endif
        
        guard let clipboard = clipboard,
              clipboard.hasPrefix("OFFER:") else {
            status = "No offer in clipboard"
            return
        }
        
        let offer = String(clipboard.dropFirst(7)) // Remove "OFFER:\n"
        status = "Creating answer..."
        webrtc_set_remote_description(self.rtc, offer)
        webrtc_create_answer(self.rtc)
        
        if let sdpPtr = webrtc_get_local_description(self.rtc),
           let answer = String(validatingUTF8: sdpPtr) {
            free_string(sdpPtr)
            #if canImport(UIKit)
            UIPasteboard.general.string = "ANSWER:\n\(answer)"
            #elseif canImport(AppKit)
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString("ANSWER:\n\(answer)", forType: .string)
            #endif
            copiedAnswer = true
            status = "Answer copied! Waiting for connection..."
            // Don't set isConnected here - wait for ICE to complete
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                self.isConnected = true
                self.status = "Connected!"
                self.sendTestMessages()
            }
        }
    }
    
    func processClipboardAnswer() {
        let clipboard: String?
        #if canImport(UIKit)
        clipboard = UIPasteboard.general.string
        #elseif canImport(AppKit)
        clipboard = NSPasteboard.general.string(forType: .string)
        #endif
        
        guard let clipboard = clipboard,
              clipboard.hasPrefix("ANSWER:") else {
            status = "No answer in clipboard"
            return
        }
        
        let answer = String(clipboard.dropFirst(8)) // Remove "ANSWER:\n"
        status = "Connected!"
        webrtc_set_remote_description(self.rtc, answer)
        self.isConnected = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            self.sendTestMessages()
        }
    }
    
    func sendMessage(_ message: String) {
        webrtc_send_message(rtc, message)
    }
    
    func refreshMessages() {
        var count: size_t = 0
        guard let messagesPtr = webrtc_get_messages(rtc, &count), count > 0 else { return }
        
        var newMessages: [String] = []
        for i in 0..<count {
            if let msgPtr = messagesPtr.advanced(by: i).pointee {
                newMessages.append(String(cString: msgPtr))
            }
        }
        
        webrtc_free_messages(messagesPtr, count)
        messages = newMessages
    }
    
    
    private func sendTestMessages() {
        // Wait longer for data channel to fully open
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            for i in 1...3 {
                let msg = "Message \(i) from \(self.copiedOffer ? "offerer" : "answerer")"
                self.sendMessage(msg)
                Thread.sleep(forTimeInterval: 0.5)
            }
            
            // Refresh messages after a delay
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                self.refreshMessages()
            }
        }
    }
    
    deinit {
        timer?.invalidate()
        webrtc_destroy(rtc)
    }
}