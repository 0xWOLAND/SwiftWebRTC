import Foundation
import SwiftWebRTC
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

class WebRTCDemo: ObservableObject {
    private let client = WebRTCClient()
    @Published var messages: [String] = []
    @Published var isConnected = false
    @Published var status = ""
    
    @Published var copiedOffer = false
    @Published var copiedAnswer = false
    
    init() {
        // Subscribe to client updates
        client.$messages.assign(to: &$messages)
        client.$isConnected.assign(to: &$isConnected)
        client.$status.assign(to: &$status)
    }
    
    func createOffer() {
        status = "Creating offer..."
        if let offer = client.createOffer() {
            #if canImport(UIKit)
            UIPasteboard.general.string = "OFFER:\n\(offer)"
            #elseif canImport(AppKit)
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString("OFFER:\n\(offer)", forType: .string)
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
        
        let offer = String(clipboard.dropFirst(7))
        status = "Creating answer..."
        client.setRemoteDescription(offer)
        
        if let answer = client.createAnswer() {
            #if canImport(UIKit)
            UIPasteboard.general.string = "ANSWER:\n\(answer)"
            #elseif canImport(AppKit)
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString("ANSWER:\n\(answer)", forType: .string)
            #endif
            copiedAnswer = true
            status = "Answer copied! Waiting for connection..."
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
        
        let answer = String(clipboard.dropFirst(8))
        status = "Connected!"
        client.setRemoteDescription(answer)
        self.isConnected = true
        DispatchQueue.main.asyncAfter(deadline: .now() + 3) {
            self.sendTestMessages()
        }
    }
    
    func sendMessage(_ message: String) {
        client.sendMessage(message)
    }
    
    func refreshMessages() {
        messages = client.getMessages()
    }
    
    private func sendTestMessages() {
        DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
            for i in 1...3 {
                let msg = "Message \(i) from \(self.copiedOffer ? "offerer" : "answerer")"
                self.sendMessage(msg)
                Thread.sleep(forTimeInterval: 0.5)
            }
            
            DispatchQueue.main.asyncAfter(deadline: .now() + 2) {
                self.refreshMessages()
            }
        }
    }
}