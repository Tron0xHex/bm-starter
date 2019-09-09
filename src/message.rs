#[repr(C, packed)]
pub struct Payload {
    pub address: i32,
    pub value: i32,
}

#[repr(C, packed)]
pub struct Message {
    pub unk_var: i32,
    pub window_handle: i32,
    pub pad: [i32; 2],
    pub in_message_id: i32,
    pub payload: Payload,
}
