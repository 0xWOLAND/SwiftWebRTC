import Foundation

class WebRTCPeerConnection {
    private let pc: OpaquePointer
    
    init?() {
        guard let connection = create_peer_connection() else { return nil }
        self.pc = connection
    }
    
    func createOffer() -> String? {
        guard let offerPtr = create_offer(pc) else { return nil }
        let offer = String(cString: offerPtr)
        free_string(offerPtr)
        return offer
    }
    
    func setRemoteDescription(sdp: String) -> Bool {
        return set_remote_description(pc, sdp)
    }
    
    deinit {
        destroy_peer_connection(pc)
    }
}