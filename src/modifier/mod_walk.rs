use crate::modifier::mod_definition::{Choice, Modifiers, Prompt};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ModifierNode<'a> {
    Modifiers(&'a Modifiers),
    Prompt(&'a Prompt),
    Choice(&'a Choice),
}
