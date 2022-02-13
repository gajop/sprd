#[derive(Debug, PartialEq)]
pub struct Repo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, PartialEq)]
pub struct Sdp {
    pub fullname: String,
    pub something: String, // what's the purpose of this field?
    pub md5: String,
    pub alias: String,
}

#[derive(Debug, Default)]
pub struct SdpPackage {
    pub name: String,
    pub md5: [char; 32],
    pub crc32: [u8; 4],
    pub size: u32,
}
