// Minimize the number of 'if' statements to map between sets of variables.
//
// We are given a truth table with 'b' input bits and 'o' output bits.  The
// task of the program is to minimize the number of if statements required to
// generate that mapping.
// Consider this b=3 o=2 system:
//    000 => 01
//    001 => 10
//    010 => 11
//    011 => 00
//    100 => 11
//    101 => 01
//    110 => 11
//    111 => 00
// Let us call the inputs 'a', 'b', and 'c', and the output 'x' and 'y'.
// Hereafter "a" corresponds to a==1, and "a'" corresponds to a==0.
// In this case, the program should generate something like:
//    if(a'b'):
//      if(c): x = 1
//      else if(c'): y = 1
//    if(ab'):
//      y = 1
//      if(c'): x = 1
//    if(bc'): x = y = 1
// The naive minterms for each this system's outputs would be:
//    x = a'b'c + a'bc' + ab'c' + abc'
//    y = a'b'c' + a'bc' + ab'c' + ab'c + abc'
// Consider a reordering of terms:
//    x = a'b'c + a'bc' + ab'c' + abc'
//    y = a'b'c' + a'bc' + ab'c' + ab'c + abc'
//    =>
//    x = a'b'c         + ab'c' + abc' + a'bc'
//    y = a'b'c' + ab'c + ab'c' + abc' + a'bc'
// Now it is clear that
//    1) both of the 3 final terms are identical for x and y
//    2) both "abc'" and "a'bc'" appear.  The "bc'" is identical here, and
//       because both a and a' appear, we can merge these two, dropping "a".
// Thus:
//    x = a'b'c         + ab'c' + bc'
//    y = a'b'c' + ab'c + ab'c' + bc'
// Now we have a choice.  Note that y includes the two terms "ab'c" and
// "ab'c'"; by argument (2) above, we could merge those two terms.  However, it
// may be more profitable to keep the common subexpression "ab'" so that we can
// merge the solutions for "x" and "y".
extern crate csv;
extern crate docopt;
use docopt::Docopt;
use std::fmt;
use std::fs::File;
use std::path::Path;

const USAGE: &'static str = "
Usage: minterm --table <truth> --ivar=<foo>... --ovar=<bar>...

Options:
";

// A bit in our system is either on, off, or we don't care about it.  In the
// example above, dropping "a" means we don't care about it.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Bit { On, Off, NA }
impl Bit {
	fn new(b: bool) -> Self { if b { Bit::On } else { Bit::Off } }
}
impl fmt::Display for Bit {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Bit::On => write!(f, "1"),
			Bit::Off => write!(f, "0"),
			Bit::NA => write!(f, "x"),
		}
	}
}

// A single entry in a truth table.
#[derive(Clone, Debug, PartialEq)]
struct Entry {
	input: Vec<Bit>,
	output: Vec<bool>,
}
impl Entry {
	fn default() -> Self { Entry{input: vec![], output: vec![]} }
	#[allow(dead_code)]
	fn new(inp: Vec<bool>, outp: Vec<bool>) -> Self {
		let mut bits: Vec<Bit>  = vec![];
		for b in inp.iter() {
			bits.push(Bit::new(*b));
		}
		Entry{input: bits, output: outp.clone()}
	}

	fn clear(&mut self) {
		self.input.clear();
		self.output.clear();
	}

	// returns the number of bits that the inputs differ by.
	fn n_bit_differs(&self, entry: &Entry) -> usize {
		assert!(self.input.len() == entry.input.len());
		let zit = self.input.iter().zip(entry.input.iter());
		return zit.fold(0, |acc, (lhs, rhs)| {
			if *lhs == Bit::NA || *rhs == Bit::NA || *lhs == *rhs {
				acc
			} else {
				acc + 1
			}
		});
	}
}

// A Term is a product that is each state of the input bits.  For example, in
// the system: 00 -> 1, 01 -> 1, 10 -> 0, 11 -> 1, the output equation is:
//   a'b' + a'b + ab
// a'b', a'b, and ab are all terms.  We don't have symbolic names in a program,
// of course, so we just say we have a list where each element is an index and
// a boolean.  So (0, false) means "a'", whereas (1, true) means "b".
type Variable = (usize, bool);
#[derive(Clone, Debug, PartialEq)]
struct Term {
	bits: Vec<Variable>,
}
impl fmt::Display for Term {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// just used as symbolic names, to avoid calling them "index 7" etc.
		let names = ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o',
		             'p','q','r','s','t','u','v','w','x','y','z'];
		for var in self.bits.iter() {
			// completely valid to extend the list of names... but this quickly gets
			// larger than something that would be computable in finite time.
			assert!(var.0 < names.len()); // so we can convert to a "simple" name.
			if var.1 {
				try!(write!(f, "{}", names[var.0]));
			} else {
				try!(write!(f, "{}'", names[var.0]));
			}
		}
		write!(f, "")
	}
}

