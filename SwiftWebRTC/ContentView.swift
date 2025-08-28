//
//  ContentView.swift
//  SwiftWebRTC
//
//  Created by Bhargav Annem on 8/28/25.
//

import SwiftUI

struct ContentView: View {
    @State private var n: UInt32 = 0
    @State private var fibValue: UInt32 = 0
    @State private var sdp: String = "No SDP"
    @State private var pc: WebRTCPeerConnection?
    
    var body: some View {
        VStack {
            Text("Fibonacci(\(n)) = \(fibValue)")
            Button("Next Fibonacci") {
                fibValue = fibonacci(n)
                n += 1
            }
            
            Divider()
            
            Text("WebRTC Demo")
                .font(.headline)
            
            Button("Create Peer Connection") {
                pc = WebRTCPeerConnection()
                sdp = pc != nil ? "PC Created" : "Failed"
            }
            
            Button("Create Offer") {
                if let offer = pc?.createOffer() {
                    sdp = String(offer.prefix(100)) + "..."
                }
            }
            
            Text(sdp)
                .font(.system(.caption, design: .monospaced))
                .frame(maxWidth: 300)
        }
        .padding()
    }
}

#Preview {
    ContentView()
}
