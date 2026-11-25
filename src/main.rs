use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs::{File};
use std::io;
use std::io::{Write, Read, stdin};
use savefile::prelude::*;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut filenames: Vec<String> = vec![];
    let mut out = io::stdout();
    loop {
        print!("Enter assembly file name (blank to quit): ");
        out.flush()?;
        let mut outstring = String::new();
        stdin().read_line(&mut outstring)?;
        let outstring = outstring.trim_end_matches(['\r', '\n']).to_string();
        if outstring.is_empty() {
            break;
        }
        filenames.push(outstring.to_owned());
    }
    let mut parser = Parser::new();
    for filename in filenames.into_iter() {
        parser.load(&filename)?;
    }
    let mut binfiles: Vec<String> = vec![];
    loop {
        print!("Enter binary file name (blank to quit): ");
        out.flush()?;
        let mut outstring = String::new();
        stdin().read_line(&mut outstring)?;
        let outstring = outstring.trim_end_matches(['\r', '\n']).to_string();
        if outstring.is_empty() {
            break;
        }
        binfiles.push(outstring.to_owned());
    }

    for binfile in binfiles.into_iter() {
        parser.link(&binfile)?;
    }

    parser.assemble()?;

    let mut outfile = String::new();
    print!("Enter output file name: ");
    out.flush()?;
    stdin().read_line(&mut outfile)?;
    let outfile = outfile.trim_end_matches(['\r', '\n']).to_string();
    let mut binstring: String = outfile.clone();
    binstring.push_str(".bin");


    parser.save(&binstring)?;
    println!("Saved binary opcode output to ./{}", binstring);

    let mut labelstring: String = outfile.clone();
    labelstring.push_str(".label");
    let mut varstring: String = outfile.clone();
    varstring.push_str(".var");

    save_file(labelstring, 0, &parser.get_labels())?;
    save_file(varstring, 0, &parser.get_variables())?;

    print!("Select file to run (blank to quit): ");
    out.flush()?;
    let mut outstring = String::new();
    stdin().read_line(&mut outstring)?;
    let outfile = outstring.trim_end_matches(['\r', '\n']).to_string();

    let mut root = Node::blank();
    root.load(&outfile)?;

    let mut data: Vec<u64> = vec![];

    loop {
        print!("Enter u64 value for initial data (blank to quit): ");
        out.flush()?;
        let mut outint: String = "".to_owned();
        stdin().read_line(&mut outint)?;
        let outint = outint.trim_end_matches(['\r', '\n']).to_string();
        if outint.is_empty() {
            break;
        }
        data.push(outint.parse::<u64>()?);
    }

    root.run(data);

    Ok(())
}




