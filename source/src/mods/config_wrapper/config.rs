
#[derive(Debug)]
pub struct Elements{
    network:String,
    trusted_pubk:Vec<String>,
}

pub fn init()->Elements{
    Elements{network:"seed.yutadd.com".to_string(),trusted_pubk:Vec::new()}
}