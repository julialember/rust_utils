#![allow(unused)]
use std::{env, fmt, fs, io::{self, stdin, Read}};

#[derive(Debug)]
enum CryptorErr {
    UnExpectedArg(String),
    FileError(io::Error),
    Help,
    NoFiles(),
    InvalidKey(String),
}

impl fmt::Display for CryptorErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnExpectedArg(e) => write!(f, "unexpected arg: {}", e),
            Self::FileError(e) => write!(f, "error with file: {}", e),
            Self::Help => write!(f, "helping!"),
            Self::InvalidKey(e) => write!(f, "invalid keyword: {}", e),
            Self::NoFiles() => write!(f, "no file to crypy/decrypt"),
        }
    }
}

impl From<io::Error> for CryptorErr {
    fn from(err: io::Error) -> Self {
        CryptorErr::FileError(err)
    }
}


struct Cryptor<'a> {
    filename: Option<&'a String>, 
    keyword: Option<&'a String>,
    mode: bool, 
    without_ask: bool,
}

impl<'a> Cryptor<'a> {
    fn help() -> () {
        println!("you should write:
        [NAME OF THE FILE]
        [MODE] -- ((-c|--crypt)/(-d|--decrypt))
        [ASKING](if uncrypt) -- -wa|--without-ask");
    } 

    fn valid_key(str: &str) -> bool {
        !str.contains(|x: char| !x.is_alphabetic())
    }
    
    fn start(&self) -> Result<(), CryptorErr> {
        let chars: Vec<char> = self.keyword.unwrap().chars().collect();
        let file = self.filename.unwrap();
        if self.mode {
            fs::write(file, Self::crypt(fs::read_to_string(file)?, &chars))?
        } else {
                let text: String = 
                    Self::decrypt(fs::read_to_string(file)?, &chars);  
                if self.without_ask {
                    fs::write(file, text)?;  
                }
                else {
                    for i in text.lines().filter(|x| !x.is_empty()).take(3) 
                        {println!("{}", i)};
                    println!("is it valid?");
                    let mut answer = [0_u8];
                    stdin().read_exact(&mut answer)?;
                    if answer[0] == b'y' {
                        fs::write(file, text)?;
                        println!("file successfyly rewrited!");
                    } else {
                        println!("invalid key!");
                        return Err(
                            CryptorErr::InvalidKey(
                                self.keyword.unwrap().clone()));
                        }
                }
        }
        Ok(())
    }

    fn decrypt(to_decrypt: String, keyword: &[char]) -> String {
        let mut key_index: usize = 0;
        let len = keyword.len();
        
        to_decrypt.chars().map(|x| {
            if x.is_alphabetic() {
                let base = if x.is_lowercase() {b'a'} else {b'A'};
                let key_val = if  base == b'A' 
                { keyword[key_index % len].to_ascii_uppercase()} else {
                    keyword[key_index %len].to_ascii_lowercase()} as u8 - base;
                key_index+=1;
                ((x as u8 - base + 26 - key_val) % 26 + base) as char 

            } else {x}
        }).collect()

    }

    fn crypt(to_crypt: String, keyword: &[char]) -> String {
        let mut key_index: usize = 0;
        let len = keyword.len();

        to_crypt.chars().map(|x| {
            if x.is_alphabetic() {
                let base = if x.is_lowercase() {b'a'} else {b'A'};
                let key_val = if base == b'A' 
                { keyword[key_index % len].to_ascii_uppercase()} else {
                    keyword[key_index % len].to_ascii_lowercase()} as u8 - base;
                key_index+=1;
                ((x as u8 - base + key_val) % 26 + base ) as char 
            } else {x}
            
        }).collect()    
    }

    fn new(vec: &'a [String]) -> Result<Self, CryptorErr> {
        let mut new_c: Cryptor<'a> = Cryptor { 
            filename: None, 
            keyword: None, 
            mode: false, 
            without_ask: false };
        for i in vec {
            if i.starts_with('-') {
                match i.trim() {
                    "-c" | "--crypt" => new_c.mode = true,
                    "-d" | "--decrypt" => new_c.mode = false,
                    "-wa"| "--without-ask" => new_c.without_ask = true,
                    "-h" | "--help" => {
                        Self::help();
                        return Err(CryptorErr::Help);
                    }
                    _ => return Err(CryptorErr::UnExpectedArg(i.clone()))
                }
            } else if new_c.filename.is_none(){
               new_c.filename = Some(i);
            } else {
                if !Self::valid_key(i) {
                    return Err(CryptorErr::InvalidKey(i.clone()));
                } 
                new_c.keyword = Some(i);
            }
             
        }
        if new_c.filename.is_none() {
            return Err(CryptorErr::NoFiles());
        } else if new_c.keyword.is_none() {
            return Err(CryptorErr::InvalidKey(String::from("[NO KEY ARGUMENT]")));
        }
        Ok(new_c)
    }
}

