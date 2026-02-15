use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use fxhash::FxHashMap;
use keympostor::action::{KeyAction, KeyActionSequence};
use keympostor::event::KeyEvent;
use keympostor::key::{Key};
use keympostor::modifiers::KeyModifiers;
use keympostor::modifiers::KeyModifiers::{All, Any};
use keympostor::rules::KeyTransformRule;
use keympostor::transition::KeyTransition;
use keympostor::transition::KeyTransition::{Down, Up};
use keympostor::trigger::KeyTrigger;

type Group = FxHashMap<KeyModifiers, KeyTransformRule>;

trait KeyTransformMap {
    fn get(&self, event: &KeyEvent) -> Option<&KeyTransformRule>;
    fn put(&mut self, rule: KeyTransformRule);
}

#[derive(Debug, Default)]
pub struct KeyTransformHashMap {
    map: FxHashMap<KeyAction, Group>,
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
            matrix: vec![vec![vec![vec![None; 256]; 136]; 2]; 2].into_boxed_slice(),
        }
    }
}

impl KeyTransformMatrix {
    fn get_group_mut(&mut self, action: &KeyAction) -> &mut Option<Group> {
        &mut self.matrix[action.transition as usize][action.key.is_ext_sc() as usize]
            [action.key.sc() as usize][action.key.vk() as usize]
    }

    fn get_group(&self, action: &KeyAction) -> &Option<Group> {
        &self.matrix[action.transition as usize][action.key.is_ext_sc() as usize]
            [action.key.sc() as usize][action.key.vk() as usize]
    }

    fn put_group(&mut self, action: &KeyAction, group: Group) {
        self.matrix[action.transition as usize][action.key.is_ext_sc() as usize]
            [action.key.sc() as usize][action.key.vk() as usize] = Some(group);
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
        key: Key::from_code(vk, sc, ext).unwrap(),
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

fn create_event(vk: u8, sc: u8, ext: bool, trans: KeyTransition) -> KeyEvent {
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
    for vk in 0..255 {
        for sc in 0..135 {
            f(vk, sc, false, Down);
            f(vk, sc, false, Up);
            f(vk, sc, true, Down);
            f(vk, sc, true, Up);
        }
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
