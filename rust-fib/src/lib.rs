use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};
use webrtc::api::APIBuilder;
use webrtc::peer_connection::configuration::RTCConfiguration;

#[no_mangle]
pub extern "C" fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

#[no_mangle]
pub extern "C" fn webrtc_new() -> *mut WebRTC {
    Box::into_raw(Box::new(WebRTC::new()))
}

#[no_mangle]
pub extern "C" fn webrtc_create_offer(rtc: *mut WebRTC) {
    if rtc.is_null() { return; }
    let rtc = unsafe { &mut *rtc };
    rtc.create_offer();
}

#[no_mangle]
pub extern "C" fn webrtc_create_answer(rtc: *mut WebRTC) {
    if rtc.is_null() { return; }
    let rtc = unsafe { &mut *rtc };
    rtc.create_answer();
}

#[no_mangle]
pub extern "C" fn webrtc_set_remote_description(rtc: *mut WebRTC, sdp: *const c_char) {
    if rtc.is_null() || sdp.is_null() { return; }
    let rtc = unsafe { &mut *rtc };
    let sdp_str = unsafe { 
        match CStr::from_ptr(sdp).to_str() {
            Ok(s) => s,
            Err(_) => return,
        }
    };
    if !sdp_str.trim().is_empty() {
        rtc.set_remote_description(sdp_str);
    }
}

