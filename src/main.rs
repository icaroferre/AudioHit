// AUDIOHIT
// DEVELOPED BY ÍCARO FERRE
// @icaroferre 


use std::path::Path;
use std::fs;
use decibel::{AmplitudeRatio, DecibelRatio};
use argparse::{ArgumentParser, Store};
use stopwatch::{Stopwatch};
use rand::Rng;
use std::process::Command;
// use std::thread;
// use std::sync::{Arc, Mutex};


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

struct SampleRegion{
    start_point: u32,
    end_point: u32
}


struct SampleData{
    file_path: String,
    file_name: String,
    destination_folder: String,
    duration: u32,
    samples: Vec<i16>,
    regions: Vec<SampleRegion>,
    spec: hound::WavSpec
}

struct ProcessResults{
    sample: Option<SampleData>,
    generated_files: Vec<String>
}

#[derive(Clone)]
struct ProcessingPreset {
    mode: String,
    autoconvert : bool,
    split: bool,
    min_length : f32,
    fade_in: f32,
    fade_out: f32,
    thresh_db: f32,
    speed_up: u32,
    slow_down: u32,
    reduce_sr: u32,
    evenly_spaced : bool,
    ot_file : bool,
    ot_random : bool,
    pitch_offset : isize,
    pitch_range : isize,
    normalize: bool
}


fn stof(s:&str) -> f32 {
	let x = s.to_string().parse().unwrap();
	x
}

fn main() {
	//Version number
    let versionnumber = "0.8";

    // Set default preset
    let  default = getpreset(0);
    
    // Start timer
    let sw = Stopwatch::start_new();

    // Initialize variables
    let mut op_mode = "trim".to_string();
	let mut filename = "0".to_string();
    let mut folder_path = "0".to_string();
    let mut autoconvert = "true".to_string();

    let mut split = "false".to_string();
	let mut fade_in_ms = default.0;
    let mut fade_out_ms = default.1;
    let mut min_length = "450.0".to_string();
    let mut thresh_db = default.2;

    let mut speed_up = "0".to_string();
    let mut slow_down = "0".to_string();
    let mut reduce_sr = "1".to_string();


    let mut ot_file = "false".to_string();
    let mut ot_random = "false".to_string();
    let mut ot_evenspace = "false".to_string();
    let mut normalize = "true".to_string();

    // SCALE MODE ARGUMENTS
    let mut pitch_offset = "0".to_string();
    let mut pitch_range = "12".to_string();

	{  // Get variable values from arguments
        let mut ap = ArgumentParser::new();
        ap.set_description("Greet somebody.");
        ap.refer(&mut op_mode)
            .add_option(&["--mode"], Store,
            "Set the operation mode (trim / ref / scale).");
        ap.refer(&mut filename)
            .add_option(&["--file"], Store,
            "Filename.");
        ap.refer(&mut folder_path)
            .add_option(&["--folder"], Store,
            "Set folder path for batch processing.");
        ap.refer(&mut split)
            .add_option(&["--split"], Store,
            "Splits processed file into multiple files based on the threshold.");
        ap.refer(&mut min_length)
            .add_option(&["--minlength"], Store,
            "Set the mininum length for each split region.");
                
        ap.refer(&mut fade_in_ms)
            .add_option(&["--fadein"], Store,
            "Fade in Time (default 0.2 ms).");
        ap.refer(&mut fade_out_ms)
            .add_option(&["--fadeout"], Store,
            "Fade Out Time (default 2 ms).");
        ap.refer(&mut autoconvert)
            .add_option(&["--autoconvert"], Store,
            "Enables or disables auto-convertion via SoX.");
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
        ap.refer(&mut pitch_offset)
            .add_option(&["--pitch_offset"], Store,
            "Set the pitch offset (root note) for the scale mode. C = 0 Db = 1 D = 2 ...");
        ap.refer(&mut pitch_range)
            .add_option(&["--pitch_range"], Store,
            "Set the pitch range for the scale mode.");
        ap.refer(&mut normalize)
            .add_option(&["--normalize"], Store,
            "Enable individual sample normalization to -0.1dB.");  
        ap.parse_args_or_exit();
    }

    // Fill processing preset using arguments
    let params = ProcessingPreset{
        mode: op_mode,
        autoconvert : autoconvert.parse().unwrap(),
        split: split.parse().unwrap(),
        min_length : stof(&min_length),
        fade_in: stof(&fade_in_ms), 
        fade_out: stof(&fade_out_ms), 
        thresh_db: stof(&thresh_db),
        speed_up: stof(&speed_up) as u32,
        slow_down: stof(&slow_down) as u32,
        reduce_sr : stof(&reduce_sr) as u32,
        evenly_spaced : ot_evenspace.parse().unwrap(),
        ot_file : ot_file.parse().unwrap(),
        ot_random : ot_random.parse().unwrap(),
        pitch_offset : pitch_offset.parse().unwrap(),
        pitch_range : pitch_range.parse().unwrap(),
        normalize: normalize.parse().unwrap()
    };


    println!(" ");
    println!("AUDIOHIT - DEVELOPED BY ICARO FERRE");
    println!("Version {}", versionnumber);
    println!(" ");


    let mut processed_files:u16 = 0;

    let mut ot_slicer = Slicer::new();

    
    // Single file processing
    if filename != "0" {
        let check_file: &Path = &filename.as_ref();
        if check_file.is_file() {
            processed_files += process_file(filename, &params, &mut ot_slicer, true).generated_files.len() as u16;
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
                    processed_files += process_folder(&folder, &mut ot_slicer, &params);
                }

                // Process files found in main folder
                processed_files += process_folder(&folder_path, &mut ot_slicer, &params );

                
            }
            else {
                println!("ERROR: Folder not found: {:?}", check_file);
            }
        }
    };
    println!(" ");
    println!("Processed files: {}", processed_files);
    println!("Total time lapsed: {}ms", sw.elapsed_ms());
    println!("Average time per sample: {}ms", sw.elapsed_ms() as f32 / processed_files as f32);
    println!(" ");

    
    
}

