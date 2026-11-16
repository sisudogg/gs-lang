fn main() {
    println!("Hello, world!");
}

fn parse_blocks(prog: String) {
    
}

struct Node {
    rule: Vec<u64>,
    stack: Vec<u64>,
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Self {
            rule: self.rule.clone(),
            stack: vec![],
        }
    }
}

impl Node {
    pub fn forward(&mut self, data1: Option<u64>, data2: Option<u64>) -> u64 {
        if let Some(a) = data1 {
            self.stack.push(a);
        }
        if let Some(b) = data2 {
            self.stack.push(b)
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
}

enum Branch {
    Live(Box<Node>, Box<Branch>, Box<Branch>),
    Through(Box<Node>, Box<Branch>),
    End(Box<Node>)
}

pub trait Pass {
    fn forward(&mut self, data: u64) -> u64;
}

impl Pass for Branch {
    fn forward(&mut self, data: u64) -> u64 {
        let x = match self {
            Branch::Live(next, past1, past2) => {next.forward(Some(past1.forward(data)), Some(past2.forward(data)))},
            Branch::Through(next, past) => {next.forward(Some(past.forward(data)), None)},
            Branch::End(next) => {next.forward(Some(data), None)},
        };
        x
    }
}


