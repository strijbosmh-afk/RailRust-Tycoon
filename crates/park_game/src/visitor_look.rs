//! Original, deterministic visitor looks. They are derived from stable IDs;
//! no appearance data or random draws are added to simulation saves.

use park_sim::GuestId;

pub(crate) const LOOK_COUNT: usize = 48;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct VisitorLook {
    pub shirt: [u8; 3],
    pub skin: [u8; 3],
    pub hair: [u8; 3],
    pub bag: [u8; 3],
    pub hair_style: usize,
    pub cap: bool,
}

const SHIRTS: [[u8; 3]; 8] = [
    [213, 77, 82],
    [66, 126, 188],
    [150, 88, 174],
    [242, 184, 65],
    [53, 154, 140],
    [222, 119, 161],
    [242, 113, 49],
    [91, 169, 77],
];
const SKINS: [[u8; 3]; 4] = [
    [174, 119, 84],
    [239, 192, 143],
    [118, 77, 54],
    [216, 157, 115],
];
const HAIR: [[u8; 3]; 4] = [[67, 53, 42], [48, 44, 39], [120, 76, 47], [190, 142, 73]];
const BAGS: [[u8; 3]; 3] = [[47, 79, 70], [142, 71, 53], [69, 75, 122]];

pub(crate) fn look(id: GuestId) -> VisitorLook {
    let index = (id.0 % LOOK_COUNT as u64) as usize;
    VisitorLook {
        shirt: SHIRTS[index % SHIRTS.len()],
        skin: SKINS[(index / 3) % SKINS.len()],
        hair: HAIR[(index / 2) % HAIR.len()],
        bag: BAGS[index % BAGS.len()],
        hair_style: index % 3,
        cap: index.is_multiple_of(7),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_are_stable_and_repeat_after_forty_eight_ids() {
        for id in 0..192 {
            assert_eq!(look(GuestId(id)), look(GuestId(id + 48)));
        }
        assert_ne!(look(GuestId(1)), look(GuestId(2)));
        assert!(look(GuestId(0)).cap);
        assert!(!look(GuestId(1)).cap);
    }
}