// Loads a WAV file and parses it as a SampleData struct
fn load_audio_file(filename:String, params: &ProcessingPreset) -> Option<SampleData> {
    println!("Loading file: {}", filename);

    let filename_clone = filename.clone();
    let path : &Path = filename_clone.as_ref();
    let output_folder = create_output_folder(filename.clone());

    let reader = hound::WavReader::open(filename.clone());
    
    match reader {
        Ok(mut reader) => {

            // Convert sample if it's not compatible (16 bits / mono)
            if params.autoconvert && (reader.spec().bits_per_sample != 16 || reader.spec().channels > 1) {
                let converted_file = auto_convert(filename.clone(), &reader.spec());
                reader = hound::WavReader::open(converted_file).unwrap();
            }

            let mut sample_data = SampleData {
                file_path: filename,
                file_name: format!("{}", path.file_stem().unwrap().to_str().unwrap()),
                destination_folder: output_folder,
                duration: reader.len() / reader.spec().channels as u32,
                samples: reader.samples().map(|s| s.unwrap()).collect(),
                spec : reader.spec(),
                regions: vec!{}
            };

            
            // Normalize buffer to improve detection
            sample_data.samples = normalize_buffer(sample_data.samples, -1.0);
            println!("Sample-rate: {} | Bit-Depth: {} | Channels: {}", sample_data.spec.sample_rate, sample_data.spec.bits_per_sample, sample_data.spec.channels);

            assert_eq!(sample_data.spec.channels, 1, "INVALID FILE. Convert audio file to mono and try again.");
            assert_eq!(sample_data.spec.bits_per_sample, 16, "INVALID FILE. Convert audio file to 16 bits and try again.");
            Some(sample_data)
        },
        Err(_) => {
            
            None
        }
    }
    
}


