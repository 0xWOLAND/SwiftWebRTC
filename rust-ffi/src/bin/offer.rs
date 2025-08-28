use rust_ffi::WebRTC;
use std::{fs, thread::sleep, time::Duration};

fn main() {
    let (offer_file, answer_file) = ("/tmp/webrtc_offer.sdp", "/tmp/webrtc_answer.sdp");
    fs::remove_file(offer_file).ok();
    fs::remove_file(answer_file).ok();
    
    let mut rtc = WebRTC::new();
    rtc.create_offer();
    fs::write(offer_file, rtc.get_local_description().unwrap()).unwrap();
    println!("Offer created, waiting for answer...");
    
    while !fs::read_to_string(answer_file).is_ok_and(|s| !s.is_empty()) {
        sleep(Duration::from_millis(100));
    }
    
    rtc.set_remote_description(&fs::read_to_string(answer_file).unwrap());
    println!("Connected! Waiting for data channel...");
    
    sleep(Duration::from_secs(3));
    
    // Send messages
    for i in 1..=3 {
        let msg = format!("Message {} from offer", i);
        rtc.send_message(&msg);
        sleep(Duration::from_millis(500));
    }
    
    // Check received messages
    sleep(Duration::from_secs(2));
    println!("\n=== Messages received at offer ===");
    for msg in rtc.get_messages() {
        println!("  {}", msg);
    }
    
    loop { sleep(Duration::from_secs(1)); }
}