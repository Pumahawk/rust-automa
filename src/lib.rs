use std::rc::Rc;
use std::cell::RefCell;
use std::collections::LinkedList;

pub struct ANode<I, R, C> {
    node: Rc<RefCell<Node<I, R, C>>>,
}

impl <I, R, C> ANode<I, R, C> {
    pub fn new() -> ANode<I, R, C> {
        ANode {
            node: Rc::new(RefCell::new(Node::new())),
        }
    }

    pub fn clone(&self) -> ANode<I, R, C> {
        ANode {
            node: Rc::clone(&self.node),
        }
    }
}

impl <I, R, C> Linkable<I, R, C> for ANode<I, R, C> {
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I, R, C>>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I, R, C>)
    {
        self.node.borrow_mut().link_update(destination, condition, update_link);
    }
}

pub trait Linkable<I, R, C> {
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I, R, C>>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I, R, C>);

    fn link<F>(&mut self, destination: Option<&ANode<I, R, C>>, condition: F)
    where
        F : Fn(&I) -> bool + 'static
    {
        self.link_update(destination, condition, |_|{});
    }

    fn link_function<F, FPr>(&mut self, destination: Option<&ANode<I, R, C>>, condition: F, process: FPr)
    where
        F : Fn(&I) -> bool + 'static,
        FPr: Fn(&I, &mut C) -> Option<R> + 'static
    {
        self.link_update(destination, condition, |link| link.set_function(process));
    }

    fn link_process<F, FPr>(&mut self, destination: Option<&ANode<I, R, C>>, condition: F, process: FPr)
    where
        F : Fn(&I) -> bool + 'static,
        FPr: Fn(&I, &mut C) + 'static
    {
        self.link_update(destination, condition, |link| link.set_process(process));
    }
}

pub struct Node<I, R, C> {
    links: LinkedList<Link<I, R, C>>,
}

impl <I, R, C> Node<I, R, C> {
    pub fn new() -> Node<I, R, C> {
        Node {
            links: LinkedList::new(),
        }
    }
}

impl <I, R, C> Linkable<I, R, C> for Node<I, R, C> {
    
    fn link_update<F, FUpd>(&mut self, destination: Option<&ANode<I, R, C>>, condition: F, update_link: FUpd)
    where
        F : Fn(&I) -> bool + 'static,
        FUpd: FnOnce(&mut Link<I, R, C>)
    {
        self.links.push_back(Link::new(destination, condition));
        update_link(self.links.back_mut().unwrap());
    }
}

pub struct Link<I, R, C> {
    condition: Box<dyn Fn(&I) -> bool>,
    process: Option<Box<dyn Fn(&I, &mut C) -> Option<R>>>,
    destination: Option<ANode<I, R, C>>,
}

impl <I, R, C> Link<I, R, C> {
    pub fn new<F: Fn(&I) -> bool + 'static>(destination: Option<&ANode<I, R, C>>, condition: F) -> Link<I, R, C> {
        Link {
            condition: Box::new(condition),
            process: None,
            destination: destination.map(|destination| destination.clone()),
        }
    }

    pub fn condition(&self, input: &I) -> bool {
        (self.condition)(input)
    }

    pub fn process(&self, input: &I, context: &mut C) -> Option<R> {
        if let Some(fun) = &self.process {
            fun(input, context)
        } else {
            None
        }
    }

    pub fn set_process<F: Fn(&I, &mut C) + 'static>(&mut self, fun: F) {
        self.set_function(move |input, context| {fun(input, context); None});
    }

    pub fn set_function<F: Fn(&I, &mut C) -> Option<R> + 'static>(&mut self, fun: F) {
        self.process = Some(Box::new(fun));
    }
}

pub struct Cursor<I, R, C> {
    context: C,
    node: ANode<I, R, C>,
}

impl <I, R, C> Cursor<I, R, C> {
    pub fn new(context: C, node: &ANode<I, R, C>) -> Cursor<I, R, C> {
        Cursor {
            context,
            node: node.clone(),
        }
    }

    pub fn action(&mut self, input: &I) -> Option<R> {
        let mut node = None;
        let mut res = None;
        for link in self.node.node.borrow_mut().links.iter() {
            if link.condition(input) {
                res = link.process(input, &mut self.context);
                node = if let Some(destination) = &link.destination { Some(destination.clone()) } else { None };
                break;
            }
        }
        if let Some(node) = node {
            self.node = node;
        }

        res
    }
}

pub fn eq<T: std::cmp::PartialEq>(input: T) -> impl Fn(&T) -> bool {
    move |el| el == &input
}

pub fn not_eq<T: std::cmp::PartialEq>(input: T) -> impl Fn(&T) -> bool {
    move |el| el != &input
}

pub fn node<I, R, C>() -> ANode<I, R, C> {
    ANode::new()
} 

#[cfg(test)]
mod tests {

    use crate::*;

    type TNode = ANode<char, (), ()>;
    
    #[test]
    fn create_automa() {

        let v = Rc::new(RefCell::new(Vec::new()));
        let v2 = Rc::clone(&v);


        let mut node1 = TNode::new();
        let node2 = TNode::new();

        node1.link_update(Some(&node2), eq('A'), |link| {
            link.set_process(move |_,_|v2.borrow_mut().push("help"));
        });

        let mut cursor = Cursor::new((), &node1);
        cursor.action(&'B');
        assert_eq!(0, v.borrow().len());
        cursor.action(&'A');

        assert_eq!("help", v.borrow()[0]);
    }

    #[test]
    fn read_string() {

        enum StrStatus {
            StringEnd,
        }

        type StrNode = ANode<char, StrStatus, ()>;

        let string = Rc::new(RefCell::new(Vec::<char>::new()));


        let mut n1 = StrNode::new();
        let mut n2 = StrNode::new();
        let n3 = StrNode::new();

        n1.link(Some(&n2), eq('"'));

        let strc = Rc::clone(&string);
        n2.link_process(None, |input| input >= &'a' && input <= &'z', move |input,_| strc.borrow_mut().push(*input));

        n2.link_function(Some(&n3), eq('"'), |_,_| Some(StrStatus::StringEnd));

        let input = r#""stringa"#;
        let mut cursor = Cursor::new((), &n1);
        input.chars().for_each(|c| { cursor.action(&c); });

        match cursor.action(&'"') {
            Some(StrStatus::StringEnd) => {
                let output: String = string.borrow().iter().collect();
                assert_eq!("stringa", output);
            },
            _ => assert!(false),
        }

    }
}