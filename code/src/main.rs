// AUDIOHIT
// DEVELOPED BY ÍCARO FERRE
// @icaroferre 


use std::path::Path;
use std::fs;
use decibel::{AmplitudeRatio, DecibelRatio};
use argparse::{ArgumentParser, Store};
use stopwatch::{Stopwatch};
use rand::Rng;


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

struct ProcessingPreset {
    fade_in: f32,
    fade_out: f32,
    thresh_db: f32,
    speed_up: u32,
    slow_down: u32,
    reduce_sr: u32,
    evenly_spaced : bool
}


fn stof(s:&str) -> f32 {
	let x = s.to_string().parse().unwrap();
	x
}

fn main() {
	//Version number
    let versionnumber = "0.4";

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
    let mut ot_random = "".to_string();

    let mut ot_evenspace = "false".to_string();
    let mut speed_up = "0".to_string();
    let mut slow_down = "0".to_string();
    let mut reduce_sr = "1".to_string();

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
            "Concatenate samples and generate Octatrack .ot file (true / false).");    
        ap.refer(&mut ot_random)
            .add_option(&["--ot_random"], Store,
            "If true, only 1 sample chain will be generated and 64 samples will be chosen randomly.");    
        ap.refer(&mut ot_evenspace)
            .add_option(&["--ot_evenspace"], Store,
            "If true, samples will be evenly space in the concatenated wav file.");    
        ap.refer(&mut speed_up)
            .add_option(&["--speedup"], Store,
            "Speeds up audio samples by a x times the original speed.");
        ap.refer(&mut slow_down)
            .add_option(&["--slowdown"], Store,
            "Slows down audio samples by 1/x of the original speed.");
        ap.refer(&mut reduce_sr)
            .add_option(&["--reducesr"], Store,
            "Divides the sample rate of audio samples by x.");
        ap.parse_args_or_exit();
    }


    println!(" ");
    println!("AUDIOHIT - DEVELOPED BY ICARO FERRE");
    println!("Version {}", versionnumber);
    println!(" ");


    let mut processed_files:u16 = 0;

    let mut OT_Slicer = Slicer::new();

    let params = ProcessingPreset{
        fade_in: stof(&fade_in_ms), 
        fade_out: stof(&fade_out_ms), 
        thresh_db: stof(&thresh_db),
        speed_up: stof(&speed_up) as u32,
        slow_down: stof(&slow_down) as u32,
        reduce_sr : stof(&reduce_sr) as u32,
        evenly_spaced : ot_evenspace.parse().unwrap()
    };

    // Single file processing
    if filename != "0" {
        let check_file: &Path = &filename.as_ref();
        if check_file.is_file() {
            process(&filename, &params);
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
                    processed_files += process_folder(&folder, &mut OT_Slicer, &ot_file, &params, ot_random.clone());
                }

                // Process files found in main folder
                processed_files += process_folder(&folder_path, &mut OT_Slicer, &ot_file, &params, ot_random.clone());

                
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

fn process(filename:&String, params: &ProcessingPreset) -> String{

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
    let thresh_value: AmplitudeRatio<_> = DecibelRatio(params.thresh_db).into();
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
    	println!("File already exists. Deleting existing file...");
        fs::remove_file(new_file).unwrap();
    };

    let mut writer = hound::WavWriter::create(new_file, spec).unwrap();
    let new_file_dur = (file_dur  - start_point) - (file_dur - end_point);

	let fade_in = (params.fade_in * (file_sr as f32 * 0.001)).floor() as i32;
    let fade_out = (params.fade_out * (file_sr as f32 * 0.001)).floor() as i32;
    println!("Fade durations: {}ms ({} samples) / {}ms ({} samples)", params.fade_in, fade_in, params.fade_out, fade_out);
    println!("New file duration: {} samples", new_file_dur);

    

    let mut new_audio_buffer: Vec<i16> = Vec::new();

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
            new_audio_buffer.push(amplitude as i16);
            
    	}
    	
    }

    if params.reduce_sr > 1 {
        println!("Reducing sample rate to: {}", reader.spec().sample_rate / params.reduce_sr);
        new_audio_buffer  = reduce_sr_buffer(new_audio_buffer.clone(), params.reduce_sr);
    }

    if params.speed_up > 0 {
        println!("Speeding up buffer to {}x the original speed", params.speed_up);
        new_audio_buffer  = speed_buffer(new_audio_buffer.clone(), params.speed_up);
    }

    if params.slow_down > 0 {
        println!("Slowing down buffer to 1/{} of the original speed", params.slow_down);
        new_audio_buffer  = slow_buffer(new_audio_buffer.clone(), params.slow_down);
    }

    println!("Filling file...");
    for i in new_audio_buffer {
        writer.write_sample(i).unwrap();
    }


    writer.finalize().unwrap();

    new_filename

}

