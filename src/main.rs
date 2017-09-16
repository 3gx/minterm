extern crate csv;
use std::io;

// A single entry in a truth table.
#[derive(Clone, Debug, PartialEq)]
struct Entry {
	input: Vec<bool>,
	output: Vec<bool>,
}
impl Entry {
	fn default() -> Self { Entry{input: vec![], output: vec![]} }
	#[allow(dead_code)]
	fn new(inp: Vec<bool>, outp: Vec<bool>) -> Self {
		Entry{input: inp.clone(), output: outp.clone()}
	}

	fn clear(&mut self) {
		self.input.clear();
		self.output.clear();
	}
}

struct Truth {
	table: Vec<Entry>,
}

impl Truth {
	fn default() -> Self { Truth{table: vec![]} }
	#[allow(dead_code)]
	fn new(inp: Vec<Vec<bool>>, outp: Vec<Vec<bool>>) -> Self {
		assert_eq!(inp.len(), outp.len());
		let mut entlist: Vec<Entry> = vec![];
		for i in 0..inp.len() {
			entlist.push(Entry::new(inp[i].clone(), outp[i].clone()));
		}
		Truth{table: entlist}
	}

	fn solution(&self, inp: Vec<bool>) -> Vec<bool> {
		let foo = self.table.iter().find(|tbl| { tbl.input == inp });
		match foo {
			None => panic!("cannot find bit pattern {:?}", inp),
			Some(x) => x.output.clone(),
		}
	}

	fn len(&self) -> usize { return self.table.len() }
}

fn main() {
	let input_bits = 7;
	let output_bits = 4;
	let header_lines = 2;
	let tbl = parse(io::stdin(), header_lines, input_bits, output_bits);
	for ent in tbl.table.iter() {
		if ent.input.len() != input_bits {
			println!("Incorrect number of bits ({}, should be {}) for elem {:?}.",
			         ent.input.len(), input_bits, ent.input);
			std::process::exit(1);
		}
	}
	let two: i32 = 2;
	if tbl.len() != two.pow(input_bits as u32) as usize {
		println!("Table is too short ({} elems) for {} bits.", tbl.len(),
		         input_bits);
		std::process::exit(1);
	}
	println!("Parsed truth table with {} input bits -> {} output bits",
	         input_bits, output_bits);
	println!("({} input lines.)", tbl.len());

	let gray = gray_code(input_bits);
	for g in gray.iter() {
		assert!(g.len() == input_bits);
		for bit in g.iter() {
			print!("{}", if *bit { 1 } else { 0 });
		}
		println!("");
	}
	let blah = vec![false,false,false,true,true,false,false];
	println!("soln: {:?}", tbl.solution(blah));
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
	Truth {
	let mut rdr = csv::ReaderBuilder::new()
		.has_headers(false)
		.from_reader(data);
	let mut iter = rdr.records();
	let mut line: usize = 0;
	for _ in 0..nheader { // skip header lines.
		iter.next();
		line = line + 1;
	}
	let mut tbl = Truth::default();
	let mut ent = Entry::default();

	for result in iter {
		ent.clear();

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
			ent.input.push(on);
		}

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
			ent.output.push(on);
		}
		tbl.table.push(ent.clone());
		ent.clear()
	}
	return tbl;
}

#[cfg(test)]
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
		let tbl = parse(eg.as_bytes(), 2, 8, 4);
		// should be the same number of lines:
		assert_eq!(tbl.len(), 2);
	}
}
