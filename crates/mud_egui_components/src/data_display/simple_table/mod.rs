mod mud_simple_table;
mod simple_table_internal;

pub use mud_simple_table::{
    elevation_shadow, resolve_column_widths, row_height, MudSimpleTable, RowUi, SimpleTableUi,
    CELL_PADDING_X, CELL_PADDING_Y, DENSE_ROW_HEIGHT, FALLBACK_WIDTH, MAX_ELEVATION, ROW_HEIGHT,
};