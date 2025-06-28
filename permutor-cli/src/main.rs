use clap::Parser;

use cli::cli_util::log_cli_header;
use codecs::amf::Amf;
use codecs::apple_silicon::Apple;
use codecs::av1_qsv::AV1QSV;
use codecs::get_vendor_for_codec;
use codecs::nvenc::Nvenc;
use codecs::permute::Permute;
use codecs::qsv::QSV;
use codecs::vendor::Vendor;
use engine::permutation_engine::PermutationEngine;
use permutation::permutation::Permutation;

use crate::permutor_cli::PermutorCli;

mod permutor_cli;

enum Codecs {
    AMF(codecs::amf::Amf),
    APPLE(codecs::apple_silicon::Apple),
    AV1QSV(codecs::av1_qsv::AV1QSV),
    NVENC(codecs::nvenc::Nvenc),
    QSV(codecs::qsv::QSV)
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

fn main() {
    log_cli_header(String::from("Permutation Tool"));
    let mut cli = PermutorCli::parse();
    cli.validate();

    log_special_arguments(&cli);

    let mut engine = PermutationEngine::new(cli.log_output_directory.clone());
    let vendor = get_vendor_for_codec(&cli.encoder.clone());
    
    let mut codec: Codecs;
    match vendor {
        Vendor::Nvidia => {
            let nvenc = Nvenc::new(cli.encoder == "hevc_nvenc", cli.gpu, cli.no_b_frame);
            codec = Codecs::NVENC(nvenc);
            //build_nvenc_setting_permutations(&mut engine, &cli, bitrate);
        }
        Vendor::AMD => {
            let amf = Amf::new(cli.encoder == "hevc_amf", cli.gpu);
            codec = Codecs::AMF(amf);
            //build_amf_setting_permutations(&mut engine, &cli, bitrate);
        }
        Vendor::IntelQSV => {
            if cli.encoder.contains("av1") {
                let intel_av1 = AV1QSV::new();
                codec = Codecs::AV1QSV(intel_av1);
                //build_intel_av1_permutations(&mut engine, &cli, bitrate);
            } else {
                let intel_i_gpu = QSV::new(cli.encoder == "hevc_qsv");
                codec = Codecs::QSV(intel_i_gpu);
                //build_intel_igpu_permutations(&mut engine, &cli, bitrate);
            }
        }
        Vendor::Apple => {
            // this can probably be simplified with a type
            let h264 = cli.encoder.contains("h264");
            let prores = cli.encoder.contains("prores");
            let apple_silicon = Apple::new(h264, prores);
            codec = Codecs::APPLE(apple_silicon);
            //build_apple_silicon_h264_permutations(&mut engine, &cli, bitrate);
        }
        Vendor::Unknown => { panic!("Unknown"); }
    }

    for bitrate in get_bitrate_permutations(cli.bitrate, cli.max_bitrate_permutation.unwrap()) {

        // initialize the permutations each time
        codec.init();

        while let Some((_encoder_index, settings)) = codec.next() {
            let mut permutation = Permutation::new(cli.source_file.clone(), cli.encoder.clone());
            permutation.video_file = cli.source_file.clone();
            permutation.encoder_settings = settings;
            permutation.bitrate = bitrate;
            permutation.check_quality = cli.check_quality;
            permutation.verbose = cli.verbose;
            permutation.detect_overload = cli.detect_overload;
            permutation.allow_duplicates = cli.allow_duplicate_scores;
            permutation.verbose = cli.verbose;
            permutation.ten_bit = cli.ten_bit;
            engine.add(permutation);

            // break out early here to just make 1 permutation
            if cli.test_run {
                break;
            }
        }

        if cli.test_run {
            break;
        }
    }

    engine.run();
}

fn log_special_arguments(cli: &PermutorCli) {
    if cli.has_special_options() {
        println!("\nOptions:");
        if cli.detect_overload {
            println!("  -encoding will stop if overload detected");
        }

        if cli.check_quality {
            println!("  -calculating vmaf score");
        }

        if cli.allow_duplicate_scores {
            println!("  -ignoring whether expected vmaf score will be duplicated");
        }

        if cli.verbose {
            println!("  -verbose enabled");
        }

        if cli.test_run {
            println!("  -test run, will only run 1 permutation");
        }
    }
}

fn get_bitrate_permutations(starting_bitrate: u32, max_bitrate: u32) -> Vec<u32> {
    let interval = 5;
    let mut bitrates = Vec::new();
    for i in 0..(((max_bitrate - starting_bitrate) / interval) + 1) {
        bitrates.push(starting_bitrate + (interval * i));
    }

    return bitrates;
}
