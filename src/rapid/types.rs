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

#[derive(Debug, Default, PartialEq)]
pub struct SdpPackage {
    pub name: String,
    pub md5: [u8; 32],
    pub md5_bin: [u8; 16],
    pub crc32: [u8; 4],
    pub size: u32,
}
