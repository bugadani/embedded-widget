use core::marker::PhantomData;

use object_chain::{Chain, ChainElement, Link};

use crate::{
    input::event::InputEvent,
    widgets::{
        layouts::linear::{Cell, CellWeight, LinearLayoutChainElement},
        ParentHolder, Widget, WidgetStateHolder,
    },
    BoundingBox, Canvas, MeasureConstraint, MeasureSpec, MeasuredSize, Position, WidgetRenderer,
};

pub struct Column<C, CE> {
    parent_index: usize,
    bounds: BoundingBox,
    widgets: CE,
    _marker: PhantomData<C>,
}

impl<C> Column<C, ()>
where
    C: Canvas,
{
    pub fn new<W, CW>(widget: Cell<W, CW>) -> Column<C, Chain<Cell<W, CW>>>
    where
        W: Widget,
        CW: CellWeight,
    {
        Column {
            parent_index: 0,
            bounds: BoundingBox::default(),
            widgets: Chain::new(widget),
            _marker: PhantomData,
        }
    }
}

impl<C, CE> Column<C, CE>
where
    C: Canvas,
    CE: LinearLayoutChainElement<C> + ChainElement,
{
    pub fn add<W, CW>(self, widget: Cell<W, CW>) -> Column<C, Link<Cell<W, CW>, CE>>
    where
        W: Widget,
        CW: CellWeight,
    {
        Column {
            parent_index: self.parent_index,
            bounds: self.bounds,
            widgets: self.widgets.append(widget),
            _marker: PhantomData,
        }
    }

    fn locate(&self, mut idx: usize) -> Option<(usize, usize)> {
        let children = self.widgets.len();

        for i in 0..children {
            let child = self.widgets.at(i).widget();
            let grandchildren = child.children();
            if idx <= grandchildren {
                return Some((i, idx));
            }

            idx -= grandchildren + 1;
        }

        None
    }
}

impl<C, CE> Widget for Column<C, CE>
where
    C: Canvas,
    CE: LinearLayoutChainElement<C> + ChainElement,
{
    fn bounding_box(&self) -> BoundingBox {
        self.bounds
    }

    fn bounding_box_mut(&mut self) -> &mut BoundingBox {
        &mut self.bounds
    }

    fn measure(&mut self, measure_spec: MeasureSpec) {
        let max_height = match measure_spec.height {
            MeasureConstraint::AtMost(max) | MeasureConstraint::Exactly(max) => max,
            MeasureConstraint::Unspecified => {
                // We can do whatever
                let count = self.widgets.len();
                for i in 0..count {
                    self.widgets.at_mut(i).widget_mut().measure(measure_spec);
                }
                return;
            }
        };

        let count = self.widgets.len();
        let mut fixed_heights = 0;
        let mut max_width = 0;
        let mut total_weight = 0;

        // Count the height of the widgets that don't have a weight
        for i in 0..count {
            let cell = self.widgets.at_mut(i);
            let weight = cell.weight();
            if weight == 0 {
                let spec = MeasureSpec {
                    width: measure_spec.width,
                    height: MeasureConstraint::AtMost(max_height - fixed_heights),
                };

                let widget = cell.widget_mut();
                widget.measure(spec);
                fixed_heights += widget.bounding_box().size.height;
                max_width = max_width.max(widget.bounding_box().size.width);
            } else {
                total_weight += weight;
            }
        }

        // Divide the rest of the space among the weighted widgets
        if total_weight != 0 {
            let remaining_space = max_height - fixed_heights;
            let height_per_weight_unit = remaining_space / total_weight;
            // in case we have some stray pixels, divide them up evenly
            let remainder = remaining_space % total_weight;

            for i in 0..count {
                let cell = self.widgets.at_mut(i);
                let weight = cell.weight();
                if weight != 0 {
                    let spec = MeasureSpec {
                        width: measure_spec.width,
                        height: MeasureConstraint::Exactly(
                            height_per_weight_unit * weight + ((i as u32) < remainder) as u32,
                        ),
                    };

                    let widget = cell.widget_mut();
                    widget.measure(spec);
                    max_width = max_width.max(widget.bounding_box().size.width);
                }
            }
        }

        self.set_measured_size(MeasuredSize {
            height: if total_weight == 0 {
                fixed_heights
            } else {
                max_height
            },
            width: measure_spec.width.apply_to_measured(max_width),
        })
    }

    fn arrange(&mut self, mut position: Position) {
        self.bounding_box_mut().position = position;

        let count = self.widgets.len();
        for i in 0..count {
            let widget = self.widgets.at_mut(i).widget_mut();

            widget.arrange(position);

            let height = widget.bounding_box().size.height;
            position.y += height as i32;
        }
    }

    fn children(&self) -> usize {
        let count = self.widgets.len();
        let mut children = count;
        for i in 0..count {
            children += self.widgets.at(i).widget().children();
        }

        children
    }

    fn get_child(&self, idx: usize) -> &dyn Widget {
        let (child, grandchild) = self.locate(idx).unwrap();

        let widget = self.widgets.at(child).widget();
        if grandchild == 0 {
            widget
        } else {
            widget.get_child(grandchild - 1)
        }
    }

    fn get_mut_child(&mut self, idx: usize) -> &mut dyn Widget {
        let (child, grandchild) = self.locate(idx).unwrap();

        let widget = self.widgets.at_mut(child).widget_mut();
        if grandchild == 0 {
            widget
        } else {
            widget.get_mut_child(grandchild - 1)
        }
    }

    fn test_input(&mut self, event: InputEvent) -> Option<usize> {
        let mut index = 1;

        let count = self.widgets.len();
        for i in 0..count {
            let widget = self.widgets.at_mut(i).widget_mut();

            if let Some(idx) = widget.test_input(event) {
                return Some(idx + index);
            }

            index += widget.children() + 1;
        }

        None
    }
}

impl<C, CE> ParentHolder for Column<C, CE>
where
    C: Canvas,
{
    fn parent_index(&self) -> usize {
        self.parent_index
    }

    fn set_parent(&mut self, index: usize) {
        self.parent_index = index;
    }
}

impl<C, CE> WidgetStateHolder for Column<C, CE>
where
    C: Canvas,
{
    fn change_state(&mut self, _state: u32) {}

    fn change_selection(&mut self, _state: bool) {}
}

impl<C, CE> WidgetRenderer<C> for Column<C, CE>
where
    CE: LinearLayoutChainElement<C> + ChainElement,
    C: Canvas,
{
    fn draw(&self, canvas: &mut C) -> Result<(), C::Error> {
        let count = self.widgets.len();
        for i in 0..count {
            self.widgets.at(i).draw(canvas)?;
        }
        Ok(())
    }
}
