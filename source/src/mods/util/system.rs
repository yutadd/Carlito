pub fn exit_with_error(message:String){
    eprintln!("Error: {}", message);
        std::process::exit(1)
}