// Fids different regions in a sample data and add found regions to the sample data struct
fn find_regions(mut sample_data: SampleData, params: &ProcessingPreset) -> SampleData {


    let max = match sample_data.spec.bits_per_sample {
    	16 => <u16>::max_value(),
    	_ => 0
    };


    // Convert dB to linear amplitude
    let thresh_value: AmplitudeRatio<_> = DecibelRatio(params.thresh_db).into();
    let thresh:f32 = max as f32 * thresh_value.amplitude_value() as f32;
    // println!("Threshold set to: {} ({})", thresh, thresh_value.amplitude_value());

    if sample_data.spec.channels == 1 {
        // Scan through audio file
        let mut last_start_point : u32 = 0;
        let mut last_valid_point = 0;
        let mut silence_buffer : u32 = 0;
        let min_length = ((params.min_length / 1000.0) * sample_data.spec.sample_rate as f32) as u32;
        let min_silence = (min_length as f32 / 2.5) as u32;
    	for i in 0..sample_data.duration as usize {
    		let samp_pos = i;

            // Check if sample exists
    		let sample_exists = match sample_data.samples.get(samp_pos + 1) {
    			Some(_) => true,
    			None => false
    		};

    		if sample_exists == true {

                // Get absolute value of sample at index
                // println!("{}", samples[samp_pos]);
    			let abs_val = match sample_data.samples[samp_pos] {
    				_ if (sample_data.samples[samp_pos] > 0) => sample_data.samples[samp_pos],
    				_ if (sample_data.samples[samp_pos] < 0) =>  {
                        let mut value = sample_data.samples[samp_pos];
                        if value <  -32767 {
                            value = -32767;
                        };
                        value * -1
                        // inv
                    },
    				_ => 0
    			};

                // If start point has not been found and value is > thresh, set startpoint
    			if abs_val as f32 > thresh {
                    if last_start_point == 0 {
                        last_start_point = samp_pos as u32;
                    } else {
                        last_valid_point = samp_pos as u32;
                    }
            	}

                // Keep updating the end point while sample value > thresh
            	if abs_val as f32 <= thresh && last_start_point > 0 {
                    let end_point = samp_pos as u32;
                    silence_buffer += 1;
                    if params.split && end_point - last_start_point > min_length && silence_buffer > min_silence {
                        let new_region = SampleRegion{
                            start_point : last_start_point.clone(),
                            end_point : end_point
                        };
                        last_start_point = 0;
                        println!("New region found: {} / {}", new_region.start_point, new_region.end_point);
                        sample_data.regions.push(new_region);
                        silence_buffer = 0;
                    }
                    
            	}
    		}
        }
        
        if params.split == false {
            let new_region = SampleRegion{
                start_point : last_start_point.clone(),
                end_point : last_valid_point
            };
            sample_data.regions.push(new_region);
        } 

        sample_data.regions.reverse();
        
        // let num_of_regions = if params.split { regions.len()} else {1};
        // for i in 0..num_of_regions {
        //     let re = regions.pop().unwrap();
        //     let new_file = create_trimmed_file(to_process.clone(), output_folder.clone(), re, params , i as u8);
        //     new_files.push(new_file);
            
        // }
        
       
    } 
    sample_data
}


// Generates separate audio files for each region in the sample data
fn generate_samples_from_regions(sample_data: &SampleData, params: &ProcessingPreset) -> Vec<String> {
    let num_of_regions = sample_data.regions.len();
    let mut generated_files: Vec<String> = vec!{};
    for i in 0..num_of_regions {
        let region = SampleRegion {
            start_point: sample_data.regions[i].start_point.clone(),
            end_point: sample_data.regions[i].end_point.clone()
        };
        let new_file = create_trimmed_file(sample_data, params, region , i as u8);
        generated_files.push(new_file);
    }
    generated_files
}

