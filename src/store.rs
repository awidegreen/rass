extern crate gpgme;

use std::path::PathBuf;
use std::env;
use std::ffi;
use std::fmt;
use std::error;
use std::io;
use std::fs::File;
use std::fs;
use std::io::prelude::*;
use std::result;

use gpgme::Data;

use ::vcs;

pub static PASS_ENTRY_EXTENSION: &'static str = "gpg";
pub static PASS_GPGID_FILE: &'static str = ".gpg-id";

#[derive(Debug)]
pub enum PassStoreError {
    GPG(gpgme::Error),
    Io(io::Error),
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
        }
    }
}
impl error::Error for PassStoreError {
    fn description(&self) -> &str {
        match *self {
            PassStoreError::GPG(_) => "gpg error",
            PassStoreError::Io(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            PassStoreError::GPG(ref err) => Some(err),
            PassStoreError::Io(ref err) => Some(err),
        }
    }
}


/// Represents the underlying directory structure of a password store. 
/// The folder structure is inherit from pass(1).
#[derive(Debug)]
pub struct PassStore {
    passhome: PathBuf,
    entries: Vec<PassEntry>,
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
    pub fn new() -> PassStore {
        let def_path = PassStore::get_default_location();
        let mut store =  PassStore { 
            entries: vec![],
            passhome: def_path.clone(),
            gpgid: String::new(),
            verbose: false,
        };
        store.fill();
        store
    }

    /// Constructs a new `PassStore` using the provided location.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use std::path::PathBuf;
    /// use rass::store::PassStore;
    ///
    /// let p = PathBuf::from("/home/bar/.store");
    ///
    /// let store = PassStore::from(&p);
    ///
    /// ```
    pub fn from(path: &PathBuf) -> PassStore {
        let mut store =  PassStore { 
            entries: vec![],
            passhome: path.clone(),
            gpgid: String::new(),
            verbose: false,
        };
        store.fill();
        store
    }

    /// Set the verbose printouts for the store.
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose
    }

    /// Returns all `PassStore` entries.
    pub fn entries(&self) -> &Vec<PassEntry> {
        &self.entries
    }

    /// Returns the absolute_path of a given `PassEntry`.
    pub fn absolute_path(&self, entry: &PassEntry) -> PathBuf {
        self.passhome.clone().join(&entry.path)
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
            if p.is_file() {
                if p.extension() == Some(ending) {
                    let e =  PassEntry::new(&p, &self.passhome);
                    self.entries.push(e);
                    continue;
                }
                let gpgid_fname = ffi::OsStr::new(PASS_GPGID_FILE);
                if p.file_name() == Some(gpgid_fname) {
                    self.gpgid = match get_gpgid_from_file(&p) {
                        Ok(id) => id,
                        Err(_) => panic!("Unable to open file: {}", 
                                         PASS_GPGID_FILE)
                    }
                }
            }
            else if p.is_dir() {
                self.parse(&p);
            }
        }
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

    /// Find an entry in the `PassStore` by its location.
    pub fn find_by_location<S>(&self, query: S) -> Vec<&PassEntry> 
        where S: Into<String> {

        let query = query.into();
        let r: Vec<&PassEntry> = self.entries
            .iter()
            .filter(|&x| x.location().contains(&query))
            .collect();
        
        r
    }

    /// Find and returns a Vector of `PassEntry`s by its name.
    pub fn find_by_name<S>(&self, query: S) -> Vec<&PassEntry> 
        where S: Into<String> {

        let query = query.into();
        let r: Vec<&PassEntry> = self.entries
            .iter()
            .filter(|&x| x.name().contains(&query) )
            .collect();
        r
    }

    /// Get a `PassEntry` first based on its location, if not found, try by
    /// its name.
    pub fn get<S>(&self, pass: S) -> Option<&PassEntry> 
        where S: Into<String>{

        let pass = pass.into();
        let r = self.entries.iter().find(|&x| x.location() == pass);
        match r {
            Some(_) => r,
            None => { 
                self.entries.iter().find(|&x| x.name() == pass)
            }
        }
    }

    /// Reads and returns the content of the given `PassEntry`. The for the 
    /// gpg-file related to the `PassEntry` encrypt. 
    pub fn read(&self, entry: &PassEntry) -> Option<String> {
        let p = self.passhome.clone().join(&entry.path);
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
                       entry: PassEntry) -> Result<()> 
            where VCS: vcs::VersionControl 
    {
        if self.verbose {
            println!("Remove {}", entry.location());
        }

        self.entries.retain(|ref e| e.location() != entry.location());
    
        let mut p = self.absolute_path(&entry);
        p.set_extension(PASS_ENTRY_EXTENSION);
        println!("{:?}", p);
        try!(fs::remove_file(&p));

        try!(vcs.remove(p.to_str().unwrap()));
        try!(vcs.commit(&format!("Remove {} from store.", entry.location())));

        Ok(())
    }
}

/// Represents an entry in a `PassStore` relative to the stores location.
#[derive(Debug, Clone)]
pub struct PassEntry {
    path: PathBuf,
}

impl PassEntry {
    /// Constructs a new `PassEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    /// use rass::store::PassEntry;
    ///
    /// let entry_path = PathBuf::from("/home/bar/.store/foobar.gpg");
    /// let store_path = PathBuf::from("/home/bar/.store");
    ///
    /// let entry = PassEntry::new(&entry_path, &store_path);
    /// ```
    /// 
    pub fn new(path: &PathBuf, passhome: &PathBuf) -> PassEntry {
        let path = ::util::strip_path(path, passhome);

        PassEntry {
            path: path,
        }
    }

    /// Returns the name of the `PassEntry` as String. Returns only the name 
    /// of the gpg file without the gpg file extension.
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use rass::store::PassEntry;
    ///
    /// let entry_path = PathBuf::from("/home/bar/.store/bla/foobar.gpg");
    /// let store_path = PathBuf::from("/home/bar/.store");
    ///
    /// let entry = PassEntry::new(&entry_path, &store_path);
    ///
    /// assert_eq!("foobar", entry.name()); 
    /// ```
    pub fn name(&self) -> String {
        let mut tp = self.path.clone();
        tp.set_extension("");
        tp.file_name().unwrap().to_str().unwrap().to_string()
    }

    /// Returns the path of the `PassEntry` as String, relative to the store and
    /// without gpg file extension.
    ///
    /// # Examples
    /// ```
    /// use std::path::PathBuf;
    /// use rass::store::PassEntry;
    ///
    /// let entry_path = PathBuf::from("/home/bar/.store/bla/foobar.gpg");
    /// let store_path = PathBuf::from("/home/bar/.store");
    ///
    /// let entry = PassEntry::new(&entry_path, &store_path);
    ///
    /// assert_eq!("bla/foobar", entry.location()); 
    /// ```
    pub fn location(&self) -> String {
        let mut tp = self.path.clone();
        tp.set_extension("");
        tp.to_str().unwrap().to_string()
    }
}

fn get_gpgid_from_file(path: &PathBuf) -> Result<String> {
    let f = try!(fs::File::open(path));
    let mut  reader = io::BufReader::new(f);

    let mut buffer = String::new();
    reader.read_line(&mut buffer).unwrap();
    Ok(buffer.trim().to_string())
}
