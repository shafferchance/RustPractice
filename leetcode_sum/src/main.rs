use core::panic;
use std::boxed::Box;

// Definition for singly-linked list.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ListNode {
  pub val: i32,
  pub next: Option<Box<ListNode>>
}

impl ListNode {
  #[inline]
  fn new(val: i32) -> Self {
    ListNode {
      next: None,
      val
    }
  }
}

fn get_value_from_nodes(node_1: Option<&Box<ListNode>>, node_2: Option<&Box<ListNode>>, carry_flag: bool) -> i32 {
  match (node_1, node_2) {
      (Some(node_1_value), Some(node_2_value)) => {
          node_1_value.val + node_2_value.val + if carry_flag { 1 } else { 0 }
      }
      (Some(node_1_value), None) => {
          node_1_value.val + if carry_flag { 1 } else { 0 }
      }
      (None, Some(node_2_value)) => {
          node_2_value.val + if carry_flag { 1 } else { 0 }
      }
      (None, None) => {
        panic!("Should break before getting here")
      }
  }
}

fn get_final_value (total: i32, carry_flag: bool) -> i32 {
  if carry_flag { total % 10 } else { total }
}

fn append_final_carry(new_list_cursor: Option<&mut Box<ListNode>>, carry_flag: bool) {
  if carry_flag {
    if let Some(end_of_new_list) = new_list_cursor {
      end_of_new_list.next = Some(Box::from(ListNode::new(1)));
    }
  }
}

fn main() {
    let mut l1_1 = Box::from(ListNode::new(9));
    let mut l1_2 = Box::from(ListNode::new(9));
    let mut l1_3 = Box::from(ListNode::new(9));
    let mut l1_4 = Box::from(ListNode::new(9));
    let mut l1_5 = Box::from(ListNode::new(9));
    let mut l1_6 = Box::from(ListNode::new(9));
    let l1_7 = Box::from(ListNode::new(9));
    l1_6.next = Option::from(l1_7);
    l1_5.next = Option::from(l1_6);
    l1_4.next = Option::from(l1_5);
    l1_3.next = Option::from(l1_4);
    l1_2.next = Option::from(l1_3);
    l1_1.next = Option::from(l1_2);

    let mut l2_1 = Box::from(ListNode::new(9));
    let mut l2_2= Box::from(ListNode::new(9));
    let mut l2_3 = Box::from(ListNode::new(9));
    let l2_4 = Box::from(ListNode::new(9));
    l2_3.next = Option::from(l2_4);
    l2_2.next = Option::from(l2_3);
    l2_1.next = Option::from(l2_2);


    let mut current_node_1 = Option::from(l1_1);
    let mut current_node_2 = Option::from(l2_1);
    let mut new_list: Box<ListNode> =  Box::from(ListNode::new(0));
    let mut new_list_cursor: Option<&mut Box<ListNode>> = None;
    let mut carry_flag = false;

    loop {
      let total = get_value_from_nodes(current_node_1.as_ref(), current_node_2.as_ref(), carry_flag);

      carry_flag = total > 9;

      // Append values to new linked list
      new_list_cursor = if let Some(current_new_list_node) = new_list_cursor {
        current_new_list_node.next = Some(Box::from(ListNode::new(get_final_value(total, carry_flag))));
        current_new_list_node.next.as_mut()
      } else {
        new_list.val = get_final_value(total, carry_flag);
        Some(&mut new_list)
      };

      match (&current_node_1, &current_node_2) {
        (Some(current_node_1_val), Some(current_node_2_val)) => {
          if current_node_1_val.next.is_none() && current_node_2_val.next.is_none() {
            append_final_carry(new_list_cursor, carry_flag);
            break;
          }
        }
        (Some(current_node_1), None) => {
          if current_node_1.next.is_none() {
            append_final_carry(new_list_cursor, carry_flag);
            break;
          }
        }
        (None, Some(current_node_2)) => {
          if current_node_2.next.is_none() {
            append_final_carry(new_list_cursor, carry_flag);
            break;
          }
        }
        (None, None) => panic!("Neither are set what?!")
      }

      // Steps forward in both linked lists
      current_node_1 = match current_node_1.as_ref() {
        Some(current_node_1_val) => current_node_1_val.next.clone(),
        None => None
      };

      current_node_2 = match current_node_2.as_ref() {
        Some(current_node_2_val) => current_node_2_val.next.clone(),
        None => None
      };
    }

    println!("Result: {:?}", new_list)
}
