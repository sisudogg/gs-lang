use std::sync::{Arc, Mutex};

fn main() {
    println!("Hello, world!");
}

fn parse_blocks(prog: String) {
    
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
        while i <= self.rule.len() {
            if let Some(code) = self.rule.get(i) {
                match code {
                    0 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a + b) }, //add
                    1 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a - b) }, //sub
                    2 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a * b) }, //mul
                    3 => { let a = self.stack.pop()?; let b = self.stack.pop()?; self.stack.push(a / b) }, //div
                    4 => { i += self.rule.get([i+1])? }
                    5 => { if self.stack.pop()? == 0 { i += (*self.rule.get(i+1)? as usize); } }, //jmp0


                    42 => {self.flash(); break;}
                    _ => break,
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
        self.forward(out_vec)
    }
}




