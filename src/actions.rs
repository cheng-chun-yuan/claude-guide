/// Application actions that can be triggered by keybindings
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    // Navigation
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    NextTab,
    PrevTab,
    GoToTab(usize),

    // Selection
    Select,
    Cancel,

    // CRUD operations
    Add,
    Edit,
    Delete,
    Save,
    Toggle,

    // Application
    ShowHelp,
    Quit,
    ForceQuit,
    Refresh,

    // Modal
    Confirm,
    Dismiss,

    // Text input
    InputChar(char),
    InputBackspace,
    InputDelete,
    InputLeft,
    InputRight,
    InputHome,
    InputEnd,
}

impl Action {
    pub fn description(&self) -> &'static str {
        match self {
            Action::MoveUp => "Move up",
            Action::MoveDown => "Move down",
            Action::MoveLeft => "Move left",
            Action::MoveRight => "Move right",
            Action::NextTab => "Next tab",
            Action::PrevTab => "Previous tab",
            Action::GoToTab(_) => "Go to tab",
            Action::Select => "Select item",
            Action::Cancel => "Cancel",
            Action::Add => "Add new item",
            Action::Edit => "Edit selected item",
            Action::Delete => "Delete selected item",
            Action::Save => "Save changes",
            Action::Toggle => "Toggle item",
            Action::ShowHelp => "Show help",
            Action::Quit => "Quit",
            Action::ForceQuit => "Force quit",
            Action::Refresh => "Refresh",
            Action::Confirm => "Confirm",
            Action::Dismiss => "Dismiss",
            Action::InputChar(_) => "Input character",
            Action::InputBackspace => "Backspace",
            Action::InputDelete => "Delete",
            Action::InputLeft => "Cursor left",
            Action::InputRight => "Cursor right",
            Action::InputHome => "Cursor to start",
            Action::InputEnd => "Cursor to end",
        }
    }
}
