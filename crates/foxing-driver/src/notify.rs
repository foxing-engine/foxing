//! The operator's channel. Notices (an arrival worth knowing about) are posted to a
//! Discord webhook from a thread of their own; the world never waits on the network.
//! Without a webhook they go to stderr, so a local run still shows them.

use std::sync::mpsc;

/// Start the notifier; returns the sender the tick loop feeds.
pub fn start(webhook: Option<String>, world: &str) -> mpsc::Sender<String> {
    let (tx, rx) = mpsc::channel::<String>();
    let world = world.to_string();
    std::thread::Builder::new()
        .name("foxing-notify".into())
        .spawn(move || {
            let send = |line: &str| match &webhook {
                Some(url) => post(url, &world, line),
                None => eprintln!("notice: {line}"),
            };
            let mut crowd = Crowd::default();
            loop {
                match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(line) => {
                        if let Some(out) = crowd.take(line, std::time::Instant::now()) {
                            send(&out);
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
                if let Some(out) = crowd.due(std::time::Instant::now()) {
                    send(&out);
                }
            }
        })
        .expect("notifier thread");
    tx
}

/// Arrivals said one by one are welcome until they come as a crowd. The first
/// [`SINGLY`] in any [`WINDOW`] go out as they are; the rest of that window are held
/// and said in one notice when it closes, so a load test or a busy evening is a line
/// a minute and not a line a name.
pub const SINGLY: usize = 3;
pub const WINDOW: std::time::Duration = std::time::Duration::from_secs(60);
/// How many names a crowd's notice spells out before it only counts.
pub const NAMED: usize = 12;
const ARRIVED: &str = " arrived for the first time";

#[derive(Default)]
struct Crowd {
    /// When the window opened, and how many arrivals it has passed singly.
    opened: Option<std::time::Instant>,
    passed: usize,
    held: Vec<String>,
}

impl Crowd {
    /// A line in: what to send now, if anything. Only arrivals are ever held.
    fn take(&mut self, line: String, now: std::time::Instant) -> Option<String> {
        if !line.contains(ARRIVED) {
            return Some(line);
        }
        if self.opened.is_none() {
            self.opened = Some(now);
            self.passed = 0;
        }
        if self.passed < SINGLY {
            self.passed += 1;
            return Some(line);
        }
        let name = line
            .split(ARRIVED)
            .next()
            .unwrap_or(&line)
            .trim()
            .to_string();
        self.held.push(name);
        None
    }

    /// The window's close: the held names as one notice. A window that held any opens
    /// the next one already full, so a steady crowd stays one line a window.
    fn due(&mut self, now: std::time::Instant) -> Option<String> {
        let opened = self.opened?;
        if now.duration_since(opened) < WINDOW {
            return None;
        }
        if self.held.is_empty() {
            self.opened = None;
            return None;
        }
        let held = std::mem::take(&mut self.held);
        self.opened = Some(now);
        self.passed = SINGLY;
        let named = held
            .iter()
            .take(NAMED)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        let more = match held.len().saturating_sub(NAMED) {
            0 => String::new(),
            n => format!(", and {n} more"),
        };
        Some(if held.len() == 1 {
            format!("{named} arrived for the first time.")
        } else {
            format!(
                "{} more arrived for the first time in the last minute: {named}{more}.",
                held.len()
            )
        })
    }
}

fn post(url: &str, world: &str, line: &str) {
    let body = serde_json::json!({
        "username": format!("{world} (the world)"),
        "content": line,
        "allowed_mentions": {"parse": []},
    });
    match ureq::post(url)
        .set("content-type", "application/json")
        .timeout(std::time::Duration::from_secs(10))
        .send_string(&body.to_string())
    {
        Ok(_) => {}
        Err(ureq::Error::Status(code, _)) => eprintln!("notice: webhook answered {code}: {line}"),
        Err(e) => eprintln!("notice: webhook failed: {e}: {line}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crowd_of_arrivals_is_one_notice_a_minute_and_a_few_are_said_singly() {
        let t0 = std::time::Instant::now();
        let mut c = Crowd::default();
        let line = |n: usize| format!("**Gn{n}** arrived for the first time, via mcp.");
        for n in 0..SINGLY {
            assert_eq!(c.take(line(n), t0), Some(line(n)));
        }
        for n in SINGLY..SINGLY + 20 {
            assert_eq!(c.take(line(n), t0), None, "held past the threshold");
        }
        assert_eq!(
            c.take("the log refused a write".into(), t0).as_deref(),
            Some("the log refused a write")
        );
        assert_eq!(c.due(t0 + WINDOW / 2), None);
        let said = c.due(t0 + WINDOW).expect("the window closes");
        assert!(
            said.starts_with("20 more arrived for the first time in the last minute: **Gn3**, ")
        );
        assert!(said.ends_with(", and 8 more."), "{said}");
        // The crowd goes on: nothing singly in the window after a full one.
        assert_eq!(c.take(line(99), t0 + WINDOW), None);
        let said = c.due(t0 + WINDOW * 2).expect("the next window");
        assert_eq!(said, "**Gn99** arrived for the first time.");
        // A quiet window ends it, and arrivals are said singly again.
        assert_eq!(c.due(t0 + WINDOW * 3), None);
        assert_eq!(c.take(line(100), t0 + WINDOW * 3), Some(line(100)));
    }
}
