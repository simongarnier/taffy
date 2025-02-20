use core::cell::RefCell;
use std::rc::Rc;

use crate::{style::StyleBuilder, StyleBuilderError};

use super::{NodeId, TaffyError, TaffyTree};

struct StyleNode {
    style_builder: StyleBuilder,
    children: Vec<Box<StyleNode>>,
    node_id_handle: Option<NodeIdHandle>,
}

struct NodeIdHandle(Rc<RefCell<Option<NodeId>>>);

impl NodeIdHandle {
    fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    fn set(&self, node_id: NodeId) {
        *self.0.borrow_mut() = Some(node_id)
    }

    pub fn get(&self) -> Option<NodeId> {
        self.0.borrow().clone()
    }
}

#[derive(Debug)]
pub enum StyleNodeError {
    BuilderFailed(StyleBuilderError),
    TaffyComputeError(TaffyError),
}

impl From<StyleBuilderError> for StyleNodeError {
    fn from(value: StyleBuilderError) -> Self {
        Self::BuilderFailed(value)
    }
}

impl From<TaffyError> for StyleNodeError {
    fn from(value: TaffyError) -> Self {
        Self::TaffyComputeError(value)
    }
}

type TaffyBuilderResult<T> = Result<T, StyleNodeError>;

impl StyleNode {
    /// Create a new StyleTree
    fn new() -> Self {
        Self { style_builder: StyleBuilder::default(), children: Vec::new(), node_id_handle: None }
    }

    /// Allows for customizing the style.
    fn style<F>(&mut self, f: F) -> &mut StyleNode
    where
        F: FnOnce(&mut StyleBuilder),
    {
        f(&mut self.style_builder);
        self
    }

    /// Allows for adding a child node.
    fn child(&mut self, style_node: StyleNode) -> &mut StyleNode {
        self.children.push(Box::new(style_node));
        self
    }

    /// Allows for setting a handle where the node id will be set once Materialized
    fn handle(&mut self, node_id_handle: Option<NodeIdHandle>) -> &mut StyleNode {
        self.node_id_handle = node_id_handle;
        self
    }

    /// Materialize the node and all its children into the provided tree.
    fn build(&self, tree: &mut TaffyTree) -> TaffyBuilderResult<NodeId> {
        let root_id = tree.new_leaf(self.style_builder.build()?)?;

        if let Some(node_id_handle) = self.node_id_handle.as_ref() {
            node_id_handle.set(root_id);
        }

        self.children.iter().try_for_each(|child| child.build(tree).map(|_| ()))?;

        Ok(root_id)
    }
}
