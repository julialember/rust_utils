#![allow(unused)]
use std::io;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;


#[derive(Debug)]
pub enum GrepError {
    UnknownArgument(String),
    FileError(io::Error),
    NoPattern,
    NoOutFile,
}

impl std::fmt::Display for GrepError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownArgument(e) => write!(f, "grep: unknown argument: {}", e),
            Self::FileError(e) => write!(f, "grep: file error: {}", e),
            Self::NoPattern => write!(f, "grep: no patter"),
            Self::NoOutFile => write!(f, "grep: need the [OUTFILE] near the [-o | --output]"),
        } 
    }
}

impl From<std::io::Error> for GrepError {
    fn from(error: std::io::Error) -> Self {
        GrepError::FileError(error)
    }
}

enum FileToRead {
    Stdin,
    File(BufReader<File>)
}

pub struct Grep<'a> {
    pattern: Option<&'a String>, 
    search_in: FileToRead,
    outfile: Box<dyn Write>,
    ignore_case: bool,
    count: bool,
    line_number: bool,
}

impl<'a> Grep<'a> {
    fn set_pattern(&mut self, pat: &'a String) {
        self.pattern = Some(pat); 
    }
    fn set_search(&mut self, filename: &str) -> Result<(), io::Error> {
        let file = File::open(filename)?;
        self.search_in = FileToRead::File(BufReader::new(file));
        Ok(())
    } 
    fn set_out(&mut self, filename: &str) -> Result<(), io::Error> {
        let file = File::create(filename)?;
        self.outfile = Box::new(BufWriter::new(file));
        Ok(())
    }
    pub fn new(args: &'a Vec<String>) -> Result<Grep<'a>, GrepError>{
        let mut grep = Grep { 
        pattern: None,
        search_in: FileToRead::Stdin, 
        outfile: Box::new(BufWriter::new(io::stdout())),
        ignore_case: false,
        count: false,
        line_number: false,
        };
        let mut i = 1;
        while i < args.len() {
            if args[i].starts_with('-') {
                match args[i].as_str() {
                    "-o" | "--output" => if i + 1 < args.len() {
                            i+=1;
                            grep.set_out(args[i].as_str())?;
                        } else {return Err(GrepError::NoOutFile)} 
                    "-i" | "--ignore-case" => grep.ignore_case = true,
                    "-" => grep.search_in = FileToRead::Stdin,
                    "-c" | "--count" => grep.count = true,
                    "-n" | "--line_number" => grep.line_number = true,
                    _ => return Err(GrepError::UnknownArgument(args[i].clone())),
                } 
            } 
            else {
                if grep.pattern.is_none() {grep.set_pattern(&args[i])}
                else {grep.set_search(args[i].as_str())?}
            }
            i+=1;
        } 
        if grep.pattern.is_none() {Err(GrepError::NoPattern)}
        else {Ok(grep)}
    }
    
    fn check(ig: bool, buffer: &str, pattern: &str) -> bool {
        if ig {
            buffer.to_ascii_lowercase().contains(&pattern)
        } else {
            buffer.contains(pattern)
        }
    }

    pub fn start(mut self) -> Result<(), GrepError>{
        if  let Some(pattern) = self.pattern {
            let pat = if self.ignore_case {&pattern.to_ascii_lowercase()} else {&pattern};
            if  self.count {
                match self.search_in {
                    FileToRead::Stdin => writeln!(self.outfile, "0")?,
                    FileToRead::File(file) => writeln!(self.outfile, "{}",
                        file.lines().filter_map(Result::ok)
                            .filter(|li| Self::check(self.ignore_case, &li, &pat)).count())?,
                }
                self.outfile.flush()?;
                return Ok(());
            }
            match self.search_in {
                FileToRead::Stdin => {
                    let mut line_count = 0;
                    let mut buffer = String::new();
                    while io::stdin().read_line(&mut buffer)? != 0{
                        if Self::check(self.ignore_case, &buffer, pat) {
                            if self.line_number {write!(self.outfile, "{}:{}", line_count, buffer)?;}
                            else {write!(self.outfile, "{}", buffer)?};
                            self.outfile.flush()?;
                        }
                        line_count+=1;
                        buffer.clear();
                    }     
                } 
                FileToRead::File(file) => {
                    for (pos, line) in file.lines().enumerate().filter_map(|(p, res)| res.ok().map(|line| (p, line)) )
                        .filter(|(_, l)| Self::check(self.ignore_case, l, pat)) {
                        if self.line_number {writeln!(self.outfile, "{}:{}", pos+1, line)?;}
                        else {writeln!(self.outfile, "{}", line)?};
                    }     
                    self.outfile.flush()?;
                }
            }
        }
        Ok(())
    }
}

