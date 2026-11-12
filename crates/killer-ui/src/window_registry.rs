use crate::window::{Window, WindowId};
use crate::window_children::WindowChildren;
use crate::window_dispatch::WindowDispatch;
use crate::window_instance::WindowInstance;
use crate::window_manager::{ManagerError, WindowManager};
use crate::window_owner::{
    VIRTUAL_ROOT_CHILDREN, WindowOwner, WindowParent, WindowRegistration, original_owner, original_registration, original_root_registration,
};
use crate::window_records::RecordKey;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;

#[derive(Debug, PartialEq, Eq)]
pub enum RegistryError {
    UnknownWindow(WindowId),
    MetadataMismatch(WindowId),
    DuplicateInstance(WindowId),
    DuplicateOwnerSlot(WindowId),
    UnreachableWindow(WindowId),
    MissingParent,
    UnknownRoot,
    UnknownVtable(u32),
    RegistrationLimit,
    AlreadyRegistered(WindowInstance),
    Manager(ManagerError<Infallible>),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "registro da interface: {self:?}")
    }
}

impl std::error::Error for RegistryError {}

#[derive(Debug)]
pub struct RegisteredInstance {
    pub record: RecordKey,
    pub registration_flags: u32,
    pub vtable: u32,
    pub dispatch: WindowDispatch,
    pub children: WindowChildren<RecordKey>,
}

#[derive(Debug)]
pub struct WindowRegistry {
    pub manager: WindowManager<WindowInstance>,
    pub primary: WindowInstance,
    pub secondary: WindowInstance,
    nodes: HashMap<WindowInstance, RegisteredInstance>,
}

#[derive(Clone, Copy)]
struct Installation<'a> {
    window: &'a Window,
    owner: WindowOwner,
    registration: WindowRegistration,
}

impl WindowRegistry {
    pub fn from_windows(windows: &[Window]) -> Result<Self, RegistryError> {
        let mut instances = HashSet::new();
        let mut owner_slots = HashSet::new();
        let mut installations = Vec::new();
        for window in windows.iter().filter(|window| !VIRTUAL_ROOT_CHILDREN.contains(&window.id)) {
            let expected_owner = original_owner(window.id).ok_or(RegistryError::UnknownWindow(window.id))?;
            let expected_registration =
                original_registration(window.id).ok_or(RegistryError::UnknownWindow(window.id))?;
            if window.owner != Some(expected_owner) || window.registration != Some(expected_registration) {
                return Err(RegistryError::MetadataMismatch(window.id));
            }
            if !instances.insert(window.instance()) {
                return Err(RegistryError::DuplicateInstance(window.id));
            }
            if !owner_slots.insert((expected_owner.parent, expected_owner.slot)) {
                return Err(RegistryError::DuplicateOwnerSlot(window.id));
            }
            installations.push(Installation { window, owner: expected_owner, registration: expected_registration });
        }
        installations.sort_by_key(|installation| installation.registration.call_address);
        let mut registry = Self {
            manager: WindowManager::default(),
            primary: WindowInstance::default(),
            secondary: WindowInstance::default(),
            nodes: HashMap::new(),
        };
        for (parent, instance) in [
            (WindowParent::PrimaryRoot, registry.primary.clone()),
            (WindowParent::SecondaryRoot, registry.secondary.clone()),
        ] {
            let registration = original_root_registration(parent).ok_or(RegistryError::UnknownRoot)?;
            registry.register_instance(instance.clone(), None, registration.flags_argument, registration.vtable)?;
            registry.install_children(&installations, parent, &instance)?;
        }
        for installation in installations.iter().filter(|installation| installation.owner.parent == WindowParent::ConsoleManager) {
            let instance = installation.window.instance();
            registry.register_instance(
                instance.clone(),
                None,
                installation.registration.flags_argument,
                installation.registration.vtable,
            )?;
            registry.install_children(&installations, WindowParent::Window(installation.window.id), &instance)?;
        }
        for window in windows.iter().filter(|window| !VIRTUAL_ROOT_CHILDREN.contains(&window.id)) {
            if !registry.nodes.contains_key(&window.instance()) {
                return Err(RegistryError::UnreachableWindow(window.id));
            }
        }
        Ok(registry)
    }

