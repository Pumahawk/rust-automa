use std::rc::Rc;
use std::cell::RefCell;
use std::collections::LinkedList;

pub struct ANode<T, I, R, C> {
    node: Rc<RefCell<Node<T, I, R, C>>>,
}

impl <T, I, R, C> ANode<T, I, R, C> {
    pub fn new() -> ANode<T, I, R, C> {
        ANode {
            node: Rc::new(RefCell::new(Node::new())),
        }
    }

    pub fn clone(&self) -> ANode<T, I, R, C> {
        ANode {
            node: Rc::clone(&self.node),
        }
    }
}

impl <T, I, R, C> From<T> for ANode<T, I, R, C> {
    fn from(value: T) -> Self {
        ANode {
            node: Rc::new(RefCell::new(Node::from(value))),
        }
    }
}

impl <T, I, R, C> Linkable<T, I, R, C> for ANode<T, I, R, C> {
    fn link_update<F, FUpd, FPr>(&mut self, destination: Option<&ANode<T, I, R, C>>, condition: F, process: FPr, update_link: FUpd)
    where
        F : Fn(&I, &C) -> bool + 'static,
        FUpd: FnOnce(&mut Link<T, I, R, C>),
        FPr: Fn(I, &mut C) -> R + 'static
    {
        self.node.borrow_mut().link_update(destination, condition, process, update_link);
    }
}

pub trait Linkable<T, I, R, C> {
    fn link_update<F, FUpd, FPr>(&mut self, destination: Option<&ANode<T, I, R, C>>, condition: F, process: FPr, update_link: FUpd)
    where
        F : Fn(&I, &C) -> bool + 'static,
        FUpd: FnOnce(&mut Link<T, I, R, C>),
        FPr: Fn(I, &mut C) -> R + 'static;

    fn link_function<F, FPr>(&mut self, destination: Option<&ANode<T, I, R, C>>, condition: F, process: FPr)
    where
        F : Fn(&I, &C) -> bool + 'static,
        FPr: Fn(I, &mut C) -> R + 'static
    {
        self.link_update(destination, condition, process, |_| {});
    }
}

pub trait LinkProcess<T, I, R, C> {
    fn link_process<F, FPr>(&mut self, destination: Option<&ANode<T, I, Option<R>, C>>, condition: F, process: FPr)
    where
        F : Fn(&I, &C) -> bool + 'static,
        FPr: Fn(I, &mut C) + 'static;
    
    fn link<F>(&mut self, destination: Option<&ANode<T, I, Option<R>, C>>, condition: F)
    where
        F : Fn(&I, &C) -> bool + 'static
    {
        self.link_process(destination, condition, |_,_|{});
    }
}

impl <T, I, R, C, Tr: Linkable<T, I, Option<R>, C>> LinkProcess<T, I, R, C> for Tr {
    fn link_process<F, FPr>(&mut self, destination: Option<&ANode<T, I, Option<R>, C>>, condition: F, process: FPr)
    where
        F : Fn(&I, &C) -> bool + 'static,
        FPr: Fn(I, &mut C) + 'static
    {
        self.link_update(destination, condition, move |input, ctx| {process(input, ctx); None},  |_| {});
    }

}

pub struct Node<T, I, R, C> {
    value: Option<T>,
    links: LinkedList<Link<T, I, R, C>>,
}

impl <T, I, R, C> Node<T, I, R, C> {
    pub fn new() -> Node<T, I, R, C> {
        Node {
            value: None,
            links: LinkedList::new(),
        }
    }
}

impl <T, I, R, C> From<T> for Node<T, I, R, C> {
    fn from(value: T) -> Self {
        Node {
            value : Some(value),
            links: LinkedList::new(),
        }
    }
}

impl <T, I, R, C> Linkable<T, I, R, C> for Node<T, I, R, C> {
    
    fn link_update<F, FUpd, FPr>(&mut self, destination: Option<&ANode<T, I, R, C>>, condition: F, process: FPr, update_link: FUpd)
    where
        F : Fn(&I, &C) -> bool + 'static,
        FUpd: FnOnce(&mut Link<T, I, R, C>),
        FPr: Fn(I, &mut C) -> R + 'static
    {
        self.links.push_back(Link::new(destination, condition, process));
        update_link(self.links.back_mut().unwrap());
    }
}

pub struct Link<T, I, R, C> {
    condition: Box<dyn Fn(&I, &C) -> bool>,
    process: Box<dyn Fn(I, &mut C) -> R>,
    destination: Option<ANode<T, I, R, C>>,
}

