// AUDIOHIT
// DEVELOPED BY ÍCARO FERRE
// @icaroferre 


use std::path::Path;
// use std::env;
use std::fs;
use decibel::{AmplitudeRatio, DecibelRatio};
use argparse::{ArgumentParser, Store};
use stopwatch::{Stopwatch};

extern crate argparse;
extern crate hound;
extern crate decibel;
extern crate stopwatch;


fn stof(s:&str) -> f32 {
	let x = s.to_string().parse().unwrap();
	x
}

fn main() {
	//Version number
    let versionnumber = "0.1.0";

    // Start timer
    let sw = Stopwatch::start_new();

    // Initialize variables
	let mut filename = "0".to_string();
    let mut folder_path = "0".to_string();
	let mut fade_in_ms = "0.2".to_string();
	let mut fade_out_ms = "2".to_string();
	let mut thresh_db = "-36".to_string();

	{  // Get variable values from arguments
        let mut ap = ArgumentParser::new();
        ap.set_description("Greet somebody.");
        ap.refer(&mut filename)
            .add_option(&["--file"], Store,
            "Filename.");
        ap.refer(&mut folder_path)
            .add_option(&["--folder"], Store,
            "Set folder path for batch processing.");
        ap.refer(&mut fade_in_ms)
            .add_option(&["--fadein"], Store,
            "Fade in Time (default 0.2 ms).");
        ap.refer(&mut fade_out_ms)
            .add_option(&["--fadeout"], Store,
            "Fade Out Time (default 2 ms).");
        ap.refer(&mut thresh_db)
            .add_option(&["--thresh"], Store,
            "Threshold value in decibels (default -36dB).");
        ap.parse_args_or_exit();
    }

    println!(" ");
    println!("AUDIOHIT - DEVELOPED BY ICARO FERRE");
    println!("Version {}", versionnumber);
    println!(" ");


    let mut processed_files:i16 = 0;

    // Single file processing
    if filename != "0" {
        let check_file: &Path = &filename.as_ref();
        if check_file.is_file() {
            process(&filename, &fade_in_ms, &fade_out_ms, &thresh_db);
        } else {
            println!("ERROR: File not found.");
        }
    }
    else {
        // Folder batch processing
        if folder_path != "0" {
            let check_file: &Path = &filename.as_ref();

            // Validate directory
            if check_file.is_dir() {

                // Get list of files
                let paths = fs::read_dir(folder_path).unwrap();
                for path in paths {
                    let file_path = path.unwrap().path();
                    let file_name = &file_path.file_name();
                    let file_ext = match &file_path.extension(){
                        &Some(x) => x.to_str().unwrap(),
                        &None => " "
                    };

                    // Process WAV files
                    if file_ext == "wav" {
                        processed_files += 1;
                        match *file_name {
                            Some(_x) => process(&file_path.to_str().unwrap().to_string(), &fade_in_ms, &fade_out_ms, &thresh_db),
                            None => println!("Error")
                        }; 
                    } 
                }
            }
            else {
                println!("ERROR: Folder not found.");
            }
        }
    };
    println!(" ");
    println!("Processed files: {}", processed_files);
    println!("Total time lapsed: {}ms", sw.elapsed_ms());
    println!(" ");
}

