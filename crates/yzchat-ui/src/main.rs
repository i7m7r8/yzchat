//! Cross-platform Signal-like UI for yzchat (Slint, all-Rust).
//!
//! Dark theme, conversation list, message bubbles, voice/video calls,
//! screenshare, and end-to-end encrypted messaging.

use slint::{SharedString, VecModel};
use slint::slint;
use std::rc::Rc;

use yzchat_crypto::{hybrid_kem::HybridKem, CryptoError, Result};
use yzchat_media::MediaManager;

slint::slint! {
    import { VerticalBox, HorizontalBox, Text, Rectangle, Button, LineEdit, Scroll, ListView, TouchArea } from "std-widgets.slint";

    export component MessageBubble extends Rectangle {
        property <string> text: "";
        property <bool> mine: false;

        height: 40px;
        width: parent.width;
        padding: 8px;
        background: if mine then #0078ff else #2a2a2a;
        border-radius: 12px;
        cursor: default;

        Text {
            text: root.text;
            color: if mine then white else #e0e0e0;
            x: root.width * 0.05;
            y: root.height * 0.25;
            wrap: true;
            width: root.width * 0.9;
        }
    }

    export component ChatView extends VerticalBox {
        property <string> peerId: "";

        VerticalBox {
            width: parent.width;
            height: parent.height;
            spacing: 0px;

            // Header
            Rectangle {
                width: parent.width;
                height: 56px;
                background: #1a1a1a;

                HorizontalBox {
                    x: 0px; y: 0px;
                    width: parent.width; height: parent.height;

                    TouchArea => Button {
                        text: "\u{2190}";
                        width: 40px; height: parent.height;
                    }

                    Text {
                        text: root.peerId;
                        color: white;
                        font-size: 18px;
                        x: 40px;
                        y: (parent.height - 24px) / 2;
                    }

                    Button {
                        text: "\u{1F4F2}";
                        width: 44px; height: parent.height;
                        x: parent.width - 44px;
                    }

                    Button {
                        text: "\u{1F508}";
                        width: 44px; height: parent.height;
                        x: parent.width - 88px;
                    }
                }
            }

            // Messages area
            Scroll {
                width: parent.width;
                height: parent.height - 56px - 72px;

                VerticalBox {
                    id: messageList;
                    width: parent.width;
                    spacing: 8px;
                    padding: 12px;
                }
            }

            // Input
            Rectangle {
                width: parent.width;
                height: 72px;
                background: #1a1a1a;

                HorizontalBox {
                    x: 0px; y: 0px;
                    width: parent.width; height: parent.height;
                    padding: 12px;
                    spacing: 8px;

                    TouchArea => Button {
                        text: "\u{1F4CE}";
                        width: 40px; height: 40px;
                    }

                    LineEdit {
                        id: messageInput;
                        width: parent.width - 120px;
                        height: 40px;
                        placeholder-text: "Message...";
                    }

                    TouchArea => Button {
                        text: "\u{2714}";
                        width: 40px; height: 40px;
                        background: #0078ff;
                    }
                }
            }
        }
    }

    export component MainWindow inherits Window {
        property <string> selectedPeer: "";

        width: 1100px;
        height: 750px;
        background: #121212;

        VerticalBox {
            width: parent.width;
            height: parent.height;

            // Top bar
            Rectangle {
                width: parent.width;
                height: 56px;
                background: #1a1a1a;

                HorizontalBox {
                    x: 0px; y: 0px;
                    width: parent.width; height: parent.height;

                    TouchArea => Button {
                        text: "\u{2630}";
                        width: 48px; height: parent.height;
                    }

                    Text {
                        text: "yzchat";
                        color: white;
                        font-size: 20px;
                        weight: 700;
                        x: 48px;
                        y: (parent.height - 26px) / 2;
                    }

                    LineEdit {
                        width: 260px;
                        height: 32px;
                        placeholder-text: "Search...";
                        x: parent.width - 340px;
                        y: (parent.height - 32px) / 2;
                    }

                    Button {
                        text: "\u{1F4E2}";
                        width: 40px; height: parent.height;
                        x: parent.width - 80px;
                    }

                    Button {
                        text: "\u{23F3}";
                        width: 40px; height: parent.height;
                        x: parent.width - 40px;
                    }
                }
            }

            // Body: sidebar + chat
            HorizontalBox {
                width: parent.width;
                height: parent.height - 56px;

                // Sidebar
                Rectangle {
                    width: 280px;
                    height: parent.height;
                    background: #1a1a1a;

                    VerticalBox {
                        width: parent.width;
                        height: parent.height;

                        TouchArea => Button {
                            text: "\u{2709} New chat";
                            width: parent.width; height: 48px;
                            background: transparent;
                            color: #b0b0b0;
                        }

                        Text {
                            text: "CONTACTS";
                            color: #666666;
                            font-size: 11px;
                            x: 16px; y: 16px;
                        }

                        Scroll {
                            width: parent.width;
                            height: parent.height - 90px;

                            ListView {
                                id: contactList;
                                width: parent.width;
                                row-height: 56px;
                                background: transparent;
                            }
                        }
                    }
                }

                // Chat area
                ChatView {
                    width: parent.width;
                    height: parent.height;
                    peerId: root.selectedPeer;
                }
            }
        }
    }
}

struct Backend {
    media: MediaManager,
    contacts: Rc<VecModel<SharedString>>,
}

impl Backend {
    fn new() -> Result<Self> {
        Ok(Self {
            media: MediaManager::new(),
            contacts: Rc::new(VecModel::default()),
        })
    }

    fn generate_identity(&self) -> Result<yzchat_crypto::hybrid_kem::HybridKemKeypair> {
        HybridKem::generate()
    }
}

fn main() -> Result<()> {
    let backend = Rc::new(Backend::new()?);
    let _identity = backend.generate_identity()?;

    let window = MainWindow::new();
    window.show();

    slint::platform::event_loop().map_err(|e| CryptoError::Other(e.into()))?;

    Ok(())
}
