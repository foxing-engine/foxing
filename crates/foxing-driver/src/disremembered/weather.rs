//! Weather and hours in `look` (R78): a second sentence that varies with the
//! Clocktower's rates and the hour, so two looks at one room an hour apart read
//! differently, and the rates are felt where they act. The fog is thick or thin by
//! `mend`; the Orchard's hours ripe or green by `price`; the Shallows' water by
//! `respawn`; the lanterns by `half_life`; the Long Afternoon's sun a hand's width
//! off by the hour; Morphos's visit changes the sky everywhere outdoors.

use crate::world::World;

/// The line, if the room has weather, for the world as it stands.
pub fn line(world: &World, key: &str) -> Option<String> {
    let rate = |n: &str| super::economy::rate(world, n);
    let hour = world.now().0.div_euclid(1000).rem_euclid(86_400) / 3600;
    let night = !(6..18).contains(&hour);
    let morphos = super::morphos::present(world).is_some();
    let outdoors = matches!(
        key,
        "threshold" | "fogbank" | "shallows" | "orchard" | "mudroom" | "midden" | "lantern-row"
    ) || key.starts_with("trail-");
    if morphos && outdoors {
        return Some(
            "The sky is the wrong color, as it is when the traveler is in the world.".to_string(),
        );
    }
    let s = match key {
        "fogbank" => {
            let mend = rate("mend");
            if mend > 1.05 {
                "The fog is thin tonight, and mends what it touches."
            } else if mend < 0.95 {
                "The fog lies thick and still, and keeps every wound in it."
            } else if night {
                "The fog is as it always is, moon-colored."
            } else {
                "The fog is as it always is, gray by day."
            }
        }
        "orchard" => {
            let price = rate("price");
            if price > 1.05 {
                "The hours on the boughs are green and hard this hour, and dear."
            } else if price < 0.95 {
                "The hours hang soft and cheap this hour, and some have dropped."
            } else {
                "The hours on the boughs are as unripe as ever."
            }
        }
        "shallows" => {
            let respawn = rate("respawn");
            if respawn > 1.05 {
                "The water is up, and things come out of it quickly."
            } else if respawn < 0.95 {
                "The water is low and slow, and what is in it stays down."
            } else {
                "The water is at its usual mark."
            }
        }
        "lantern-row" => {
            let hl = rate("half_life");
            if hl > 1.05 {
                "The lanterns burn long tonight; what is read here is kept longer."
            } else if hl < 0.95 {
                "The lanterns gutter; what is read here fades sooner."
            } else {
                "The lanterns burn as they always burn."
            }
        }
        "threshold" | "mudroom" => {
            let pay = rate("pay");
            if pay > 1.05 {
                "The air carries sound tonight; a story told here goes further."
            } else if pay < 0.95 {
                "The air is dead and close; stories carry less this hour."
            } else if night {
                "The moon is up, as it always is."
            } else {
                "The moon is up, as it always is, pale against the day."
            }
        }
        "trail-afternoon" => {
            // The sun a hand's width off by the hour: it is always four, but the
            // hand is bigger some hours.
            let off = ((hour - 16).rem_euclid(24)).min(24 - (hour - 16).rem_euclid(24));
            return Some(match off {
                0 => "The sun is exactly where it stuck, a hand's width above the wall.".to_string(),
                1..=3 => "The sun is a hand's width above the wall, and the shadows are nearly honest.".to_string(),
                _ => format!("The sun is a hand's width above the wall, which it should not be at this hour ({} off); the shadows point wrong.", off),
            });
        }
        _ => return None,
    };
    Some(s.to_string())
}
