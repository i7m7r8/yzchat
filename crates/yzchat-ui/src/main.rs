//! Cross-platform Signal-like UI for yzchat (Slint, all-Rust).
//! Dark theme window; conversation list and message bubbles render here.

use slint::{SharedString, VecModel};
use std::rc::Rc;

use yzchat_crypto::{hybrid_kem::HybridKem, CryptoError, Result};
use yzchat_media::MediaManager;

slint::slint! {
    #[slint::edition = "1.19"]
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

                Text {
                    text: "yzchat";
                    color: white;
                    font-size: 20px;
                    weight: 700;
                    x: 48px;
                    y: (parent.height - 26px) / 2;
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

                        Text {
                            text: "CONTACTS";
                            color: #666666;
                            font-size: 11px;
                            x: 16px; y: 16px;
                        }

                        ListView {
                            id: contactList;
                            width: parent.width;
                            row-height: 56px;
                            background: transparent;
                        }
                    }
                }

                // Chat area
                VerticalBox {
                    width: parent.width;
                    height: parent.height;
                    spacing: 0px;

                    // Header
                    Rectangle {
                        width: parent.width;
                        height: 56px;
                        background: #1a1a1a;

                        Text {
                            text: root.selectedPeer;
                            color: white;
                            font-size: 18px;
                            x: 40px;
                            y: (parent.height - 24px) / 2;
                        }
                    }

                    // Messages area
                    VerticalBox {
                        id: messageList;
                        width: parent.width;
                        height: parent.height - 56px - 72px;
                        spacing: 8px;
                        padding: 12px;
                    }

                    // Input
                    Rectangle {
                        width: parent.width;
                        height: 72px;
                        background: #1a1a1a;

                        LineEdit {
                            id: messageInput;
                            width: parent.width - 120px;
                            height: 40px;
                            x: 60px;
                            y: 16px;
                            placeholder-text: "Message...";
                        }
                    }
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
