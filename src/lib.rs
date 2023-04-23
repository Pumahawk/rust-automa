use std::rc::Rc;
use std::cell::RefCell;
use std::collections::LinkedList;

type ANode<I> = Rc<RefCell<Node<I>>>;

pub trait Linkable<I> {
    fn link_update<F, FUpd>(&mut self, destination: &ANode<I>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I>);

    fn link<F, FUpd>(&mut self, destination: &ANode<I>, condition: F)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I>)
    {
        self.link_update(destination, condition, |_|{});
    }
}

pub struct Node<I> {
    links: LinkedList<Link<I>>,
}

impl <I> Node<I> {
    pub fn new() -> ANode<I> {
        Rc::new(RefCell::new(Node {
            links: LinkedList::new(),
        }))
    }
}

impl <I> Linkable<I> for Node<I> {
    
    fn link_update<F, FUpd>(&mut self, destination: &ANode<I>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I>)
    {
        self.links.push_front(Link::new(destination, condition));
        update_link(self.links.front_mut().unwrap());
    }
}

pub struct Link<I> {
    condition: Box<dyn Fn(&I) -> bool>,
    process: Option<Box<dyn Fn()>>,
    destination: ANode<I>,
}

impl <I> Link<I> {
    pub fn new<F: Fn(&I) -> bool + 'static>(destination: &ANode<I>, condition: F) -> Link<I> {
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
    node: ANode<I>,
}

impl <I> Cursor<I> {
    pub fn new(node: &ANode<I>) -> Cursor<I> {
        Cursor {
            node: Rc::clone(node),
        }
    }

    pub fn action(&mut self, input: &I) {
        let mut node = None;
        for link in self.node.borrow_mut().links.iter() {
            if link.condition(input) {
                link.process();
                node = Some(Rc::clone(&link.destination));
                break;
            }
        }
        if let Some(node) = node {
            self.node = node;
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

        let v = Rc::new(RefCell::new(Vec::new()));
        let v2 = Rc::clone(&v);


        let node1 = Node::<char>::new();
        let node2 = Node::<char>::new();

        node1.borrow_mut().link_update(&node2, eq('A'), |link| {
            link.set_process(move ||v2.borrow_mut().push("help"));
        });

        let mut cursor = Cursor::new(&node1);
        cursor.action(&'B');
        assert_eq!(0, v.borrow().len());
        cursor.action(&'A');

        assert_eq!("help", v.borrow()[0]);
    }
}