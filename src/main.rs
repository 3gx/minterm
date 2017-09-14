extern crate csv;
use std::io;

fn main() {
	let input_bits = 7;
	let output_bits = 4;
	let header_lines = 2;
	let (inp, outp) = parse(io::stdin(), header_lines, input_bits, output_bits);
	if (inp.len()/input_bits) != (outp.len()/output_bits) {
		println!("Different number of input/output lines?  Parse error.");
		std::process::exit(1);
	}
	let two: i32 = 2;
	if inp.len()/input_bits != two.pow(input_bits as u32) as usize {
		println!("Incorrect number of inputs ({}) for {} bits.",
		         inp.len()/input_bits, input_bits);
		std::process::exit(1);
	}
	println!("Parsed truth table with {} input bits -> {} output bits",
	         input_bits, output_bits);
	println!("({} input lines.)", inp.len()/input_bits);

	let gray = gray_code(input_bits);
	for g in gray.iter() {
		for bit in g.iter() {
			print!("{}", if *bit { 1 } else { 0 });
		}
		println!("");
	}
}

// really this returns a Vec<[usize; nbits]>, but Rust's variable-length arrays
// are vectors.
fn gray_code(nbits: usize) -> Vec<Vec<bool>> {
	let gray1: Vec<Vec<bool>> = vec![vec![false], vec![true]];
	let mut cur = gray1;
	for _ in 1..nbits {
		cur = gray_code_r(cur);
	}
	cur
}

// takes an 'n' bit gray code and computes the gray code for n+1 bits
fn gray_code_r(gray: Vec<Vec<bool>>) -> Vec<Vec<bool>> {
	// prepend 0's (false) to the original list
	let list0: Vec<Vec<bool>> =	gray.iter().map(|bitstring| {
		let mut copy = bitstring.clone();
		copy.insert(0, false);
		copy
	}).collect();
	// prepend 1's (true) to the reversed original list
	let mut list1: Vec<Vec<bool>> =	gray.iter().rev().map(|bitstring| {
		let mut copy = bitstring.clone();
		copy.insert(0, true);
		copy
	}).collect();
	// return the concatenation of the old and new lists.
	let mut concat = list0;
	concat.append(&mut list1);
	concat
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
	let mut line: usize = 0;
	for _ in 0..nheader { // skip header lines.
		iter.next();
		line = line + 1;
	}
	let mut inputs: Vec<bool> = vec![];
	let mut outputs: Vec<bool> = vec![];

	for result in iter {
		let record = result.expect("a CSV record");
		line = line + 1;
		for i in 0..nin {
			let on: bool = match record[i].parse::<i32>() {
				Ok(b) => b != 0,
				Err(e) => {
					println!("WARNING: ignoring input '{}' ({}) on line {}:{}",
					         record[i].to_string(), e, line, i);
					false
				},
			};
			inputs.push(on);
		}
		assert_eq!(inputs.len()%nin, 0);

		// we take the right*most* NOUT columns for the outputs.  Note that this is
		// not columns nin through nin+nout: there could be "spacer" columns
		// between the inputs and outputs.
		let mincol = record.len() - nout;
		for j in mincol .. record.len() {
			let on: bool = match record[j].parse::<i32>() {
				Ok(b) => b != 0,
				Err(e) => {
					println!("WARNING: ignoring output '{}' ({}) on line {}:{}",
					         record[j].to_string(), e, line, j);
					false
				},
			};
			outputs.push(on);
		}
		assert!(outputs.len() % nout == 0);
	}
	assert_eq!(inputs.len()/nin, line-nheader);
	assert_eq!(outputs.len()/nout, line-nheader);
	println!("retvals have {}, {} lines.", inputs.len()/nin, outputs.len()/nout);
	return (inputs, outputs);
}

#[test]
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
		let (inp, outp) = parse(eg.as_bytes(), 2, 8, 4);
		// should be the same number of lines:
		assert_eq!(inp.len() % 8, outp.len() % 4);
	}
}