fn normalize_buffer(buffer: Vec<i16>, gain: f32) -> Vec<i16> {
    // println!("Normalizing buffer...");
    
    let total_gain: AmplitudeRatio<_> = DecibelRatio(gain).into();
    let total_gain_mult : f32 = i16::MAX as f32 * total_gain.amplitude_value() as f32;
    
    let mut new_buffer : Vec<i16> = Vec::new();
    let mut max_gain : i32 = 0;
    
    for i in 0..buffer.len() {
        let sample = (buffer[i]as i32).abs();
        if sample > max_gain {
            max_gain = sample;
        }
    };
    let multiplier: f32 = total_gain_mult / max_gain as f32;
    // println!("Normalizing ratio: {}", multiplier);
    for i in 0..buffer.len() {
        let sample = (buffer[i] as f32 * multiplier) as i16;
        new_buffer.push(sample);
    }
    new_buffer
}

fn create_trimmed_file(sample_data: &SampleData, params: &ProcessingPreset, trim_data: SampleRegion, file_num: u8) -> String {
    // WRITER STAGE

    let file_path : &Path = sample_data.file_path.as_ref();
    

    // let output_folder_string = create_output_folder(file_name.clone());
    let output_folder_path : &Path = sample_data.destination_folder.as_ref();
    let mut wav_file_name : String = file_path.file_stem().unwrap().to_str().unwrap().to_string();

    if params.split {
        wav_file_name = format!("{}_{:2<0}.wav", wav_file_name, file_num + 1);
    } else {
        wav_file_name = format!("{}.wav", wav_file_name);
    }

    let mut new_file = Path::join(output_folder_path, wav_file_name);
   	let new_file_string = new_file.display().to_string();

    if new_file.is_file() {
    	println!("File already exists. Deleting existing file...");
        fs::remove_file(&mut new_file).unwrap();
    };

    let mut writer = hound::WavWriter::create(&mut new_file, sample_data.spec).unwrap();
    let new_file_dur = (sample_data.duration  - trim_data.start_point) - (sample_data.duration - trim_data.end_point);

	let fade_in = (params.fade_in * (sample_data.spec.sample_rate as f32 * 0.001)).floor() as i32;
    let fade_out = (params.fade_out * (sample_data.spec.sample_rate as f32 * 0.001)).floor() as i32;
    

    let mut new_audio_buffer: Vec<i16> = Vec::new();

    // Start writing new file
    for i in 0..new_file_dur as usize  {

        // Add start point to index
    	let index = i + trim_data.start_point as usize; 
    	
        // Check if sample is valid
        let sample_exists = match sample_data.samples.get(index) {
    			Some(_) => true,
    			None => false
    	};
    	if sample_exists == true{ 
	        let amplitude = match i {
                // Fade in
	        	_ if (i < fade_in as usize) => (sample_data.samples[index] as f32 * (i as f32 / fade_in as f32)) as i16,
	        	// Fade Out
                _ if (i > (new_file_dur as i32 - fade_out) as usize) => (sample_data.samples[index] as f32 * (1.0 - ((i as f32 - (new_file_dur as f32 - fade_out as f32) as f32) as f32 / fade_out as f32))) as i16,
	        	// In between
                _ => sample_data.samples[index]
            };
            new_audio_buffer.push(amplitude as i16);
            
    	}
    	
    }

    if params.reduce_sr > 1 {
        println!("Reducing sample rate to: {}", sample_data.spec.sample_rate / params.reduce_sr);
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

    // Normalize samples
    if params.normalize {
        new_audio_buffer = normalize_buffer(new_audio_buffer, -0.1);
    }

    println!("Creating new file: {} ({} samples)", new_file_string, new_file_dur);
    for i in new_audio_buffer {
        writer.write_sample(i).unwrap();
    }

    writer.finalize().unwrap();

    new_file_string

}

fn createdir(dir:String) {
    match fs::create_dir(&dir) {
        Err(_) => {},
        _ => {}
    };
} 

fn process_folder(folder_path:&String,  ot_slicer: &mut Slicer, params: &ProcessingPreset) -> u16 {
    let check_file: &Path = &folder_path.as_ref();
    let mut processed_files : u16 = 0;
    let  octatrack_file = params.ot_file.clone();
    let random_files = params.ot_random;

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
            ot_slicer.clear();

            // Create output folder
            let mut output_folder = folder_path.clone();
            output_folder.push_str("/output/");
            createdir(output_folder.clone());
            ot_slicer.output_folder = output_folder;
            

            // Set OT file to the same name as the folder
            ot_slicer.output_filename = check_file.file_name().unwrap().to_str().unwrap().to_string();

            // Add suffix to OT file if num_octa_files is greater than 0
            let mut max_files : usize = valid_files.len();

            if num_octa_files > 1 {
                let suffix = format!("_{}", i + 1);
                ot_slicer.output_filename.push_str(suffix.as_str());
                max_files = octatrack_max_files;
            }

            if random_files {
                // Calculate the number of files in random mode. 
                if valid_files.len() > octatrack_max_files {
                    max_files = octatrack_max_files;
                } else 
                if (valid_files.len() / 8) > 0 {
                    let num_of_groups : usize = valid_files.len() / 8;
                    max_files = num_of_groups * 8;
                }
                else {
                    max_files = valid_files.len() / 2;
                }
            }
    
            // let mut files_to_process: Vec<String> = vec!{};
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

                if file_pos < valid_files.len() { // If sample position is valid
                    let file =  &valid_files[file_pos]; // select file
                    println!("Processing file {}: {}", file_pos, file.display());
                    let file_path : String = file.to_str().unwrap().to_string(); // Copy file path as string

                    // files_to_process.push(file_path);
                    process_file(file_path, params, ot_slicer, false);
                    
                }

            }

            // MULTITHREAD TEST
            // let param_mutex = Arc::new(Mutex::new(params.clone()));
            // let slicer_mutex = Arc::new(Mutex::new(ot_slicer));

            
            // let mut handles = vec!{};

            // for i in files_to_process {
            //     let p_l = Arc::clone(&param_mutex);
            //     let s_l = Arc::clone(&slicer_mutex);
            //     let handle = thread::spawn(move || {
            //             // let mut new_slicer = Slicer::new();
            //             let p = p_l.lock().unwrap();
            //             let mut s = s_l.lock().unwrap();
            //             println!("{}", i);
            //             process_file(i, &p, s, false);
            //     });
            //     handles.push(handle);
            // }

            // for handle in handles {
            //     handle.join().unwrap();
            // }

            // Finish the ot_util process and generate the sample chain .wav and .ot files
            if octatrack_file {
                ot_slicer.generate_ot_file(params.evenly_spaced).unwrap();
            };

        }
    }
    processed_files // Return the number of processed files
}


