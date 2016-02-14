extern crate rass;
extern crate clap;
extern crate rpassword;

use std::io;
use std::io::prelude::*;
use std::env;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, SubCommand};

use rass::store::PassStore;
use rass::vcs;

static STORE_DIR_ENV_NAME: &'static str = "PASSWORD_STORE_DIR";

fn main() {
    let store = match env::var(STORE_DIR_ENV_NAME) {
        Ok(val) => {         
            let p = PathBuf::from(val);
            PassStore::from(&p)
        },
        Err(_)  => PassStore::new(),
    };
    let vcs = vcs::GitWrapper::new(&store.get_location());

    let mut app = PassstoreApp {
        store: store,
    };

    let matches = get_matches();

    if matches.is_present("verbose") {
        app.store.set_verbose(true);
    }
    
    let ran_subcommand = match matches.subcommand() {
        ("find", Some(matches)) =>   { app.find(&matches); true }
        ("insert", Some(matches)) => { app.insert(vcs, &matches); true }
        ("add", Some(matches)) =>    { app.insert(vcs, &matches); true } // alias for insert
        ("show", Some(matches)) =>   { app.show(&matches); true }
        ("ls", Some(matches)) =>     { app.list(&matches); true }
        ("rm", Some(matches)) =>     { app.remove(vcs, &matches); true }
        _ => false
    };

    if !ran_subcommand {              
        if  matches.is_present("PASS") {
            app.show(&matches); 
        }
        else {
            app.list(&matches);
        }
    }
}

#[derive(Debug)]
struct PassstoreApp {
    store: PassStore,
}

impl PassstoreApp {
    fn insert<T: vcs::VersionControl>(&mut self, vcs: T, matches: &ArgMatches) {
        let pass = matches.value_of("PASS").unwrap_or("");

        match self.store.get(pass) {
            Some(_) => { 
                let q = format!("An entry already exists for {}.\
                                Overwrite it? [y/N] ", pass);
                match yes_no(q.as_ref(), YesNoAnswer::NO) {
                    YesNoAnswer::NO  => return,
                    YesNoAnswer::YES => (),
                }
            },
            None => (),
        };

        let multiline = matches.is_present("multiline");

        let stdin = io::stdin();
        let mut buffer = vec![];

        if multiline {
            println!("Enter contents for {} and press Ctrl+D when finsihed:\n", pass);
            match stdin.lock().read_to_end(&mut buffer) {
                Ok(..) => (),
                Err(err) => panic!("Something went wrong: {}", err)
            }
        } else {
            let pw = single_line_password(pass);
            buffer = pw.into_bytes();
            buffer.push('\n' as u8);
        }

        match self.store.insert(vcs, pass, buffer) {
            Ok(_) => (),
            Err(err) => panic!("{}", err)
        }
    }

    fn list(&self, matches: &ArgMatches) {
        for entry in self.store.entries() {
            if matches.is_present("long") {
                println!("{}", self.store.absolute_path(entry).to_str().unwrap())
            }
            else {
                println!("{}", entry.location())
            }
        }
    }

    fn show(&self, matches: &ArgMatches) {
        let pass = matches.value_of("PASS").unwrap_or("");
        match self.store.get(pass) {
            Some(entry) => { 
                match self.store.read(entry) {
                    Some(x) => print!("{}", x),
                    None => println!("Unable to read!"),
                }           
            },
            None => {
                println!("Error: {} is not in the password store.", pass);
            }
        }
    }

    fn find(&self, matches: &ArgMatches) {
        let query = matches.value_of("QUERY").unwrap();
        let print = matches.is_present("print");
        let matches = match matches.is_present("name") {
            true => self.store.find_by_name(query),
            _    => self. store.find_by_location(query),
        };

        for e in matches {        
            if print {
                match self.store.read(e) {
                    Some(x) => println!("{}:\n{}", e.location(), x),
                    None => println!("Unable to read!"),
                }
            }
            else {
                println!("{}", e.location())
            }
        }
    }