    fn install_children(
        &mut self,
        installations: &[Installation<'_>],
        owner: WindowParent,
        parent: &WindowInstance,
    ) -> Result<(), RegistryError> {
        for installation in installations.iter().filter(|installation| installation.owner.parent == owner) {
            let instance = installation.window.instance();
            if self.nodes.contains_key(&instance) {
                return Err(RegistryError::DuplicateInstance(installation.window.id));
            }
            let record = self.nodes.get(parent).ok_or(RegistryError::MissingParent)?.record;
            self.register_instance(
                instance.clone(),
                Some(record),
                installation.registration.flags_argument,
                installation.registration.vtable,
            )?;
            let child =
                self.nodes.get(&instance).ok_or(RegistryError::UnreachableWindow(installation.window.id))?.record;
            let parent_node = self.nodes.get_mut(parent).ok_or(RegistryError::MissingParent)?;
            parent_node.children.insert(child, || installation.registration.flags_argument);
            self.install_children(installations, WindowParent::Window(installation.window.id), &instance)?;
        }
        Ok(())
    }

    fn register_instance(
        &mut self,
        instance: WindowInstance,
        parent: Option<RecordKey>,
        flags: u32,
        vtable: u32,
    ) -> Result<(), RegistryError> {
        let dispatch = WindowDispatch::from_vtable(vtable)
            .or_else(|| {
                matches!(vtable, 0x1019f8b8 | 0x101a2b40 | 0x101a2ec0 | 0x101ceb60 | 0x101a76e8).then_some(
                    WindowDispatch {
                        focus: 0x10035af0,
                        focus_mode: 0x100023c0,
                        movement: 0x100351c0,
                        resize: 0x10036de0,
                        show: 0x10034590,
                        hide: 0x10035a90,
                        move_dispatch: 0x10054f40,
                    },
                )
            })
            .ok_or(RegistryError::UnknownVtable(vtable))?;
        let stored_flags = Cell::new(0);
        let record = self
            .manager
            .register(
                instance.clone(),
                flags,
                parent,
                |_, _, value| {
                    stored_flags.set(value);
                    Ok::<_, Infallible>(())
                },
                |_, _| Ok(stored_flags.get()),
            )
            .map_err(RegistryError::Manager)?
            .ok_or(RegistryError::RegistrationLimit)?;
        self.nodes.insert(
            instance,
            RegisteredInstance {
                record,
                registration_flags: stored_flags.get(),
                vtable,
                dispatch,
                children: WindowChildren::default(),
            },
        );
        Ok(())
    }

    pub fn register_child(
        &mut self,
        parent: &WindowInstance,
        instance: WindowInstance,
        flags: u32,
        vtable: u32,
    ) -> Result<RecordKey, RegistryError> {
        if self.nodes.contains_key(&instance) {
            return Err(RegistryError::AlreadyRegistered(instance));
        }
        let parent_record = self.nodes.get(parent).ok_or(RegistryError::MissingParent)?.record;
        self.register_instance(instance.clone(), Some(parent_record), flags, vtable)?;
        let child_record = self.nodes.get(&instance).ok_or(RegistryError::MissingParent)?.record;
        self.nodes.get_mut(parent).ok_or(RegistryError::MissingParent)?.children.insert(child_record, || flags);
        Ok(child_record)
    }

    pub fn node_mut(&mut self, instance: &WindowInstance) -> Option<&mut RegisteredInstance> {
        self.nodes.get_mut(instance)
    }

    pub fn node(&self, instance: &WindowInstance) -> Option<&RegisteredInstance> {
        self.nodes.get(instance)
    }

    pub fn parent_instance(&self, instance: &WindowInstance) -> Option<&WindowInstance> {
        let record = self.nodes.get(instance)?.record;
        let parent = self.manager.records.get(record)?.parent?;
        self.manager.records.get(parent)?.window.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::window::Rect;

    fn window(id: WindowId) -> Window {
        let mut window = Window::new(id, Rect::new(13, 27, 100, 50), Rect::new(0, 0, 100, 10));
        window.owner = original_owner(id);
        window.registration = original_registration(id);
        window.visible = false;
        window
    }

    #[test]
    fn installation_preserves_desktop_and_uses_native_parents_not_paint_order() {
        let windows = vec![
            window(WindowId::OPTION),
            window(WindowId::SYSTEM_MSG),
            window(WindowId::CHAT),
            window(WindowId::STATUS),
        ];
        let before = windows.clone();
        let registry = WindowRegistry::from_windows(&windows).unwrap();
        assert_eq!(windows, before);
        assert_eq!(registry.manager.count, 6);
        assert_eq!(registry.manager.roots.len(), 2);
        assert_eq!(registry.parent_instance(&windows[0].instance()), Some(&registry.secondary));
        assert_eq!(registry.parent_instance(&windows[1].instance()), Some(&windows[2].instance()));
        assert_eq!(registry.parent_instance(&windows[2].instance()), Some(&registry.primary));
        let registered: Vec<_> = registry
            .manager
            .all
            .iter()
            .map(|key| registry.manager.records.get(*key).unwrap().window.clone().unwrap())
            .collect();
        assert_eq!(
            registered,
            [
                registry.primary.clone(),
                windows[3].instance(),
                windows[2].instance(),
                windows[1].instance(),
                registry.secondary.clone(),
                windows[0].instance()
            ]
        );
        let node = registry.node(&registry.primary).unwrap();
        let first = node.children.first().unwrap();
        let second = node.children.next(first).unwrap();
        assert_eq!(node.children.record(first), Some(registry.node(&windows[3].instance()).unwrap().record));
        assert_eq!(node.children.record(second), Some(registry.node(&windows[2].instance()).unwrap().record));
        assert_eq!(node.children.next(second), None);
    }

    #[test]
    fn console_manager_windows_are_registered_directly_as_parentless_roots_in_call_order() {
        let ordered: Vec<_> = crate::window_owner::MANAGER_DIRECT_WINDOWS.iter().map(|id| window(*id)).collect();
        let reversed: Vec<_> = ordered.iter().rev().cloned().collect();
        let registry = WindowRegistry::from_windows(&reversed).unwrap();
        assert_eq!(registry.manager.count, 5);
        assert_eq!(registry.manager.roots.len(), 5);
        let tail: Vec<_> = registry.manager.all[2..]
            .iter()
            .map(|key| registry.manager.records.get(*key).unwrap().window.clone().unwrap())
            .collect();
        let expected: Vec<_> = ordered.iter().map(Window::instance).collect();
        assert_eq!(tail, expected);
        for window in &ordered {
            assert_eq!(registry.parent_instance(&window.instance()), None);
        }
        let node = registry.node(&ordered[0].instance()).unwrap();
        assert_eq!(node.registration_flags, 2);
        assert_eq!(node.vtable, 0x101b31d8);
    }

    #[test]
    fn installation_rejects_missing_metadata_duplicates_and_orphans() {
        let first = window(WindowId::STATUS);
        assert!(matches!(
            WindowRegistry::from_windows(&[first.clone(), first]),
            Err(RegistryError::DuplicateInstance(WindowId::STATUS))
        ));
        assert!(matches!(
            WindowRegistry::from_windows(&[window(WindowId::STATUS), window(WindowId::STATUS)]),
            Err(RegistryError::DuplicateOwnerSlot(WindowId::STATUS))
        ));
        assert!(matches!(
            WindowRegistry::from_windows(&[window(WindowId::SYSTEM_MSG)]),
            Err(RegistryError::UnreachableWindow(WindowId::SYSTEM_MSG))
        ));
        let mut bad = window(WindowId::CHAT);
        bad.registration.as_mut().unwrap().flags_argument = 0;
        assert!(matches!(WindowRegistry::from_windows(&[bad]), Err(RegistryError::MetadataMismatch(WindowId::CHAT))));
        assert!(matches!(
            WindowRegistry::from_windows(&[window(WindowId::LOADING)]),
            Err(RegistryError::UnknownWindow(WindowId::LOADING))
        ));
    }
    #[test]
    fn dynamic_child_registration_keeps_instance_identity_flags_and_insertion_order() {
        let status = window(WindowId::STATUS);
        let status_instance = status.instance();
        let mut registry = WindowRegistry::from_windows(&[status]).unwrap();
        let first = WindowInstance::default();
        let second = WindowInstance::default();
        let first_record = registry.register_child(&status_instance, first.clone(), 2, 0x101a9bf0).unwrap();
        let second_record = registry.register_child(&status_instance, second.clone(), 0x800002, 0x101a9bf0).unwrap();
        assert_eq!(registry.parent_instance(&first), Some(&status_instance));
        assert_eq!(registry.parent_instance(&second), Some(&status_instance));
        let node = registry.node(&status_instance).unwrap();
        let head = node.children.first().unwrap();
        assert_eq!(node.children.record(head), Some(second_record));
        assert_eq!(node.children.record(node.children.next(head).unwrap()), Some(first_record));
        assert_eq!(registry.node(&second).unwrap().registration_flags, 0x800002);
        let count = registry.manager.count;
        assert_eq!(
            registry.register_child(&status_instance, second, 2, 0x101a9bf0),
            Err(RegistryError::AlreadyRegistered(
                registry.manager.records.get(second_record).unwrap().window.clone().unwrap()
            ))
        );
        assert_eq!(registry.manager.count, count);
        assert_eq!(
            registry.register_child(&WindowInstance::default(), WindowInstance::default(), 2, 0x101a9bf0),
            Err(RegistryError::MissingParent)
        );
        assert_eq!(
            registry.register_child(&status_instance, WindowInstance::default(), 2, 0),
            Err(RegistryError::UnknownVtable(0))
        );
        assert_eq!(registry.manager.count, count);
    }
}
