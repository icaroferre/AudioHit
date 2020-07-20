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
extern crate ot_utils;

use ot_utils::Slicer;


struct Preset {
    fade_in: String,
    fade_out: String,
    thresh_db: String,
}



fn stof(s:&str) -> f32 {
	let x = s.to_string().parse().unwrap();
	x
}

fn main() {
	//Version number
    let versionnumber = "0.2.0";

    // Set default preset
    let  default = getpreset(0);
    
    // Start timer
    let sw = Stopwatch::start_new();
    getpreset(1);
    // Initialize variables
	let mut filename = "0".to_string();
    let mut folder_path = "0".to_string();
	let mut fade_in_ms = default.0;
	let mut fade_out_ms = default.1;
    let mut thresh_db = default.2;
    let mut ot_file = "".to_string();

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
        ap.refer(&mut ot_file)
            .add_option(&["--ot_file"], Store,
            "Concatenate samples and generate Octatrack .ot file (true / false)");    
        ap.parse_args_or_exit();
    }

    let test_num : u32 = 124 * 6; 
    println!("{:?}", test_num.to_le_bytes());
    println!(" ");
    println!("AUDIOHIT - DEVELOPED BY ICARO FERRE");
    println!("Version {}", versionnumber);
    println!(" ");


    let mut processed_files:i16 = 0;

    let mut OT_Slicer = Slicer::new();


    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    

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
        if &folder_path != "0" {
            let check_file: &Path = &folder_path.as_ref();

            // Validate directory
            if check_file.is_dir() {

                // Get list of files
                let paths = fs::read_dir(&folder_path).unwrap();

                OT_Slicer.output_folder = folder_path.clone();

                OT_Slicer.output_filename = check_file.file_name().unwrap().to_str().unwrap().to_string();


                for path in paths {
                    // Get file info (path, name, and extension)
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
                            Some(_x) => {
                                let new_file = process(&file_path.to_str().unwrap().to_string(), &fade_in_ms, &fade_out_ms, &thresh_db);
                                if ot_file == "true" {
                                    OT_Slicer.add_file(new_file).unwrap();
                                }
                            },
                            None => println!("Error")
                        }; 
                    } 
                }
            }
            else {
                println!("ERROR: Folder not found: {:?}", check_file);
            }
        }
    };
    println!(" ");
    println!("Processed files: {}", processed_files);
    println!("Total time lapsed: {}ms", sw.elapsed_ms());
    println!(" ");

    if ot_file == "true" {
        OT_Slicer.generate_ot_file().unwrap();
    }
    
}

