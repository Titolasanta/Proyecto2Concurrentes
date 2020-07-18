
use std::io::{Write};

pub fn write_log(msg: std::string::String,lock: &std::sync::Mutex<std::fs::File>) -> () {

	match lock.lock(){
		Ok(file) => match writeln!( &*file, "{}",msg){
			Ok(()) => {},
			Err(e) => panic!("can t write in log {}",e )

		},
		Err(e) => panic!("can t write in log {}",e)
	};
        
       

}