use std::fs::File;
use std::path::Path;
use std::io::{prelude::*, ErrorKind};
use crate::mods::util::system;
pub fn create_file_exit_on_fail(name:String)->Option<File>{
    match File::create(name){
        Ok(file)=>Some(file),
        Err(E)=>{print!("{}",E.to_string());None},
    }
}
pub fn get_file(name:String)->File{
    let f;
    if(!Path::new(&name).exists()){
       f =create_file_exit_on_fail(name).expect("err on key_agent.rs:19");//this error message won't show.
    }else{
        match File::open(name){
            Ok(file)  => f=file,
            Err(e) => {system::exit_with_error(format!("ERROR->{}",e.to_string()));}
        };
    }
    f
}