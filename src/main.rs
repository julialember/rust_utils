use std::env;

mod utils {
    pub mod grep;
    pub mod cat;
}

//grep 
#[allow(unused)]
use utils::grep::{ Grep, GrepError};
//cat
#[allow(unused)]
use utils::cat::{Cat, CatError};

fn main() -> Result<(), CatError>{
    let args: Vec<String> = env::args().collect::<Vec<String>>();
    let cat = Cat::new(args)?;
    cat.start()?;
    Ok(())
}