impl <T, I, R, C> Link<T, I, R, C> {
    pub fn new<F, FPr>(destination: Option<&ANode<T, I, R, C>>, condition: F, process: FPr) -> Link<T, I, R, C>
    where
        F : Fn(&I, &C) -> bool + 'static,
        FPr: Fn(I, &mut C) -> R + 'static
    {
        Link {
            condition: Box::new(condition),
            process: Box::new(process),
            destination: destination.map(|destination| destination.clone()),
        }
    }

    pub fn condition(&self, input: &I, context: &C) -> bool {
        (self.condition)(input, context)
    }

    pub fn process(&self, input: I, context: &mut C) -> R {
        (self.process)(input, context)
    }

    pub fn set_function<F: Fn(I, &mut C) -> R + 'static>(&mut self, fun: F) {
        self.process = Box::new(fun);
    }
}

impl <T, I, R, C> Link<T, I, Option<R>, C> {

    pub fn set_process<F: Fn(I, &mut C) + 'static>(&mut self, fun: F) {
        self.set_function(move |input, context| {fun(input, context); None});
    }
}

pub struct Cursor<T, I, R, C> {
    context: C,
    node: ANode<T, I, R, C>,
}

impl <T, I, R, C> Cursor<T, I, R, C> {
    pub fn new(context: C, node: &ANode<T, I, R, C>) -> Cursor<T, I, R, C> {
        Cursor {
            context,
            node: node.clone(),
        }
    }

    pub fn action(&mut self, input: I) -> Option<R> {
        let mut node = None;
        let mut res = None;
        for link in self.node.node.borrow_mut().links.iter() {
            if link.condition(&input, &self.context) {
                res = Some(link.process(input, &mut self.context));
                node = if let Some(destination) = &link.destination { Some(destination.clone()) } else { None };
                break;
            }
        }
        if let Some(node) = node {
            self.node = node;
        }

        res
    }

    pub fn context(&self) -> &C {
        &self.context
    }

    pub fn access_data<F: FnOnce(Option<&T>, &mut C)>(&mut self, fun: F) {
        fun(self.node.node.borrow().value.as_ref(), &mut self.context);
    }
    
    pub fn into_context(self) -> C {
        self.context
    }
}

pub fn eq<T: std::cmp::PartialEq, C>(input: T) -> impl Fn(&T, &C) -> bool {
    move |el, _| el == &input
}

pub fn not_eq<T: std::cmp::PartialEq, C>(input: T) -> impl Fn(&T, &C) -> bool {
    move |el, _| el != &input
}

pub fn node<T, I, R, C>() -> ANode<T, I, R, C> {
    ANode::new()
} 

#[cfg(test)]
mod tests {

    use crate::*;

    type TNode = ANode<(), char, (), ()>;
    
    #[test]
    fn create_automa() {

        let v = Rc::new(RefCell::new(Vec::new()));
        let v2 = Rc::clone(&v);


        let mut node1 = TNode::new();
        let node2 = TNode::new();

        node1.link_function(Some(&node2), eq('A'), move |_,_| v2.borrow_mut().push("help"));

        let mut cursor = Cursor::new((), &node1);
        cursor.action('B');
        assert_eq!(0, v.borrow().len());
        cursor.action('A');

        assert_eq!("help", v.borrow()[0]);
    }

    #[test]
    fn read_string() {

        enum StrStatus {
            StringEnd,
        }

        type StrNode = ANode<bool, char, Option<StrStatus>, (bool, Vec<char>)>;

        let mut n1 = StrNode::new();
        let mut n2 = StrNode::new();
        let n3 = StrNode::from(true);

        n1.link(Some(&n2), eq('"'));

        n2.link_process(None, |input,_| input >= &'a' && input <= &'z', |input, context| context.1.push(input));

        n2.link_function(Some(&n3), eq('"'), |_,_| Some(StrStatus::StringEnd));

        let input = r#""stringa"#;
        let mut cursor = Cursor::new((false, Vec::new()), &n1);
        input.chars().for_each(|c| { cursor.action(c); });

        
        let result = cursor.action('"');
        cursor.access_data(|d, ctx| ctx.0 = *d.unwrap());

        assert_eq!(true, cursor.context().0);

        match result.flatten() {
            Some(StrStatus::StringEnd) => {
                let output: String = cursor.into_context().1.iter().collect();
                assert_eq!("stringa", output);
            },
            _ => assert!(false),
        }

    }
}