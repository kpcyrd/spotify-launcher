use log::info;
use std::sync::Arc;
use zbus::Result;

pub fn is_spotify_running() -> bool {
    match zbus::blocking::Connection::session() {
        Ok(c) => c
            .call_method(
                Some("org.mpris.MediaPlayer2.spotify"),
                "/org/mpris/MediaPlayer2",
                Some("org.freedesktop.DBus.Peer"),
                "Ping",
                &(),
            )
            .is_ok(),
        _ => false,
    }
}

pub fn play_remote(uri: &String) -> Result<Arc<zbus::Message>> {
    info!("Playing uri {uri} in already running instance over dbus");
    let c = zbus::blocking::Connection::session()?;
    // This dbus interface supports URIs in the spotify:<type>:<UUID> format,
    // which conveniently is also the format used by the "--uri" arg
    c.call_method(
        Some("org.mpris.MediaPlayer2.spotify"),
        "/org/mpris/MediaPlayer2",
        Some("org.mpris.MediaPlayer2.Player"),
        "OpenUri",
        &(uri),
    )
}
