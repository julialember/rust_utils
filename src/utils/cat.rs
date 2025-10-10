use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::io;

#[derive(Debug)]
pub enum CatError {
    UnknownArgument(String),
    FileError(io::Error),
    NoOutFile,
   // UnepectedFlag(String),
}

impl std::fmt::Display for CatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownArgument(e) => write!(f, "cat: unknown argument: {}", e),
            Self::FileError(e) => write!(f, "cat: file error: {}", e),
            Self::NoOutFile => write!(f, "cat: need [OUTPUT] file near the [TO]"),
         //   Self::UnepectedFlag(e) => write!(f, "you can't use the {} flag",  e),
        } 
    }
}

impl From<std::io::Error> for CatError {
    fn from(error: std::io::Error) -> Self {
        CatError::FileError(error)
    }
}


enum ReadFile {
    Stdin,
    File(BufReader<File>)
}

pub struct Cat {
    outfile: Box<dyn Write>,
    files_to_read: Vec<ReadFile>,
    numer_lines: bool,
    end_lines: bool
}

impl Cat {
    fn set_out(&mut self, filename: &String) -> Result<(), CatError> {
        let file = File::create(filename)?; 
        self.outfile = Box::new(BufWriter::new(file));
        Ok(())
    }
    fn add_to_read(&mut self, filename: &String) -> Result<(), CatError> {
        if filename == "-" {
            self.files_to_read.push(ReadFile::Stdin);
        } else {
            self.files_to_read.push(ReadFile::File(BufReader::new(File::open(filename)?)));
        }
        Ok(())
    }
    pub fn new(args: Vec<String>) -> Result<Cat, CatError>{
        let mut cat = Cat {
            outfile: Box::new(BufWriter::new(io::stdout())),
            files_to_read: Vec::new(),
            numer_lines: false,
            end_lines: false,
        };
        let mut i = 1;
        while i < args.len() {
            if args[i].starts_with('-') {
                match args[i].as_str() {
                    "-o" | "--output" => if  i + 1 < args.len() {
                        i+=1;
                        cat.set_out(&args[i])?;
                    } else {return Err(CatError::NoOutFile)}
                    "-" | "--stdin" => cat.files_to_read.push(ReadFile::Stdin),
                    "-n" | "--number-lines" => cat.numer_lines = true,
                    "-e" | "-E" | "--End-of-the-line" => cat.end_lines = true,
                    _ => return Err(CatError::UnknownArgument(args[i].clone())),
                }
            }
            else {
                cat.add_to_read(&args[i])?;
            };
            i+=1;
        };
        if cat.files_to_read.is_empty() {cat.files_to_read.push(ReadFile::Stdin)};
        Ok(cat)
    }

    pub fn start(mut self) -> Result<(), CatError> {
        for i in self.files_to_read {
            match i {
                ReadFile::Stdin => {
                    let mut line = 1;
                    let mut buffer = String::new();
                    while io::stdin().read_line(&mut buffer)? != 0 {
                        if self.numer_lines {
                            writeln!(self.outfile, "{}.{}{}", line, buffer.trim_end(), if self.end_lines {"$"} else {""})?;}
                        else {writeln!(self.outfile, "{}{}", buffer.trim_end(), if self.end_lines {"$"} else {""})?};
                        self.outfile.flush()?;
                        line+=1;
                        buffer.clear();
                    }
                } 
                ReadFile::File(mut file) => {
                    if self.numer_lines{
                        for (line_number, buffer) in file.lines().enumerate(){
                            writeln!(self.outfile, "{}.{}{}", line_number+1, buffer?, if self.end_lines {"$"} else {""})?;
                        };
                    } else if self.end_lines{
                        for buffer in file.lines() {
                            writeln!(self.outfile, "{}$", buffer?)?;
                        }
                    }
                    else {io::copy(&mut file, &mut self.outfile)?;};
                    self.outfile.flush()?;
                }
            }
        } 
        Ok(())
    }
}
