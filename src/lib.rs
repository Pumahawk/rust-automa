use std::rc::Rc;
use std::cell::RefCell;
use std::collections::LinkedList;

pub struct Node<I> {
    links: LinkedList<Link<I>>,
}

impl <I> Node<I> {
    pub fn new() -> Rc<RefCell<Node<I>>> {
        Rc::new(RefCell::new(Node {
            links: LinkedList::new(),
        }))
    }

    pub fn link<F: Fn(&I) -> bool + 'static>(&mut self, destination: &Rc<RefCell<Node<I>>>, condition: F) -> &mut Link<I> {
        self.links.push_front(Link::new(destination, condition));
        self.links.front_mut().unwrap()
    }
}

pub struct Link<I> {
    condition: Box<dyn Fn(&I) -> bool>,
    process: Option<Box<dyn Fn()>>,
    destination: Rc<RefCell<Node<I>>>,
}

impl <I> Link<I> {
    pub fn new<F: Fn(&I) -> bool + 'static>(destination: &Rc<RefCell<Node<I>>>, condition: F) -> Link<I> {
        Link {
            condition: Box::new(condition),
            process: None,
            destination: Rc::clone(destination),
        }
    }

    pub fn condition(&self, input: &I) -> bool {
        (self.condition)(input)
    }

    pub fn process(&self) {
        if let Some(fun) = &self.process {
            fun();
        } 
    }

    pub fn set_process<F: Fn() + 'static>(&mut self, fun: F) {
        self.process = Some(Box::new(fun));
    }
}

pub struct Cursor<I> {
    node: Rc<RefCell<Node<I>>>,
}

impl <I> Cursor<I> {
    pub fn new(node: &Rc<RefCell<Node<I>>>) -> Cursor<I> {
        Cursor {
            node: Rc::clone(node),
        }
    }

    pub fn action(&mut self, input: &I) {
        for link in (self.node.borrow_mut()).links.iter() {
            if link.condition(input) {
                link.process();
                self.node = Rc::clone(&link.destination);
                break;
            }
        }
    }
}

pub fn eq<T: std::cmp::PartialEq>(input: T) -> impl Fn(&T) -> bool {
    move |el| el == &input
} 

#[cfg(test)]
mod tests {

    use crate::*;
    
    #[test]
    fn create_automa() {
        let mut node1 = Node::<char>::new();
        let node2 = Node::<char>::new();

        Rc::get_mut(&mut node1).unwrap().get_mut().link(&node2, eq('A'));

        let mut cursor = Cursor::new(&node1);
        cursor.action(&'A');
    }
}