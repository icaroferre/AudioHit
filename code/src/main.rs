// AUDIOHIT
// DEVELOPED BY ÍCARO FERRE
// @icaroferre 


use std::path::Path;
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
    let versionnumber = "0.3";

    // Set default preset
    let  default = getpreset(0);
    
    // Start timer
    let sw = Stopwatch::start_new();

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


    println!(" ");
    println!("AUDIOHIT - DEVELOPED BY ICARO FERRE");
    println!("Version {}", versionnumber);
    println!(" ");


    let mut processed_files:u16 = 0;

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


                // Check for sub folders and process subfolders first
                let extra_folders = find_sub_folders(&folder_path);

                for folder in extra_folders {
                    processed_files += process_folder(&folder, &mut OT_Slicer, &ot_file, &fade_in_ms, &fade_out_ms, &thresh_db);
                }

                // Process files found in main folder
                processed_files += process_folder(&folder_path, &mut OT_Slicer, &ot_file, &fade_in_ms, &fade_out_ms, &thresh_db);

                
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
            writer.write_sample(amplitude as i16).unwrap();
    	}
    	
    }

    println!("---------------------------");
    println!(" ");


    writer.finalize().unwrap();

    new_filename

}

fn createdir(dir:String) -> String {
    match fs::create_dir(&dir) {
        Err(e) => {println!("{:?}", e)},
        _ => {}
    };
    dir
} 

fn process_folder(folder_path:&String,  OT_Slicer: &mut Slicer, ot_file: &String, fade_in_ms: &String, fade_out_ms: &String, thresh_db: &String) -> u16 {
    let check_file: &Path = &folder_path.as_ref();
    let mut processed_files : u16 = 0;
    // Validate directory
    if check_file.is_dir() {

        // Get list of files  
        let mut new_paths: Vec<_> = fs::read_dir(&folder_path).unwrap()
                                        .map(|r| r.unwrap())
                                        .collect();
        new_paths.sort_by_key(|dir| dir.path());

        let mut valid_files : Vec<std::path::PathBuf> = Vec::new();


        for path in new_paths {
            // Get file info (path, name, and extension)
            let file_path = path.path();
            let file_name = &file_path.file_name();
            let file_ext = match &file_path.extension(){
                &Some(x) => x.to_str().unwrap(),
                &None => " "
            };


            // Find WAV files and add to valid_files vector
            if file_ext == "wav" {
                match *file_name {
                    Some(_x) => {
                        valid_files.push(file_path);
                        processed_files += 1;
                    },
                    None => println!("Error")
                };
            } 
        }

        println!("Valid files found: {}", valid_files.len());


        // Because the Octatrack OT files can only support up to 64 files we need to batch process files in groups of 64 files or less
        // While 64 should be the theoretical max, we must do 32 at the time so the OT checksum doesn't overflow

        let octatrack_max_files = 64;
        let num_octa_files = ((valid_files.len() as f64 / octatrack_max_files as f64) as f64).ceil() as isize;
        println!("Number of Octatrack OT files: {}", num_octa_files);

        for i in 0..num_octa_files {
  
            // Reset the slice vector
            OT_Slicer.clear();

            // Create output folder
            let mut output_folder = folder_path.clone();
            output_folder.push_str("/output/");
            output_folder = createdir(output_folder);
            OT_Slicer.output_folder = output_folder;
            

            // Set OT file to the same name as the folder
            OT_Slicer.output_filename = check_file.file_name().unwrap().to_str().unwrap().to_string();

            // Add suffix to OT file if num_octa_files is greater than 0
            let mut max_files : usize = valid_files.len();

            if num_octa_files > 1 {
                let suffix = format!("_{}", i + 1);
                OT_Slicer.output_filename.push_str(suffix.as_str());
                max_files = octatrack_max_files;
            }
    
            for file in 0..max_files {
                let file_pos = file + (i * max_files as isize) as usize;
                if file_pos < valid_files.len() {
                    let file =  &valid_files[file_pos];
                    println!("Processing file {}: {}", file_pos, file.display());
                    match ot_file.as_str() {
                        "true" => {
                            let new_file = process(&file.to_str().unwrap().to_string(), fade_in_ms, fade_out_ms, thresh_db);
                            OT_Slicer.add_file(new_file).unwrap();
                        },
                        "only" => {
                            OT_Slicer.add_file(file.to_str().unwrap().to_string()).unwrap();
                        }
                        _ => {
                            let _ = process(&file.to_str().unwrap().to_string(), fade_in_ms, fade_out_ms, thresh_db);
                        }
                    }
                }
            }


            if ot_file == "true" || ot_file == "only" {
                OT_Slicer.generate_ot_file().unwrap();
            }

        }

        processed_files = valid_files.len() as u16;
    }
    processed_files
}


/// Look for folders inside a folder and returns a vector of paths as strings
fn find_sub_folders (folder_path: &String) -> Vec<String> {
    let check_file: &Path = &folder_path.as_ref();
    let mut valid_files : Vec<String> = Vec::new();

    // Validate directory
    if check_file.is_dir() {
        let mut files_found: Vec<_> = fs::read_dir(&folder_path).unwrap()
                                        .map(|r| r.unwrap())
                                        .collect();
        files_found.sort_by_key(|dir| dir.path());

        

        for file in files_found {
            if file.path().is_dir() {
                valid_files.push(file.path().to_str().unwrap().to_string());
            }
        }
    }
    valid_files

}


/// Get preset data for fade_in, fade_out and threshold values
fn getpreset(pnum:usize) -> (String, String, String) {
    let drums = Preset{fade_in: "3".to_string(), fade_out: "10".to_string(), thresh_db: "-48".to_string()};
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

    
}

