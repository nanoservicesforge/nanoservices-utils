pub mod bit_codec;
pub mod codec;
pub mod version_codec;


pub enum Codec {
    BincodeCodec,
    VersionCodec,
}
