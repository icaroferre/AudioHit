# AudioHit

-----

## About 

AudioHit is a CLI utility written in Rust and designed to trim and fade audio samples (specially single hits such as drum hits and bass notes).  
The app detects the start and end points of the main content of the audio sample based on amplitude (whatever is above a certain threshold), trims the audio file to those start / end points, and apply a linear fade in/out.

## Limitations

- AudioHit can only process 16-bit mono wave files.

## How to Install

If you're running **macOS**, download the [ZIP of this repo](https://github.com/icaroferre/AudioHit/archive/master.zip) and copy the audiohit binary file from the ```code/target/debug``` folder to your ```/usr/bin``` folder.
If you're running other OS, you'll need to install the latest version of Rust + Cargo and compile the code for your own system (cd into the folder and ```cargo build```).

## How to Use

To use AudioHit, open the terminal and use the ```audiohit``` command to process files or folder.

To process a single file, use the command ```audiohit --file name_of_file.wav```.

To batch process an entire folder, use the command ```audiohit --folder /path/to/folder```.
 
 The new files will placed inside an output folder (which will be created if it doesn't exist).
 
 The fade in, fade out and threshold parameters can also be changed using the arguments ```--fadein```, ```--fadeout```, and ```-thresh``` respectively. Both fade times are set in ms while the threshold value is set in decibels.
 
----

Created by Ícaro Ferre  
@icaroferre  
[spektroaudio.com](http://spektroaudio.com)
 
 