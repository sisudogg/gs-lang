fn main() {
    println!("Hello, world!");
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
    pub fn forward(&mut self, data1: Option<u64>, data2: Option<u64>) -> (u64, u64) {
        if let Some(a) = data1 {
            self.stack.push(a);
        }
        if let Some(b) = data2 {
            self.stack.push(b)
        }
        self.execute();
        let x;
        if let Some(a) = self.stack.pop {
            x = a
        }
        else {
            x = 0;
        }
        let y: u64;
        if let Some(a) = self.stack.pop {
            y = a
        }
        else {
            y = 0;
        }
        (x, y)
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
        self.rule = self.stack;
        self.stack = vec![];
    }
}

enum Branch {
    Live(Box<Node>, Box<Branch>, Box<Branch>),
    Through(Box<Node>, Box<Branch>),
    End(Box<Node>)
}

trait Pass {
    pub fn forward(&mut self, data: u64) -> u64;
}

impl Pass for Branch {
    fn forward(&mut self, data: u64) -> u64 {
        match self {
            
        }
    }
}


    fn forward(&mut self, data: u64) -> u64 {
        self.0.forward(Some(data), None)
    }
}

impl Pass for Branch::Through {
    fn forward(&mut self, data: u64) -> u64 {
        self.0.forward(Some(self.1.forward(data)))
    }
}

impl Pass for Branch::Live {
    fn forward(&mut self, data: u64) -> u64 {
        self.0.forward(Some(self.1.forward(data)), Some(self.2.forward(data)))
    }
}