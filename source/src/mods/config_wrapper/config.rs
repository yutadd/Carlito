
#[derive(Debug)]
pub struct Elements{
    network:String,
}

pub fn init()->Elements{
    Elements{network:"seed.yutadd.com".to_string()}
}