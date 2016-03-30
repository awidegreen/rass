extern crate gpgme;

use std::path::PathBuf;
use std::env;
use std::ffi;
use std::fmt;
use std::convert;
use std::error;
use std::io;
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::result;

use tree;
use gpgme::Data;

use ::vcs;

pub static PASS_ENTRY_EXTENSION: &'static str = "gpg";
pub static PASS_GPGID_FILE: &'static str = ".gpg-id";

#[derive(Debug)]
pub enum PassStoreError {
    GPG(gpgme::Error),
    Io(io::Error),
    Other(String),
}

pub type Result<T> = result::Result<T, PassStoreError>;

impl From<gpgme::Error> for PassStoreError {
    fn from(err: gpgme::Error) -> PassStoreError {
        PassStoreError::GPG(err)
    }
}
impl From<io::Error> for PassStoreError {
    fn from(err: io::Error) -> PassStoreError {
        PassStoreError::Io(err)
    }
}

impl fmt::Display for PassStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PassStoreError::GPG(ref err) => write!(f, "GPG error: {}", err),
            PassStoreError::Io(ref err) => write!(f, "IO error: {}", err),
            PassStoreError::Other(ref err) => write!(f, "Other error: {}", err),
        }
    }
}
impl error::Error for PassStoreError {
    fn description(&self) -> &str {
        match *self {
            PassStoreError::GPG(_) => "gpg error",
            PassStoreError::Io(ref err) => err.description(),
            PassStoreError::Other(ref err) => err,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            PassStoreError::GPG(ref err) => Some(err),
            PassStoreError::Io(ref err) => Some(err),
            PassStoreError::Other(ref _err) => None,
        }
    }
}

pub type PassTree     = tree::Tree<PassEntry>;
pub type PassTreePath = tree::Path<PassEntry>;


/// Represents the underlying directory structure of a password store. 
/// The folder structure is inherit from pass(1).
#[derive(Debug)]
pub struct PassStore {
    passhome: PathBuf,
    entries: PassTree,
    gpgid: String,
    verbose: bool,
}       

/// Represents the underlying directory structure of a password store. 
/// The folder structure is inherit from pass(1).
///
/// On construction of the store, base directory is be walked. All found 
/// gpg-files will be treated as store entries, which are represented by 
/// `PassEntry`. 
impl PassStore {
    /// Constructs a new `PassStore` with the default store location.
    pub fn new() -> Result<PassStore> {
        let def_path = PassStore::get_default_location();
        let mut store =  PassStore { 
            entries: PassTree::default(),
            passhome: def_path.clone(),
            gpgid: String::new(),
            verbose: false,
        };
        try!(store.fill());
        Ok(store)
    }

    /// Constructs a new `PassStore` using the provided location.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use rasslib::store::PassStore;
    ///
    /// let p = PathBuf::from("/home/bar/.store");
    ///
    /// let store = PassStore::from(&p);
    ///
    /// ```
    pub fn from(path: &PathBuf) -> Result<PassStore> {
        let mut store =  PassStore { 
            entries: PassTree::default(),
            passhome: path.clone(),
            gpgid: String::new(),
            verbose: false,
        };
        try!(store.fill());
        Ok(store)
    }

