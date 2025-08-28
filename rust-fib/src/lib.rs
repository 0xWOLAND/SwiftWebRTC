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
pub extern "C" fn create_peer_connection() -> *mut PeerConnection {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let pc = rt.block_on(async {
        let api = APIBuilder::new().build();
        let config = RTCConfiguration::default();
        api.new_peer_connection(config).await.ok()
    });
    
    match pc {
        Some(conn) => Box::into_raw(Box::new(PeerConnection { conn, rt })),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn create_offer(pc: *mut PeerConnection) -> *mut c_char {
    if pc.is_null() { return std::ptr::null_mut(); }
    
    let pc = unsafe { &mut *pc };
    let offer = pc.rt.block_on(async {
        pc.conn.create_offer(None).await.ok()
    });
    
    match offer {
        Some(offer) => CString::new(offer.sdp).unwrap().into_raw(),
        None => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn set_remote_description(pc: *mut PeerConnection, sdp: *const c_char, is_offer: bool) -> bool {
    if pc.is_null() || sdp.is_null() { return false; }
    
    let pc = unsafe { &mut *pc };
    let sdp = unsafe { CStr::from_ptr(sdp).to_str().unwrap_or("") };
    
    use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
    let desc = if is_offer {
        RTCSessionDescription::offer(sdp.to_string()).ok()
    } else {
        RTCSessionDescription::answer(sdp.to_string()).ok()
    };
    
    match desc {
        Some(desc) => pc.rt.block_on(async {
            pc.conn.set_remote_description(desc).await.is_ok()
        }),
        None => false,
    }
}

#[no_mangle]
pub extern "C" fn destroy_peer_connection(pc: *mut PeerConnection) {
    if !pc.is_null() {
        unsafe { let _ = Box::from_raw(pc); }
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}

pub struct PeerConnection {
    conn: webrtc::peer_connection::RTCPeerConnection,
    rt: tokio::runtime::Runtime,
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
        
        let pc = self.rt.block_on(async {
            let api = APIBuilder::new().build();
            let mut config = RTCConfiguration::default();
            config.ice_servers = vec![RTCIceServer {
                urls: vec!["stun:stun.l.google.com:19302".to_string()],
                ..Default::default()
            }];
            Arc::new(api.new_peer_connection(config).await.unwrap())
        });
        
        // Create data channel first
        let dc = self.rt.block_on(async {
            pc.create_data_channel("chat", None).await.unwrap()
        });
        
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
            
            let pc = self.rt.block_on(async {
                let api = APIBuilder::new().build();
                let mut config = RTCConfiguration::default();
                config.ice_servers = vec![RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_string()],
                    ..Default::default()
                }];
                Arc::new(api.new_peer_connection(config).await.unwrap())
            });
            
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
            let desc = if self.is_offerer {
                RTCSessionDescription::answer(sdp.to_string()).unwrap()
            } else {
                RTCSessionDescription::offer(sdp.to_string()).unwrap()
            };
            
            self.rt.block_on(async {
                pc.set_remote_description(desc).await.unwrap();
            });
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