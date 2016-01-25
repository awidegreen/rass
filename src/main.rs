extern crate passman;
extern crate clap;

use clap::{Arg, ArgMatches, App, SubCommand};

use passman::store::PassStore;


fn main() {

    let store = PassStore::new();

    let matches = get_matches();
    
    let ran_subcommand = match matches.subcommand() {
        ("find", Some(matches)) => { find(&store, matches); true }
        ("show", Some(matches)) => { show(&store, matches); true }
        ("ls", Some(matches)) => { list(&store, matches); true }
        _ => false
    };

    if !ran_subcommand {              
        if  matches.is_present("PASS") {
            show(&store, &matches); 
        }
        else {
            list(&store, &matches);
        }
    }
}

fn list(store: &PassStore, matches: &ArgMatches) {
    for entry in store.entries() {
        if matches.is_present("long") {
            println!("{}", entry.fqn())
        }
        else {
            println!("{}", entry.location())
        }
    }
}

fn show(store: &PassStore, matches: &ArgMatches) {
    let pass = matches.value_of("PASS").unwrap_or("");
    match store.get(pass) {
        Some(entry) => { match entry.read() {
            Some(x) => println!("{}:\n{}", entry.location(), x),
            None => println!("Unable to read!"),
        }           
        },
        None => ()
    }
}

fn find(store: &PassStore, matches: &ArgMatches) {
    let query = matches.value_of("QUERY").unwrap();
    let print = matches.is_present("print");
    let matches = match matches.is_present("name") {
        true => store.find_by_name(query),
        _ => store.find_by_location(query),
    };

    match matches {
        Some(x) => {
            for e in x {        
                if print {
                    match e.read() {
                        Some(x) => println!("{}:\n{}", e.location(), x),
                        None => println!("Unable to read!"),
                    }
                }
                else {
                    println!("{}", e.location())
                }
            }
        }
        None => {
            println!("Nothing found!");
        }
    }
}


fn get_matches<'a,'b>() -> clap::ArgMatches<'a,'b> {
    App::new("passman")
        .author("Armin Widegreen, armin.widegreen@gmail.com")
        .version("0.1.0")
        .about("A manager for a pass-store, the command line password manager")
        .arg(Arg::with_name("PASS")
             .help("pass-name which shall be shown, first try pass-name (full path),\
                   if nothing is found, I'll try just the pass name.")
             .required(false)
             .index(1)
             )
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
                    .about("Show, print a given entry. First I will try to \
                            the complete location in the store, afterwards,\
                            if nothing, I will just go with the name!")
                    .arg(Arg::with_name("PASS")
                        .help("PASS-NAME which shall be shown, first try \
                               pass-name (full path), if nothing is found, I'll\
                               try just the pass name.")
                        .required(true)
                        .index(1)))
        .subcommand(SubCommand::with_name("ls")
                    .about("List the whole store")
                    .arg(Arg::with_name("long")
                         .help("Print the full qualified location instead.")
                         .short("l")))
        .get_matches()
}
