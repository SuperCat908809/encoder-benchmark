use core::panic;

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
use codecs::codecs::Codecs;
use engine::permutation_engine::PermutationEngine;
use ffmpeg::args::FfmpegQuality;
use permutation::permutation::Permutation;

use crate::permutor_cli::PermutorCli;

mod permutor_cli;

fn main() {
    log_cli_header(String::from("Permutation Tool"));
    let mut cli = PermutorCli::parse();
    cli.validate();

    log_special_arguments(&cli);

    let mut engine = PermutationEngine::new(cli.log_output_directory.clone());
    let vendor = get_vendor_for_codec(&cli.encoder.clone());
    
    let using_bitrate = true;

    let mut codec: Codecs;
    match vendor {
        Vendor::Nvidia => {
            let nvenc = Nvenc::new(cli.encoder == "hevc_nvenc", cli.gpu, cli.no_b_frame, using_bitrate);
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

    let min_quality = FfmpegQuality::Bitrate(cli.bitrate);
    let max_quality = FfmpegQuality::Bitrate(cli.max_bitrate_permutation.unwrap());
    for quality in get_quality_permutations(&min_quality, &max_quality) {

        // initialize the permutations each time
        codec.init();

        while let Some((_encoder_index, settings)) = codec.next() {
            let mut permutation = Permutation::new(cli.source_file.clone(), cli.encoder.clone());
            permutation.video_file = cli.source_file.clone();
            permutation.encoder_settings = settings;
            permutation.ffmpeg_quality = quality;
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

fn get_quality_permutations(min_quality: &FfmpegQuality, max_quality: &FfmpegQuality) -> Vec<FfmpegQuality> {
    let mut qualities = Vec::new();

    match min_quality {
        FfmpegQuality::Bitrate(b_min) => {
            let interval = 5;
            let b_max = match max_quality { FfmpegQuality::Bitrate(b) => b, _ => panic!("max_bitrate doesn't match FfmpegQuality enum of starting_bitrate"), };
            
            for i in 0..(((b_max - b_min) / interval) + 1) {
                let bitrate = b_min + (interval * i);
                qualities.push(FfmpegQuality::Bitrate(bitrate));
            }
        },
        FfmpegQuality::Quality(q_min) => {
            let interval = 2;
            let q_max = match max_quality { FfmpegQuality::Quality(q) => q, _ => panic!("max_bitrate doesn't match FfmpegQuality enum of starting_bitrate"), };
            
            // iterates from maximum quality value (lowest quality) to minimum quality value, i.e. 22 -> 18
            for i in 0..(((q_max - q_min) / interval) + 1) {
                let quality = q_min - (interval * i);
                qualities.push(FfmpegQuality::Quality(quality));
            }
        }
    }

    return qualities;
}
