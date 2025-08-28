use rust_fib::WebRTC;
use std::{fs, thread::sleep, time::Duration};

fn main() {
    let (offer_file, answer_file) = ("/tmp/webrtc_offer.sdp", "/tmp/webrtc_answer.sdp");
    println!("Waiting for offer...");
    
    while !fs::read_to_string(offer_file).is_ok_and(|s| !s.is_empty()) {
        sleep(Duration::from_millis(100));
    }
    
    let mut rtc = WebRTC::new();
    rtc.set_remote_description(&fs::read_to_string(offer_file).unwrap());
    rtc.create_answer();
    fs::write(answer_file, rtc.get_local_description().unwrap()).unwrap();
    println!("Connected! Waiting for data channel...");
    
    sleep(Duration::from_secs(4));
    
    // Send messages
    for i in 1..=3 {
        let msg = format!("Message {} from answer", i);
        rtc.send_message(&msg);
        sleep(Duration::from_millis(500));
    }
    
    // Check received messages
    sleep(Duration::from_secs(2));
    println!("\n=== Messages received at answer ===");
    for msg in rtc.get_messages() {
        println!("  {}", msg);
    }
    
    loop { sleep(Duration::from_secs(1)); }
}