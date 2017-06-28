// Use getops
extern crate getopts;

extern crate hyper;

use std::io::Read;
use hyper::{Client};

use getopts::Options;

use std::env;
use std::result;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;


fn main() {
    println!("Hello, world!");

		// Prints each argument on a separate line
		// for argument in env::args() {
		//     println!("{}", argument);
		// }

		// Using args() instead of args_os(), cause they never panic
		let commandline_args: Vec<_> = env::args().collect();
		let program = commandline_args[0].clone();

		// Use the getopts package Options structure
		let mut opts = Options::new();
    
    // Create the file argument
    opts.optopt("f", "", "Give the address of the file to download", "NAME");
    // Create help flag (-h or --help)
    opts.optflag("h", "help", "Print this help menu");
    // Create version l
    opts.optflag("V", "version", "Check the version you're running");


    // Use the innate parse() method
    // https://doc.rust-lang.org/1.2.0/book/match.html
    // https://doc.rust-lang.org/std/macro.panic.html
    let matches = match opts.parse(&commandline_args[1..]){
    	Ok(m) => {m}
    	Err(f) => {panic!(f.to_string())}
    };

    if matches.opt_present("h"){
    	  println!("THE FOOKIN' HELP IS HERE!");
    	  let brief = format!("Usage: {} FILE [options]", program);
    		print!("{}", opts.usage(&brief));
    		return;
    } else if matches.opt_present("V"){
    	  println!("THE FOOKIN' VERSION IS HERE!");
    	  return;
    }

    let source = matches.opt_str("f");
    match source {
        Some(x) => println!("THE FOOKIN' OUTPUT IS HERE: {}", x),
        None => println!("No input file specified"),
    }

    let client = Client::new();

    //let url = "http://cdn5.thr.com/sites/default/files/2011/06/nicolas_cage_2011_a_p.jpg";
    let url = "http://home.agh.edu.pl/~morchel/files/ogolne-zasady-etycznego-postepowania-inzynierow-i-technikow.pdf";

    

    let mut response = match client.get(url).send() {
        Ok(response) => response,
        Err(_) => panic!("Whoops."),
    };

    let mut buf: Vec<u8>= Vec::new();

    let path = Path::new("ORCZI.pdf");
		response.read_to_end(&mut buf).unwrap(); 

    let display = path.display();
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(file) => file,
    };

    match file.write_all(&buf[..]) {
        Err(why) => {
            panic!("couldn't write to {}: {}", display,
                                               why.description())
        },
        Ok(_) => println!("successfully wrote to {}", display),
    }

}