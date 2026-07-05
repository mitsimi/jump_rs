use maud::{Markup, html};

#[derive(Debug, Clone, Copy)]
pub enum Icon {
    AlertCircle,
    Check,
    Database,
    Download,
    Info,
    Pencil,
    Plus,
    Power,
    Search,
    Trash2,
    Upload,
    X,
}

impl Icon {
    const fn label(self) -> &'static str {
        match self {
            Self::AlertCircle => "Alert",
            Self::Check => "Success",
            Self::Database => "Data",
            Self::Download => "Download",
            Self::Info => "Information",
            Self::Pencil => "Edit",
            Self::Plus => "Add",
            Self::Power => "Wake",
            Self::Search => "Lookup",
            Self::Trash2 => "Remove",
            Self::Upload => "Upload",
            Self::X => "Close",
        }
    }
}

pub fn icon(icon: Icon) -> Markup {
    html! {
        svg
            class="icon"
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true" {
            (paths(icon))
        }
    }
}

pub fn icon_with_label(icon: Icon) -> Markup {
    html! {
        svg
            class="icon"
            xmlns="http://www.w3.org/2000/svg"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            role="img"
            aria-label=(icon.label()) {
            title { (icon.label()) }
            (paths(icon))
        }
    }
}

fn paths(icon: Icon) -> Markup {
    match icon {
        Icon::AlertCircle => html! {
            circle cx="12" cy="12" r="10" {}
            line x1="12" x2="12" y1="8" y2="12" {}
            line x1="12" x2="12.01" y1="16" y2="16" {}
        },
        Icon::Check => html! {
            path d="M20 6 9 17l-5-5" {}
        },
        Icon::Database => html! {
            ellipse cx="12" cy="5" rx="9" ry="3" {}
            path d="M3 5v14c0 1.66 4.03 3 9 3s9-1.34 9-3V5" {}
            path d="M3 12c0 1.66 4.03 3 9 3s9-1.34 9-3" {}
        },
        Icon::Download => html! {
            path d="M12 15V3" {}
            path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" {}
            path d="m7 10 5 5 5-5" {}
        },
        Icon::Info => html! {
            circle cx="12" cy="12" r="10" {}
            path d="M12 16v-4" {}
            path d="M12 8h.01" {}
        },
        Icon::Pencil => html! {
            path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" {}
            path d="m15 5 4 4" {}
        },
        Icon::Plus => html! {
            path d="M5 12h14" {}
            path d="M12 5v14" {}
        },
        Icon::Power => html! {
            path d="M12 2v10" {}
            path d="M18.4 6.6a9 9 0 1 1-12.77.04" {}
        },
        Icon::Search => html! {
            path d="m21 21-4.34-4.34" {}
            circle cx="11" cy="11" r="8" {}
        },
        Icon::Trash2 => html! {
            path d="M3 6h18" {}
            path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" {}
            path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" {}
            line x1="10" x2="10" y1="11" y2="17" {}
            line x1="14" x2="14" y1="11" y2="17" {}
        },
        Icon::Upload => html! {
            path d="M12 3v12" {}
            path d="m17 8-5-5-5 5" {}
            path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" {}
        },
        Icon::X => html! {
            path d="M18 6 6 18" {}
            path d="m6 6 12 12" {}
        },
    }
}