fn process(filename:&String, fade_in_ms:&String, fade_out_ms:&String, thresh_db:&String) {

    println!("----- INPUT FILE -----");
    println!("File name: {}", filename);
	let path: &Path = &filename.as_ref();

    // Read audio file
    let mut reader = hound::WavReader::open(path).unwrap();

    // Get audio file info
    let file_sr = reader.spec().sample_rate;
    let file_dur = reader.len() / reader.spec().channels as u32;
    let file_chs = reader.spec().channels;
    let file_bits = reader.spec().bits_per_sample;

    let stereo_file = match file_chs {
    	1 => false,
    	2 => true,
    	_ => true
    };

	
    println!("Sample Rate: {}", file_sr);
    println!("File duration: {} samples", file_dur);
    println!("Channels: {}", file_chs);
    println!("Bit-rate: {}", file_bits);

    // println!(" ");

    assert_eq!(file_bits, 16, "INVALID FILE. Convert audio file to 16 bits and try again.");

    let max = match file_bits {
    	16 => <u16>::max_value(),
    	24 => <u32>::max_value() as u16,
    	_ => 0
    };

    let thresh_value: AmplitudeRatio<_> = DecibelRatio(stof(&thresh_db)).into();
    let thresh:f32 = max as f32 * thresh_value.amplitude_value() as f32;
    // println!("Max amplitude in {} bits: {}", file_bits, max);
    println!("Threshold set to: {} ({})", thresh, thresh_value.amplitude_value());

    let mut start_point:u32 = 0;
    let mut end_point:u32 = 0;
    let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();

    if stereo_file == false {
    	for i in 0..file_dur as usize {
    		let samp_pos = i;
    		let sample_exists = match samples.get(samp_pos + 1) {
    			Some(_) => true,
    			None => false
    		};
    		if sample_exists == true{
    			let abs_val = match samples[samp_pos] {
    				_ if (samples[samp_pos] > 0) => samples[samp_pos],
    				_ if (samples[samp_pos] < 0) => samples[samp_pos] * -1,
    				_ => 0
    			};
    			// println!("{} / {} / {}", samp_pos, abs_val, thresh);
    			if abs_val as f32 > thresh && start_point == 0 {
                	start_point = samp_pos as u32;
            	}
            	if abs_val as f32 > thresh{
                	end_point = samp_pos as u32;
            	}
    		}
    	}
    	println!("Start Point: {} / End Point: {}",start_point, end_point)

    } else {
    	println!("ERROR: The file is a stereo.");
    }
    // println!(" ");
    // println!("----------------------");
    println!(" ");


	

    println!("Creating new file...");
    

 	// WRITER STAGE

 	let spec = hound::WavSpec {
        channels: 1,
        sample_rate: file_sr,
        bits_per_sample: file_bits,
        sample_format: hound::SampleFormat::Int,
    };
    let parentpath = &path.parent().unwrap().to_str().unwrap().to_string().to_owned();
    let mut new_filename  = parentpath.to_owned();
    new_filename.push_str("/output/");
    new_filename = createdir(new_filename);
   	new_filename.push_str(&path.file_name().unwrap().to_str().unwrap());

    let new_file: &Path = new_filename.as_ref();
    println!("New file: {}", new_filename);

    if new_file.is_file() {
    	println!("File already exists. Deleting existing file..");
        fs::remove_file(new_file);
    };

    let mut writer = hound::WavWriter::create(new_file, spec).unwrap();
    let new_file_dur = (file_dur  - start_point) - (file_dur - end_point);

	let fade_in = (stof(&fade_in_ms) as f32 * (file_sr as f32 * 0.001)).floor() as i32;
    let fade_out = (stof(&fade_out_ms) as f32 * (file_sr as f32 * 0.001)).floor() as i32;
    println!("Fade durations: {}ms ({} samples) / {}ms ({} samples)", &fade_in_ms, fade_in, &fade_out_ms, fade_out);
    println!("New file duration: {} samples", new_file_dur);

    println!("Filling new file...");

    


    for i in 0..new_file_dur as usize  {
    	let index = i + start_point as usize;
    	let sample_exists = match samples.get(index) {
    			Some(_) => true,
    			None => false
    	};
    	if sample_exists == true{ 
	        let amplitude = match i {
	        	_ if (i < fade_in as usize) => (samples[index] as f32 * (i as f32 / fade_in as f32)) as i16,
	        	_ if (i > (new_file_dur as i32 - fade_out) as usize) => (samples[index] as f32 * (1.0 - ((i as f32 - (new_file_dur as f32 - fade_out as f32) as f32) as f32 / fade_out as f32))) as i16,
	        	_ => samples[index]
	        };
	        // println!("{} / {}: {}", i, new_file_dur, amplitude);
	        writer.write_sample(amplitude as i16).unwrap();
    	}
    	
    }

    println!("---------------------------");
    println!(" ");


    writer.finalize().unwrap();
}

fn createdir(dir:String) -> String {
    fs::create_dir(&dir);
    // Ok(()); 
    dir

}