    fn remove<T: vcs::VersionControl>(&mut self, vcs: T, matches: &ArgMatches) {
        let pass = matches.value_of("PASS").unwrap_or("");
        let entry = match self.store.get(pass) {
            Some(e) => e.clone(),
            None => {
                println!("Error: {} is not in the password store.", pass);
                return;
            }
        };

        if !matches.is_present("force") {
            let q = format!("Are you sure you would like to delete {}? [y/N]", pass);;
            match yes_no(q.as_ref(), YesNoAnswer::NO) {
                YesNoAnswer::NO  => return,
                YesNoAnswer::YES => (),
            }
        }

        self.store.remove(vcs, entry).unwrap();
    }
}



fn get_matches<'a>() -> clap::ArgMatches<'a> {
    clap::App::new("rass")
        .author("Armin Widegreen, armin.widegreen@gmail.com")
        .version("0.1.0")
        .about("A manager for a pass-store, the command line password manager")
        .arg(Arg::with_name("PASS")
             .help("pass-name which shall be shown, first try pass-name (full path),\
                   if nothing is found, I'll try just the pass name.")
             .required(false)
             .index(1)
             )
        .arg(Arg::with_name("verbose")
             .help("Print verbose information during execution.")
             .long("verbose")
             .short("v"))
        .subcommand(SubCommand::with_name("find")
                    .about("Query a pass store entry")
                    .arg(Arg::with_name("print")
                         .short("p")
                         .long("print")
                         .help("Immediately print all results"))
                    .arg(Arg::with_name("QUERY")
                         .help("Query string use for the find command")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("name")
                         .short("n")
                         .long("name")
                         .help("use name instead of location for find")))
        .subcommand(SubCommand::with_name("show")
                    .about("Show, print a given entry. First try \
                            complete location within the store, afterwards, \
                            if nothing found, just go with the name!")
                    .arg(Arg::with_name("PASS")
                        .help("PASS which shall be shown, first try \
                               pass-name (full path), if nothing is found, I'll\
                               try just the pass name.")
                        .required(true)
                        .index(1)))
        .subcommand(SubCommand::with_name("insert")
                    .about("Inserts a new entry to the store.")
                    .arg(Arg::with_name("multiline")
                         .short("m")
                         .help("Use multiline inport for new entry."))
                    .arg(Arg::with_name("PASS")
                        .required(true)
                        .index(1)))
        .subcommand(SubCommand::with_name("add")
                    .about("Inserts a new entry to the store.")
                    .arg(Arg::with_name("multiline")
                         .short("m")
                         .help("Use multiline inport for new entry."))
                    .arg(Arg::with_name("PASS")
                        .required(true)
                        .index(1)))
                    .about("see insert")
        .subcommand(SubCommand::with_name("ls")
                    .about("List the whole store")
                    .arg(Arg::with_name("long")
                         .help("Print the full qualified location instead.")
                         .short("l")))
        .subcommand(SubCommand::with_name("rm")
                    .about("Remove entry from the store")
                    .arg(Arg::with_name("PASS")
                        .required(true)
                        .index(1))
                    //.arg(Arg::with_name("recursive")
                         //.help("Removes everything recursively.")
                         //.short("r"))
                    .arg(Arg::with_name("force")
                         .short("f")
                         .long("force")
                         .help("Forces to delete an entry, without interaction.")))
        .get_matches()
}

fn single_line_password(pass: &str) -> String {
    let mut stdout = std::io::stdout();
    loop {
        print!("Enter password for {}: ", pass);
        stdout.flush().unwrap();
        let password = rpassword::read_password().unwrap();

        print!("Confirm password for {}: ", pass);
        stdout.flush().unwrap();
        let password_confirm = rpassword::read_password().unwrap();
        if password != password_confirm {
            println!("Error: the entered passwords do not match.");
        } else {
            return password.to_string();
        }
    } 
}

#[derive(Debug)]
enum YesNoAnswer {
    YES,
    NO,
}

fn  yes_no(message: &str, default: YesNoAnswer) -> YesNoAnswer {
    let mut stdout = std::io::stdout();

    print!("{} ", message);
    stdout.flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    match input.trim() {
        "Yes" | "yes" | "Y" | "y" | "YeS" | "YES" => YesNoAnswer::YES,
        "No" | "NO" | "n" | "N"                   => YesNoAnswer::NO,
        _                                         => default,
    }
}