    /// Set the verbose printouts for the store.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose
    }

    /// Returns the absolute_path of a given `PassEntry`.
    pub fn absolute_path(&self, entry: &str) -> PathBuf {
        self.passhome.clone().join(PathBuf::from(entry))
    }

    fn fill(&mut self) -> Result<()> {
        let t = self.passhome.clone();
        self.entries = try!(self.parse(&t));

        Ok(())
    }

    fn parse(&mut self, path: &PathBuf) -> Result<PassTree>
    {
        let entry = PassEntry::new(&path, &self.passhome);

        let mut current = tree::Tree::new(entry);

        if path.is_dir() {
            let rd = match fs::read_dir(path) {
                Err(_) => {
                    let s = format!("Unable to read dir: {:?}", path);
                    return Err(PassStoreError::Other(s))
                },
                Ok(r) => r
            };

            for entry in rd {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue
                };
                let p = entry.path();

                if p.ends_with(".git") {
                    continue;
                }

                let gpgid_fname = ffi::OsStr::new(PASS_GPGID_FILE);
                if p.file_name() == Some(gpgid_fname) {
                    self.gpgid = match get_gpgid_from_file(&p) {
                        Ok(id) => id,
                        Err(_) => panic!("Unable to open file: {}", 
                                            PASS_GPGID_FILE)
                    };
                    continue;
                }
                let ending = ffi::OsStr::new(PASS_ENTRY_EXTENSION);
                if p.is_file() && p.extension() != Some(ending) {
                    continue;
                }

                let sub = try!(self.parse(&p));
                current.add(sub);
            }
        }
        Ok(current)
    } 
    
    fn get_default_location() -> PathBuf {
        let mut passhome = env::home_dir().unwrap();
        passhome.push(".password-store");
        passhome
    }

    /// Returns the location of the `PassStore` as `String`.
    pub fn get_location(&self) -> String {
        self.passhome.to_str().unwrap_or("").to_string()
    }

    /// Find and returns a Vector of `PassEntry`s by its name.
    pub fn find<S>(&self, query: S) -> Vec<PassTreePath> 
        where S: Into<String> {

        let query = query.into();
        self.entries
            .into_iter()
            .filter(|x| x.to_string().contains(&query) )
            .collect()
    }

    /// Get a `PassEntry` first based on its location, if not found, try by
    /// its name.
    pub fn get<S>(&self, pass: S) -> Option<PassTreePath> 
        where S: Into<String> {

        let pass = pass.into();
        //println!("pass term: {}", pass);
        //for e in self.entries.into_iter() {
            //println!("current: {}", e);
            //if e.to_string() == pass {
                //return Some(e);
            //}
        //}
        //return None
        self.entries
            .into_iter()
            .find(|x| x.to_string() == pass)
    }

    /// Reads and returns the content of the given `PassEntry`. The for the 
    /// gpg-file related to the `PassEntry` encrypt. 
    pub fn read(&self, entry: &PassTreePath) -> Option<String> {
        let mut p = self.passhome.clone().join(PathBuf::from(entry.to_string()));
        p.set_extension(PASS_ENTRY_EXTENSION);
        //println!("Read path: {}", p.to_str().unwrap());
        let mut input = match Data::load(p.to_str().unwrap()) {
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

    /// Inserts a new entry into the store. This creates a new encrypted 
    /// gpg-file and add it to version control system, provided via `vcs`.
    pub fn insert<VCS, D>(&mut self, vcs: VCS, entry: &str, data: D) -> Result<()>
            where D: Into<Vec<u8>>, VCS: vcs::VersionControl
    {
        let mut path = self.passhome.clone().join(entry);
        path.set_extension(PASS_ENTRY_EXTENSION);

        let mut ctx = try!(gpgme::create_context());
        let _ = ctx.set_protocol(gpgme::PROTOCOL_OPENPGP);
        let key = try!(ctx.find_key(&*self.gpgid));
        let mut input = try!(Data::from_bytes(data.into()));
        let mut output = try!(Data::new());
    
        let flags = gpgme::ops::ENCRYPT_NO_ENCRYPT_TO 
            | gpgme::ops::ENCRYPT_NO_COMPRESS;

        try!(ctx.encrypt(Some(&key), flags, &mut input, &mut output));
        
        try!(output.seek(io::SeekFrom::Start(0)));
        if self.verbose {
            println!("Going to write file: {}", path.to_str().unwrap_or(""));
        }
        let mut outfile = try!(File::create(&path));
        try!(io::copy(&mut output, &mut outfile));

        try!(vcs.add(path.to_str().unwrap()));
        try!(vcs.commit(&format!("Add given password {} to store.", entry)));

        Ok(())
    }

    /// Removes a given `PassEntry` from the store. Therefore the related
    /// gpg-file will be removed from the file-system and the internal entry 
    /// list. Further the `vcs` will use to commit that change. 
    ///
    /// Note that the `entry` passed into the function shall be a copy of the 
    /// original reference.
    pub fn remove<VCS>(&mut self, 
                       vcs: VCS,
                       entry: &PassTreePath) -> Result<()> 
            where VCS: vcs::VersionControl 
    {
        if self.verbose {
            println!("Remove {}", entry);
        }

        self.entries.remove(entry);
    
        let mut p = self.absolute_path(&entry.to_string());
        p.set_extension(PASS_ENTRY_EXTENSION);
        println!("{:?}", p);
        try!(fs::remove_file(&p));

        try!(vcs.remove(p.to_str().unwrap()));
        try!(vcs.commit(&format!("Remove {} from store.", entry.to_string())));

        Ok(())
    }

    pub fn entries<'a>(&'a self) -> &'a PassTree {
        &self.entries
    }

    pub fn print_tree(&self) {
        let printer = tree::TreePrinter::new("Password Store");
        printer.print(&self.entries);
    }
}

/// Represents an entry in a `PassStore` relative to the stores location.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PassEntry {
    name: String,
}

impl PassEntry {
    /// Constructs a new `PassEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use rasslib::store::PassEntry;
    ///
    /// let entry_path = PathBuf::from("/home/bar/.store/foobar.gpg");
    /// let store_path = PathBuf::from("/home/bar/.store");
    ///
    /// let _entry = PassEntry::new(&entry_path, &store_path);
    /// ```
    /// 
    pub fn new(path: &PathBuf, passhome: &PathBuf) -> PassEntry {
        let path = ::util::strip_path(path, passhome);

        // contains the full path!
        //let name = path.to_str().unwrap().to_string();
        let name = match path.components().last() {
            Some(last) => last.as_os_str().to_str().unwrap().to_string(),
            None => String::from(""),
        };
        PassEntry {
            name: name,
        }
    }
}

impl fmt::Display for PassEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.name.ends_with(".gpg") {
            write!(f, "{}", &self.name[..self.name.len()-4])
        } 
        else { 
            write!(f, "{}", &self.name)
        }
    }
}

impl convert::Into<String> for PassEntry {
    fn into(self) -> String {
        self.name
    }
}

fn get_gpgid_from_file(path: &PathBuf) -> Result<String> {
    let f = try!(fs::File::open(path));
    let mut  reader = io::BufReader::new(f);

    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    Ok(buffer.trim().to_string())
}
