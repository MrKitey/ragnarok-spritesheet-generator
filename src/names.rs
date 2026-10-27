//! Class / mount display names from folders.MD.

use std::collections::HashMap;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

/// Strip gender and optional trailing `_N` from a sprite stem.
/// Returns (class_key, gender).
pub fn parse_sprite_stem(stem: &str) -> Option<(String, Gender)> {
    let (before, gender) = if let Some(b) = stem.strip_suffix("_남") {
        (b, Gender::Male)
    } else if let Some(b) = stem.strip_suffix("_여") {
        (b, Gender::Female)
    } else if let Some(idx) = stem.find("_남_") {
        (&stem[..idx], Gender::Male)
    } else if let Some(idx) = stem.find("_여_") {
        (&stem[..idx], Gender::Female)
    } else {
        return None;
    };

    Some((strip_variant_suffix(before), gender))
}

fn strip_variant_suffix(base: &str) -> String {
    // Trailing _N only (palette / job variants).
    if let Some((head, tail)) = base.rsplit_once('_') {
        if !tail.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) {
            return head.to_string();
        }
    }
    base.to_string()
}

pub fn display_name(stem: &str) -> String {
    if let Some(name) = lookup(stem) {
        return name.to_string();
    }
    // Riding / chicken suffixes for 4th jobs.
    if let Some(base) = stem.strip_suffix("_riding") {
        if let Some(name) = lookup(base) {
            return format!("{name} (Riding)");
        }
    }
    if let Some(base) = stem.strip_suffix("_chicken") {
        if let Some(name) = lookup(base) {
            return format!("{name} (Chicken)");
        }
    }
    // Mount prefixes / suffixes from folders.MD patterns.
    if let Some(name) = mount_display(stem) {
        return name;
    }
    stem.to_string()
}

fn mount_display(stem: &str) -> Option<String> {
    const SUFFIXES: &[(&str, &str)] = &[
        ("포링", "Poring"),
        ("알파카", "Alpaca"),
        ("멧돼지", "Savage"),
    ];
    for (suf, label) in SUFFIXES {
        if let Some(base) = stem.strip_suffix(suf) {
            let class = display_name(base);
            return Some(format!("{class} ({label})"));
        }
    }
    const PREFIXES: &[(&str, &str)] = &[
        ("두꺼비", "Frog"),
        ("사자", "Lion"),
        ("여우", "Fox"),
        ("켈베로스", "Hyena"),
        ("타조", "Ostrich"),
        ("해태", "Haetae"),
    ];
    for (pre, label) in PREFIXES {
        if let Some(base) = stem.strip_prefix(pre) {
            if !base.is_empty() {
                let class = display_name(base);
                return Some(format!("{class} ({label})"));
            }
        }
    }
    None
}

static CLASS_MAP: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| CLASS_NAMES.iter().copied().collect());

static CLASS_MAP_LOWER: LazyLock<HashMap<String, &'static str>> = LazyLock::new(|| {
    CLASS_NAMES
        .iter()
        .filter(|(k, _)| k.is_ascii())
        .map(|(k, v)| (k.to_ascii_lowercase(), *v))
        .collect()
});

fn lookup(stem: &str) -> Option<&'static str> {
    if let Some(v) = CLASS_MAP.get(stem) {
        return Some(*v);
    }
    if stem.is_ascii() {
        return CLASS_MAP_LOWER
            .get(&stem.to_ascii_lowercase())
            .copied();
    }
    None
}