fn process(filename:&String, fade_in_ms:&String, fade_out_ms:&String, thresh_db:&String) -> String{

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

    // Check if file is mono
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
    	_ => 0
    };

    // Convert dB to linear amplitude
    let thresh_value: AmplitudeRatio<_> = DecibelRatio(stof(&thresh_db)).into();
    let thresh:f32 = max as f32 * thresh_value.amplitude_value() as f32;
    println!("Threshold set to: {} ({})", thresh, thresh_value.amplitude_value());


    let mut start_point:u32 = 0;
    let mut end_point:u32 = 0;
    let samples: Vec<i16> = reader.samples().map(|s| s.unwrap()).collect();

    if stereo_file == false {
        // Scan through audio file
    	for i in 0..file_dur as usize {
    		let samp_pos = i;

            // Check if sample exists
    		let sample_exists = match samples.get(samp_pos + 1) {
    			Some(_) => true,
    			None => false
    		};

    		if sample_exists == true {

                // Get absolute value of sample at index
                // println!("{}", samples[samp_pos]);
    			let abs_val = match samples[samp_pos] {
    				_ if (samples[samp_pos] > 0) => samples[samp_pos],
    				_ if (samples[samp_pos] < 0) =>  {
                        let mut value = samples[samp_pos];
                        if value <  -32767 {
                            value = -32767;
                        };
                        value * -1
                        // inv
                    },
    				_ => 0
    			};

                // If start point has not been found and value is > thresh, set startpoint
    			if abs_val as f32 > thresh && start_point == 0 {
                	start_point = samp_pos as u32;
            	}

                // Keep updating the end point while sample value > thresh
            	if abs_val as f32 > thresh{
                	end_point = samp_pos as u32;
            	}
    		}
    	}
    	println!("Start Point: {} / End Point: {}",start_point, end_point)

    } else {
    	println!("ERROR: The file is a stereo.");
    }
    println!(" ");
    println!("Creating new file...");
    

 	// WRITER STAGE

    // Set the specs of new audio file
 	let spec = hound::WavSpec {
        channels: 1,
        sample_rate: file_sr,
        bits_per_sample: file_bits,
        sample_format: hound::SampleFormat::Int,
    };

    // Get parent folder of audio file
    let parentpath = &path.parent().unwrap().to_str().unwrap().to_string().to_owned();
    let mut new_filename  = parentpath.to_owned();

    // Create output folder
    new_filename.push_str("/output/");
    new_filename = createdir(new_filename);
   	new_filename.push_str(&path.file_name().unwrap().to_str().unwrap());
    let new_file: &Path = new_filename.as_ref();
    println!("New file: {}", new_filename);

    if new_file.is_file() {
    	println!("File already exists. Deleting existing file..");
        fs::remove_file(new_file).unwrap();
    };

    let mut writer = hound::WavWriter::create(new_file, spec).unwrap();
    let new_file_dur = (file_dur  - start_point) - (file_dur - end_point);

	let fade_in = (stof(&fade_in_ms) as f32 * (file_sr as f32 * 0.001)).floor() as i32;
    let fade_out = (stof(&fade_out_ms) as f32 * (file_sr as f32 * 0.001)).floor() as i32;
    println!("Fade durations: {}ms ({} samples) / {}ms ({} samples)", &fade_in_ms, fade_in, &fade_out_ms, fade_out);
    println!("New file duration: {} samples", new_file_dur);

    println!("Filling new file...");

    // let mut final_array : Vec<i16> = Vec::new();

    

    // Start writing new file
    for i in 0..new_file_dur as usize  {

        // Add start point to index
    	let index = i + start_point as usize; 
    	
        // Check if sample is valid
        let sample_exists = match samples.get(index) {
    			Some(_) => true,
    			None => false
    	};
    	if sample_exists == true{ 
	        let amplitude = match i {
                // Fade in
	        	_ if (i < fade_in as usize) => (samples[index] as f32 * (i as f32 / fade_in as f32)) as i16,
	        	// Fade Out
                _ if (i > (new_file_dur as i32 - fade_out) as usize) => (samples[index] as f32 * (1.0 - ((i as f32 - (new_file_dur as f32 - fade_out as f32) as f32) as f32 / fade_out as f32))) as i16,
	        	// In between
                _ => samples[index]
	        };
	        // println!("{} / {}: {}", i, new_file_dur, amplitude);
            writer.write_sample(amplitude as i16).unwrap();
            // final_array.push(amplitude as i16);
    	}
    	
    }

    println!("---------------------------");
    println!(" ");


    writer.finalize().unwrap();

    // let new_file_path: &Path = new_filename.;
    new_filename
    // final_array

}

fn createdir(dir:String) -> String {

    match fs::create_dir(&dir) {
        Err(e) => {println!("{:?}", e)},
        _ => {}
    };
    dir
} 

fn getpreset(pnum:usize) -> (String, String, String) {
    let drums = Preset{fade_in: "0.2".to_string(), fade_out: "2".to_string(), thresh_db: "-18".to_string()};
    let bass = Preset{fade_in: "2".to_string(), fade_out: "2".to_string(), thresh_db: "-18".to_string()};

    let mut presetvec = Vec::new();
    presetvec.push(drums);
    presetvec.push(bass);

    // println!("{:?}", presetvec[pnum].fade_in);

    let validpreset = match pnum {
        _ if pnum < presetvec.len() => true,
        _ if pnum  > presetvec.len() => false,
        _ => false
    };

    assert_eq!(validpreset, true);

    let fadein = &presetvec[pnum].fade_in;
    let fadeout= &presetvec[pnum].fade_out;
    let thresh = &presetvec[pnum].thresh_db;

    (fadein.to_string(), fadeout.to_string(), thresh.to_string())

    
    // println!("Bass preset: {}, {}, {}", presetvalues.0, presetvalues.1, presetvalues.2);

}

