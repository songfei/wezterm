use crate::termwindow::render::TripleLayerQuadAllocator;
use config::DimensionContext;
use mux::tab::PositionedPane;

impl crate::TermWindow {
    /// Draws a solid border around the active pane when
    /// `active_pane_border` is configured and more than one pane
    /// is visible.
    ///
    /// Sides that face a split are centered on the split line, which
    /// sits in the middle of the one-cell gutter between panes, so
    /// borders up to one cell wide never cover text. Sides that face
    /// the window edge are placed within the window padding.
    pub fn paint_active_pane_border(
        &mut self,
        layers: &mut TripleLayerQuadAllocator,
        panes: &[PositionedPane],
    ) -> anyhow::Result<()> {
        let border_config = match self.config.active_pane_border {
            Some(border_config) => border_config,
            None => return Ok(()),
        };
        if panes.len() < 2 {
            return Ok(());
        }
        let pos = match panes.iter().find(|p| p.is_active) {
            Some(pos) => pos,
            None => return Ok(()),
        };

        let cell_width = self.render_metrics.cell_size.width as f32;
        let cell_height = self.render_metrics.cell_size.height as f32;

        let width = border_config.width.evaluate_as_pixels(DimensionContext {
            dpi: self.dimensions.dpi as f32,
            pixel_max: self.terminal_size.pixel_width as f32,
            pixel_cell: cell_width,
        });
        if width <= 0. {
            return Ok(());
        }
        let color = border_config.color.to_linear().into();

        let tab_bar_height = if self.show_tab_bar {
            self.tab_bar_pixel_height()?
        } else {
            0.
        };
        let (top_bar_height, bottom_bar_height) = if self.config.tab_bar_at_bottom {
            (0., tab_bar_height)
        } else {
            (tab_bar_height, 0.)
        };

        let os_border = self.get_os_border();
        let (padding_left, padding_top) = self.padding_left_top();

        // Pixel bounds of the pane's text area
        let text_left = padding_left + os_border.left.get() as f32 + pos.left as f32 * cell_width;
        let text_top = top_bar_height
            + padding_top
            + os_border.top.get() as f32
            + pos.top as f32 * cell_height;
        let text_right = text_left + pos.width as f32 * cell_width;
        let text_bottom = text_top + pos.height as f32 * cell_height;

        // Distance from the text edge to the center of the border line.
        // Facing a split: half a cell, matching the split line position.
        // Facing the window edge: as far out as the gutter allows,
        // without exceeding half a cell.
        let offset = |faces_split: bool, half_cell: f32, gutter: f32| {
            if faces_split {
                half_cell
            } else {
                (gutter - width / 2.).min(half_cell).max(width / 2.)
            }
        };

        let left_gutter = text_left - os_border.left.get() as f32;
        let top_gutter = text_top - top_bar_height - os_border.top.get() as f32;
        let right_gutter =
            self.dimensions.pixel_width as f32 - os_border.right.get() as f32 - text_right;
        let bottom_gutter = self.dimensions.pixel_height as f32
            - bottom_bar_height
            - os_border.bottom.get() as f32
            - text_bottom;

        let x0 = text_left - offset(pos.left > 0, cell_width / 2., left_gutter);
        let y0 = text_top - offset(pos.top > 0, cell_height / 2., top_gutter);
        let x1 = text_right
            + offset(
                pos.left + pos.width < self.terminal_size.cols as usize,
                cell_width / 2.,
                right_gutter,
            );
        let y1 = text_bottom
            + offset(
                pos.top + pos.height < self.terminal_size.rows as usize,
                cell_height / 2.,
                bottom_gutter,
            );

        let half = width / 2.;
        let outer_width = x1 - x0 + width;
        let outer_height = y1 - y0 + width;
        for rect in [
            euclid::rect(x0 - half, y0 - half, outer_width, width),
            euclid::rect(x0 - half, y1 - half, outer_width, width),
            euclid::rect(x0 - half, y0 - half, width, outer_height),
            euclid::rect(x1 - half, y0 - half, width, outer_height),
        ] {
            self.filled_rectangle(layers, 2, rect, color)?;
        }

        Ok(())
    }
}
