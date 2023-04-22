use std::rc::Rc;
use std::collections::LinkedList;

pub struct Node {
    links: LinkedList<Link>,
}

impl Node {
    pub fn new() -> Rc<Node> {
        Rc::new(Node {
            links: LinkedList::new(),
        })
    }

    pub fn link(&mut self, destination: &Rc<Node>) -> &mut Link {
        self.links.push_front(Link::new(destination));
        self.links.front_mut().unwrap()
    }
}

pub struct Link {
    destination: Rc<Node>,
}

impl Link {
    pub fn new(destination: &Rc<Node>) -> Link {
        Link {
            destination: Rc::clone(destination),
        }
    }

    pub fn condition(&self, _input: char) -> bool {
        todo!()
    }

    pub fn process(&self) {
        todo!();
    }
}

pub struct Cursor {
    node: Rc<Node>,
}

impl Cursor {
    pub fn new(node: &Rc<Node>) -> Cursor {
        Cursor {
            node: Rc::clone(node),
        }
    }

    pub fn action(&mut self, input: char) {
        for link in self.node.links.iter() {
            if link.condition(input) {
                link.process();
                self.node = Rc::clone(&link.destination);
                break;
            }
        }
    }
}