//! Callback trait definitions for FFI

use crate::types::*;
use uniffi::Object;

/// Message event callback
#[uniffi::callback(interface = "MessageEventCallback")]
pub trait MessageEventCallback: Send + Sync {
    fn on_message(&self, message: Message);
}

/// Call event callback
#[uniffi::callback(interface = "CallEventCallback")]
pub trait CallEventCallback: Send + Sync {
    fn on_call_event(&self, call_id: String, event: CallEvent);
}

/// Contact event callback
#[uniffi::callback(interface = "ContactEventCallback")]
pub trait ContactEventCallback: Send + Sync {
    fn on_contact_event(&self, contact: Contact, event: ContactEvent);
}

/// Connection change callback
#[uniffi::callback(interface = "ConnectionChangeCallback")]
pub trait ConnectionChangeCallback: Send + Sync {
    fn on_connection_change(&self, connected: bool, peer_count: u32);
}

/// Incoming call callback
#[uniffi::callback(interface = "IncomingCallCallback")]
pub trait IncomingCallCallback: Send + Sync {
    fn on_incoming_call(
        &self,
        call_id: String,
        from: Vec<u8>,
        session_type: MediaSessionType,
        quality: MediaQuality,
    );
}

/// Media stats callback
#[uniffi::callback(interface = "MediaStatsCallback")]
pub trait MediaStatsCallback: Send + Sync {
    fn on_media_stats(&self, call_id: String, stats: MediaStats);
}

/// Message callback (for P2P)
#[uniffi::callback(interface = "MessageCallback")]
pub trait MessageCallback: Send + Sync {
    fn on_message(&self, from: Vec<u8>, protocol: String, payload: Vec<u8>);
}

/// Signaling callback
#[uniffi::callback(interface = "SignalingCallback")]
pub trait SignalingCallback: Send + Sync {
    fn on_signaling_message(&self, from: Vec<u8>, message: SignalingMessage);
}
