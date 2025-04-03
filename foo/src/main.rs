// Definition for singly-linked list.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    #[inline]
    fn new(val: i32) -> Self {
        ListNode { next: None, val }
    }
}

pub fn reverse_list(head: Option<Box<ListNode>>) -> Option<Box<ListNode>> {
    let mut curr = head?;
    let mut next = curr.clone().next?.clone();
    while !next.next.is_none() {
        println!("{next:?}");
        next.next = Some(curr.clone());
        curr = next;
        next = curr.clone().next?.clone();
    }
    next.next = Some(curr);
    return Some(next.clone());
}

fn main() {
    let mut n1 = Box::new(ListNode::new(1));
    let mut n2 = Box::new(ListNode::new(2));
    let mut n3 = Box::new(ListNode::new(3));
    let n4 = Box::new(ListNode::new(4));

    n3.next = Some(n4);
    n2.next = Some(n3);
    n1.next = Some(n2);

    println!("{n1:#?}");

    // let r = reverse_list(Some(n1));

    // println!("{r:#?}");
}
