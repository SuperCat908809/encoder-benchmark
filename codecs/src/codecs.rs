
use crate::amf::Amf;
use crate::apple_silicon::Apple;
use crate::av1_qsv::AV1QSV;
use crate::nvenc::Nvenc;
use crate::qsv::QSV;
use crate::permute::Permute;

pub enum Codecs {
    AMF(Amf),
    APPLE(Apple),
    AV1QSV(AV1QSV),
    NVENC(Nvenc),
    QSV(QSV)
}

impl Iterator for Codecs {
    type Item = (usize, String);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Codecs::AMF(amf) => amf.next(),
            Codecs::APPLE(apple) => apple.next(),
            Codecs::AV1QSV(av1_qsv) => av1_qsv.next(),
            Codecs::NVENC(nvenc) => nvenc.next(),
            Codecs::QSV(qsv) => qsv.next()
        }
    }
}

impl Permute for Codecs {
    fn init(&mut self) -> &Vec<String> {
        match self {
            Codecs::AMF(amf) => amf.init(),
            Codecs::APPLE(apple) => apple.init(),
            Codecs::AV1QSV(av1_qsv) => av1_qsv.init(),
            Codecs::NVENC(nvenc) => nvenc.init(),
            Codecs::QSV(qsv) => qsv.init()
        }
    }

    fn run_standard_only(&mut self) -> &Vec<String> {
        match self {
            Codecs::AMF(amf) => amf.run_standard_only(),
            Codecs::APPLE(apple) => apple.run_standard_only(),
            Codecs::AV1QSV(av1_qsv) => av1_qsv.run_standard_only(),
            Codecs::NVENC(nvenc) => nvenc.run_standard_only(),
            Codecs::QSV(qsv) => qsv.run_standard_only()
        }
    }

    fn get_resolution_to_bitrate_map(_fps: u32) -> std::collections::HashMap<String, u32> {
        todo!()
    }
}