struct Parser {
    labels: HashMap<String, u64>,
    binary: Vec<u64>,
    assembly: String,
    variables: HashMap<String, u64>,
    offset: u64,
    source: Vec<String>,
    functions: Vec<String>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            labels: HashMap::new(),
            binary: vec![],
            assembly: String::new(),
            variables: HashMap::new(),
            offset: 0,
            source: vec![],
            functions: vec![],
        }
    }

    pub fn link(&mut self, filename: &String) -> Result<(), Box<dyn std::error::Error>> {
        self.offset = self.binary.len() as u64;
        let mut bin_name = filename.clone();
        bin_name.push_str(".bin");
        let mut binfile = File::open(&bin_name)?;
        let mut readbytes = Vec::new();
        binfile.read_to_end(&mut readbytes)?;

        let mut parsed_vals = vec![];
        for chunk in readbytes.chunks(8) {
            if chunk.len() == 8 {
                let arr: [u8; 8] = chunk.try_into()?;
                parsed_vals.push(u64::from_be_bytes(arr));
            }
        }

        self.binary.append(&mut parsed_vals);
        let mut label_name = filename.clone();
        label_name.push_str(".label");
        let label_dict: HashMap<String, u64> = load_file(label_name, 0)?;

        let mut variable_name = filename.clone();
        variable_name.push_str(".var");
        let variable_dict: HashMap<String, u64> = load_file(variable_name, 0)?;
        
        self.load_labels(label_dict)?;
        self.load_variables(variable_dict);
        Ok(())
    }

    pub fn get_labels(&self) -> HashMap<String, u64> {
        self.labels.clone()
    }

    pub fn get_variables(&self) -> HashMap<String, u64> {
        self.variables.clone()
    }

    pub fn load_source(&mut self, filename: &String) -> Result<(), Box<dyn std::error::Error>> {
        let mut outfile = filename.clone();
        outfile.push_str(".c");
        let mut outfile = File::open(outfile)?;
        let mut output = String::new();
        outfile.read_to_string(&mut output)?;
        self.source.push(output);
        Ok(())
    }


    fn load_labels(&mut self, dict: HashMap<String, u64>) -> Result<(), Box<dyn std::error::Error>> {
        let binding = dict.clone();
        let keys = binding.keys();
        for key in keys {
            if let Some(val) = self.labels.get_mut(key) {
                *val += self.offset;
            };
        }
        self.labels.extend(dict.into_iter());
        Ok(())
    }

    fn load_variables(&mut self, dict: HashMap<String, u64>) {
        let binding = dict.clone();
        let keys = binding.keys();
        for key in keys {
            if let Some(val) = self.variables.get_mut(key) {
                *val += self.offset;
            };
        }
        self.variables.extend(dict.into_iter());
    }

    pub fn assemble(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let read: Vec<String> = self.assembly.split('\n').map(|s| s.to_string()).collect();
        let lines: Vec<&str> = read.iter().map(|s| s.as_str()).collect();
        self.preprocess();
        for line in lines {
            self.parse(line)?;
        }
        Ok(())
    }

    pub fn load(&mut self, filename: &String) -> Result<(), Box<dyn std::error::Error>> {
        let mut name = filename.clone();
        name.push_str(".asm");
        let mut infile = File::open(name)?;
        infile.read_to_string(&mut self.assembly)?;
        Ok(())
    }

    pub fn get(&self) -> Vec<u64> {
        self.binary.clone()
    }

    pub fn save(&self, filename: &String) -> Result<(), Box<dyn std::error::Error>> {
        let out_name = filename.clone();
        //out_name.push_str(".bin");
        let mut outfile = File::create(out_name)?;
        let mut outvec: Vec<u8> = vec![];
        for num in self.get() {
            outvec.append(&mut Vec::from(num.to_be_bytes()));
        }
        outfile.write_all(&outvec)?;
        Ok(())
    }

    fn preprocess(&mut self) {
        let mut length = self.binary.len();
        for line in self.assembly.split("\n") {
            let instructions: Vec<&str> = line.split(' ').collect();
            if instructions[0] == "LABEL" {
                self.labels.insert(instructions[1].to_string(), length as u64);
            }
            else if instructions[0] == "VAR" {
                length += 1;
                self.variables.insert(instructions[1].to_string(), length as u64);
            }
            else if instructions[0] == "PUSH" || instructions[0] == "JMP" || instructions[0] == "JZ" || instructions[0] == "CALL" || instructions[0] == "PUSHVAR" || instructions[0] == "POPVAR" {
                length += 1
            }
            length += 1;
        }
    }

    pub fn parse(&mut self, line: &str) -> Result<(), Box<dyn std::error::Error>> {
        let instructions: Vec<&str> = line.split(' ').collect();
        let mut output: Vec<u64> = vec![];
        match instructions[0] {
            "SKIP" => output.push(0x00),
            "HALT" => output.push(0x01),
            "HLT" => output.push(0x01),
            "PSH0" => output.push(0x02),

            "ADD" => output.push(0x10),
            "SUB" => output.push(0x11),
            "MUL" => output.push(0x12),
            "DIV" => output.push(0x13),

            "AND" => output.push(0x14),
            "OR" => output.push(0x15),
            "XOR" => output.push(0x16),
            "NOT" => output.push(0x17),

            "SHL" => output.push(0x18),
            "SHR" => output.push(0x19),

            "PUSH" => {output.push(0x20); output.push(instructions[1].parse::<u64>()?)},
            "POP" => output.push(0x21),
            "DUP" => output.push(0x22),
            "SWAP" => output.push(0x23),

            "JMP" => {output.push(0x40); output.push(self.labels[instructions[1]])},
            "JZ" => {output.push(0x41); output.push(self.labels[instructions[1]])},
            "CALL" => {output.push(0x42); output.push(self.labels[instructions[1]])},
            "RET" => output.push(0x43),

            "LABEL" => {output.push(0x44);}, //{if let Some(val) = self.labels.get_mut(instructions[1]) {*val = self.binary.len() as u64}; output.push(0x44)},
            "VAR" => {output.push(0x45); output.push(0x00);}, //{if let Some(val) = self.variables.get_mut(instructions[1]) {*val = self.binary.len() as u64}; output.push(0x45); output.push(0x00);},
            "PUSHVAR" => {if let Some(val) = self.variables.get(instructions[1]) {output.push(0x46); output.push(*val)}}
            "POPVAR" => {if let Some(val) = self.variables.get(instructions[1]) {output.push(0x47); output.push(*val)}},

            "PRINT64" => {output.push(0x50)},
            "PRINT8X8" => {output.push(0x51)},
            "PRINTC64" => {output.push(0x52)},
            "PRINTC8X8" => {output.push(0x53)},
            "NEWLINE" => {output.push(0x54)},

            _ => { output.push(0x00) }
        }
        self.binary.append(&mut output);
        Ok(())
    }
}

struct Node {
    rule: Vec<u64>,
    stack: Vec<u64>,
    downs: Vec<Arc<Mutex<Node>>>,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            rule: self.rule.clone(),
            stack: vec![],
            downs: self.downs.clone(),
        }
    }
}