impl Term {
	#[cfg(test)]
	pub fn new(vals: Vec<Variable>) -> Self { Term{bits: vals} }
	pub fn compute(bits: &Vec<Bit>) -> Self {
		let mut rv = vec![];
		for (i, bit) in bits.iter().enumerate() {
			match *bit {
				Bit::On => rv.push((i, true)),
				Bit::Off => rv.push((i, false)),
				Bit::NA => panic!("NA bits during compute?"),
			};
		}
		Term{bits: rv}
	}
	pub fn len(&self) -> usize { self.bits.len() }
	// true when:
	//   - these are the same terms sans one variable is opposite (a'b' and ab').
	pub fn mergeable(&self, other: &Term) -> bool {
		if self.len() != other.len() { // mismatched terms cannot be merged.
			return false;
		}
		// they must have the same indices (represent the same vars)
		// we should probably just enforce that the ordering is monotonic; we don't
		// right now, which forces us to do a linear search every iteration.
		for t in self.bits.iter() {
			if let None = other.bits.iter().find(|&o| o.0 == t.0) {
				return false;
			}
		}

		// now count the number of bits that differ.
		let mut n_different: usize = 0;
		for t1 in self.bits.iter() {
			if let Some(bit) = other.bits.iter().find(|&o| o.0 == t1.0) {
				if bit.1 != t1.1 {
					n_different = n_different + 1;
				}
			}
		}
		return n_different == 1;
	}

	fn remove_index(&mut self, idx: usize) {
		self.bits.retain(|&b| b.0 != idx);
	}
}

// An equation is a collection of Terms, where the OR of Terms gives the
// result.
#[derive(Clone, Debug, PartialEq)]
struct Equation {
	index: usize,
	terms: Vec<Term>,
	varname: String,
}
impl Equation {
	// Takes a truth table and the index of the output variable to compute
	// equations for.
	fn new(tbl: &Truth, idx: usize, vn: &str) -> Self {
		let mut rv: Vec<Term> = vec![];
		for ent in tbl.table.iter() {
			assert!(idx < ent.output.len());
			// 0 bits don't contribute terms.
			if ent.output[idx] == false {
				continue;
			}
			// compute the term and add it to our list ...
			rv.push(Term::compute(&ent.input));
		}
		Equation{index: idx, terms: rv, varname: vn.to_string()}
	}

	// Tries to minimize this equation.
	fn simplify(&mut self) {
		// Essentially the only option we have is identifying opposite
		// subexpressions: a'b' + a'b simplifies to a'.
		let mut idx_remove: (usize, usize) = Default::default(); // index, bit.
		let mut term_remove: usize = Default::default();
		let mut found = false;
		for (t1_loc, t1) in self.terms.iter().enumerate() {
			for (t2_loc, t2) in self.terms.iter().enumerate() {
				if t1 == t2 { continue; }
				if t1.mergeable(&t2) {
					// Then we can drop the bit that differs.
					found = true;
					assert!(t1.len() == t2.len());

					let mut iter = t1.bits.iter().zip(t2.bits.iter());
					// Which bit is it?  The indices are the same, bit itself differs.
					let index = iter.find(|&(b1, b2)| b1.0 == b2.0 && b1.1 != b2.1);
					match index {
						None => panic!("mergeable but no opposite bits?"),
						Some((idx, _)) => {
							idx_remove = (t1_loc, idx.0);
							term_remove = t2_loc;
							break;
						}
					};
				}
			}
		}
		if found {
			self.terms[idx_remove.0].remove_index(idx_remove.1);
			self.terms.remove(term_remove);
			self.simplify();
		}
	}
}

impl std::fmt::Display for Equation {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		try!(write!(f, "{} = ", self.varname));
		for t in self.terms.iter() {
			try!(write!(f, "{} + ", t));
		}
		write!(f, ";")
	}
}

fn equations(truth: &Truth, outvars: Vec<&str>) -> Vec<Equation> {
	assert!(!truth.table.is_empty());
	for i in truth.table.iter() { // verify lengths are okay.
		assert!(i.input.len() == truth.table[0].input.len());
		assert!(i.output.len() == truth.table[0].output.len());
	}
	assert!(truth.table[0].output.len() == outvars.len());
	let mut rv: Vec<Equation> = vec![];
	for b in 0..truth.table[0].output.len() {
		rv.push(Equation::new(truth, b, outvars[b]));
	}
	rv
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
		// convert to a vector of bits.
		let inp_bit: Vec<Bit> = inp.iter().map(|b| { Bit::new(*b) }).collect();
		// find the entry for which the input bit pattern matches.
		let foo = self.table.iter().find(|tbl| { tbl.input == inp_bit });
		match foo {
			None => panic!("cannot find bit pattern {:?}", inp),
			Some(x) => x.output.clone(), // return the output part of the Entry.
		}
	}

	fn len(&self) -> usize { return self.table.len() }

	#[allow(dead_code)]
	fn print(&self, wrt: &mut std::io::Write) {
		for elem in self.table.iter() {
			for i in elem.input.iter() {
				write!(wrt, "{}", *i).unwrap();
			}
			write!(wrt, " -> ").unwrap();
			for o in elem.output.iter() {
				if *o {
					write!(wrt, "{}", 1).unwrap();
				} else {
					write!(wrt, "{}", 0).unwrap();
				}
			}
			write!(wrt, "\n").unwrap();
		}
	}
}

