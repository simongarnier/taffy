use core::cell::RefCell;
use std::rc::Rc;

use crate::{style::StyleBuilder, StyleBuilderError};

use super::{NodeId, TaffyError, TaffyTree};

struct StyleNode {
    style_builder: StyleBuilder,
    children: Vec<Box<StyleNode>>,
    node_id_handle: Option<NodeIdHandle>,
}

#[derive(Debug, Clone)]
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
    fn child<F>(&mut self, f: F) -> &mut StyleNode
    where
        F: FnOnce(&mut StyleNode),
    {
        let mut child_node = StyleNode::new();
        f(&mut child_node);
        self.children.push(Box::new(child_node));
        self
    }

    /// Allows for setting a handle where the node id will be set once Materialized
    fn handle(&mut self, node_id_handle: Option<NodeIdHandle>) -> &mut StyleNode {
        self.node_id_handle = node_id_handle;
        self
    }

    /// Materialize the node and all its children into the provided tree.
    fn build(&self, tree: &mut TaffyTree) -> TaffyBuilderResult<NodeId> {
        let style = self.style_builder.build()?;
        let node_id = tree.new_leaf(style)?;

        if let Some(node_id_handle) = self.node_id_handle.as_ref() {
            node_id_handle.set(node_id);
        }

        let children_node_ids: Result<Vec<_>, _> = self.children.iter().map(|child| child.build(tree)).collect();

        match children_node_ids {
            Ok(children_node_ids) => {
                tree.set_children(node_id, &children_node_ids)?;
                Ok(node_id)
            }
            Err(error) => Err(error),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        prelude::{auto, length, TaffyMaxContent},
        tree::builder::{NodeIdHandle, StyleNode},
        FlexDirection, Size, Style, TaffyTree,
    };

    #[test]
    fn readme_example() {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let header_node = tree
            .new_leaf(Style { size: Size { width: length(800.0), height: length(100.0) }, ..Default::default() })
            .unwrap();

        let body_node = tree
            .new_leaf(Style {
                size: Size { width: length(800.0), height: auto() },
                flex_grow: 1.0,
                ..Default::default()
            })
            .unwrap();

        let root_node = tree
            .new_with_children(
                Style {
                    flex_direction: FlexDirection::Column,
                    size: Size { width: length(800.0), height: length(600.0) },
                    ..Default::default()
                },
                &[header_node, body_node],
            )
            .unwrap();

        tree.compute_layout(root_node, Size::MAX_CONTENT).unwrap();

        let mut builder_tree: TaffyTree<()> = TaffyTree::new();
        let header_node_handle = NodeIdHandle::new();
        let body_node_handle = NodeIdHandle::new();

        let builder_root_node = StyleNode::new()
            .style(|s| {
                s.flex_direction(FlexDirection::Column).size(Size { width: length(800.0), height: length(600.0) });
            })
            .child(|c| {
                c.style(|s| {
                    s.size(Size { width: length(800.0), height: length(100.0) });
                })
                .handle(Some(header_node_handle.clone()));
            })
            .child(|c| {
                c.style(|s| {
                    s.size(Size { width: length(800.0), height: auto() }).flex_grow(1.0);
                })
                .handle(Some(body_node_handle.clone()));
            })
            .build(&mut builder_tree)
            .unwrap();

        builder_tree.compute_layout(builder_root_node, Size::MAX_CONTENT).unwrap();

        assert_eq!(
            tree.layout(root_node).unwrap().size.width,
            builder_tree.layout(builder_root_node).unwrap().size.width
        );
        assert_eq!(
            tree.layout(root_node).unwrap().size.height,
            builder_tree.layout(builder_root_node).unwrap().size.height
        );
        assert_eq!(
            tree.layout(header_node).unwrap().size.width,
            builder_tree.layout(header_node_handle.get().unwrap()).unwrap().size.width
        );
        assert_eq!(
            tree.layout(header_node).unwrap().size.height,
            builder_tree.layout(header_node_handle.get().unwrap()).unwrap().size.height
        );
        assert_eq!(
            tree.layout(body_node).unwrap().size.width,
            builder_tree.layout(body_node_handle.get().unwrap()).unwrap().size.width
        );
        assert_eq!(
            tree.layout(body_node).unwrap().size.height,
            builder_tree.layout(body_node_handle.get().unwrap()).unwrap().size.height
        );
    }
}
