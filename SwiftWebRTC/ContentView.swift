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
    
    var body: some View {
        VStack {
            Image(systemName: "globe")
                .imageScale(.large)
                .foregroundStyle(.tint)
            Text("Fibonacci(\(n)) = \(fibValue)")
            Button("Next Fibonacci") {
                fibValue = fibonacci(n)
                n += 1
            }
        }
        .padding()
    }
}

#Preview {
    ContentView()
}
