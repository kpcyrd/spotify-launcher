use log::info;
use regex::Regex;
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

fn parse_uri(uri: &String) -> Result<String> {
    // Remove useless tracking IDs
    let sanitized_uri = Regex::new(r"\?.*").unwrap().replace_all(uri, "");

    // URIs may be of one of the following forms:
    // 1. spotify:type:id
    // 2. spotify://type/id
    // 3. https://open.spotify.com/type/id
    //
    // Here type must be something like "track" or "album"
    // and id is a unique id consisting of alphanumerical symbols.
    // Also see: https://www.iana.org/assignments/uri-schemes/prov/spotify
    //
    // The dbus interface only understands the first syntax,
    // so we have to transform the other schemes into the first scheme.
    //
    // We employ a very strict Regex here, so that we never mistakenly pass a wrong URI over dbus by accident.
    // If the Regex doesn't match, it is better to bail out and open Spotify normally.
    let r = Regex::new(r"^(?:spotify|https):(?://(?:open\.spotify\.com/)?)?(?P<type>(?:artist|album|track|search))(?::|/)(?P<id>[[:alnum:]]+)$").unwrap();
    match r.captures(&sanitized_uri) {
        Some(c) => Ok(format!(
            "spotify:{}:{}",
            c.name("type").unwrap().as_str(),
            c.name("id").unwrap().as_str()
        )),
        None => Err(zbus::Error::Failure(
            "Unsupported URI scheme for dbus".to_string(),
        )),
    }
}

pub fn play_remote(uri: &String) -> Result<Arc<zbus::Message>> {
    info!("Playing uri {uri} in already running instance over dbus");
    let c = zbus::blocking::Connection::session()?;
    let parsed = parse_uri(uri)?;
    // This dbus interface supports URIs in the spotify:<type>:<UUID> format,
    // which conveniently is also the format used by the "--uri" arg
    c.call_method(
        Some("org.mpris.MediaPlayer2.spotify"),
        "/org/mpris/MediaPlayer2",
        Some("org.mpris.MediaPlayer2.Player"),
        "OpenUri",
        &(parsed),
    )
}
