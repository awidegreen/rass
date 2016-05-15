extern crate rasslib;
#[macro_use]
extern crate clap;
extern crate rpassword;

use std::io;
use std::io::prelude::*;
use std::env;
use std::path::PathBuf;
use std::process;
 
use clap::{App, Arg, ArgMatches, SubCommand};

use rasslib::store::PassStore;
use rasslib::vcs;

static STORE_DIR_ENV_NAME: &'static str = "PASSWORD_STORE_DIR";

fn main() {
    let store = match env::var(STORE_DIR_ENV_NAME) {
        Ok(val) => {         
            let p = PathBuf::from(val);
            PassStore::from(&p)
        },
        Err(_)  => PassStore::new(),
    };
    let store = match store {
        Ok(s) => s,
        Err(e) => 
        {
            println!("Error parsing store {}", e);
            return
        }
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
        ("git", Some(matches)) =>    { app.git_exec(vcs, &matches); true }
        ("rm", Some(matches)) =>     { app.remove(vcs, &matches); true }
        ("grep", Some(matches)) =>   { app.grep(&matches); true }
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
    fn git_exec<T: vcs::VersionControl>(&self, vcs: T, matches: &ArgMatches) {
        if !matches.is_present("PARAMS") {
            println!("Not git parameters found!");
            process::exit(-1);
        }
;
        let params: Vec<_> = matches.values_of("PARAMS").unwrap().collect();

        if let Ok(r) = vcs.cmd_dispatch(params) {
            process::exit(r.code().unwrap_or(-1))
        }
    }

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
        let pass = matches.value_of("PASS").unwrap_or_default();

        let pass = if pass.ends_with("/") { 
            &pass[0..pass.len()-1]
        } else {
            pass
        };

        if let Some(path) = self.store.get(pass) {
            self.store.print_tree(&path);
        } else {
            println!("Unable to find path for '{}'", pass);
        }
    }

    fn show(&self, matches: &ArgMatches) {
        let pass = matches.value_of("PASS").unwrap_or("");
        if let Some(entry) = self.store.get(pass) {
            if entry.is_leaf() {
                match self.store.read(&entry) {
                    Some(x) => print!("{}", x),
                    None => println!("Unable to read!"),
                }           
            } else {
                self.store.print_tree(&entry);
            }
        } else {
            println!("Error: {} is not in the password store.", pass);
        }
    }

    fn find(&self, matches: &ArgMatches) {
        let query = matches.value_of("QUERY").unwrap();
        let print = matches.is_present("print");
        //let matches = match matches.is_present("name") {
            //true => self.store.find_by_name(query),
            //_    => self. store.find_by_location(query),
        //};
        let matches = self.store.find(query);

        if matches.len() == 1 {                
            let e = &matches[0];
            println!("Only found: '{}'", e);
            if let Some(x) =  self.store.read(e) {
                println!("{}", x);
                return
            } else {
                println!("Unable to read!");
            }
        }

        for e in matches {        
            if print {
                match self.store.read(&e) {
                    Some(x) => println!("{}:\n{}", e, x),
                    None => println!("Unable to read!"),
                }
            }
            else {
                println!("{}", e)
            }
        }
    }

    fn remove<T: vcs::VersionControl>(&mut self, vcs: T, matches: &ArgMatches) {
        let pass = matches.value_of("PASS").unwrap_or("");
        if let Some(entry) = self.store.get(pass) {
            if !matches.is_present("force") {
                let q = format!("Are you sure you would like to delete {}? [y/N]", pass);;
                match yes_no(q.as_ref(), YesNoAnswer::NO) {
                    YesNoAnswer::NO  => return,
                    YesNoAnswer::YES => (),
                }
                let _ = self.store.remove(vcs, &entry);
            }
        } else {
            println!("Error: {} is not in the password store.", pass);
        }
    }

    fn grep(&self, matches: &ArgMatches) {
        let params : Vec<&str>  = matches.values_of("PARAMS").unwrap().collect();
        if params.len() < 1 {
            println!("No search team specified");
            process::exit(-1);
        }

        let searcher = matches.value_of("SEACHER").unwrap_or("grep");
        if let Ok(out) = self.store.grep(&searcher, &params) {
            println!("{}", out);
        }
    }
}



fn get_matches<'a>() -> ArgMatches<'a> {
    App::new("rass")
        .author("Armin Widegreen, armin.widegreen@gmail.com")
        .version(crate_version!())
        .about("A manager for password-store, the *nix command line password manager")
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
        .subcommand(SubCommand::with_name("ls")
                    .about("List the whole store")
                    .arg(Arg::with_name("long")
                         .help("Print the full qualified location instead.")
                         .short("l"))
                    .arg(Arg::with_name("PASS")
                         .help("Print a sub-entry instead of full store.")
                         .default_value("")
                         .required(false)
                         .index(1)))
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
        .subcommand(SubCommand::with_name("git")
                    .about("Dispatch git command to execute within the store")
                    .arg(Arg::with_name("PARAMS")
                         .multiple(true)
                         .required(true)))
        .subcommand(SubCommand::with_name("grep")
                    .about("Greps for given search term in the password store. \
                          Relays the all parameter (except searcher) to to the \
                          command specified in SEACHER parameter, default \
                          'grep'. Therefore standard grep options apply.")
                    .arg(Arg::with_name("SEARCHER")
                         .possible_values(&["ag", "grep", "ack"])
                         .short("s")
                         .long("searcher")
                         .required(false)
                         .default_value("grep"))
                    .arg(Arg::with_name("PARAMS")
                         .multiple(true)
                         .required(true)))
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
