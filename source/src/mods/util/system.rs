pub fn exit_with_error(message:String){
    eprintln!("[ERROR]Error: {}", message);
        std::process::exit(1)
}