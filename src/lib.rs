use std::rc::Rc;
use std::collections::LinkedList;

pub struct Node<I> {
    links: LinkedList<Link<I>>,
}

impl <I> Node<I> {
    pub fn new() -> Rc<Node<I>> {
        Rc::new(Node {
            links: LinkedList::new(),
        })
    }

    pub fn link<F: Fn(&I) -> bool + 'static>(&mut self, destination: &Rc<Node<I>>, condition: F) -> &mut Link<I> {
        self.links.push_front(Link::new(destination, condition));
        self.links.front_mut().unwrap()
    }
}

pub struct Link<I> {
    condition: Box<dyn Fn(&I) -> bool>,
    destination: Rc<Node<I>>,
}

impl <I> Link<I> {
    pub fn new<F: Fn(&I) -> bool + 'static>(destination: &Rc<Node<I>>, condition: F) -> Link<I> {
        Link {
            condition: Box::new(condition),
            destination: Rc::clone(destination),
        }
    }

    pub fn condition(&self, input: &I) -> bool {
        (self.condition)(input)
    }

    pub fn process(&self) {
        todo!();
    }
}

pub struct Cursor<I> {
    node: Rc<Node<I>>,
}

impl <I> Cursor<I> {
    pub fn new(node: &Rc<Node<I>>) -> Cursor<I> {
        Cursor {
            node: Rc::clone(node),
        }
    }

    pub fn action(&mut self, input: &I) {
        for link in self.node.links.iter() {
            if link.condition(input) {
                link.process();
                self.node = Rc::clone(&link.destination);
                break;
            }
        }
    }
}