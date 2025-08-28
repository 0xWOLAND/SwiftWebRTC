# SwiftWebRTC

demo swift app with rust FFI  webrtc data channels.

## Usage

1. Run on two iPhone simulators
2. **Simulator 1**: Tap "Create Offer" -> Copies offer to clipboard
3. **Simulator 2**: Tap "Process Offer" -> Creates and copies answer
4. **Simulator 1**: Tap "Process Answer" -> Connected!

Uses system clipboard for SDP exchange between simulators.