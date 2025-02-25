use super::{
    AlignContent, AlignItems, AlignSelf, BoxSizing, Dimension, Display, JustifyContent, LengthPercentage,
    LengthPercentageAuto, Overflow, Position, Style,
};
use crate::{util::sys::Vec, Line, NodeId, Point, Rect, Size, TaffyResult, TaffyTree};
use core::cell::RefCell;
use std::rc::Rc;

#[cfg(feature = "flexbox")]
use super::{FlexDirection, FlexWrap};
#[cfg(feature = "grid")]
use {
    super::{GridAutoFlow, GridPlacement, NonRepeatedTrackSizingFunction, TrackSizingFunction},
    crate::sys::GridTrackVec,
};

/// some macro
macro_rules! builder {
    // Change how we capture the cfg condition
    ($builder:ident, $(($field:ident: $type:ty $(, cfg: $($cfg:tt)+)?)),* $(,)?) => {
        /**
        * use `StyleBuilder` to construct a tree of nested style.
        */
        #[derive(Debug, Default)]
        pub struct StyleBuilder<'a> {
            children: Vec<&'a StyleBuilder<'a>>,
            ref_handle: Option<RefHandle>,
            $(
                $(#[cfg($($cfg)+)])?
                $field: Option<$type>,
            )*
        }

        impl<'a> $builder<'a> {
            $(
                $(#[cfg($($cfg)+)])?
                #[doc = concat!("Will set the `", stringify!($field), "` field to the provided value in the")]
                #[doc = "\nresulting [`Style`](super::Style) when the [`build`](StyleBuilder::build) method is called."]
                #[doc = concat!("\n\nSee [`Style::", stringify!($field), "`](super::Style::", stringify!($field), ").")]
                pub fn $field(&mut self, $field: impl Into<$type>) -> &mut Self {
                    self.$field = Some($field.into());
                    self
                }
            )*

            fn build_style(&self) -> Style {
                let mut style = Style::default();

                $(
                    $(#[cfg($($cfg)+)])?
                    if let Some(ref value) = self.$field {
                        style.$field = Clone::clone(value);
                    }
                )*

                style
            }
        }
    };
}

builder!(
    StyleBuilder,
    (display: Display),
    (item_is_table: bool),
    (box_sizing: BoxSizing),
    (overflow: Point<Overflow>),
    (scrollbar_width: f32),
    (position: Position),
    (inset: Rect<LengthPercentageAuto>),
    (size: Size<Dimension>),
    (min_size: Size<Dimension>),
    (max_size: Size<Dimension>),
    (aspect_ratio: Option<f32>),
    (margin: Rect<LengthPercentageAuto>),
    (padding: Rect<LengthPercentage>),
    (border: Rect<LengthPercentage>),
    (align_items: Option<AlignItems>, cfg: any(feature = "flexbox", feature = "grid")),
    (align_self: Option<AlignSelf>, cfg: any(feature = "flexbox", feature = "grid")),
    (justify_items: Option<AlignItems>, cfg: feature = "grid"),
    (justify_self: Option<AlignSelf>, cfg: feature = "grid"),
    (align_content: Option<AlignContent>, cfg: any(feature = "flexbox", feature = "grid")),
    (justify_content: Option<JustifyContent>, cfg: any(feature = "flexbox", feature = "grid")),
    (gap: Size<LengthPercentage>, cfg: any(feature = "flexbox", feature = "grid")),
    (text_align: super::TextAlign, cfg: feature = "block_layout"),
    (flex_direction: FlexDirection, cfg: feature = "flexbox"),
    (flex_wrap: FlexWrap, cfg: feature = "flexbox"),
    (flex_basis: Dimension, cfg: feature = "flexbox"),
    (flex_grow: f32, cfg: feature = "flexbox"),
    (flex_shrink: f32, cfg: feature = "flexbox"),
    (grid_template_rows: GridTrackVec<TrackSizingFunction>, cfg: feature = "grid"),
    (grid_template_columns: GridTrackVec<TrackSizingFunction>, cfg: feature = "grid"),
    (grid_auto_rows: GridTrackVec<NonRepeatedTrackSizingFunction>, cfg: feature = "grid"),
    (grid_auto_columns: GridTrackVec<NonRepeatedTrackSizingFunction>, cfg: feature = "grid"),
    (grid_auto_flow: GridAutoFlow, cfg: feature = "grid"),
    (grid_row: Line<GridPlacement>, cfg: feature = "grid"),
    (grid_column: Line<GridPlacement>, cfg: feature = "grid"),
);

#[derive(Debug, Clone)]
struct RefHandle(Rc<RefCell<Option<NodeId>>>);

impl RefHandle {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    fn set(&self, node_id: NodeId) {
        *self.0.borrow_mut() = Some(node_id)
    }

    pub fn get(&self) -> Option<NodeId> {
        self.0.borrow().clone()
    }
}

impl<'a> StyleBuilder<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn row() -> Self {
        let mut row = Self::new();
        row.flex_direction(FlexDirection::Row);
        row
    }

    pub fn column() -> Self {
        let mut column = Self::new();
        column.flex_direction(FlexDirection::Column);
        column
    }

    pub fn child(&'a mut self, style_builder: &'a StyleBuilder) -> &'a mut StyleBuilder<'a> {
        self.children.push(style_builder);
        self
    }

    pub fn build(&self, tree: &mut TaffyTree) -> TaffyResult<NodeId> {
        let style = self.build_style();
        let node_id = tree.new_leaf(style)?;

        if let Some(ref_handle) = self.ref_handle.as_ref() {
            ref_handle.set(node_id);
        }

        let children_node_ids = self.children.iter().map(|child| child.build(tree)).collect::<Result<Vec<_>, _>>()?;

        tree.set_children(node_id, &children_node_ids)?;

        Ok(node_id)
    }

    pub fn handle(&'a mut self, ref_handle: RefHandle) -> &'a mut StyleBuilder<'a> {
        self.ref_handle = Some(ref_handle);
        self
    }
}

#[cfg(test)]
mod test {
    use crate::{
        prelude::{auto, length, TaffyMaxContent},
        style::builder::RefHandle,
        FlexDirection, Size, TaffyTree,
    };

    use super::{Style, StyleBuilder};

    #[test]
    fn builder_defaults_match_defaults() {
        assert_eq!(StyleBuilder::default().build_style(), Style::default())
    }

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
        let header_node_handle = RefHandle::new();
        let body_node_handle = RefHandle::new();

        let builder_root_node = StyleBuilder::new()
            .flex_direction(FlexDirection::Column)
            .size(Size { width: length(800.0), height: length(600.0) })
            .child(
                StyleBuilder::new()
                    .size(Size { width: length(800.0), height: length(100.0) })
                    .handle(header_node_handle.clone()),
            )
            .child(
                StyleBuilder::new()
                    .size(Size { width: length(800.0), height: auto() })
                    .flex_grow(1.0)
                    .handle(body_node_handle.clone()),
            )
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

    #[test]
    fn row() {
        assert_eq!(
            StyleBuilder::row().build_style(),
            Style { flex_direction: FlexDirection::Row, ..Default::default() }
        )
    }

    #[test]
    fn column() {
        assert_eq!(
            StyleBuilder::column().build_style(),
            Style { flex_direction: FlexDirection::Column, ..Default::default() }
        )
    }
}
