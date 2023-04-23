use std::rc::Rc;
use std::cell::RefCell;
use std::collections::LinkedList;

pub type ANode<I> = Rc<RefCell<Node<I>>>;

pub trait Linkable<I> {
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I>>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I>);

    fn link<F>(&mut self, destination: Option<&ANode<I>>, condition: F)
    where
        F : Fn(&I) -> bool + 'static
    {
        self.link_update(destination, condition, |_|{});
    }
}

impl <I> Linkable<I> for ANode<I> {
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I>>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I>)
    {
        self.borrow_mut().link_update(destination, condition, update_link);
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
    
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I>>, condition: F, update_link: FUpd)
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
    process: Option<Box<dyn Fn(&I)>>,
    destination: Option<ANode<I>>,
}

impl <I> Link<I> {
    pub fn new<F: Fn(&I) -> bool + 'static>(destination: Option<&ANode<I>>, condition: F) -> Link<I> {
        Link {
            condition: Box::new(condition),
            process: None,
            destination: destination.map(|destination|Rc::clone(destination)),
        }
    }

    pub fn condition(&self, input: &I) -> bool {
        (self.condition)(input)
    }

    pub fn process(&self, input: &I) {
        if let Some(fun) = &self.process {
            fun(input);
        } 
    }

    pub fn set_process<F: Fn(&I) + 'static>(&mut self, fun: F) {
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
                link.process(input);
                node = link.destination.clone();
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


        let mut node1 = Node::<char>::new();
        let node2 = Node::<char>::new();

        node1.link_update(Some(&node2), eq('A'), |link| {
            link.set_process(move |_|v2.borrow_mut().push("help"));
        });

        let mut cursor = Cursor::new(&node1);
        cursor.action(&'B');
        assert_eq!(0, v.borrow().len());
        cursor.action(&'A');

        assert_eq!("help", v.borrow()[0]);
    }

    #[test]
    fn read_string() {
        let string = Rc::new(RefCell::new(Vec::<char>::new()));


        let mut n1 = Node::<char>::new();
        let mut n2 = Node::<char>::new();
        let n3 = Node::<char>::new();

        n1.link(Some(&n2), eq('"'));

        let strc = Rc::clone(&string);
        n2.link_update(None, |input| input >= &'a' || input <= &'z', |link| {
            link.set_process(move |input| strc.borrow_mut().push(*input));
        });

        n2.link(Some(&n3), eq('"'));

        let input = r#""stringa""#;
        let mut cursor = Cursor::new(&n1);
        input.chars().for_each(|c| cursor.action(&c));

        let output: String = string.borrow().iter().collect();

        assert_eq!("stringa", output);
    }
}