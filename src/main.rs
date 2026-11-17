use std::sync::{Arc, Mutex, mpsc::{Sender, Receiver}};

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
    pub fn forward(&mut self, mut data: Vec<u64>) -> u64 {
        while let Some(a) = data.pop() {
            self.stack.push(a);
        }
        self.execute();
        let x;
        if let Some(a) = self.stack.pop() {
            x = a
        }
        else {
            x = 0;
        }
        x
    }

    fn execute(&mut self) {
        for code in self.stack.iter_mut() {
            match code {
                0 => {},
                1 => {},
                2 => {},

                42 => {self.flash(); break}
                _ => {},
            }
        }
    }

    fn flash(&mut self) {
        self.rule = self.stack.clone();
        self.stack = vec![];
    }

    pub fn run (&mut self, data: Vec<u64>) -> Vec<u64> {
        let outVec = vec![];
        while let Some(a) = (self.downs.iter().next()) {
            let b = **a;
            b.lock()?;
            outVec.append(b.run(data.clone()));
        }
        self.forward(outVec)
    }
}




