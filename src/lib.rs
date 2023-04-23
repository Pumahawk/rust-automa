use std::rc::Rc;
use std::cell::RefCell;
use std::collections::LinkedList;

pub type ANode<I, R> = Rc<RefCell<Node<I, R>>>;

pub trait Linkable<I, R> {
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I, R>>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I, R>);

    fn link<F>(&mut self, destination: Option<&ANode<I, R>>, condition: F)
    where
        F : Fn(&I) -> bool + 'static
    {
        self.link_update(destination, condition, |_|{});
    }
}

impl <I, R> Linkable<I, R> for ANode<I, R> {
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I, R>>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I, R>)
    {
        self.borrow_mut().link_update(destination, condition, update_link);
    }
}

pub struct Node<I, R> {
    links: LinkedList<Link<I, R>>,
}

impl <I, R> Node<I, R> {
    pub fn new() -> ANode<I, R> {
        Rc::new(RefCell::new(Node {
            links: LinkedList::new(),
        }))
    }
}

impl <I, R> Linkable<I, R> for Node<I, R> {
    
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I, R>>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I, R>)
    {
        self.links.push_front(Link::new(destination, condition));
        update_link(self.links.front_mut().unwrap());
    }
}

pub struct Link<I, R> {
    condition: Box<dyn Fn(&I) -> bool>,
    process: Option<Box<dyn Fn(&I) -> R>>,
    destination: Option<ANode<I, R>>,
}

impl <I, R> Link<I, R> {
    pub fn new<F: Fn(&I) -> bool + 'static>(destination: Option<&ANode<I, R>>, condition: F) -> Link<I, R> {
        Link {
            condition: Box::new(condition),
            process: None,
            destination: destination.map(|destination|Rc::clone(destination)),
        }
    }

    pub fn condition(&self, input: &I) -> bool {
        (self.condition)(input)
    }

    pub fn process(&self, input: &I) -> Option<R> {
        if let Some(fun) = &self.process {
            Some(fun(input))
        } else {
            None
        }
    }

    pub fn set_process<F: Fn(&I) -> R + 'static>(&mut self, fun: F) {
        self.process = Some(Box::new(fun));
    }
}

pub struct Cursor<I, R> {
    node: ANode<I, R>,
}

impl <I, R> Cursor<I, R> {
    pub fn new(node: &ANode<I, R>) -> Cursor<I, R> {
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

    type TNode = Node<char, ()>;
    
    #[test]
    fn create_automa() {

        let v = Rc::new(RefCell::new(Vec::new()));
        let v2 = Rc::clone(&v);


        let mut node1 = TNode::new();
        let node2 = TNode::new();

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


        let mut n1 = TNode::new();
        let mut n2 = TNode::new();
        let n3 = TNode::new();

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