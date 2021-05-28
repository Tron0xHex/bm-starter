#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SecuRomRequest {
    Hit,
    Fly,
    Unknown,
}

impl From<u32> for SecuRomRequest {
    fn from(i: u32) -> Self {
        match i {
            51 => SecuRomRequest::Hit,
            100 => SecuRomRequest::Fly,
            _ => SecuRomRequest::Unknown,
        }
    }
}
