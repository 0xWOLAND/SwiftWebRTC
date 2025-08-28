import SwiftUI

struct ContentView: View {
    @State private var n: UInt32 = 0
    @State private var fibValue: UInt32 = 0
    @StateObject private var rtc = WebRTCDemo()
    
    var body: some View {
        VStack(spacing: 20) {
            Text("Fibonacci(\(n)) = \(fibValue)")
            Button("Next") {
                fibValue = fibonacci(n)
                n += 1
            }
            
            Divider()
            
            if !rtc.isConnected {
                VStack(spacing: 10) {
                    Text("Role Selection")
                        .font(.headline)
                    
                    HStack(spacing: 20) {
                        // Offerer flow
                        VStack {
                            Button("Create Offer") { 
                                rtc.createOffer() 
                            }
                            .buttonStyle(.borderedProminent)
                            
                            if rtc.copiedOffer {
                                Text("✓ Copied")
                                    .font(.caption)
                                    .foregroundColor(.green)
                                
                                Button("Process Answer") {
                                    rtc.processClipboardAnswer()
                                }
                                .buttonStyle(.bordered)
                            }
                        }
                        
                        Divider()
                            .frame(height: 50)
                        
                        // Answerer flow
                        VStack {
                            Button("Process Offer") {
                                rtc.processClipboardOffer()
                            }
                            .buttonStyle(.borderedProminent)
                            .disabled(rtc.copiedAnswer)
                            
                            if rtc.copiedAnswer {
                                Text("✓ Answer Copied")
                                    .font(.caption)
                                    .foregroundColor(.green)
                                Text("Give to Offerer")
                                    .font(.caption)
                                    .foregroundColor(.blue)
                            }
                        }
                    }
                    
                    Text(rtc.status)
                        .font(.caption)
                        .foregroundColor(.gray)
                        .padding(.top)
                }
            } else {
                Text("Connected!")
                    .foregroundColor(.green)
                HStack {
                    Button("Send") { rtc.sendMessage("Hi \(Date().timeIntervalSince1970)") }
                    Button("Refresh") { rtc.refreshMessages() }
                }
                ForEach(rtc.messages, id: \.self) { msg in
                    Text("• \(msg)")
                        .font(.caption)
                }
            }
        }
        .padding()
    }
}

#Preview {
    ContentView()
}