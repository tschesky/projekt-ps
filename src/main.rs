extern crate getopts;
extern crate hyper;
extern crate futures;
extern crate tokio_core;
extern crate hyper_tls;
extern crate pretty_env_logger;
extern crate ftp;

use std::io::Read;
use getopts::Options;
use std::str;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::io::stdin;
use std::env;
use std::io::{self, Write};
use futures::Future;
use futures::stream::Stream;
use hyper::Client;
use ftp::FtpStream;


fn main() {
		const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
		pretty_env_logger::init().unwrap();

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

    // Handle help flags
    if matches.opt_present("h"){
    	  let brief = format!("Usage: {} FILE [options]", program);
    		print!("{}", opts.usage(&brief));
    		return;
    } else if matches.opt_present("v"){
    	  println!("Version: v{}", VERSION.unwrap_or("unknown"));
    	  return;
    }

    // Check if the input file has been specified
    let input = if !matches.free.is_empty(){
    	matches.free[0].clone()
    } else {
    		let brief = format!("Usage: {} FILE [options]", program);
    		print!("{}", opts.usage(&brief));
    		return;
    };

    // Check if the destination is empty - if so, we extract the name from given source path
    let dest = match matches.opt_str("d") {
        Some(x) => x,
        None => extract_file_name_if_empty_string(input.clone()),
    };

    // Get URL to see what type of protocol we're dealing with
    let url = input.clone();
    let url = url.parse::<hyper::Uri>().unwrap();

    // Depending on the protocol - call appropriate functions
    match url.scheme(){
    	Some("http") => http_download_single_file(url, &dest[..]),
    	Some("https") => https_download_single_file(url, &dest[..]),
    	Some("ftp") => ftp_download_single_file(input, &dest[..]),
    	// Some("ftps") => ftps_download_single_file(input, &dest[..]),
    	Some(&_) => panic!("Sorry, unknown protocol!"),
    	None => panic!("Sorry, no protocol given!"),
    }
}

// Download a single file form FTP server
// fn ftps_download_single_file(input: std::string::String,  destination: &str){

// }


// Download a single file form FTP server
fn ftp_download_single_file(input: std::string::String,  destination: &str){
		let (host, directory, file) = parse_data_from_ftp_fullpath(input.clone());

		// Create a connection to an FTP server and authenticate to it.
    let mut ftp_stream = FtpStream::connect(host).unwrap_or_else(|err|
    		panic!("{}", err)
    );

    // Set transfer_type to binary so we can properly transport images
    let _ = ftp_stream.transfer_type(ftp::types::FileType::Binary);

    let (user, password) = parse_userdata_from_ftp_fullpath(input);
    let _ = ftp_stream.login(&user[..], &password[..]).unwrap();
    
    // Change into a new directory, relative to the one we are currently in.
    let _ = ftp_stream.cwd(&directory[..]).unwrap();

    let path = Path::new(destination);
    let display = path.display();


    let reader = ftp_stream.get(&file).unwrap();
    let iterator = reader.bytes();

    //Open a file in write-only mode, returns `io::Result<File>`
		let mut local_file = match File::create(&path) {
		   Err(why) => panic!("couldn't create {}: {}",
		                      display,
		                      why.description()),
		   Ok(file) => file,
		};

    for byte in iterator {
    	// println!("{}", byte.unwrap());
    	match local_file.write(&[byte.unwrap()])  {
				Err(why) => {
			      panic!("couldn't write to {}: {}", display,
			                                         why.description())
			  },
			  Ok(_) => (),
			};
    }

    let _ = local_file.flush();

    //  -- BufReader, iteracja po byte'ach --
   	//  let mut reader = ftp_stream.get(file).unwrap();
    
   	//  //Open a file in write-only mode, returns `io::Result<File>`
   	//  let mut local_file = match File::create(&path) {
   	//      Err(why) => panic!("couldn't create {}: {}",
   	//                         display,
   	//                         why.description()),
   	//      Ok(file) => file,
   	//  };

   	//  loop{
   	//  		let chunk = read_n(&mut reader, 5);
   	//  		match chunk {
   	//  				Ok(v) => match io::stdout().write_all(&v)  {
		//     					Err(why) => {
		// 			            panic!("couldn't write to {}: {}", display,
		// 			                                               why.description())
		// 			        },
		// 			        Ok(_) => (),
		//     	},
   	//  				Err(0) => return,
   	//  				Err(_) => panic!("OMG!"),
   	//  		};
  	// }

    // -- simple_retr --
    // let remote_file = ftp_stream.simple_retr("file").unwrap();
    // println!("Read file with contents\n{}\n", str::from_utf8(&remote_file.into_inner()).unwrap());

    // Terminate the connection to the server.
    let _ = ftp_stream.quit();
}

#[allow(dead_code)]
fn read_n<R>(reader: R, bytes_to_read: u64) -> Result<Vec<u8>, i32>
    where R: Read,
{
    let mut buf = vec![];
    let mut chunk = reader.take(bytes_to_read);
    let status = chunk.read_to_end(&mut buf);
    // Do appropriate error handling
    match status {
        Ok(0) => Err(0),
        Ok(_) => Ok(buf),
        _ => panic!("Didn't read enough"),
    }
}

// Function that uses futures
#[allow(dead_code)]
#[allow(unused_variables, unused_mut)]
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

fn extract_file_name_if_empty_string(fullpath: std::string::String) -> std::string::String {
			let split: Vec<&str> = fullpath.split("/").collect();
			std::string::String::from(*split.last().unwrap())
}

fn parse_data_from_ftp_fullpath(input: std::string::String) -> (std::string::String, std::string::String, std::string::String){
		let replace = input.replace("ftp://", "");
		let split: Vec<&str> = replace.split("/").collect();
		let split2 = split.clone();
		let split3: Vec<&str> = split2.first().unwrap().split("@").collect();

		let host = split3.last().unwrap();
		let proper_host = format!("{}:21", host);
		let file = split.last().unwrap();
		let directory = split[1..split.len()-1].join("/");

		(proper_host, directory, std::string::String::from(*file))
}

fn parse_userdata_from_ftp_fullpath(input: std::string::String) -> (std::string::String, std::string::String){
		let replace = input.replace("ftp://", "");
		let mut username = std::string::String::new();
		let mut password = std::string::String::new();

		if replace.contains("@") {
				let split: Vec<&str> = replace.split("@").collect();
				let split2: Vec<&str> = split.first().unwrap().split(":").collect();
				username = std::string::String::from(*split2.first().unwrap()).clone();
				password = std::string::String::from(*split2.last().unwrap()).clone();
		} else {
				println!("User: ");
				stdin().read_line(&mut username).expect("Did not enter a correct string");
		    if let Some('\n')=username.chars().next_back() {
		        username.pop();
		    }
		    if let Some('\r')=username.chars().next_back() {
		        username.pop();
		    }
				println!("Password: ");
				stdin().read_line(&mut password).expect("Did not enter a correct string");
		    if let Some('\n')=password.chars().next_back() {
		        password.pop();
		    }
		    if let Some('\r')=password.chars().next_back() {
		        password.pop();
		    }
		}

		(username, password)
}