# AudioHit

-----

## About 

AudioHit is a CLI utility written in Rust and designed to trim and fade audio samples (specially single hits such as drum hits and bass notes).  
The app detects the start and end points of the main content of the audio sample based on amplitude (whatever is above a certain threshold), trims the audio file to those start / end points, and apply a linear fade in/out.

The generated samples can also be concatenated into a single .wav file a .ot slice file for the Elektron Octatrack can be generated automatically.

## Limitations

- AudioHit can only process 16-bit mono wave files.

## How to Install

If you're running **macOS**, [download the latest release version](https://github.com/icaroferre/AudioHit/releases). To use it, ```cd``` into the folder and use the ```./audiohit``` command.  
If you're running other OS, you'll need to install the latest version of Rust + Cargo and compile the code for your own system (cd into the folder and ```cargo build --release```).

## How to Use

To use AudioHit, open the terminal and use the ```./audiohit``` command to process files or folder.

To process a single file, use the command ```./audiohit --file name_of_file.wav```.

To batch process an entire folder, use the command ```./audiohit --folder /path/to/folder```.  
Sub-folders found in the specified --folder argument will also be processed (one level). 
 
The new files will placed inside an output folder (which will be created if it doesn't exist).

The fade in, fade out and threshold parameters can also be changed using the arguments ```--fadein```, ```--fadeout```, and ```--thresh``` respectively. Both fade times are set in ms while the threshold value is set in decibels (e.g. -48).
 
When processing an entire folder, the ```--ot_file true``` argument can be used to automatically concatenate all the trimmed files generated by audiohit and generate a .ot slice file for the Elektron Octatrack (via my [ot_utils](https://github.com/icaroferre/ot_utils) Rust library). The final file will be generated to the output folder and will both .wav and .ot files will have the same name.  
If more than 64 samples are found within that folder then multiple OT files will be created, each containing 64 samples / slices.

It's also possible to only generate the Octatrack OT file based on the original samples contained inside the specified ```--folder``` by using the ```--ot_file only``` argument. 

The ```--ot_random true``` argument can be used to generate a single chain (concatenated wav + ot file for the Octatrack) using 64 samples randomly picked from the folder specified in the ```--folder``` argument. AudioHit will make sure not to pick the same file more than once.
 
## Roadmap / To-Do

- Add support for stereo files
- Add support for 24 and 32-bit audio files

## Compiling instructions for different platforms

While a compiled binary for macOS is available on the [Releases page](https://github.com/icaroferre/AudioHit/releases), compiling binaries for different platforms is quite easy with Rust.

Here's a step by step instruction:

1 - [Download the zip file for this repository](https://github.com/icaroferre/AudioHit/archive/master.zip) and unzip it

2 - [Install Rust on your device](https://www.rust-lang.org/tools/install)

3 - Open the terminal and navegate to the the code folder of the repository: ```cd path/to/AudioHit-master/code/```. If you're unsure about the path just enter ```cd ``` and drag and drop the code folder on the terminal window.

4 - Use the command ```cargo build --release``` to build a binary for your system. The cargo system will take care of downloading all the necessary dependencies and compile everything automatically. The final binary will be available at the target / release folder.


----

Created by Ícaro Ferre  
@icaroferre  
[spektroaudio.com](http://spektroaudio.com)
 
 