#[no_mangle]
pub extern "C" fn webrtc_get_local_description(rtc: *mut WebRTC) -> *mut c_char {
    if rtc.is_null() { return std::ptr::null_mut(); }
    let rtc = unsafe { &*rtc };
    match rtc.get_local_description() {
        Some(desc) => CString::new(desc).unwrap().into_raw(),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn webrtc_send_message(rtc: *mut WebRTC, msg: *const c_char) {
    if rtc.is_null() || msg.is_null() { return; }
    let rtc = unsafe { &*rtc };
    let msg = unsafe { CStr::from_ptr(msg).to_str().unwrap_or("") };
    rtc.send_message(msg);
}

#[no_mangle]
pub extern "C" fn webrtc_get_messages(rtc: *mut WebRTC, count: *mut usize) -> *mut *mut c_char {
    if rtc.is_null() || count.is_null() { return std::ptr::null_mut(); }
    let rtc = unsafe { &*rtc };
    let messages = rtc.get_messages();
    unsafe { *count = messages.len(); }
    
    let mut c_messages: Vec<*mut c_char> = messages
        .into_iter()
        .map(|s| CString::new(s).unwrap().into_raw())
        .collect();
    c_messages.shrink_to_fit();
    let ptr = c_messages.as_mut_ptr();
    std::mem::forget(c_messages);
    ptr
}

#[no_mangle]
pub extern "C" fn webrtc_destroy(rtc: *mut WebRTC) {
    if !rtc.is_null() {
        unsafe { let _ = Box::from_raw(rtc); }
    }
}

#[no_mangle]
pub extern "C" fn webrtc_free_messages(messages: *mut *mut c_char, count: usize) {
    if messages.is_null() { return; }
    unsafe {
        let messages = Vec::from_raw_parts(messages, count, count);
        for msg in messages {
            let _ = CString::from_raw(msg);
        }
    }
}

pub struct WebRTC {
    pc: Option<Arc<webrtc::peer_connection::RTCPeerConnection>>,
    rt: tokio::runtime::Runtime,
    local_desc: Option<String>,
    dc: Arc<Mutex<Option<Arc<webrtc::data_channel::RTCDataChannel>>>>,
    messages: Arc<Mutex<Vec<String>>>,
    is_offerer: bool,
}

impl WebRTC {
    pub fn new() -> Self {
        Self {
            pc: None,
            rt: tokio::runtime::Runtime::new().unwrap(),
            local_desc: None,
            dc: Arc::new(Mutex::new(None)),
            messages: Arc::new(Mutex::new(Vec::new())),
            is_offerer: false,
        }
    }
    
    pub fn create_offer(&mut self) {
        use webrtc::ice_transport::ice_server::RTCIceServer;
        
        self.is_offerer = true;
        let messages = Arc::clone(&self.messages);
        let dc_store = Arc::clone(&self.dc);
        
        let pc = match self.rt.block_on(async {
            let api = APIBuilder::new().build();
            let mut config = RTCConfiguration::default();
            config.ice_servers = vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            }];
            api.new_peer_connection(config).await
        }) {
            Ok(conn) => {
                println!("Peer connection created successfully (offer)");
                Arc::new(conn)
            },
            Err(e) => {
                println!("Failed to create peer connection: {}", e);
                return;
            }
        };
        
        // Create data channel first
        let dc = match self.rt.block_on(async {
            pc.create_data_channel("chat", None).await
        }) {
            Ok(channel) => {
                println!("Data channel created");
                channel
            },
            Err(e) => {
                println!("Failed to create data channel: {}", e);
                return;
            }
        };
        
        let msg_clone = Arc::clone(&messages);
        dc.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
            if let Ok(text) = String::from_utf8(msg.data.to_vec()) {
                println!("Received: {}", text);
                msg_clone.lock().unwrap().push(text);
            }
            Box::pin(async {})
        }));
        
        dc.on_open(Box::new(|| {
            println!("Data channel opened on offer side!");
            Box::pin(async {})
        }));
        
        *dc_store.lock().unwrap() = Some(dc);
        
        // Create and set offer
        let offer = self.rt.block_on(async {
            let offer = pc.create_offer(None).await.unwrap();
            pc.set_local_description(offer.clone()).await.unwrap();
            // Wait for ICE gathering to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            
            // Get updated local description with ICE candidates
            if let Some(local) = pc.local_description().await {
                return local;
            }
            offer
        });
        
        self.local_desc = Some(offer.sdp);
        self.pc = Some(pc);
    }
    
    pub fn create_answer(&mut self) {
        if let Some(ref pc) = self.pc {
            let answer = self.rt.block_on(async {
                let answer = pc.create_answer(None).await.unwrap();
                pc.set_local_description(answer.clone()).await.unwrap();
                // Wait for ICE gathering
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                
                // Get updated local description with ICE candidates
                if let Some(local) = pc.local_description().await {
                    return local;
                }
                answer
            });
            self.local_desc = Some(answer.sdp);
        }
    }
    
    pub fn set_remote_description(&mut self, sdp: &str) {
        use webrtc::ice_transport::ice_server::RTCIceServer;
        use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
        
        if self.pc.is_none() {
            let messages = Arc::clone(&self.messages);
            let dc_store = Arc::clone(&self.dc);
            
            let pc = match self.rt.block_on(async {
                let api = APIBuilder::new().build();
                let mut config = RTCConfiguration::default();
                config.ice_servers = vec![RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    ..Default::default()
                }];
                api.new_peer_connection(config).await
            }) {
                Ok(conn) => {
                    println!("Peer connection created successfully (answer)");
                    Arc::new(conn)
                },
                Err(e) => {
                    println!("Failed to create peer connection: {}", e);
                    return;
                }
            };
            
            pc.on_data_channel(Box::new(move |dc: Arc<webrtc::data_channel::RTCDataChannel>| {
                let msg_clone = Arc::clone(&messages);
                let dc_clone = Arc::clone(&dc);
                let dc_store_clone = Arc::clone(&dc_store);
                
                dc.on_message(Box::new(move |msg: webrtc::data_channel::data_channel_message::DataChannelMessage| {
                    if let Ok(text) = String::from_utf8(msg.data.to_vec()) {
                        println!("Received: {}", text);
                        msg_clone.lock().unwrap().push(text);
                    }
                    Box::pin(async {})
                }));
                
                dc.on_open(Box::new(move || {
                    println!("Data channel opened on answer side!");
                    *dc_store_clone.lock().unwrap() = Some(dc_clone);
                    Box::pin(async {})
                }));
                
                Box::pin(async {})
            }));
            
            self.pc = Some(pc);
        }
        
        if let Some(ref pc) = self.pc {
            let desc = match if self.is_offerer {
                RTCSessionDescription::answer(sdp.to_string())
            } else {
                RTCSessionDescription::offer(sdp.to_string())
            } {
                Ok(d) => d,
                Err(_) => return,
            };
            
            if let Err(e) = self.rt.block_on(async {
                pc.set_remote_description(desc).await
            }) {
                println!("Failed to set remote description: {}", e);
            } else {
                println!("Remote description set successfully");
            }
        }
    }
    
    pub fn get_local_description(&self) -> Option<String> {
        self.local_desc.clone()
    }
    
    pub fn send_message(&self, msg: &str) {
        if let Some(ref dc) = *self.dc.lock().unwrap() {
            let data = msg.as_bytes().to_vec();
            let dc = Arc::clone(dc);
            self.rt.block_on(async move {
                if let Err(e) = dc.send(&bytes::Bytes::from(data)).await {
                    println!("Error sending: {}", e);
                } else {
                    println!("Sent: {}", msg);
                }
            });
        }
    }
    
    pub fn get_messages(&self) -> Vec<String> {
        self.messages.lock().unwrap().clone()
    }
}