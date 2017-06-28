// Use getops
extern crate getopts;

extern crate hyper;

use std::io::Read;

use getopts::Options;
use std::result;
use std::str;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

extern crate futures;
extern crate tokio_core;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate ftp;

use std::env;
use std::io::{self, Write};

use futures::Future;
use futures::stream::Stream;

use hyper::Client;


fn main() {
		const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

		pretty_env_logger::init().unwrap();

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
    opts.optopt("d", "", "Specify destination file", "NAME");
    // Create help flag (-h or --help)
    opts.optflag("h", "help", "Print this help menu");
    // Create version l
    opts.optflag("v", "version", "Check the version you're running");


    // Use the innate parse() method
    // https://doc.rust-lang.org/1.2.0/book/match.html
    // https://doc.rust-lang.org/std/macro.panic.html
    let matches = match opts.parse(&commandline_args[1..]){
    	Ok(m) => { m }  
    	Err(f) => {panic!(f.to_string())}
    };

    if matches.opt_present("h"){
    	  let brief = format!("Usage: {} FILE [options]", program);
    		print!("{}", opts.usage(&brief));
    		return;
    } else if matches.opt_present("v"){
    	  println!("Version: v{}", VERSION.unwrap_or("unknown"));
    	  return;
    }

    let dest = matches.opt_str("d");
    
    let input = if !matches.free.is_empty(){
    	matches.free[0].clone()
    } else {
    		let brief = format!("Usage: {} FILE [options]", program);
    		print!("{}", opts.usage(&brief));
    		return;
    };

    let url = input.clone();
    let url = url.parse::<hyper::Uri>().unwrap();

    match url.scheme(){
    	Some("http") => http_download_single_file(url, &dest.unwrap()[..]),
    	Some("https") => https_download_single_file(url, &dest.unwrap()[..]),
    	Some("ftp") => ftp_download_single_file(input, &dest.unwrap()[..]),
    	Some(&_) => panic!("Sorry, unknown protocol!"),
    	None => panic!("Sorry, no protocol given!"),
    }
}

fn ftp_download_single_file(input: std::string::String,  destination: &str){
		use ftp::FtpStream;
		use std::io::Cursor;

		let mut replace = input.replace("ftp://", "");
		let mut split: Vec<&str> = replace.split("/").collect();

		let host = split.first().unwrap();
		let proper_host = format!("{}:21", host);
		let file = split.last().unwrap();
		let directory = split[1..split.len()-1].join("/");

		println!("{}", proper_host);
		println!("{}", file);
		println!("{}", directory);

		// Create a connection to an FTP server and authenticate to it.
    let mut ftp_stream = FtpStream::connect(proper_host).unwrap_or_else(|err|
    		panic!("{}", err)
    );

    let _ = ftp_stream.login("anonymous", "").unwrap();

    // Get the current directory that the client will be reading from and writing to.
    println!("Current directory: {}", ftp_stream.pwd().unwrap());
    
    // Change into a new directory, relative to the one we are currently in.
    let _ = ftp_stream.cwd(&directory[..]).unwrap();

    println!("Current directory: {}", ftp_stream.pwd().unwrap());

    let path = Path::new(file);
    let display = path.display();

    


    ftp_stream.retr(file, |stream| {
		    let mut buf = Vec::new();
		    // Open a file in write-only mode, returns `io::Result<File>`
		    let mut local_file = match File::create(&path) {
		        Err(why) => panic!("couldn't create {}: {}",
		                           display,
		                           why.description()),
		        Ok(file) => file,
		    };
		    stream.read_to_end(&mut buf).map(|_|
		        match local_file.write_all(&buf)  {
    					Err(why) => {
			            panic!("couldn't write to {}: {}", display,
			                                               why.description())
			        },
			        Ok(_) => (),
    	}
		    ).map_err(|e| ftp::types::FtpError::ConnectionError(e))
		});

    // Retrieve (GET) a file from the FTP server in the current working directory.
    // let remote_file = ftp_stream.simple_retr("file").unwrap();
    // println!("Read file with contents\n{}\n", str::from_utf8(&remote_file.into_inner()).unwrap());

    // Terminate the connection to the server.
    let _ = ftp_stream.quit();
}

// Function that uses futures
fn http_download_single_file_work(url: hyper::Uri,  destination: &str){

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let client = Client::new(&handle);

    let work = client.get(url).and_then(|res| {
        println!("Response: {}", res.status());
        println!("Headers: \n{}", res.headers());

        res.body().for_each(|chunk| {
            io::stdout().write_all(&chunk).map_err(From::from)
        })
    }).map(|_| {
        println!("\n\nDone.");
    });

    core.run(work).unwrap();
}


// Function that downloads a single file
// It doesnt user futures - blocking and not very effective
fn http_download_single_file(url: hyper::Uri, destination: &str){

		let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
		let client = Client::new(&handle);

    let work = client.get(url);
    let reponse = core.run(work).unwrap();

    let buf2 = reponse.body().collect();
    let finally = match core.run(buf2){
    	Ok(res) => res,
    	Err(_) => panic!("OMG"),
    };


    let path = Path::new(destination);

    let display = path.display();
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(file) => file,
    };

    for x in &finally {
    	match file.write_all(&x) {
    		Err(why) => {
            panic!("couldn't write to {}: {}", display,
                                               why.description())
        },
        Ok(_) => (),
    	}
    }

    println!("successfully wrote to {}", display);
}

// Function that downloads a single file
// It doesnt user futures - blocking and not very effective
fn https_download_single_file(url: hyper::Uri, destination: &str){

		let mut core = tokio_core::reactor::Core::new().unwrap();
		let client = Client::configure().connector(::hyper_tls::HttpsConnector::new(4, &core.handle()).unwrap()).build(&core.handle());

    let work = client.get(url);
    let reponse = core.run(work).unwrap();

    let buf2 = reponse.body().collect();
    let finally = match core.run(buf2){
    	Ok(res) => res,
    	Err(_) => panic!("OMG"),
    };


    let path = Path::new(destination);

    let display = path.display();
    // Open a file in write-only mode, returns `io::Result<File>`
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}",
                           display,
                           why.description()),
        Ok(file) => file,
    };

    for x in &finally {
    	match file.write_all(&x) {
    		Err(why) => {
            panic!("couldn't write to {}: {}", display,
                                               why.description())
        },
        Ok(_) => (),
    	}
    }

    println!("successfully wrote to {}", display);
}