fn auto_convert(file_path : String, spec: &hound::WavSpec)  -> String {
    
    // Compatible spec
    let compatible_spec = hound::WavSpec {
        channels: 1,
        sample_rate: spec.sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    println!("Converting file {} to compatible format...", file_path);
    let path : &Path = file_path.as_ref(); // Get path of file
    let output_folder = create_output_folder(file_path.clone()); // Create output folder 
    let file_name = path.file_name().unwrap().to_str().unwrap(); // Get the file name
    let new_filename = format!("{}{}", output_folder, file_name); // Create path for new file
    

    assert_eq!(convert_to_format(file_path.clone(), new_filename.clone(), &compatible_spec).success(), true, "Failed to convert file (is SoX installed?).");


    new_filename // Return path of the converted file
}


fn process_file (file_path: String, params: &ProcessingPreset, ot_slicer: &mut Slicer, generate_ot_file: bool) -> ProcessResults {

    // Generates an empty result struct
    let mut results = ProcessResults{
        sample: None,
        generated_files: vec!{}
    };

    // Load WAV file and generate sample data struct
    let sample_data = load_audio_file(file_path, params);

    match sample_data {
        Some(mut sample_data) => {

            if params.ot_file && generate_ot_file {
                // Set OT file to the same name as the folder
                ot_slicer.clear();
                ot_slicer.output_folder = sample_data.destination_folder.clone();
                ot_slicer.output_filename = sample_data.file_name.clone();
            }

            match params.mode.as_str() {
                "trim" => {
                    sample_data = find_regions(sample_data, params);
                    results.generated_files = generate_samples_from_regions(&sample_data, params);
                    if params.ot_file {
                        for i in results.generated_files.clone() {
                            ot_slicer.add_file(i).unwrap();
                        }
                    }
                },
                "scale" => {
                    results.generated_files = generate_sample_scale(&sample_data, params);
                    ot_slicer.output_filename.push_str("_scale");
                    for i in results.generated_files.clone() {
                        if params.ot_file {ot_slicer.add_file(i).unwrap();};      
                    }
                },
                "ref" => {
                    add_reference_track(&sample_data);
                }
                &_ => {}
            };
            if generate_ot_file && params.ot_file {
                ot_slicer.generate_ot_file(params.evenly_spaced).unwrap();
            }
            println!("Finished processing: {}", sample_data.file_path);
            println!("");
            results.sample = Some(sample_data);
        },
        None => {}
    }
    
    results

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

fn create_output_folder (filepath: String) -> String {
    let path : &Path = filepath.as_ref();
    let parentpath = &path.parent().unwrap().to_str().unwrap().to_string().to_owned();
    let mut new_filename  = parentpath.to_owned();
    new_filename.push_str("/output/");
    let output_folder_path :&Path = new_filename.as_ref();
    if !output_folder_path.is_dir() {
        createdir(new_filename.clone());
    };
    new_filename
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

fn convert_to_format(filepath: String, new_filepath: String, specs : &hound::WavSpec) -> std::process::ExitStatus {

    let bits = format!("{}", specs.bits_per_sample);
    let sr  = format!("{}", specs.sample_rate);
    let chs = format!("{}", specs.channels);
    let commands = ["-v", "0.8", filepath.as_str(), "-b", bits.as_str(), "-c", chs.as_str(), new_filepath.as_str(), "rate", "-v", "-I", "-b", "90", "-a", sr.as_str()];
    Command::new("sox").args(&commands).status().expect("Audio convertion failed")
}

fn generate_sample_scale(sample_data: &SampleData, params : &ProcessingPreset) -> Vec<String> {
    let final_folder_path = format!("{}{}_scale",sample_data.destination_folder, sample_data.file_name);
    createdir(final_folder_path.clone());

    let mut new_files : Vec<String> = Vec::new();
  
    println!("Creating chromatic scale folder for {}", sample_data.file_path);
    for i in 0..params.pitch_range {
        let new_pitch = (i as isize - params.pitch_offset) * 100 as isize;
        let new_filepath = format!("{}/{}_{:0>2}.wav", final_folder_path, sample_data.file_name, i + 1);
        println!("{} > {}: {}", sample_data.file_path, new_filepath, new_pitch);
        new_files.push(new_filepath.clone());
        let _result = Command::new("sox").args(&[
            sample_data.file_path.as_str(),
            "-b", "16", "-c", "1",
            new_filepath.as_str(), 
            "pitch", 
            new_pitch.to_string().as_str(),
            "rate", "-v", "-I", "-b", "90", "-a", "44100"
        
        ]).status().expect("Audio convertion failed");
    }
    
    new_files
}

fn add_reference_track(sample_data: &SampleData) {
    println!("Adding pitch track to : {}", sample_data.file_path);

    let new_filepath = format!("{}{}_ref.wav", sample_data.destination_folder, sample_data.file_name);
    let sine_file = format!("{}{}_sine.wav", sample_data.destination_folder, sample_data.file_name);

    let duration_in_ms : f32 = sample_data.duration as f32 / sample_data.spec.sample_rate as f32;
    println!("File length: {}", duration_in_ms);


    let _result = Command::new("sox").args(&["-n", "-r", sample_data.spec.sample_rate.to_string().as_str(), "-b", sample_data.spec.bits_per_sample.to_string().as_str(), sine_file.as_str(), "synth", duration_in_ms.to_string().as_str(), "sine", "130.81"]).status().expect("Audio convertion failed");
    let _result = Command::new("sox").args(&[sample_data.file_path.as_str(), sine_file.as_str(), new_filepath.as_str(), "-MS"]).status().expect("Audio convertion failed");
    fs::remove_file(sine_file).unwrap();
}