fn main() {
	let args = Docopt::new(USAGE)
		.unwrap_or_else(|e| e.exit())
		.parse()
		.unwrap_or_else(|e| e.exit());
	println!("vars: '{:?}'", args.get_vec("--ivar"));
	println!("truth: '{}'", args.get_str("<truth>"));
	println!("map: '{:?}'", args);
	let input_bits = args.get_count("--ivar") as usize;
	let output_bits = args.get_count("--ovar") as usize;
	let header_lines = 2;
	let csvtable = Path::new(args.get_str("<truth>"));
	let fp = match File::open(&csvtable) {
		Err(e) => panic!("error {} opening {}", e, args.get_str("<truth>")),
		Ok(f) => f,
	};
	let tbl = parse(fp, header_lines, input_bits, output_bits);
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

	let mut eqns = equations(&tbl, args.get_vec("--ovar"));
	assert_eq!(eqns.len(), tbl.table[0].output.len());
	for e in 0..eqns.len() {
		eqns[e].simplify();
		println!("{}", eqns[e]);
	}
}

// really this returns a Vec<[usize; nbits]>, but Rust's variable-length arrays
// are vectors.
#[allow(dead_code)]
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
			ent.input.push(Bit::new(on));
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

	// a faux example with just 3 inputs and 2 outputs, for validation against.
	// if the inputs are 'a','b','c' and the outputs are 'x','y', then the
	// basic solution is:
	//   x = a'b'c + a'bc' + ab'c' + abc'
	//   y = a'b'c' + a'bc' + ab'c' + ab'c + abc'
	// i.e. a solution of:
	//   x = y = 0
	//   if(a'bc'): x = y = 1
	//   if(abc'): x = y = 1

	//   if(ab'c'): x = y = 1
	//   if(ab'c): y = 1

	//   if(a'b'c): x = 1
	//   if(a'b'c'): y = 1
	// that can be simplified to:
	//   if(a'b'):
	//    if(c): x = 1
	//    else if(c'): y = 1
	//   if(ab'):
	//    y = 1
	//    if(c'): x = 1
	//   if(bc'): x = y = 1
	fn small_example() -> String {
		let s =
			"0,0,0,,0,1\n".to_string() +
			"0,0,1,,1,0\n" +
			"0,1,0,,1,1\n" +
			"0,1,1,,0,0\n" +
			"1,0,0,,1,1\n" +
			"1,0,1,,0,1\n" +
			"1,1,0,,1,1\n" +
			"1,1,1,,0,0\n";
		s
	}

	#[test]
	fn read_test() {
		let eg = example_head();
		let tbl = parse(eg.as_bytes(), 2, 8, 4);
		// should be the same number of lines:
		assert_eq!(tbl.len(), 2);
	}

	#[test]
	fn parse_small() {
		let small = small_example();
		let truth = parse(small.as_bytes(), 0, 3, 2);
		assert_eq!(truth.len(), 8);
	}

	#[test]
	fn term_merge() {
		let t1 = Term::new(vec![(0,false), (1,false), (2,false)]);
		let t2 = Term::new(vec![(0,false), (1,true), (2,false)]);
		let t3 = Term::new(vec![(0,false), (1,true), (2,false), (3,true)]);
		let t4 = Term::new(vec![(0,false), (1,true), (2,false), (3,false)]);
		assert!(t1.mergeable(&t2));
		assert!(!t1.mergeable(&t3));
		assert!(!t1.mergeable(&t4));
		assert!(t2.mergeable(&t1));
		assert!(!t2.mergeable(&t3));
		assert!(!t2.mergeable(&t4));
		assert!(!t3.mergeable(&t1));
		assert!(!t3.mergeable(&t2));
		assert!(t3.mergeable(&t4));
		assert!(!t4.mergeable(&t1));
		assert!(!t4.mergeable(&t2));
		assert!(t4.mergeable(&t3));
	}

	#[test]
	fn small_simplify() {
		let small = small_example();
		let truth = parse(small.as_bytes(), 0, 3, 2);
		assert_eq!(truth.len(), 8);
		let mut eqns = equations(&truth, vec!["foo", "bar"]);
		assert_eq!(eqns.len(), truth.table[0].output.len());
		for e in 0..eqns.len() {
			println!("{}", eqns[e]);
			eqns[e].simplify();
		}
	}
}