/// folders.MD CLASSES NAME + useful mounts.
const CLASS_NAMES: &[(&str, &str)] = &[
    ("abyss_chaser", "Abyss Chaser"),
    ("arch_mage", "Arch Mage"),
    ("biolo", "Biolo"),
    ("cardinal", "Cardinal"),
    ("dragon_knight", "Dragon Knight"),
    ("elemetal_master", "Elemental Master"),
    ("hyper_novice", "Hyper Novice"),
    ("imperial_guard", "Imperial Guard"),
    ("inquisitor", "Inquisitor"),
    ("kagerou", "Kagerou"),
    ("meister", "Meister"),
    ("night_watch", "Night Watch"),
    ("oboro", "Oboro"),
    ("rebellion", "Rebellion"),
    ("shadow_cross", "Shadow Cross"),
    ("shinkiro", "Shinkiro"),
    ("shiranui", "Shiranui"),
    ("sky_emperor2", "Sky Emperor (Fusion)"),
    ("sky_emperor", "Sky Emperor"),
    ("soul_ascetic", "Soul Ascetic"),
    ("troubadour", "Troubadour"),
    ("trouvere", "Trouvere"),
    ("windhawk", "Wind Hawk"),
    ("frog_kagerou", "Kagerou (Frog)"),
    ("frog_oboro", "Oboro (Frog)"),
    ("peco_rebellion", "Rebellion (Bike)"),
    ("meister_madogear1", "Meister (Madogear Old)"),
    ("meister_madogear2", "Meister (Madogear New)"),
    ("wolf_windhawk", "Wind Hawk (Wolf)"),
    ("alitea", "Alitea"),
    ("druid", "Druid"),
    ("karnos", "Karnos"),
    ("가드", "Royal Guard"),
    ("건너", "Gunslinger"),
    ("검사", "Swordsman"),
    ("결혼", "Tuxedo/Wedding Dress"),
    ("검용병", "Novice (Alternative)"),
    ("궁수", "Archer"),
    ("권성", "Star Gladiator"),
    ("권성융합", "Star Gladiator (Fusion)"),
    ("기사_h", "Knight (Alt)"),
    ("기사", "Knight"),
    ("길로틴크로스", "Guillotine Cross"),
    ("닌자", "Ninja"),
    ("도둑", "Thief"),
    ("드래곤나이트", "Rune Knight"),
    ("레인져", "Ranger"),
    ("로그_h", "Rogue (Alt)"),
    ("로그", "Rogue"),
    ("로드나이트", "Lord Knight"),
    ("로드페코", "Lord Knight (Peco)"),
    ("룬나이트", "Rune Knight"),
    ("리벨리온", "Rebellion"),
    ("마도기어", "Madogear"),
    ("마도아머", "Madogear 2"),
    ("마법사", "Magician"),
    ("몽크_h", "Monk (Alt)"),
    ("몽크", "Monk"),
    ("무희", "Dancer"),
    ("무희바지", "Dancer (Pants)"),
    ("미케닉", "Mechanic"),
    ("민스트럴", "Minstrel"),
    ("바드_h", "Bard (Alt)"),
    ("바드", "Bard"),
    ("산타", "Santa Suit"),
    ("상인", "Merchant"),
    ("팔라딘", "Paladin"),
    ("성제", "Star Emperor"),
    ("성제융합", "Star Emperor (Fusion)"),
    ("성직자_h", "Acolyte (Alt)"),
    ("성직자", "Acolyte"),
    ("성투사2", "High Priest"),
    ("성투사", "Priest"),
    ("세이지_h", "Sage (Alt)"),
    ("세이지", "Sage"),
    ("소서러", "Sorcerer"),
    ("소울리퍼", "Soul Reaper"),
    ("소울링커", "Soul Linker"),
    ("쉐도우체이서", "Shadow Chaser"),
    ("슈라", "Sura"),
    ("슈퍼노비스", "Super Novice"),
    ("스나이퍼", "Sniper"),
    ("스토커", "Stalker"),
    ("아크비숍", "Arch Bishop"),
    ("어세신_h", "Assassin (Alt)"),
    ("어세신", "Assassin"),
    ("어쌔신크로스", "Assassin Cross"),
    ("여름2", "Summer Suit 2"),
    ("여름", "Summer Suit"),
    ("연금술사_h", "Alchemist (Alt)"),
    ("연금술사", "Alchemist"),
    ("옥토버패스트", "Oktoberfest Suit"),
    ("운영자2", "Game Master (Madogear)"),
    ("운영자", "Game Master"),
    ("워록", "Warlock"),
    ("위저드_h", "Wizard (Alt)"),
    ("위저드", "Wizard"),
    ("인퀴지터", "Inquisitor"),
    ("제네릭", "Genetic"),
    ("제철공_h", "Blacksmith (Alt)"),
    ("제철공", "Blacksmith"),
    ("창용병", "Spear Mercenary"),
    ("용병", "Sword Mercenary"),
    ("챔피온", "Champion"),
    ("초보자", "Novice"),
    ("크루세이더_h", "Crusader (Alt)"),
    ("크루세이더", "Crusader"),
    ("크리에이터", "Creator"),
    ("클라운", "Clown/Gypsy"),
    ("태권소년", "Taekwon"),
    ("턱시도", "Tuxedo"),
    ("프로페서", "Professor"),
    ("프리스트_h", "Priest (Alt)"),
    ("프리스트", "Priest"),
    ("하이위저드", "High Wizard"),
    ("하이프리", "High Priest"),
    ("한복", "Hanbok Suit"),
    ("헌터_h", "Hunter (Alt)"),
    ("헌터", "Hunter"),
    ("화이트스미스", "Whitesmith"),
    ("레인져늑대", "Ranger (Wolf)"),
    ("신페코크루세이더_h", "Crusader (Peco Alt)"),
    ("신페코크루세이더", "Crusader (Peco)"),
    ("구페코크루세이더", "Crusader (Peco Old)"),
    ("그리폰가드", "Royal Guard (Gryphon)"),
    ("룬나이트쁘띠2", "Rune Knight (Violet Dragon)"),
    ("룬나이트쁘띠3", "Rune Knight (White Dragon)"),
    ("룬나이트쁘띠4", "Rune Knight (Blue Dragon)"),
    ("룬나이트쁘띠5", "Rune Knight (Red Dragon)"),
    ("룬나이트쁘띠", "Rune Knight (Dragon)"),
    ("페코페코_기사_h", "Knight (Peco Alt)"),
    ("페코페코_기사", "Knight (Peco)"),
    ("페코건너", "Gunslinger (Bike)"),
    ("페코rebellion", "Rebellion on Peco"),
    ("노비스", "Novice"),
    ("하이프리스트", "High Priest"),
    ("어쎄신크로스", "Assassin Cross"),
    ("로얄가드", "Royal Guard"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_gender() {
        let (stem, g) = parse_sprite_stem("초보자_남").unwrap();
        assert_eq!(stem, "초보자");
        assert_eq!(g, Gender::Male);
        let (stem, g) = parse_sprite_stem("초보자_여").unwrap();
        assert_eq!(stem, "초보자");
        assert_eq!(g, Gender::Female);
    }

    #[test]
    fn parse_variant_suffix() {
        let (stem, g) = parse_sprite_stem("슈라_남_1").unwrap();
        assert_eq!(stem, "슈라");
        assert_eq!(g, Gender::Male);
    }

    #[test]
    fn display_known_and_riding() {
        assert_eq!(display_name("초보자"), "Novice");
        assert_eq!(display_name("hyper_novice_riding"), "Hyper Novice (Riding)");
        assert_eq!(display_name("unknown_xyz"), "unknown_xyz");
    }
}
