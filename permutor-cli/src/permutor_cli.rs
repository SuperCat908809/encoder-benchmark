use clap::Parser;

use cli::cli_util::{error_with_ack, standard_cli_check};

use ffmpeg::args::FfmpegQuality;

#[derive(Parser)]
pub struct PermutorCli {
    /// the encoder you wish to benchmark: [h264_nvenc, hevc_nvenc, etc]
    #[arg(short, long, value_name = "encoder_name", default_value = "encoder")]
    pub encoder: String,
    /// target bitrate (in Mb/s) to output; in combination with --max-bitrate-permutation, this is the starting permutation
    #[arg(short, long, value_name = "bitrate")]
    pub bitrate: Option<u32>,
    /// target quality (in 0..51 range, smaller is better) to output; in combination with --max-quality-permutation, this is the starting permutation
    #[arg(short, long, value_name = "quality")]
    pub quality: Option<u32>,
    /// whether to run vmaf score on each permutation or not
    #[arg(short, long)]
    pub check_quality: bool,
    /// when used with check_quality, encodes that produce the same quality will still be encoded
    #[arg(short, long)]
    pub(crate) allow_duplicate_scores: bool,
    // stop an encoding session if the encoder can't keep up with the input file's FPS
    #[arg(short, long)]
    pub detect_overload: bool,
    /// the source file you wish to benchmark; if not provided, will run standard benchmark on all supported resolutions
    #[arg(short, long, value_name = "source.y4m", default_value = "")]
    pub source_file: String,
    /// the directory you wish the benchmark to look for your encoder files; can be used with --source_file/-s if you wish
    #[arg(short, long, value_name = "folder/to/files", default_value = "")]
    pub files_directory: String,
    /// the directory you wish for the logs this tool produces to go into; defaults to the current directory. Does NOT support spaces in directories
    #[arg(long, value_name = "folder/to/log/output", default_value = "")]
    pub log_output_directory: String,
    /// runs just the first permutation for given encoder; useful for testing the tool & output
    #[arg(short, long)]
    pub test_run: bool,
    /// adds in '-pix_fmt yuv420p10le' to force 10-bit encoding
    #[arg(long)]
    pub ten_bit: bool,
    /// maximum value to increase the bitrate or quality to. If not specified, tool will not permute over bitrate values
    #[arg(short, long, value_name = "bitrate/quality")]
    pub max_quality_permutation: Option<u32>,
    /// bitrate or quality interval to increase the bitrate or quality by. (Default is 5Mb/s or 4 CQ intervals); only used if max_quality_permutation is specified
    #[arg(short, long, value_name = "bitrate/quality interval")]
    pub interval_quality_permutation: Option<u32>,
    /// logs useful information to help troubleshooting
    #[arg(short, long)]
    pub verbose: bool,
    /// lists the supported/implemented supported that this tool supports
    #[arg(short, long)]
    pub list_supported_encoders: bool,
    /// the GPU you wish to run the encode on; defaults to the first/only GPU found in your system
    #[arg(short, long, default_value = "0")]
    pub gpu: u8,
    /// opt-out of using b frames for either H264 or HEVC encoders; currently only supported for Nvidia GPUs
    #[arg(short, long)]
    pub no_b_frame: bool,
}

impl PermutorCli {
    pub fn validate(&mut self) {
        standard_cli_check(
            self.list_supported_encoders,
            &self.encoder,
            &self.source_file,
            &self.files_directory,
            false,
        );

        if self.source_file.is_empty() {
            println!("Error: No source file was provided to run on, please specify an input file");
            error_with_ack(false);
        }

        if self.bitrate.is_none() && self.quality.is_none() {
            println!("Error: Neither constant bitrate or quality was provided, please specified one of the two");
            error_with_ack(false);
        }

        if !self.bitrate.is_none() && !self.quality.is_none() {
            println!("Error: Both constant bitrate and constant was provided, please specified one of the two");
            error_with_ack(false);
        }

        if self.max_quality_permutation.is_none() {
            self.max_quality_permutation = if !self.bitrate.is_none() {
                Option::from(self.bitrate)
            }
            else {
                Option::from(self.quality)
            }
        }
        else {
            if !self.bitrate.is_none() && self.fetch_quality_interval() == 0 {
                println!("Error: Bitrate interval cannot be zero");
                error_with_ack(false);
            } else if !self.quality.is_none() && self.fetch_quality_interval() == 0 {
                println!("Error: Quality interval cannot be zero");
                error_with_ack(false);
            }
        }

        if self.source_file.is_empty() && !self.files_directory.is_empty() {
            // internally map the source_file and source_files_directory together
            self.source_file = format!("{}/{}", self.files_directory, self.source_file);
        }
    }

    pub fn has_special_options(&self) -> bool {
        return self.check_quality
            || self.detect_overload
            || self.verbose
            || self.test_run
            || self.allow_duplicate_scores;
    }

    pub fn fetch_quality_args(&self) -> FfmpegQuality {
        if !self.bitrate.is_none() {
            FfmpegQuality::ConstantBitrate(self.bitrate.unwrap())
        }
        else {
            FfmpegQuality::ConstantQuality(self.quality.unwrap())
        }
    }

    pub fn fetch_quality_interval(&self) -> i32 {
        match self.fetch_quality_args() {
            FfmpegQuality::ConstantBitrate(_) => self.interval_quality_permutation.unwrap_or(5) as i32,
            FfmpegQuality::ConstantQuality(_) => -(self.interval_quality_permutation.unwrap_or(4) as i32), // quality increases as CQ decreases
        }
    }
}
