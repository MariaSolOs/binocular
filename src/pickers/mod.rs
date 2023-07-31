use anyhow::Result;
use ratatui::widgets::ListItem;
use tokio::sync::mpsc::Sender;

mod grep;
pub use grep::{GrepItem, GrepPicker};

/// An item returned by a Binocular picker.
pub trait PickerItem {
    /// Returns a `ratatui` list item representing the match.
    fn as_list_item(&self) -> ListItem;

    /// Returns a preview of the match to be displayed in the TUI.
    fn preview(&self) -> String;
}

/// A Binocular picker.
pub trait Picker<I: PickerItem> {
    /// Returns the picker's name.
    fn name(&self) -> &'static str;

    /// Returns the picker's preview title.
    fn preview_title(&self) -> &'static str;

    /// Handles changes in the search input field.
    /// `sender` can be used to communicate back with the application.
    fn handle_input_change(&self, input: String, sender: Sender<Vec<I>>);

    /// Handles selection events.
    fn handle_selection(&self, item: &I) -> Result<()>;
}
