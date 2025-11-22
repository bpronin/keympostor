use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, criterion_group, criterion_main};
use fxhash::FxHashMap;
use keympostor::keyboard::action::{KeyAction, KeyActionSequence};
use keympostor::keyboard::event::KeyEvent;
use keympostor::keyboard::key::KEYS;
use keympostor::keyboard::key::Key;
use keympostor::keyboard::modifiers::KeyModifiers;
use keympostor::keyboard::modifiers::KeyModifiers::{All, Any};
use keympostor::keyboard::rules::KeyTransformRule;
use keympostor::keyboard::sc::MAX_SCAN_CODE;
use keympostor::keyboard::transition::KeyTransition;
use keympostor::keyboard::transition::KeyTransition::{Down, Up};
use keympostor::keyboard::trigger::KeyTrigger;
use keympostor::keyboard::vk::MAX_VK_CODE;

type Group = FxHashMap<KeyModifiers, KeyTransformRule>;

//type TheMap = HashMap<KeyAction, Group>;
type TheMap = FxHashMap<KeyAction, Group>;
// type TheMap = HashMap<KeyAction, Group, ahash::RandomState>;

trait KeyTransformMap {
    fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule>;
    fn put(&mut self, rule: KeyTransformRule);
}

#[derive(Debug, Default)]
pub struct KeyTransformHashMap {
    map: TheMap,
}

impl KeyTransformMap for KeyTransformHashMap {
    fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule> {
        let map = self.map.get(&event.action)?;
        map.get(&All(event.modifiers)).or_else(|| map.get(&Any))
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = rule.trigger;
        self.map
            .entry(trigger.action)
            .or_default()
            .insert(trigger.modifiers, rule);
    }
}

#[derive(Debug)]
pub struct KeyTransformMatrix {
    matrix: Box<[Vec<Vec<Vec<Option<Group>>>>]>,
}

impl Default for KeyTransformMatrix {
    fn default() -> Self {
        Self {
            matrix: vec![vec![vec![vec![None; MAX_VK_CODE]; MAX_SCAN_CODE]; 2]; 2]
                .into_boxed_slice(),
        }
    }
}

impl KeyTransformMatrix {
    fn get_group_mut(&mut self, action: &KeyAction) -> &mut Option<Group> {
        &mut self.matrix[action.transition as usize][action.key.scan_code.1 as usize]
            [action.key.scan_code.0 as usize][action.key.vk_code as usize]
    }

    fn get_group(&self, action: &KeyAction) -> &Option<Group> {
        &self.matrix[action.transition as usize][action.key.scan_code.1 as usize]
            [action.key.scan_code.0 as usize][action.key.vk_code as usize]
    }

    fn put_group(&mut self, action: &KeyAction, group: Group) {
        self.matrix[action.transition as usize][action.key.scan_code.1 as usize]
            [action.key.scan_code.0 as usize][action.key.vk_code as usize] = Some(group);
    }
}

impl KeyTransformMap for KeyTransformMatrix {
    fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule> {
        if let Some(map) = self.get_group(&event.action) {
            map.get(&All(event.modifiers)).or_else(|| map.get(&Any))
        } else {
            None
        }
    }

    fn put(&mut self, rule: KeyTransformRule) {
        let trigger = rule.trigger;
        let action = trigger.action;

        if let Some(map) = self.get_group_mut(&action) {
            map.insert(trigger.modifiers, rule);
        } else {
            let mut map = Group::default();
            map.insert(trigger.modifiers, rule);
            self.put_group(&action, map);
        }
    }
}

fn create_action(vk: u8, sc: u8, ext: bool, trans: KeyTransition) -> KeyAction {
    KeyAction {
        key: Key {
            vk_code: vk,
            scan_code: (sc, ext),
        },
        transition: trans,
    }
}

fn crete_rule(vk: u8, sc: u8, ext: bool, trans: KeyTransition) -> KeyTransformRule {
    KeyTransformRule {
        trigger: KeyTrigger {
            action: create_action(vk, sc, ext, trans),
            modifiers: Any,
        },
        actions: KeyActionSequence::new(vec![]),
    }
}

fn create_event(vk: u8, sc: u8, ext: bool, trans: KeyTransition) -> KeyEvent<'static> {
    KeyEvent {
        action: create_action(vk, sc, ext, trans),
        modifiers: Default::default(),
        rule: None,
        time: 0,
        is_injected: false,
        is_private: false,
    }
}

pub fn for_all<F>(mut f: F)
where
    F: FnMut(u8, u8, bool, KeyTransition) -> (),
{
    for (_name, key) in KEYS {
        f(key.vk_code, key.scan_code.0, key.scan_code.1, Down);
        f(key.vk_code, key.scan_code.0, key.scan_code.1, Up);
    }
}

fn bench_map<M: KeyTransformMap>(group: &mut BenchmarkGroup<WallTime>, id: &str, mut map: M) {
    for_all(|vk, sc, ext, trans| {
        map.put(crete_rule(vk, sc, ext, trans));
    });
    group.bench_function(id, move |b| {
        b.iter(|| {
            for_all(|vk, sc, ext, trans| {
                let _ = map.get(&create_event(vk, sc, ext, trans));
            })
        })
    });
}

pub(crate) fn bench_transform_container(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_container_benchmark");

    bench_map(&mut group, "Map", KeyTransformHashMap::default());
    bench_map(&mut group, "Matrix", KeyTransformMatrix::default());

    group.finish();
}

criterion_group!(benches, bench_transform_container);
// criterion_group!(benches, get_keyboard_state);
criterion_main!(benches);
