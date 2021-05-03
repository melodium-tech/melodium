
use std::rc::{Rc, Weak};
use std::cell::RefCell;
use super::super::designer::SequenceDesigner;

pub struct SequenceBuilder {
    pub designer: Rc<RefCell<SequenceDesigner>>
}
