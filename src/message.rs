#[repr(C)]
#[derive(Debug, Clone)]
pub struct Payload {
    pub address: u32,
    pub value: u32,
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Message {
    pub unk_var: u32,
    pub window_handle: u32,
    pub pad: [u32; 2],
    pub in_message_id: u32,
    pub payload: Payload,
}
