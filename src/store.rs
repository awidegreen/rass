use std::path::PathBuf;
use std::env;
use std::fs;
use std::ffi;
use std::io;
use std::io::prelude::*;

use gpgme;
use gpgme::Data;

pub static PASS_ENTRY_EXTENSION: &'static str = "gpg";

pub struct PassStore {
    entries: Vec<PassEntry>,
    passhome: PathBuf,
}       


impl PassStore {
    pub fn new() -> PassStore {
        let def_path = PassStore::get_default_location();
        let mut store =  PassStore { 
            entries: vec![],
            passhome: def_path,
        };
        store.fill();
        store
    }

    pub fn from(path: &PathBuf) -> PassStore {
        let mut store =  PassStore { 
            entries: vec![],
            passhome: path.clone(),
        };
        store.fill();
        store
    }

    fn add(&mut self, entry: PassEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &Vec<PassEntry> {
        &self.entries
    }

    fn fill(&mut self) {
        let t = self.passhome.clone();
        self.parse(&t);
    }

    fn parse(&mut self, path: &PathBuf) {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let p = entry.path();

            if p.ends_with(".git") {
                continue;
            }

            let ending = ffi::OsStr::new(PASS_ENTRY_EXTENSION);
            if p.is_file() && p.extension() == Some(ending) {
                let e =  PassEntry::new(&p);
                self.add(e);
            }
            else if p.is_dir() {
                self.parse(&p);
            }
        }
    } 
    
    pub fn get_default_location() -> PathBuf {
        let mut passhome = env::home_dir().unwrap();
        passhome.push(".password-store");
        passhome
    }

    pub fn find_by_location<S>(&self, query: S) -> Option<Vec<&PassEntry>> 
        where S: Into<String> {

        let query = query.into();
        let r: Vec<&PassEntry> = self.entries
            .iter()
            .filter(|&x| x.location.contains(&query))
            .collect();
        
        match r.len() {
            x if x > 0 => Some(r),
            _ => None,
        }
    }

    pub fn find_by_name<S>(&self, query: S) -> Option<Vec<&PassEntry>> 
        where S: Into<String> {

        let query = query.into();
        let r: Vec<&PassEntry> = self.entries
            .iter()
            .filter(|&x| x.name.contains(&query) )
            .collect();
        
        match r.len() {
            x if x > 0 => Some(r),
            _ => None,
        }
    }

    pub fn get<S>(&self, pass: S) -> Option<&PassEntry> 
        where S: Into<String>{

        let pass = pass.into();
        let r = self.entries
            .iter()
            .find(|&x| x.location == pass);
        match r {
            Some(_) => r,
            None => { 
                self.entries
                    .iter()
                    .find(|&x| x.name == pass)
            }
        }
    }
}

pub struct PassEntry {
    fqn: PathBuf,
    location: String,
    name: String,
}

impl PassEntry {
    fn new(path: &PathBuf) -> PassEntry {
        let fname = path.file_name().unwrap().to_str().unwrap();
        if !path.is_file() {
            panic!("{} is not a file", fname);
        }

        let name = match fname.rfind(PASS_ENTRY_EXTENSION) {
            Some(x) => {
                let (p, _) = fname.split_at(x-1);
                p.to_string()
            },
            None => fname.to_string()
        };

        let mut loc = ::util::strip_path(path, &PassStore::get_default_location());
        loc.set_extension("");

        PassEntry {
            fqn: path.clone(),
            location: loc.to_str().unwrap().to_string(),
            name: name,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn location(&self) -> &String {
        &self.location
    }

    pub fn fqn(&self) -> &str {
        &self.fqn.to_str().unwrap()
    }

    pub fn read(&self) -> Option<String> {
        let mut input = match Data::load(self.fqn.as_path()) {
            Ok(input) => input,
            Err(_) => {
                return None;
            }
        };

        let mut ctx = gpgme::create_context().unwrap();
        let _ = ctx.set_protocol(gpgme::PROTOCOL_OPENPGP);
        let mut output = Data::new().unwrap();
        match ctx.decrypt(&mut input, &mut output) {
            Ok(..) => (),
            Err(_) => {
                return None;
            }
        }

        let mut result = String::new();
        let _ = output.seek(io::SeekFrom::Start(0));
        let _ = output.read_to_string(&mut result);

        Some(result)
    }

}