impl Node {
    pub fn new(rule: Vec<u64>, subordinates: Vec<Arc<Mutex<Node>>>) -> Node {
        Node {
            rule,
            stack: vec![],
            downs: subordinates,
        }
    }

    pub fn blank() -> Node {
        Node {
            rule: vec![],
            stack: vec![],
            downs: vec![],
        }
    }

    pub fn load(&mut self, filename: &String) -> Result<(), Box<dyn std::error::Error>> {
        let mut bin_name = filename.clone();
        bin_name.push_str(".bin");
        let mut binfile = File::open(&bin_name).expect(format!("Failed to open file {}", bin_name).as_str());
        let mut readbytes = Vec::new();
        binfile.read_to_end(&mut readbytes)?;

        let mut parsed_vals = vec![];
        for chunk in readbytes.chunks(8) {
            if chunk.len() == 8 {
                let arr: [u8; 8] = chunk.try_into()?;
                parsed_vals.push(u64::from_be_bytes(arr));
            }
        }
        self.rule = parsed_vals;
        Ok(())
    }

    pub fn forward(&mut self, mut data: Vec<u64>) -> Vec<u64> {
        self.stack.append(&mut data);
        self.execute();
        let mut out_vec: Vec<u64> = vec![];
        let num_read;
        if let Some(top) = self.stack.pop() {
            num_read = top;
            for _ in 0..num_read {
                if let Some(next_value) = self.stack.pop() {
                    out_vec.push(next_value);
                }
            }
        }
        else {
            out_vec.push(0);
        }
        out_vec
    }

    fn execute(&mut self) -> Option<()> {
        let mut i: usize = 0;
        let mut ret: Vec<usize> = vec![];
        while i <= self.rule.len() {
            if let Some(code) = self.rule.get(i) {
                match code {
                    //basic codes
                    0x00 => {},
                    0x01 => {break; }
                    0x02 => {self.stack.push(0)}
                    
                    //arithmetic
                    0x10 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a + b) }, //add
                    0x11 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a - b) }, //sub
                    0x12 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a * b) }, //mul
                    0x13 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a / b) }, //div

                    //bitwise
                    0x14 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a & b) }, //andB
                    0x15 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a | b) }, //orB
                    0x16 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a ^ b) }, //xorB
                    0x17 => { let a = self.stack.pop()?; self.stack.push(!a) }, //notB

                    //bitshift
                    0x18 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a << b % 64) }, //shL
                    0x19 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a >> b % 64) }, //shR

                    //stack manipulation
                    0x20 => { self.stack.push(*self.rule.get(i+1)?); i += 1;} //push
                    0x21 => { self.stack.pop(); } //pop
                    0x22 => { let a = self.stack.pop()?; self.stack.push(a); self.stack.push(a);} //dup
                    0x23 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a); self.stack.push(b);} //swap

                    //control flow            
                    0x40 => { i = *self.rule.get(i+1)? as usize }, //jmp
                    0x41 => { if self.stack.pop()? == 0 { i = *self.rule.get(i+1)? as usize; } else {i += 1}}, //jz
                    0x42 => {ret.push(i+2); i = self.rule[i+1] as usize;  }, //call
                    0x43 => { i = ret.pop()? }, //return

                    //Labels and variables
                    0x44 => {}, //label
                    0x45 => {i += 1;}, //var //don't execute data of var
                    0x46 => {self.stack.push(*self.rule.get(*self.rule.get(i+1)? as usize)?); i += 1;}, //push variable to stack
                    0x47 => {let index = *self.rule.get(i+1)? as usize; self.rule[index] = self.stack.pop()?; i += 1;} //pop and write to variable
                    //risky operation: 0x48 => { i += self.stack.pop()? as usize }, //jmps //Jump to index at top of stack
                    //risky operation: 0x49 => { if self.stack.pop()? == 0 { i += self.stack.pop()? as usize } }, //jzs //Jump to value 2nd of stack if 1st = 0

                    //Basic IO operations
                    0x50 => {print!("{}", self.stack.pop()?)},
                    0x51 => {let bytes = self.stack.pop()?.to_be_bytes(); for num in bytes {println!("{}", num)}},
                    0x52 => {},
                    0x53 => {let bytes = self.stack.pop()?.to_be_bytes(); for num in bytes { println!("{}", char::from(num))}},
                    0x54 => println!(),


                    42000000 => {self.flash(); break;},
                    _ => {},
                }
            
        }
        i += 1;
        };
        Some(())
    }

    fn flash(&mut self) {
        self.rule = self.stack.clone();
        self.stack = vec![];
    }

    pub fn run (&mut self, data: Vec<u64>) -> Vec<u64> {
        let mut out_vec = vec![];
        while let Some(a) = self.downs.iter().next() {
            let b = a;
            out_vec.append(&mut b.lock().unwrap().run(data.clone()));
        }
        if out_vec.len() == 0 {
            out_vec = data.clone()
        }
        self.forward(out_vec)
    }
}


