use std::io::{Write};

pub fn write_log(msg: std::string::String, lock: &std::sync::Mutex<std::fs::File>, write: &bool) -> () {
	if *write {
		match lock.lock(){
			Ok(file) => match writeln!(&*file, "{}", msg){
				Ok(()) => (),
				Err(e) => panic!("Failed to write to log! {:?}", e)
			},
			Err(e) => panic!("Poisoned! {:?}",e)
		};
	}
}
