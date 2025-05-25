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
use ffmpeg::args::FfmpegQuality;
use permutation::permutation::Permutation;

use crate::permutor_cli::PermutorCli;

mod permutor_cli;

fn main() {
    log_cli_header(String::from("Permutation Tool"));
    let mut cli = PermutorCli::parse();
    cli.validate();

    log_special_arguments(&cli);

    let cli_quality = cli.fetch_quality_args();
    let quality_interval = cli.fetch_quality_interval();

    let mut engine = PermutationEngine::new(cli.log_output_directory.clone());
    let vendor = get_vendor_for_codec(&cli.encoder.clone());
    for quality in get_quality_permutations(cli_quality, cli.max_quality_permutation.unwrap(), quality_interval) {
        match vendor {
            Vendor::Nvidia => {
                build_nvenc_setting_permutations(&mut engine, &cli, quality);
            }
            Vendor::AMD => {
                panic!(); //build_amf_setting_permutations(&mut engine, &cli, bitrate);
            }
            Vendor::IntelQSV => {
                if cli.encoder.contains("av1") {
                    panic!(); //build_intel_av1_permutations(&mut engine, &cli, bitrate);
                } else {
                    panic!(); //build_intel_igpu_permutations(&mut engine, &cli, bitrate);
                }
            }
            Vendor::Apple => {
                panic!(); //build_apple_silicon_h264_permutations(&mut engine, &cli, bitrate);
            }
            Vendor::Unknown => {}
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

fn build_nvenc_setting_permutations(
    engine: &mut PermutationEngine,
    cli: &PermutorCli,
    quality: FfmpegQuality,
) {
    let mut nvenc = Nvenc::new(cli.encoder == "hevc_nvenc", cli.gpu, cli.no_b_frame);

    // initialize the permutations each time
    nvenc.init();

    while let Some((_encoder_index, settings)) = nvenc.next() {
        let mut permutation = Permutation::new(cli.source_file.clone(), cli.encoder.clone());
        permutation.video_file = cli.source_file.clone();
        permutation.encoder_settings = settings;
        permutation.quality = quality;
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
}

/*
fn build_amf_setting_permutations(engine: &mut PermutationEngine, cli: &PermutorCli, bitrate: u32) {
    let mut amf = Amf::new(cli.encoder == "hevc_amf", cli.gpu);

    // initialize the permutations each time
    amf.init();

    while let Some((_encoder_index, settings)) = amf.next() {
        let mut permutation = Permutation::new(cli.source_file.clone(), cli.encoder.clone());
        permutation.video_file = cli.source_file.clone();
        permutation.encoder_settings = settings;
        permutation.bitrate = bitrate;
        permutation.check_quality = cli.check_quality;
        permutation.verbose = cli.verbose;
        permutation.detect_overload = cli.detect_overload;
        permutation.allow_duplicates = cli.allow_duplicate_scores;
        permutation.ten_bit = cli.ten_bit;
        engine.add(permutation);

        // break out early here to just make 1 permutation
        if cli.test_run {
            break;
        }
    }
}

fn build_intel_av1_permutations(engine: &mut PermutationEngine, cli: &PermutorCli, bitrate: u32) {
    let mut intel_av1 = AV1QSV::new();

    // initialize the permutations each time
    intel_av1.init();

    while let Some((_encoder_index, settings)) = intel_av1.next() {
        let mut permutation = Permutation::new(cli.source_file.clone(), cli.encoder.clone());
        permutation.video_file = cli.source_file.clone();
        permutation.encoder_settings = settings;
        permutation.bitrate = bitrate;
        permutation.check_quality = cli.check_quality;
        permutation.verbose = cli.verbose;
        permutation.detect_overload = cli.detect_overload;
        permutation.allow_duplicates = cli.allow_duplicate_scores;
        permutation.ten_bit = cli.ten_bit;
        engine.add(permutation);

        // break out early here to just make 1 permutation
        if cli.test_run {
            break;
        }
    }
}

fn build_intel_igpu_permutations(engine: &mut PermutationEngine, cli: &PermutorCli, bitrate: u32) {
    let mut intel_i_gpu = QSV::new(cli.encoder == "hevc_qsv");

    // initialize the permutations each time
    intel_i_gpu.init();

    while let Some((_encoder_index, settings)) = intel_i_gpu.next() {
        let mut permutation = Permutation::new(cli.source_file.clone(), cli.encoder.clone());
        permutation.video_file = cli.source_file.clone();
        permutation.encoder_settings = settings;
        permutation.bitrate = bitrate;
        permutation.check_quality = cli.check_quality;
        permutation.verbose = cli.verbose;
        permutation.detect_overload = cli.detect_overload;
        permutation.allow_duplicates = cli.allow_duplicate_scores;
        permutation.ten_bit = cli.ten_bit;
        engine.add(permutation);

        // break out early here to just make 1 permutation
        if cli.test_run {
            break;
        }
    }
}

// TODO: we'll probably need to do more of these per apple silicon one
fn build_apple_silicon_h264_permutations(
    engine: &mut PermutationEngine,
    cli: &PermutorCli,
    bitrate: u32,
) {
    // this can probably be simplified with a type
    let h264 = cli.encoder.contains("h264");
    let prores = cli.encoder.contains("prores");
    let mut apple_silicon = Apple::new(h264, prores);

    // initialize the permutations each time
    apple_silicon.init();

    while let Some((_encoder_index, settings)) = apple_silicon.next() {
        let mut permutation = Permutation::new(cli.source_file.clone(), cli.encoder.clone());
        permutation.video_file = cli.source_file.clone();
        permutation.encoder_settings = settings;
        permutation.bitrate = bitrate;
        permutation.check_quality = cli.check_quality;
        permutation.verbose = cli.verbose;
        permutation.detect_overload = cli.detect_overload;
        permutation.allow_duplicates = cli.allow_duplicate_scores;
        permutation.ten_bit = cli.ten_bit;
        engine.add(permutation);

        // break out early here to just make 1 permutation
        if cli.test_run {
            break;
        }
    }
}
*/

fn get_quality_permutations(starting_quality: FfmpegQuality, max_quality: u32, interval: i32) -> Vec<FfmpegQuality> {

    let mut qualities = Vec::new();
    let min: i32;
    //let interval: i32;
    match starting_quality {
        FfmpegQuality::ConstantBitrate(b) => {
            min = b as i32;
            //interval = 5;
        },
        FfmpegQuality::ConstantQuality(q) => {
            min  = q as i32;
            //interval = -4;
        }, // note interval is negative since quality increases with smaller CQs
    }

    for i in 0..(((max_quality as i32 - min) / interval) + 1) {
        qualities.push(match starting_quality {
            FfmpegQuality::ConstantBitrate(_) => FfmpegQuality::ConstantBitrate((min + (interval * i)) as u32),
            FfmpegQuality::ConstantQuality(_) => FfmpegQuality::ConstantQuality((min + (interval * i)) as u32),
        });
    }

    return qualities;
}