fn createdir(dir:String) -> String {
    match fs::create_dir(&dir) {
        Err(_) => {},
        _ => {}
    };
    dir
} 

fn process_folder(folder_path:&String,  OT_Slicer: &mut Slicer, ot_file: &String, params: &ProcessingPreset, ot_random: String) -> u16 {
    let check_file: &Path = &folder_path.as_ref();
    let mut processed_files : u16 = 0;
    let mut random_files: bool = false;

    if ot_random == "true".to_string() {
        random_files = true;
    }
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
        let mut num_octa_files = ((valid_files.len() as f64 / octatrack_max_files as f64) as f64).ceil() as isize;

        if random_files {
            num_octa_files = 1;
        };

        let mut selected_files : Vec<usize> = Vec::new();

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

            if random_files {
                max_files = octatrack_max_files;
            }
    
            for file in 0..max_files {
                let mut file_pos : usize = file + (i * max_files as isize) as usize;

                // Set file position to a random file that hasn't been picked yet
                if random_files {
                    let mut rng = rand::thread_rng();
                    let mut found_valid_file: bool = false;
                    while found_valid_file == false {
                        let random_file = rng.gen_range(0, valid_files.len()); // Get random position
                        if selected_files.contains(&random_file) == false { // Check if has been picked before
                            file_pos = random_file.clone(); // Set file position to the random value
                            println!("Randomly selected file #{} ({} / 64)", random_file, file + 1);
                            selected_files.push(random_file); // Add random value to list of already picked values
                            found_valid_file = true; // Exit loop
                        }
                    }
                }

                if file_pos < valid_files.len() {
                    let file =  &valid_files[file_pos];
                    println!("Processing file {}: {}", file_pos, file.display());
                    let file_path : String = file.to_str().unwrap().to_string();
                    match ot_file.as_str() {
                        "true" => {
                            let new_file = process(&file_path, params);
                            OT_Slicer.add_file(new_file).unwrap();
                        },
                        "only" => {
                            OT_Slicer.add_file(file_path).unwrap();
                        }
                        _ => {
                            let _ = process(&file_path, params);
                        }
                    }
                }
            }


            if ot_file == "true" || ot_file == "only" {
                OT_Slicer.generate_ot_file(params.evenly_spaced).unwrap();
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
            if file.path().is_dir() && file.path().file_name().unwrap() != "output" {
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


fn speed_buffer(buf: Vec<i16>, multiplier: u32) -> Vec<i16> {
    let mut new_buffer : Vec<i16> = Vec::new();
    for i in 0..buf.len() {
        if i % multiplier as usize == 0 {
            new_buffer.push(buf[i]);
        }
    }
    new_buffer
}

fn reduce_sr_buffer(buf: Vec<i16>, divider: u32) -> Vec<i16> {
    let mut new_buffer : Vec<i16> = Vec::new();
    let mut last_value: i16 = 0;
    for i in 0..buf.len() {
        if i % divider as usize == 0 {
            last_value = buf[i];
        }
        new_buffer.push(last_value);
    }
    new_buffer
}

fn slow_buffer(buf: Vec<i16>, divider: u32) -> Vec<i16> {
    let mut new_buffer : Vec<i16> = Vec::new();
    for i in 0..buf.len() {
        for _ in 0..divider as usize {
            new_buffer.push(buf[i]);
        }
    }
    new_buffer
}