extern crate csv;

fn main() {
}

// parses a truth table in a CSV file with
//   NHEADER header (ignored) rows
//   NIN inputs as the leftmost NIN columns
//   NOUT outputs as the rightmost NOUT columns
fn parse<T: std::io::Read>(data: T, nheader: usize, nin: usize, nout: usize) ->
	(Vec<bool>, Vec<bool>) {
	let mut rdr = csv::ReaderBuilder::new()
		.has_headers(false)
		.from_reader(data);
	let mut iter = rdr.records();
	for _ in 0..nheader { // skip header lines.
		iter.next();
	}
	let mut inputs: Vec<bool> = Default::default();
	let mut outputs: Vec<bool> = Default::default();

	for result in iter {
		let record = result.expect("a CSV record");
		println!("{:?}", record);
		for i in 0..nin {
			println!("input {}: {}", i, record[i].to_string());
			let on: bool = match record[i].parse::<i32>() {
				Ok(b) => b != 0,
				Err(e) => {
					println!("WARNING: ignoring input '{}' ({})",
					         record[i].to_string(), e);
					false
				},
			};
			inputs.push(on);
		}
		assert!(inputs.len() % nin == 0);

		// we take the right*most* NOUT columns for the outputs.  Note that this is
		// not columns nin through nin+nout: there could be "spacer" columns
		// between the inputs and outputs.
		let mincol = record.len() - nout;
		for j in mincol .. record.len() {
			println!("output {}: {}", j, record[j].to_string());
			let on: bool = match record[j].parse::<i32>() {
				Ok(b) => b != 0,
				Err(e) => {
					println!("WARNING: ignoring output '{}' ({})",
					         record[j].to_string(), e);
					false
				},
			};
			outputs.push(on);
		}
		assert!(outputs.len() % nout == 0);
	}
	return (vec![], vec![]);
}

mod test {
	use super::*;

	fn example_head() -> String {
		let s = ",COMPONENTS,,,HAVE,,,,,REQUIRED_VARS includes,,,\n".to_string() +
			"REQUIRED,OGL,GLX,EGL,OGL,GLX,EGL,GL,,OGL,GLX,EGL,GL\n" +
			"0,0,0,0,0,0,0,0,,1,1,0,0\n" +
			"0,0,0,0,0,0,0,1,,0,0,0,1\n";
		s
	}

	#[test]
	fn read_test() {
		let eg = example_head();
		parse(eg.as_bytes(), 2, 8, 4);
	}
}
