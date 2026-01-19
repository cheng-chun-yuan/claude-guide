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
    ChangePlatform,
    ManageVersions,
    ToggleScope,
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
