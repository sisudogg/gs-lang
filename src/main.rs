use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::fs::{File};
use std::io::{self, Write, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    Ok(())
}

fn assemble(filename: String) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(filename)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let instructions: Vec<&str> = content.split("\n").collect();
    let mut output: Vec<u64> = vec![];
    let mut parser = Parser::new();
    for (index, line) in instructions.iter().enumerate() {
        parser.parse(line, index.try_into()?)?;
    }
    output.append(&mut parser.get());
    Ok(String::new())
}


struct Parser {
    labels: HashMap<String, u64>,
    binary: Vec<u64>
}

impl Parser {
    pub fn new() -> Parser {
        Parser {
            labels: HashMap::new(),
            binary: vec![],
        }
    }

    pub fn get(&self) -> Vec<u64> {
        self.binary.clone()
    }

    pub fn parse(&mut self, line: &str, index: u64) -> Result<(), Box<dyn std::error::Error>> {
        let instructions: Vec<&str> = line.split(' ').collect();
        let mut output: Vec<u64> = vec![];
        match instructions[0] {
            "SKIP" => output.push(0x00),
            "HALT" => output.push(0x01),
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

            "LABEL" => {if let Some(val) = self.labels.get_mut(instructions[1]) { *val = (self.binary.len() + 1) as u64; }; output.push(0x44)},

            _ => { output.push(0x00) }
        }
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
            rule: rule,
            stack: vec![],
            downs: subordinates,
        }
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
                    0x20 => { self.stack.push(*self.rule.get(i+1)?)} //push
                    0x21 => { self.stack.pop(); } //pop
                    0x22 => { let a = self.stack.pop()?; self.stack.push(a); self.stack.push(a);} //dup
                    0x23 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a); self.stack.push(b);} //swap

                    //control flow            
                    0x40 => { i += *self.rule.get(i+1)? as usize }, //jmp
                    0x41 => { if self.stack.pop()? == 0 { i += (*self.rule.get(i+1)? as usize); } }, //jz
                    0x42 => {ret.push(i); i = self.rule[i+1] as usize;  }, //call
                    0x43 => { i = ret.pop()? }, //return
                    0x44 => {}, //label
                    //risky operation: 0x48 => { i += self.stack.pop()? as usize }, //jmps
                    //risky operation: 0x49 => { if self.stack.pop()? == 0 { i += self.stack.pop()? as usize } }, //jzs

                    42000000 => {self.flash(); break;},
                    _ => {},
                }
                i += 1;
        }
        };
        Some(())
    }

    fn flash(&mut self) {
        self.rule = self.stack.clone();
        self.stack = vec![];
    }

    pub fn run (&mut self, data: Vec<u64>) -> Vec<u64> {
        let mut out_vec = vec![];
        while let Some(a) = (self.downs.iter().next()) {
            let b = a;
            out_vec.append(&mut b.lock().unwrap().run(data.clone()));
        }
        if out_vec.len() == 0 {
            out_vec = data.clone()
        }
        self.forward(out_vec)
    }
}




