use killer_ui::initial_position::{InitialPositionBackend, UndefinedInitialCoordinate};
use killer_ui::positioning_rect::PositioningRect;
use killer_ui::window::{Desktop, Rect};
use killer_ui::window_children::ChildNodeId;
use killer_ui::window_instance::WindowInstance;
use killer_ui::window_move::{WindowMoveBackend, move_window};
use killer_ui::window_position::{RelativePositionBackend, set_relative_position};
use killer_ui::window_registry::WindowRegistry;
use killer_ui::window_restore::float_size_to_int;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, PartialEq)]
pub struct GeometryNode {
    pub rect: PositioningRect,
    pub window_flags: Option<u32>,
    pub anchor: u32,
    pub anchor_offset: (i32, i32),
    pub relations: [Vec<Option<WindowInstance>>; 4],
    pub stuck_flags: Option<u32>,
    pub ratios: Option<[f32; 2]>,
    pub resize_class_chain: Option<Vec<u32>>,
    pub frame: Option<WindowInstance>,
    pub projected_to_desktop: bool,
    pub initial_position: Option<killer_ui::initial_position::InitialPosition>,
    projected: Rect,
}

impl GeometryNode {
    pub fn new(rect: PositioningRect) -> Self {
        Self {
            projected: project(rect),
            rect,
            window_flags: None,
            anchor: 0,
            anchor_offset: (0, 0),
            relations: std::array::from_fn(|_| Vec::new()),
            stuck_flags: None,
            ratios: None,
            resize_class_chain: None,
            frame: None,
            projected_to_desktop: false,
            initial_position: Some(killer_ui::initial_position::DISABLED_INITIAL_POSITION),
        }
    }

    fn clear_relations(&mut self) {
        self.relations = std::array::from_fn(|_| Vec::new());
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InitializeError {
    MissingInstance(WindowInstance),
    MissingVisibility(WindowInstance),
    MissingWindowFlags(WindowInstance),
    MissingControlState(WindowInstance, u32),
    MissingRecord(killer_ui::window_records::RecordKey),
    UnsupportedVisibility(WindowInstance, u32),
    Promotion(killer_ui::window_children::PromotionError),
    MissingGeometry(WindowInstance),
    MissingDesktopWindow(WindowInstance),
    MissingInitialPosition(WindowInstance),
    MissingStuckFlags(WindowInstance),
    MissingRatios(WindowInstance),
    MissingResizeClass(WindowInstance),
    UnsupportedFrameResize(WindowInstance),
    UnsupportedResize(WindowInstance, u32),
    UnsupportedClipHide(WindowInstance),
    InvalidInitialPosition(UndefinedInitialCoordinate),
    InvalidChild(WindowInstance, ChildNodeId),
    CyclicParent(WindowInstance),
    UnsupportedMovement(WindowInstance, u32),
    UnsupportedMoveDispatch(WindowInstance, u32),
}

impl std::fmt::Display for InitializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Initialize: {self:?}")
    }
}

impl std::error::Error for InitializeError {}

#[derive(Default)]
pub struct InitializeGeometry {
    pub nodes: HashMap<WindowInstance, GeometryNode>,
    pub owner_slots: HashMap<(WindowInstance, u32), WindowInstance>,
}

fn project(rect: PositioningRect) -> Rect {
    Rect::new(rect.x, rect.y, float_size_to_int(rect.width), float_size_to_int(rect.height))
}

fn native_rect(rect: Rect) -> PositioningRect {
    PositioningRect { x: rect.x, y: rect.y, width: rect.w as f32, height: rect.h as f32 }
}

impl InitializeGeometry {
    pub fn capture(desktop: &Desktop, registry: &WindowRegistry) -> Result<Self, InitializeError> {
        let mut geometry = Self::default();
        let root = PositioningRect { x: 0, y: 0, width: desktop.size.0 as f32, height: desktop.size.1 as f32 };
        for instance in [&registry.primary, &registry.secondary] {
            if registry.node(instance).is_none() {
                return Err(InitializeError::MissingInstance(instance.clone()));
            }
            geometry.nodes.insert(instance.clone(), GeometryNode::new(root));
        }
        for window in desktop.windows.iter().filter(|window| !killer_ui::window_owner::VIRTUAL_ROOT_CHILDREN.contains(&window.id)) {
            let instance = window.instance();
            if registry.node(&instance).is_none() {
                return Err(InitializeError::MissingInstance(instance));
            }
            let mut node = GeometryNode::new(native_rect(window.rect));
            node.projected_to_desktop = true;
            node.initial_position = window.initial_position;
            if let Some(owner) = window.owner
                && let Some(parent) = registry.parent_instance(&instance)
            {
                geometry.owner_slots.insert((parent.clone(), owner.slot), instance.clone());
            }
            geometry.nodes.insert(instance, node);
        }
        Ok(geometry)
    }

    pub fn import_desktop(&mut self, desktop: &Desktop, registry: &WindowRegistry) -> Result<(), InitializeError> {
        for window in desktop.windows.iter().filter(|window| !killer_ui::window_owner::VIRTUAL_ROOT_CHILDREN.contains(&window.id)) {
            let instance = window.instance();
            if registry.node(&instance).is_none() {
                return Err(InitializeError::MissingInstance(instance));
            }
            if !self.nodes.contains_key(&instance) {
                return Err(InitializeError::MissingGeometry(instance));
            }
        }
        for window in &desktop.windows {
            if let Some(node) = self.nodes.get_mut(&window.instance()) {
                node.rect.x = window.rect.x;
                node.rect.y = window.rect.y;
                if window.rect.w != node.projected.w {
                    node.rect.width = window.rect.w as f32;
                }
                if window.rect.h != node.projected.h {
                    node.rect.height = window.rect.h as f32;
                }
                node.projected = window.rect;
            }
        }
        Ok(())
    }

    pub fn window_flags(&self, instance: &WindowInstance) -> Result<u32, InitializeError> {
        self.nodes
            .get(instance)
            .and_then(|node| node.window_flags)
            .ok_or_else(|| InitializeError::MissingWindowFlags(instance.clone()))
    }

    pub fn rect(&self, instance: &WindowInstance) -> Result<PositioningRect, InitializeError> {
        self.nodes.get(instance).map(|node| node.rect).ok_or_else(|| InitializeError::MissingGeometry(instance.clone()))
    }

    fn validate_tree(
        &self,
        desktop: &Desktop,
        registry: &WindowRegistry,
        instance: &WindowInstance,
        dispatch: bool,
    ) -> Result<(), InitializeError> {
        let mut pending = vec![instance.clone()];
        let mut seen = HashSet::new();
        while let Some(instance) = pending.pop() {
            if !seen.insert(instance.clone()) {
                return Err(InitializeError::CyclicParent(instance));
            }
            let registered =
                registry.node(&instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?;
            let state = self.nodes.get(&instance).ok_or_else(|| InitializeError::MissingGeometry(instance.clone()))?;
            if state.projected_to_desktop && desktop.instance(&instance).is_none() {
                return Err(InitializeError::MissingDesktopWindow(instance));
            }
            if registered.dispatch.movement != 0x100351c0 {
                return Err(InitializeError::UnsupportedMovement(instance, registered.dispatch.movement));
            }
            if dispatch && registered.dispatch.move_dispatch != 0x10054f40 {
                return Err(InitializeError::UnsupportedMoveDispatch(instance, registered.dispatch.move_dispatch));
            }
            if state.anchor != 0
                && let Some(parent) = registry.parent_instance(&instance)
            {
                self.rect(parent)?;
            }
            let mut child = registered.children.first();
            while let Some(node) = child {
                pending.push(child_instance(registry, &instance, node)?);
                child = registered.children.next(node);
            }
        }
        Ok(())
    }

    pub fn move_instance(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        instance: &WindowInstance,
        delta: (i32, i32),
    ) -> Result<(), InitializeError> {
        self.validate_tree(desktop, registry, instance, false)?;
        let mut backend =
            GeometryBackend { geometry: self, desktop, registry, instance: instance.clone(), error: None };
        move_window(&mut backend, delta.0, delta.1);
        backend.finish()
    }

    pub fn set_position(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        instance: &WindowInstance,
        position: (i32, i32),
        root: PositioningRect,
    ) -> Result<(), InitializeError> {
        self.validate_tree(desktop, registry, instance, true)?;
        let mut backend =
            GeometryBackend { geometry: self, desktop, registry, instance: instance.clone(), error: None };
        set_relative_position(&mut backend, position.0, position.1, root);
        backend.finish()
    }

    pub fn restore_initial_position(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        instance: &WindowInstance,
    ) -> Result<(), InitializeError> {
        let state = self.nodes.get(instance).ok_or_else(|| InitializeError::MissingGeometry(instance.clone()))?;
        let initial = if state.projected_to_desktop {
            desktop
                .instance(instance)
                .ok_or_else(|| InitializeError::MissingDesktopWindow(instance.clone()))?
                .initial_position
        } else {
            state.initial_position
        }
        .ok_or_else(|| InitializeError::MissingInitialPosition(instance.clone()))?;
        if !initial.enabled {
            return Ok(());
        }
        self.validate_tree(desktop, registry, instance, true)?;
        self.rect(&registry.primary)?;
        let screen = desktop.size;
        let mut backend =
            GeometryBackend { geometry: self, desktop, registry, instance: instance.clone(), error: None };
        initial.restore(screen, &mut backend).map_err(InitializeError::InvalidInitialPosition)?;
        backend.finish()
    }

    pub fn restore_primary_children(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
    ) -> Result<(), InitializeError> {
        let root = registry
            .node(&registry.primary)
            .ok_or_else(|| InitializeError::MissingInstance(registry.primary.clone()))?;
        let mut current = root.children.first();
        while let Some(node) = current {
            let instance = child_instance(registry, &registry.primary, node)?;
            let registered =
                registry.node(&instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?;
            if registered.vtable != 0x101adc48 {
                self.restore_initial_position(desktop, registry, &instance)?;
            }
            current = registry.node(&registry.primary).and_then(|root| root.children.next(node));
        }
        Ok(())
    }

    pub fn set_window_size(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        instance: &WindowInstance,
        request: killer_ui::window_set_size::SetWindowSize,
    ) -> Result<(), InitializeError> {
        self.validate_tree(desktop, registry, instance, true)?;
        if self.window_flags(instance)? & 0x100000 != 0 {
            return Err(InitializeError::UnsupportedClipHide(instance.clone()));
        }
        if request.remember && self.nodes.get(instance).and_then(|node| node.initial_position).is_none() {
            return Err(InitializeError::MissingInitialPosition(instance.clone()));
        }
        let mut backend =
            GeometryBackend { geometry: self, desktop, registry, instance: instance.clone(), error: None };
        killer_ui::window_set_size::set_window_size(&mut backend, request);
        backend.finish()
    }

    pub fn resize_instance(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        instance: &WindowInstance,
        size: (f32, f32),
    ) -> Result<(), InitializeError> {
        let registered = registry.node(instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?;
        let handler = killer_ui::window_resize_dispatch::ResizeHandler::from_address(registered.dispatch.resize)
            .ok_or_else(|| InitializeError::UnsupportedResize(instance.clone(), registered.dispatch.resize))?;
        let mut backend =
            GeometryBackend { geometry: self, desktop, registry, instance: instance.clone(), error: None };
        killer_ui::window_resize_dispatch::resize_with_handler(&mut backend, handler, size.0, size.1)?;
        backend.finish()
    }

    fn resize_base_instance(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        instance: &WindowInstance,
        size: (f32, f32),
    ) -> Result<(), InitializeError> {
        self.validate_tree(desktop, registry, instance, false)?;
        let registered = registry.node(instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?;
        let node = self.nodes.get(instance).ok_or_else(|| InitializeError::MissingGeometry(instance.clone()))?;
        if self.window_flags(instance)? & 0x80 != 0
            && let Some(frame) = &node.frame
        {
            let frame_node = registry.node(frame).ok_or_else(|| InitializeError::MissingInstance(frame.clone()))?;
            if frame_node.vtable != 0x101a2b40 {
                return Err(InitializeError::UnsupportedFrameResize(frame.clone()));
            }
            self.validate_tree(desktop, registry, frame, false)?;
            if self.nodes.get(frame).and_then(|node| node.resize_class_chain.as_ref()).is_none() {
                return Err(InitializeError::MissingResizeClass(frame.clone()));
            }
        }
        let chain =
            node.resize_class_chain.as_ref().ok_or_else(|| InitializeError::MissingResizeClass(instance.clone()))?;
        let propagates = chain.iter().any(|class| killer_ui::window_resize::PROPAGATING_CLASSES.contains(class));
        if propagates {
            let mut current = registered.children.first();
            while let Some(child_node) = current {
                let child = child_instance(registry, instance, child_node)?;
                let state = self.nodes.get(&child).ok_or_else(|| InitializeError::MissingGeometry(child.clone()))?;
                if state.stuck_flags.is_none() {
                    return Err(InitializeError::MissingStuckFlags(child));
                }
                if state.ratios.is_none() {
                    return Err(InitializeError::MissingRatios(child));
                }
                current = registered.children.next(child_node);
            }
        }
        let mut backend =
            GeometryBackend { geometry: self, desktop, registry, instance: instance.clone(), error: None };
        killer_ui::window_resize::resize_window(&mut backend, size.0, size.1);
        backend.finish()
    }

    fn collect_group(
        &self,
        instance: &WindowInstance,
        output: &mut Vec<WindowInstance>,
    ) -> Result<(), InitializeError> {
        if output.contains(instance) {
            return Ok(());
        }
        output.push(instance.clone());
        let mut stack = vec![(instance.clone(), 0, 0)];
        while let Some((instance, side, index)) = stack.last_mut() {
            if *side == 4 {
                stack.pop();
                continue;
            }
            let state = self.nodes.get(instance).ok_or_else(|| InitializeError::MissingGeometry(instance.clone()))?;
            if *index == state.relations[*side].len() {
                *side += 1;
                *index = 0;
                continue;
            }
            let next = state.relations[*side][*index].clone();
            *index += 1;
            if let Some(next) = next
                && !output.contains(&next)
            {
                output.push(next.clone());
                stack.push((next, 0, 0));
            }
        }
        Ok(())
    }

    pub fn rearrange(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        instance: &WindowInstance,
        old: (f32, f32),
        new: (f32, f32),
    ) -> Result<(), InitializeError> {
        let node = self.nodes.get(instance).ok_or_else(|| InitializeError::MissingGeometry(instance.clone()))?;
        let flags = node.stuck_flags.ok_or_else(|| InitializeError::MissingStuckFlags(instance.clone()))?;
        if flags & 1 != 0 {
            return Ok(());
        }
        let mut ratios = node.ratios.ok_or_else(|| InitializeError::MissingRatios(instance.clone()))?;
        let mut bounds = node.rect;
        let mut owners = [instance.clone(), instance.clone()];
        let mut group = Vec::new();
        self.collect_group(instance, &mut group)?;
        for member in &group {
            self.validate_tree(desktop, registry, member, false)?;
            let state = self.nodes.get(member).ok_or_else(|| InitializeError::MissingGeometry(member.clone()))?;
            if state.stuck_flags.is_none() {
                return Err(InitializeError::MissingStuckFlags(member.clone()));
            }
            let rect = state.rect;
            if rect.x < bounds.x {
                bounds.width = (f64::from(bounds.x.wrapping_sub(rect.x)) + f64::from(bounds.width)) as f32;
                bounds.x = rect.x;
                owners[0] = member.clone();
            }
            if rect.y < bounds.y {
                bounds.height = (f64::from(bounds.y.wrapping_sub(rect.y)) + f64::from(bounds.height)) as f32;
                bounds.y = rect.y;
                owners[1] = member.clone();
            }
            let right = f64::from(rect.x) + f64::from(rect.width);
            if right > f64::from(bounds.x) + f64::from(bounds.width) {
                bounds.width = (right - f64::from(bounds.x)) as f32;
            }
            let bottom = f64::from(rect.y) + f64::from(rect.height);
            if bottom > f64::from(bounds.y) + f64::from(bounds.height) {
                bounds.height = (bottom - f64::from(bounds.y)) as f32;
            }
        }
        let x = self.rect(&owners[0])?.x;
        let y = self.rect(&owners[1])?.y;
        let precise_x = f64::from(x);
        let rounded_y = f64::from(y as f32);
        if f64::from(bounds.width) + 200.0 < f64::from(old.0) {
            ratios[0] = (precise_x / (f64::from(old.0) - f64::from(bounds.width))) as f32;
        }
        if f64::from(bounds.height) + 200.0 < f64::from(old.1) {
            ratios[1] = (rounded_y / (f64::from(old.1) - f64::from(bounds.height))) as f32;
        }
        let dx = if new.0 > bounds.width {
            ((f64::from(new.0) - f64::from(bounds.width)) * f64::from(ratios[0]) - precise_x) as f32
        } else {
            x.wrapping_neg() as f32
        };
        let dy = if new.1 > bounds.height {
            ((f64::from(new.1) - f64::from(bounds.height)) * f64::from(ratios[1]) - rounded_y) as f32
        } else {
            y.wrapping_neg() as f32
        };
        if let Some(state) = self.nodes.get_mut(instance) {
            state.ratios = Some(ratios);
        }
        for member in group {
            if let Some(state) = self.nodes.get_mut(&member)
                && let Some(flags) = state.stuck_flags.as_mut()
            {
                *flags |= 1;
            }
            self.move_instance(desktop, registry, &member, (float_size_to_int(dx), float_size_to_int(dy)))?;
        }
        Ok(())
    }
}

pub fn child_instance(
    registry: &WindowRegistry,
    parent: &WindowInstance,
    child: ChildNodeId,
) -> Result<WindowInstance, InitializeError> {
    registry
        .node(parent)
        .and_then(|node| node.children.record(child))
        .and_then(|record| registry.manager.records.get(record))
        .and_then(|record| record.window.clone())
        .ok_or_else(|| InitializeError::InvalidChild(parent.clone(), child))
}

struct GeometryBackend<'a> {
    geometry: &'a mut InitializeGeometry,
    desktop: &'a mut Desktop,
    registry: &'a WindowRegistry,
    instance: WindowInstance,
    error: Option<InitializeError>,
}

impl GeometryBackend<'_> {
    fn finish(self) -> Result<(), InitializeError> {
        self.error.map_or(Ok(()), Err)
    }

    fn fail(&mut self, error: InitializeError) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn read_rect(&mut self, instance: &WindowInstance) -> PositioningRect {
        match self.geometry.rect(instance) {
            Ok(rect) => rect,
            Err(error) => {
                self.fail(error);
                PositioningRect { x: 0, y: 0, width: 0.0, height: 0.0 }
            }
        }
    }
}

impl WindowMoveBackend for GeometryBackend<'_> {
    type ChildNode = ChildNodeId;
    fn anchor(&mut self) -> u32 {
        self.geometry.nodes.get(&self.instance).map_or(0, |node| node.anchor)
    }
    fn parent_rect(&mut self) -> Option<PositioningRect> {
        let parent = self.registry.parent_instance(&self.instance)?.clone();
        Some(self.read_rect(&parent))
    }
    fn rect(&mut self) -> PositioningRect {
        self.read_rect(&self.instance.clone())
    }
    fn anchor_offset(&mut self) -> (i32, i32) {
        self.geometry.nodes.get(&self.instance).map_or((0, 0), |node| node.anchor_offset)
    }
    fn set_position(&mut self, x: i32, y: i32) {
        if self.error.is_some() {
            return;
        }
        if let Some(node) = self.geometry.nodes.get_mut(&self.instance) {
            node.rect.x = x;
            node.rect.y = y;
            node.projected = project(node.rect);
            if let Some(window) = self.desktop.instance_mut(&self.instance) {
                window.rect = node.projected;
            }
        } else {
            self.fail(InitializeError::MissingGeometry(self.instance.clone()));
        }
    }
    fn first_child(&mut self) -> Option<ChildNodeId> {
        self.registry.node(&self.instance)?.children.first()
    }
    fn next_child(&mut self, node: ChildNodeId) -> Option<ChildNodeId> {
        self.registry.node(&self.instance)?.children.next(node)
    }
    fn move_child(&mut self, node: ChildNodeId, dx: i32, dy: i32) {
        if self.error.is_some() {
            return;
        }
        match child_instance(self.registry, &self.instance, node) {
            Ok(instance) => {
                let mut child = GeometryBackend {
                    geometry: self.geometry,
                    desktop: self.desktop,
                    registry: self.registry,
                    instance,
                    error: None,
                };
                move_window(&mut child, dx, dy);
                if let Err(error) = child.finish() {
                    self.fail(error);
                }
            }
            Err(error) => self.fail(error),
        }
    }
}

impl RelativePositionBackend for GeometryBackend<'_> {
    fn clear_relations(&mut self) {
        if self.error.is_none()
            && let Some(node) = self.geometry.nodes.get_mut(&self.instance)
        {
            node.clear_relations();
        }
    }
    fn position(&mut self) -> (i32, i32) {
        let rect = WindowMoveBackend::rect(self);
        (rect.x, rect.y)
    }
    fn dispatch_move(&mut self, dx: i32, dy: i32) {
        if self.error.is_none() {
            move_window(self, dx, dy);
        }
    }
}

impl InitialPositionBackend for GeometryBackend<'_> {
    type RootRect = PositioningRect;
    fn root_rect(&mut self) -> PositioningRect {
        self.read_rect(&self.registry.primary.clone())
    }
    fn set_position(&mut self, x: i32, y: i32, root: PositioningRect) {
        if self.error.is_none() {
            set_relative_position(self, x, y, root);
        }
    }
}

impl killer_ui::window_set_size::SetWindowSizeBackend for GeometryBackend<'_> {
    type ChildNode = ChildNodeId;
    fn initial_position(&mut self) -> killer_ui::initial_position::InitialPosition {
        match self.geometry.nodes.get(&self.instance).and_then(|node| node.initial_position) {
            Some(initial) => initial,
            None => {
                self.fail(InitializeError::MissingInitialPosition(self.instance.clone()));
                killer_ui::initial_position::DISABLED_INITIAL_POSITION
            }
        }
    }
    fn set_initial_position(&mut self, initial: killer_ui::initial_position::InitialPosition) {
        if self.error.is_some() {
            return;
        }
        if let Some(node) = self.geometry.nodes.get_mut(&self.instance) {
            node.initial_position = Some(initial);
        }
        if let Some(window) = self.desktop.instance_mut(&self.instance) {
            window.initial_position = Some(initial);
        }
    }
    fn screen_width(&mut self) -> i32 {
        self.desktop.size.0
    }
    fn screen_height(&mut self) -> i32 {
        self.desktop.size.1
    }
    fn clear_relations(&mut self) {
        RelativePositionBackend::clear_relations(self);
    }
    fn position(&mut self) -> (i32, i32) {
        RelativePositionBackend::position(self)
    }
    fn set_position(&mut self, position: (i32, i32)) {
        WindowMoveBackend::set_position(self, position.0, position.1);
    }
    fn set_size(&mut self, size: (f32, f32)) {
        killer_ui::window_resize::WindowResizeBackend::set_size(self, size);
    }
    fn first_child(&mut self) -> Option<ChildNodeId> {
        WindowMoveBackend::first_child(self)
    }
    fn move_child(&mut self, child: ChildNodeId, delta: (i32, i32)) {
        WindowMoveBackend::move_child(self, child, delta.0, delta.1);
    }
    fn next_child(&mut self, child: ChildNodeId) -> Option<ChildNodeId> {
        WindowMoveBackend::next_child(self, child)
    }
    fn parent_position(&mut self) -> Option<(i32, i32)> {
        WindowMoveBackend::parent_rect(self).map(|rect| (rect.x, rect.y))
    }
    fn flags(&mut self) -> u32 {
        match self.geometry.window_flags(&self.instance) {
            Ok(flags) => flags,
            Err(error) => {
                self.fail(error);
                0
            }
        }
    }
    fn apply_zero_rect(&mut self) {
        self.fail(InitializeError::UnsupportedClipHide(self.instance.clone()));
    }
}

impl killer_ui::window_resize_dispatch::ResizeDispatchBackend for GeometryBackend<'_> {
    type Window = WindowInstance;
    type Error = InitializeError;
    fn resize_base(&mut self, width: f32, height: f32) -> Result<(), InitializeError> {
        self.geometry.resize_base_instance(self.desktop, self.registry, &self.instance, (width, height))
    }
    fn child(&mut self, slot: u32) -> Option<WindowInstance> {
        self.geometry.owner_slots.get(&(self.instance.clone(), slot)).cloned()
    }
    fn size(&mut self) -> (f32, f32) {
        let rect = WindowMoveBackend::rect(self);
        (rect.width, rect.height)
    }
    fn resize_child(&mut self, child: WindowInstance, width: f32, height: f32) -> Result<(), InitializeError> {
        self.geometry.resize_instance(self.desktop, self.registry, &child, (width, height))
    }
}

impl killer_ui::window_resize::WindowResizeBackend for GeometryBackend<'_> {
    type ChildNode = ChildNodeId;
    fn clear_relations(&mut self) {
        RelativePositionBackend::clear_relations(self);
    }
    fn has_frame_flag(&mut self) -> bool {
        killer_ui::window_set_size::SetWindowSizeBackend::flags(self) & 0x80 != 0
    }
    fn has_frame(&mut self) -> bool {
        self.geometry.nodes.get(&self.instance).is_some_and(|node| node.frame.is_some())
    }
    fn frame_height(&mut self) -> f32 {
        let frame = self.geometry.nodes.get(&self.instance).and_then(|node| node.frame.clone());
        match frame {
            Some(frame) => self.read_rect(&frame).height,
            None => {
                self.fail(InitializeError::UnsupportedFrameResize(self.instance.clone()));
                0.0
            }
        }
    }
    fn resize_frame(&mut self, width: i32, height: i32) {
        if self.error.is_some() {
            return;
        }
        let frame = self.geometry.nodes.get(&self.instance).and_then(|node| node.frame.clone());
        match frame {
            Some(frame) => {
                let result = self.geometry.resize_base_instance(
                    self.desktop,
                    self.registry,
                    &frame,
                    (width as u32 as f32, height as u32 as f32),
                );
                if let Err(error) = result {
                    self.fail(error);
                }
            }
            None => self.fail(InitializeError::UnsupportedFrameResize(self.instance.clone())),
        }
    }
    fn size(&mut self) -> (f32, f32) {
        let rect = WindowMoveBackend::rect(self);
        (rect.width, rect.height)
    }
    fn set_size(&mut self, size: (f32, f32)) {
        if self.error.is_some() {
            return;
        }
        if let Some(node) = self.geometry.nodes.get_mut(&self.instance) {
            node.rect.width = size.0;
            node.rect.height = size.1;
            node.projected = project(node.rect);
            if let Some(window) = self.desktop.instance_mut(&self.instance) {
                window.rect = node.projected;
            }
        } else {
            self.fail(InitializeError::MissingGeometry(self.instance.clone()));
        }
    }
    fn has_anchor(&mut self) -> bool {
        WindowMoveBackend::anchor(self) != 0
    }
    fn update_anchor(&mut self, x: i32, y: i32) {
        if self.error.is_none() {
            move_window(self, x, y);
        }
    }
    fn inherits(&mut self, class: u32) -> bool {
        self.geometry
            .nodes
            .get(&self.instance)
            .and_then(|node| node.resize_class_chain.as_ref())
            .is_some_and(|chain| chain.contains(&class))
    }
    fn first_child(&mut self) -> Option<ChildNodeId> {
        WindowMoveBackend::first_child(self)
    }
    fn next_child(&mut self, child: ChildNodeId) -> Option<ChildNodeId> {
        WindowMoveBackend::next_child(self, child)
    }
    fn prepare_child(&mut self, child: ChildNodeId, mode: i32) {
        if self.error.is_some() {
            return;
        }
        match child_instance(self.registry, &self.instance, child) {
            Ok(instance) => {
                if let Some(node) = self.geometry.nodes.get_mut(&instance)
                    && let Some(flags) = node.stuck_flags.as_mut()
                {
                    *flags &= !(mode as u32);
                } else {
                    self.fail(InitializeError::MissingStuckFlags(instance));
                }
            }
            Err(error) => self.fail(error),
        }
    }
    fn rearrange_child(&mut self, child: ChildNodeId, old: (f32, f32), new: (f32, f32)) {
        if self.error.is_some() {
            return;
        }
        let result = child_instance(self.registry, &self.instance, child)
            .and_then(|instance| self.geometry.rearrange(self.desktop, self.registry, &instance, old, new));
        if let Err(error) = result {
            self.fail(error);
        }
    }
}

#[derive(Clone, Debug)]
pub struct StatusWidthState {
    pub owner: WindowInstance,
    pub border: WindowInstance,
    pub bar_or_close: WindowInstance,
    pub name: WindowInstance,
    pub maximum: i32,
    pub accumulated: i32,
    pub target: bool,
    pub target_name_width: i32,
}

impl StatusWidthState {
    pub fn request_width(
        &mut self,
        geometry: &mut InitializeGeometry,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        width: i32,
    ) -> Result<(), InitializeError> {
        let rect = geometry.rect(&self.owner)?;
        let delta = killer_ui::window_restore::x87_to_int(f64::from(width) - f64::from(rect.width));
        self.apply_delta(geometry, desktop, registry, delta)
    }

    pub fn apply_delta(
        &mut self,
        geometry: &mut InitializeGeometry,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        mut delta: i32,
    ) -> Result<(), InitializeError> {
        let rect = geometry.rect(&self.owner)?;
        if self.target {
            self.maximum = 202;
        }
        if delta > 0 && self.accumulated < self.maximum {
            self.accumulated = self.accumulated.wrapping_add(delta);
            if self.accumulated > self.maximum {
                delta = delta.wrapping_add(self.maximum.wrapping_sub(self.accumulated));
                self.accumulated = self.maximum;
            }
        } else if delta < 0 && self.accumulated > 0 {
            self.accumulated = self.accumulated.wrapping_add(delta);
            if self.accumulated < 0 {
                delta = delta.wrapping_sub(self.accumulated);
                self.accumulated = 0;
            }
        } else {
            return Ok(());
        }
        let delta = delta as f32;
        let width = (f64::from(rect.width) + f64::from(delta)) as f32;
        geometry.resize_instance(desktop, registry, &self.owner, (width, rect.height))?;
        let border = geometry.rect(&self.border)?;
        let owner = geometry.rect(&self.owner)?;
        geometry.set_window_size(
            desktop,
            registry,
            &self.border,
            killer_ui::window_set_size::SetWindowSize {
                position: (killer_ui::window_restore::x87_to_int(f64::from(owner.width) - f64::from(border.width)), 0),
                size: (float_size_to_int(border.width), float_size_to_int(border.height)),
                remember: false,
                modes: (0, 3),
            },
        )?;
        if self.target {
            let owner = geometry.rect(&self.owner)?;
            geometry.set_window_size(
                desktop,
                registry,
                &self.bar_or_close,
                killer_ui::window_set_size::SetWindowSize {
                    position: (killer_ui::window_restore::x87_to_int(f64::from(owner.width) - 19.0), 6),
                    size: (15, 15),
                    remember: false,
                    modes: (0, 3),
                },
            )?;
            self.target_name_width =
                killer_ui::window_restore::x87_to_int(f64::from(geometry.rect(&self.owner)?.width) - 22.0);
        } else {
            let bar = geometry.rect(&self.bar_or_close)?;
            geometry.resize_instance(
                desktop,
                registry,
                &self.bar_or_close,
                ((f64::from(bar.width) + f64::from(delta)) as f32, 12.0),
            )?;
        }
        let name = geometry.rect(&self.name)?;
        geometry.resize_instance(
            desktop,
            registry,
            &self.name,
            ((f64::from(name.width) + f64::from(delta)) as f32, name.height),
        )
    }
}

#[derive(Clone, Debug)]
pub struct VisibilityNode {
    pub flags: u32,
    pub focused: bool,
    pub minimized: Option<WindowInstance>,
    pub dialog_input_mode: Option<u32>,
    pub kill_focus_address: u32,
    pub resize_border_pressed: Option<(u32, u32)>,
}

#[derive(Default)]
pub struct InitializeVisibility {
    pub nodes: HashMap<WindowInstance, VisibilityNode>,
    pub focused_record: Option<killer_ui::window_records::RecordKey>,
}

impl InitializeVisibility {
    fn node(&self, instance: &WindowInstance) -> Result<&VisibilityNode, InitializeError> {
        self.nodes.get(instance).ok_or_else(|| InitializeError::MissingVisibility(instance.clone()))
    }

    pub fn visible(&self, instance: &WindowInstance) -> Result<bool, InitializeError> {
        Ok(self.node(instance)?.flags & 2 != 0)
    }

    fn set_flags(
        &mut self,
        desktop: &mut Desktop,
        geometry: &mut InitializeGeometry,
        instance: &WindowInstance,
        flags: u32,
    ) -> Result<(), InitializeError> {
        let node = self.nodes.get_mut(instance).ok_or_else(|| InitializeError::MissingVisibility(instance.clone()))?;
        geometry
            .nodes
            .get_mut(instance)
            .ok_or_else(|| InitializeError::MissingGeometry(instance.clone()))?
            .window_flags = Some(flags);
        node.flags = flags;
        if let Some(window) = desktop.instance_mut(instance) {
            window.visible = flags & 2 != 0;
        }
        Ok(())
    }

    fn kill_focus(&mut self, registry: &WindowRegistry, instance: &WindowInstance) -> Result<(), InitializeError> {
        let registered = registry.node(instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?;
        let node = self.nodes.get_mut(instance).ok_or_else(|| InitializeError::MissingVisibility(instance.clone()))?;
        match node.kill_focus_address {
            0x10034790 => {}
            0x10011a80 => {
                if node.resize_border_pressed.is_none() {
                    return Err(InitializeError::MissingControlState(instance.clone(), 0x274));
                }
            }
            address => return Err(InitializeError::UnsupportedVisibility(instance.clone(), address)),
        }
        if self.focused_record == Some(registered.record) {
            self.focused_record = None;
        }
        node.focused = false;
        if node.kill_focus_address == 0x10011a80 {
            node.resize_border_pressed = Some((0, 0));
        }
        Ok(())
    }

    fn inherits(geometry: &InitializeGeometry, instance: &WindowInstance, class: u32) -> Result<bool, InitializeError> {
        let chain = geometry
            .nodes
            .get(instance)
            .and_then(|state| state.resize_class_chain.as_ref())
            .ok_or_else(|| InitializeError::MissingResizeClass(instance.clone()))?;
        Ok(chain.contains(&class))
    }

    fn search_focus(
        &mut self,
        desktop: &mut Desktop,
        registry: &mut WindowRegistry,
        geometry: &InitializeGeometry,
        instance: &WindowInstance,
        by_class: bool,
    ) -> Result<bool, InitializeError> {
        let registered = registry.node(instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?;
        let address = registered.dispatch.focus_mode;
        if by_class {
            if Self::inherits(geometry, instance, killer_ui::window_focus::FOCUS_CLASS)? {
                if let Some(parent) = registry.parent_instance(instance).cloned() {
                    self.focus(desktop, registry, geometry, &parent)?;
                }
                return Ok(true);
            }
        } else {
            let mode = killer_ui::window_focus_dispatch::focus_mode_from_address(address)
                .ok_or_else(|| InitializeError::UnsupportedVisibility(instance.clone(), address))?;
            if mode && self.visible(instance)? {
                self.focus(desktop, registry, geometry, instance)?;
                return Ok(true);
            }
        }
        let mut current = registry.node(instance).and_then(|node| node.children.first());
        while let Some(node) = current {
            let child = child_instance(registry, instance, node)?;
            if self.search_focus(desktop, registry, geometry, &child, by_class)? {
                return Ok(true);
            }
            current = registry.node(instance).and_then(|owner| owner.children.next(node));
        }
        Ok(false)
    }

    fn focus_base(
        &mut self,
        desktop: &mut Desktop,
        registry: &mut WindowRegistry,
        geometry: &InitializeGeometry,
        instance: &WindowInstance,
    ) -> Result<(), InitializeError> {
        let record = registry.node(instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?.record;
        if self.focused_record == Some(record) {
            return Ok(());
        }
        if let Some(old) = self.focused_record {
            let owner = registry
                .manager
                .records
                .get(old)
                .and_then(|record| record.window.clone())
                .ok_or(InitializeError::MissingRecord(old))?;
            self.kill_focus(registry, &owner)?;
            self.focused_record = None;
        }
        let address = registry
            .node(instance)
            .ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?
            .dispatch
            .focus_mode;
        let mode = killer_ui::window_focus_dispatch::focus_mode_from_address(address)
            .ok_or_else(|| InitializeError::UnsupportedVisibility(instance.clone(), address))?;
        if !mode && self.search_focus(desktop, registry, geometry, instance, false)? {
            return Ok(());
        }
        if self.node(instance)?.flags & 0x200000 == 0
            && !Self::inherits(geometry, instance, killer_ui::window_focus::FOCUS_CLASS)?
        {
            let mut direct = false;
            let mut current = registry.node(instance).and_then(|node| node.children.first());
            while let Some(node) = current {
                let child = child_instance(registry, instance, node)?;
                if Self::inherits(geometry, &child, killer_ui::window_focus::FOCUS_CLASS)? {
                    direct = true;
                    break;
                }
                current = registry.node(instance).and_then(|owner| owner.children.next(node));
            }
            if !direct && self.search_focus(desktop, registry, geometry, instance, true)? {
                return Ok(());
            }
        }
        self.nodes.get_mut(instance).ok_or_else(|| InitializeError::MissingVisibility(instance.clone()))?.focused =
            true;
        self.focused_record = Some(record);
        let mut flags = HashMap::new();
        for record in &registry.manager.all {
            let window = registry
                .manager
                .records
                .get(*record)
                .and_then(|record| record.window.as_ref())
                .ok_or(InitializeError::MissingRecord(*record))?;
            flags.insert(*record, self.node(window)?.flags);
        }
        let mut child = instance.clone();
        while let Some(parent) = registry.parent_instance(&child).cloned() {
            let record = registry.node(&child).ok_or_else(|| InitializeError::MissingInstance(child.clone()))?.record;
            let parent_node =
                registry.node_mut(&parent).ok_or_else(|| InitializeError::MissingInstance(parent.clone()))?;
            let mut missing = None;
            parent_node
                .children
                .promote(record, |_, record| match flags.get(&record) {
                    Some(flags) => *flags,
                    None => {
                        missing = Some(record);
                        0
                    }
                })
                .map_err(InitializeError::Promotion)?;
            if let Some(record) = missing {
                return Err(InitializeError::MissingRecord(record));
            }
            child = parent;
        }
        self.project_order(desktop, registry)?;
        Ok(())
    }

    fn project_order(&self, desktop: &mut Desktop, registry: &WindowRegistry) -> Result<(), InitializeError> {
        fn append(
            registry: &WindowRegistry,
            parent: &WindowInstance,
            order: &mut Vec<WindowInstance>,
        ) -> Result<(), InitializeError> {
            let mut current =
                registry.node(parent).ok_or_else(|| InitializeError::MissingInstance(parent.clone()))?.children.last();
            while let Some(node) = current {
                let child = child_instance(registry, parent, node)?;
                order.push(child.clone());
                append(registry, &child, order)?;
                current = registry.node(parent).and_then(|owner| owner.children.previous(node));
            }
            Ok(())
        }
        for root in [&registry.primary, &registry.secondary] {
            let mut order = Vec::new();
            append(registry, root, &mut order)?;
            let ranks: HashMap<_, _> =
                order.into_iter().enumerate().map(|(index, instance)| (instance, index)).collect();
            let indices: Vec<_> = desktop
                .windows
                .iter()
                .enumerate()
                .filter_map(|(index, window)| ranks.contains_key(&window.instance()).then_some(index))
                .collect();
            let mut windows: Vec<_> = indices.iter().map(|index| desktop.windows[*index].clone()).collect();
            windows.sort_by_key(|window| ranks.get(&window.instance()).copied());
            for (index, window) in indices.into_iter().zip(windows) {
                desktop.windows[index] = window;
            }
        }
        Ok(())
    }

    pub fn focus(
        &mut self,
        desktop: &mut Desktop,
        registry: &mut WindowRegistry,
        geometry: &InitializeGeometry,
        instance: &WindowInstance,
    ) -> Result<(), InitializeError> {
        let address =
            registry.node(instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?.dispatch.focus;
        match address {
            0x10035af0 => self.focus_base(desktop, registry, geometry, instance),
            0x10007e10 => {
                self.focus_base(desktop, registry, geometry, instance)?;
                let mode = self
                    .node(instance)?
                    .dialog_input_mode
                    .ok_or_else(|| InitializeError::MissingControlState(instance.clone(), 0x250))?;
                if mode != 0 {
                    let child = geometry.owner_slot(instance, 0x240)?.clone();
                    self.focus(desktop, registry, geometry, &child)?;
                }
                Ok(())
            }
            0x10053c10 => {
                let child = geometry.owner_slot(instance, 0x134)?.clone();
                self.focus(desktop, registry, geometry, &child)?;
                self.kill_focus(registry, instance)
            }
            _ => Err(InitializeError::UnsupportedVisibility(instance.clone(), address)),
        }
    }

    fn select_after_hide(
        &mut self,
        desktop: &mut Desktop,
        registry: &mut WindowRegistry,
        geometry: &InitializeGeometry,
        instance: &WindowInstance,
    ) -> Result<(), InitializeError> {
        let mut descendant = false;
        let mut cursor = instance.clone();
        while let Some(parent) = registry.parent_instance(&cursor).cloned() {
            if parent == registry.primary {
                descendant = true;
                break;
            }
            cursor = parent;
        }
        if !self.visible(&registry.primary)? || !descendant {
            return Ok(());
        }
        let root = registry.primary.clone();
        let mut current = registry.node(&root).and_then(|node| node.children.first());
        while let Some(node) = current {
            let Some(next) = registry.node(&root).and_then(|owner| owner.children.next(node)) else {
                break;
            };
            let child = child_instance(registry, &root, next)?;
            let flags = self.node(&child)?.flags;
            if self.visible(&child)? && flags & 0x80000 == 0 {
                self.focus(desktop, registry, geometry, &child)?;
                break;
            }
            current = Some(next);
        }
        Ok(())
    }

    pub fn hide(
        &mut self,
        desktop: &mut Desktop,
        registry: &mut WindowRegistry,
        geometry: &mut InitializeGeometry,
        instance: &WindowInstance,
    ) -> Result<(), InitializeError> {
        let registered = registry.node(instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?;
        let record = registered.record;
        if !matches!(registered.dispatch.hide, 0x10035a90 | 0x10083ef0) {
            return Err(InitializeError::UnsupportedVisibility(instance.clone(), registered.dispatch.hide));
        }
        let mut current = registered.children.first();
        self.set_flags(desktop, geometry, instance, self.node(instance)?.flags & !2)?;
        while let Some(node) = current {
            let child = child_instance(registry, instance, node)?;
            self.hide(desktop, registry, geometry, &child)?;
            current = registry.node(instance).and_then(|owner| owner.children.next(node));
        }
        if self.node(instance)?.focused {
            if self.focused_record == Some(record) {
                self.focused_record = None;
            }
            self.nodes.get_mut(instance).ok_or_else(|| InitializeError::MissingVisibility(instance.clone()))?.focused =
                false;
            self.select_after_hide(desktop, registry, geometry, instance)?;
        }
        Ok(())
    }

    pub fn show(
        &mut self,
        desktop: &mut Desktop,
        registry: &mut WindowRegistry,
        geometry: &mut InitializeGeometry,
        instance: &WindowInstance,
    ) -> Result<(), InitializeError> {
        let address =
            registry.node(instance).ok_or_else(|| InitializeError::MissingInstance(instance.clone()))?.dispatch.show;
        if !matches!(address, 0x10034590 | 0x100568e0) {
            return Err(InitializeError::UnsupportedVisibility(instance.clone(), address));
        }
        if self.node(instance)?.flags & 0x2000 != 0 {
            let minimized = self
                .node(instance)?
                .minimized
                .clone()
                .ok_or_else(|| InitializeError::MissingControlState(instance.clone(), 0x3c))?;
            self.hide(desktop, registry, geometry, &minimized)?;
        }
        self.set_flags(desktop, geometry, instance, self.node(instance)?.flags | 2)?;
        let mut current = registry.node(instance).and_then(|node| node.children.first());
        while let Some(node) = current {
            let child = child_instance(registry, instance, node)?;
            self.show(desktop, registry, geometry, &child)?;
            current = registry.node(instance).and_then(|owner| owner.children.next(node));
        }
        if address == 0x100568e0 {
            geometry.rearrange_system_message(desktop, registry, self, instance)?;
        }
        Ok(())
    }
}

impl InitializeGeometry {
    pub fn owner_slot(&self, owner: &WindowInstance, slot: u32) -> Result<&WindowInstance, InitializeError> {
        self.owner_slots
            .get(&(owner.clone(), slot))
            .ok_or_else(|| InitializeError::MissingControlState(owner.clone(), slot))
    }

    pub fn reposition_notifications(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        visibility: &InitializeVisibility,
    ) -> Result<(), InitializeError> {
        let root = self.rect(&registry.primary)?;
        let chat = self.owner_slot(&registry.primary, 0xf4)?;
        let chat_y = self.rect(chat)?.y;
        let message = self.owner_slot(chat, 0x1a0)?;
        let y = chat_y.wrapping_sub(if visibility.visible(message)? { 167 } else { 37 });
        for (slot, x) in [(0x17c, 5), (0x180, 42), (0x184, 79)] {
            let button = self.owner_slot(&registry.primary, slot)?.clone();
            self.set_position(desktop, registry, &button, (x, y), root)?;
        }
        Ok(())
    }

    pub fn rearrange_system_message(
        &mut self,
        desktop: &mut Desktop,
        registry: &WindowRegistry,
        visibility: &InitializeVisibility,
        instance: &WindowInstance,
    ) -> Result<(), InitializeError> {
        let chat = self.owner_slot(&registry.primary, 0xf4)?.clone();
        let chat_y = self.rect(&chat)?.y;
        self.reposition_notifications(desktop, registry, visibility)?;
        if let Some(child) = self.owner_slots.get(&(chat, 0x1b0)).cloned() {
            let delta = chat_y.wrapping_sub(self.rect(&child)?.y).wrapping_sub(if visibility.visible(instance)? {
                321
            } else {
                191
            });
            self.move_instance(desktop, registry, &child, (0, delta))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::initial_position::{DISABLED_INITIAL_POSITION, STATUS_INITIAL_POSITION};
    use killer_ui::window::{Window, WindowId};
    use killer_ui::window_owner::{original_owner, original_registration};

    fn test_node(rect: PositioningRect) -> GeometryNode {
        let mut node = GeometryNode::new(rect);
        node.window_flags = Some(0);
        node
    }

    fn fixture() -> (Desktop, WindowRegistry, InitializeGeometry) {
        let mut desktop = Desktop::new((1000, 800));
        for (id, rect) in [
            (WindowId::STATUS, Rect::new(400, 100, 200, 84)),
            (WindowId::CHAT, Rect::new(10, 300, 348, 247)),
            (WindowId::SYSTEM_MSG, Rect::new(10, 153, 348, 130)),
            (WindowId::OPTION, Rect::new(30, 40, 256, 480)),
        ] {
            let mut window = Window::new(id, rect, Rect::default());
            window.owner = original_owner(id);
            window.registration = original_registration(id);
            window.initial_position = Some(STATUS_INITIAL_POSITION);
            desktop.windows.push(window);
        }
        let registry = WindowRegistry::from_windows(&desktop.windows).unwrap();
        let mut geometry = InitializeGeometry::capture(&desktop, &registry).unwrap();
        for (instance, node) in &mut geometry.nodes {
            node.window_flags = Some(registry.node(instance).unwrap().registration_flags);
        }
        (desktop, registry, geometry)
    }

    #[test]
    fn movement_reaches_live_registered_children_independent_of_paint_order() {
        let (mut desktop, registry, mut geometry) = fixture();
        let chat = desktop.window(WindowId::CHAT).unwrap().instance();
        desktop.windows.reverse();
        geometry.move_instance(&mut desktop, &registry, &chat, (70, -30)).unwrap();
        assert_eq!(desktop.window(WindowId::CHAT).unwrap().rect, Rect::new(80, 270, 348, 247));
        assert_eq!(desktop.window(WindowId::SYSTEM_MSG).unwrap().rect, Rect::new(80, 123, 348, 130));
        assert_eq!(desktop.window(WindowId::OPTION).unwrap().rect, Rect::new(30, 40, 256, 480));
    }

    #[test]
    fn restore_uses_primary_root_and_moves_child_with_separate_axis_dispatch() {
        let (mut desktop, registry, mut geometry) = fixture();
        let chat = desktop.window(WindowId::CHAT).unwrap().instance();
        let state = geometry.nodes.get_mut(&chat).unwrap();
        state.relations[0].push(Some(chat.clone()));
        state.stuck_flags = Some(0xa0);
        let root = geometry.nodes.get_mut(&registry.primary).unwrap();
        root.rect.x = 20;
        root.rect.y = 30;
        geometry.restore_initial_position(&mut desktop, &registry, &chat).unwrap();
        assert_eq!(desktop.window(WindowId::CHAT).unwrap().rect, Rect::new(20, 30, 348, 247));
        assert_eq!(desktop.window(WindowId::SYSTEM_MSG).unwrap().rect, Rect::new(20, -117, 348, 130));
        assert!(geometry.nodes[&chat].relations.iter().all(Vec::is_empty));
        assert_eq!(geometry.nodes[&chat].stuck_flags, Some(0xa0));
        assert_eq!(desktop.window(WindowId::CHAT).unwrap().initial_position, Some(STATUS_INITIAL_POSITION));
    }

    #[test]
    fn native_fractional_size_survives_integer_projection_and_external_drag() {
        let (mut desktop, registry, mut geometry) = fixture();
        let status = desktop.window(WindowId::STATUS).unwrap().instance();
        geometry.nodes.get_mut(&status).unwrap().rect.width = 200.75;
        geometry.move_instance(&mut desktop, &registry, &status, (1, 2)).unwrap();
        desktop.window_mut(WindowId::STATUS).unwrap().rect.x = 99;
        geometry.import_desktop(&desktop, &registry).unwrap();
        assert_eq!(geometry.rect(&status).unwrap().width, 200.75);
        assert_eq!(geometry.rect(&status).unwrap().x, 99);
        desktop.window_mut(WindowId::STATUS).unwrap().rect.w = 250;
        geometry.import_desktop(&desktop, &registry).unwrap();
        assert_eq!(geometry.rect(&status).unwrap().width, 250.0);
    }

    #[test]
    fn absent_or_replaced_instance_fails_before_moving_the_owner() {
        let (mut desktop, registry, mut geometry) = fixture();
        let chat = desktop.window(WindowId::CHAT).unwrap().instance();
        let old = desktop.window(WindowId::SYSTEM_MSG).unwrap().instance();
        desktop.windows.retain(|window| window.id != WindowId::SYSTEM_MSG);
        desktop.windows.push(Window::new(WindowId::SYSTEM_MSG, Rect::default(), Rect::default()));
        let before = desktop.windows.clone();
        assert_eq!(
            geometry.move_instance(&mut desktop, &registry, &chat, (30, 40)),
            Err(InitializeError::MissingDesktopWindow(old))
        );
        assert_eq!(desktop.windows, before);
    }

    #[test]
    fn initialize_child_pass_preserves_chat_and_secondary_settings_and_hidden_state() {
        let (mut desktop, registry, mut geometry) = fixture();
        desktop.window_mut(WindowId::STATUS).unwrap().visible = false;
        let before = desktop.windows.clone();
        geometry.restore_primary_children(&mut desktop, &registry).unwrap();
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect, Rect::new(0, 0, 200, 84));
        assert!(!desktop.window(WindowId::STATUS).unwrap().visible);
        for id in [WindowId::CHAT, WindowId::SYSTEM_MSG, WindowId::OPTION] {
            assert_eq!(desktop.window(id), before.iter().find(|window| window.id == id));
        }
        desktop.window_mut(WindowId::STATUS).unwrap().initial_position = Some(DISABLED_INITIAL_POSITION);
        let status = desktop.window(WindowId::STATUS).unwrap().instance();
        geometry.nodes.remove(&registry.primary);
        geometry.restore_initial_position(&mut desktop, &registry, &status).unwrap();
    }

    #[test]
    fn rearrange_uses_instance_groups_flags_and_live_child_movement() {
        let (mut desktop, registry, mut geometry) = fixture();
        let chat = desktop.window(WindowId::CHAT).unwrap().instance();
        let status = desktop.window(WindowId::STATUS).unwrap().instance();
        for instance in [&chat, &status] {
            let node = geometry.nodes.get_mut(instance).unwrap();
            node.stuck_flags = Some(0xa0);
            node.ratios = Some([0.5, 0.5]);
        }
        geometry.nodes.get_mut(&chat).unwrap().relations[0].push(Some(status.clone()));
        geometry.rearrange(&mut desktop, &registry, &chat, (1000.0, 800.0), (2000.0, 1600.0)).unwrap();
        assert_eq!(geometry.nodes[&chat].stuck_flags, Some(0xa1));
        assert_eq!(geometry.nodes[&status].stuck_flags, Some(0xa1));
        let before = desktop.windows.clone();
        geometry.rearrange(&mut desktop, &registry, &status, (1.0, 1.0), (1.0, 1.0)).unwrap();
        assert_eq!(desktop.windows, before);
        let chat_rect = desktop.window(WindowId::CHAT).unwrap().rect;
        let child_rect = desktop.window(WindowId::SYSTEM_MSG).unwrap().rect;
        assert_eq!(child_rect.y.wrapping_sub(chat_rect.y), -147);
    }
    #[test]
    fn resize_base_updates_live_geometry_and_anchor_but_preserves_fractional_native_dimensions() {
        let (mut desktop, registry, mut geometry) = fixture();
        let status = desktop.window(WindowId::STATUS).unwrap().instance();
        let state = geometry.nodes.get_mut(&status).unwrap();
        state.resize_class_chain = Some(Vec::new());
        state.anchor = 9;
        state.anchor_offset = (7, -11);
        state.relations[0].push(Some(status.clone()));
        geometry.resize_instance(&mut desktop, &registry, &status, (200.75, 84.5)).unwrap();
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect, Rect::new(806, 704, 200, 84));
        assert_eq!(geometry.rect(&status).unwrap().width, 200.75);
        assert_eq!(geometry.rect(&status).unwrap().height, 84.5);
        assert!(geometry.nodes[&status].relations.iter().all(Vec::is_empty));
        geometry.resize_instance(&mut desktop, &registry, &status, (-10.0, -0.0)).unwrap();
        assert_eq!(geometry.rect(&status).unwrap().width, 0.0);
        assert_eq!(geometry.rect(&status).unwrap().height.to_bits(), (-0.0_f32).to_bits());
    }

    #[test]
    fn resize_does_not_invent_class_chain_or_root_override() {
        let (mut desktop, registry, mut geometry) = fixture();
        let status = desktop.window(WindowId::STATUS).unwrap().instance();
        let before = desktop.windows.clone();
        assert_eq!(
            geometry.resize_instance(&mut desktop, &registry, &status, (172.0, 84.0)),
            Err(InitializeError::MissingResizeClass(status))
        );
        assert_eq!(
            geometry.resize_instance(&mut desktop, &registry, &registry.primary, (1000.0, 800.0)),
            Err(InitializeError::MissingResizeClass(registry.primary.clone()))
        );
        assert_eq!(desktop.windows, before);
    }
    fn width_fixture(target: bool) -> (Desktop, WindowRegistry, InitializeGeometry, StatusWidthState) {
        let (desktop, mut registry, mut geometry) = fixture();
        let owner = desktop.window(WindowId::STATUS).unwrap().instance();
        geometry.nodes.get_mut(&owner).unwrap().resize_class_chain = Some(Vec::new());
        let mut controls = Vec::new();
        for (vtable, rect) in [
            (0x101a2ec0, Rect::new(590, 100, 10, 84)),
            (if target { 0x101a9bf0 } else { 0x101ceb60 }, Rect::new(416, 141, 178, 12)),
            (0x101a76e8, Rect::new(440, 109, 130, 14)),
        ] {
            let instance = WindowInstance::default();
            registry.register_child(&owner, instance.clone(), 2, vtable).unwrap();
            let mut node = test_node(native_rect(rect));
            node.resize_class_chain = Some(Vec::new());
            geometry.nodes.insert(instance.clone(), node);
            controls.push(instance);
        }
        let state = StatusWidthState {
            owner,
            border: controls[0].clone(),
            bar_or_close: controls[1].clone(),
            name: controls[2].clone(),
            maximum: 202,
            accumulated: 28,
            target,
            target_name_width: 178,
        };
        (desktop, registry, geometry, state)
    }

    #[test]
    fn native_status_width_updates_the_live_window_and_all_registered_controls() {
        for target in [false, true] {
            let (mut desktop, registry, mut geometry, mut width) = width_fixture(target);
            width.apply_delta(&mut geometry, &mut desktop, &registry, 1000).unwrap();
            assert_eq!(width.accumulated, 202);
            assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.w, 374);
            let before = desktop.windows.clone();
            width.apply_delta(&mut geometry, &mut desktop, &registry, 1).unwrap();
            assert_eq!(desktop.windows, before);
            width.request_width(&mut geometry, &mut desktop, &registry, 172).unwrap();
            assert_eq!(width.accumulated, 0);
            assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.w, 172);
            assert_eq!(geometry.rect(&width.border).unwrap().x, 562);
            assert_eq!(geometry.rect(&width.border).unwrap().y, 100);
            assert_eq!(geometry.rect(&width.name).unwrap().width, 102.0);
            if target {
                assert_eq!(width.maximum, 202);
                assert_eq!(width.target_name_width, 150);
                assert_eq!(project(geometry.rect(&width.bar_or_close).unwrap()), Rect::new(553, 106, 15, 15));
            } else {
                assert_eq!(geometry.rect(&width.bar_or_close).unwrap().width, 150.0);
                assert_eq!(geometry.rect(&width.bar_or_close).unwrap().height, 12.0);
            }
            assert_eq!(desktop.window(WindowId::STATUS).unwrap().initial_position, Some(STATUS_INITIAL_POSITION));
        }
    }
    #[test]
    fn frame_resize_uses_unsigned_integer_conversion_and_mutates_the_registered_frame() {
        let (mut desktop, mut registry, mut geometry) = fixture();
        let owner = desktop.window(WindowId::STATUS).unwrap().instance();
        let frame = WindowInstance::default();
        registry.register_child(&owner, frame.clone(), 2, 0x101a2b40).unwrap();
        geometry.nodes.get_mut(&owner).unwrap().window_flags = Some(0x80);
        let mut node = test_node(PositioningRect { x: 400, y: 85, width: 200.0, height: 15.75 });
        node.resize_class_chain = Some(Vec::new());
        geometry.nodes.insert(frame.clone(), node);
        let owner_node = geometry.nodes.get_mut(&owner).unwrap();
        owner_node.frame = Some(frame.clone());
        owner_node.resize_class_chain = Some(Vec::new());
        geometry.resize_instance(&mut desktop, &registry, &owner, (300.75, 84.0)).unwrap();
        assert_eq!(geometry.rect(&frame).unwrap().width, 300.0);
        assert_eq!(geometry.rect(&frame).unwrap().height, 15.0);
        assert_eq!(geometry.rect(&owner).unwrap().width, 300.75);
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.w, 300);
    }

    #[test]
    fn root_resize_rearranges_registered_children_and_preserves_secondary_tree() {
        let (mut desktop, registry, mut geometry) = fixture();
        geometry.nodes.get_mut(&registry.primary).unwrap().resize_class_chain = Some(vec![0x102d5280]);
        for window in &desktop.windows {
            let node = geometry.nodes.get_mut(&window.instance()).unwrap();
            node.stuck_flags = Some(0xa3);
            node.ratios = Some([0.5, 0.5]);
        }
        let option_before = desktop.window(WindowId::OPTION).unwrap().clone();
        geometry.resize_instance(&mut desktop, &registry, &registry.primary, (2000.0, 1600.0)).unwrap();
        assert_eq!(geometry.rect(&registry.primary).unwrap().width, 2000.0);
        assert_eq!(desktop.window(WindowId::STATUS).unwrap().rect.x, 900);
        assert_eq!(desktop.window(WindowId::OPTION), Some(&option_before));
        let chat = desktop.window(WindowId::CHAT).unwrap();
        let child = desktop.window(WindowId::SYSTEM_MSG).unwrap();
        assert_eq!(child.rect.y - chat.rect.y, -147);
        assert_eq!(geometry.nodes[&chat.instance()].stuck_flags, Some(0xa1));
    }
    fn visibility_fixture(registry: &WindowRegistry, geometry: &mut InitializeGeometry) -> InitializeVisibility {
        let mut visibility = InitializeVisibility::default();
        for record in &registry.manager.all {
            let instance = registry.manager.records.get(*record).unwrap().window.as_ref().unwrap();
            let node = registry.node(instance).unwrap();
            visibility.nodes.insert(
                instance.clone(),
                VisibilityNode {
                    flags: node.registration_flags,
                    focused: false,
                    minimized: None,
                    dialog_input_mode: None,
                    kill_focus_address: 0x10034790,
                    resize_border_pressed: None,
                },
            );
            geometry.nodes.get_mut(instance).unwrap().resize_class_chain = Some(Vec::new());
        }
        visibility
    }

    #[test]
    fn concrete_hide_reaches_children_and_focus_promotes_registered_instances() {
        let (mut desktop, mut registry, mut geometry) = fixture();
        let chat = desktop.window(WindowId::CHAT).unwrap().instance();
        let message = desktop.window(WindowId::SYSTEM_MSG).unwrap().instance();
        let status = desktop.window(WindowId::STATUS).unwrap().instance();
        let option_index = desktop.windows.iter().position(|w| w.id == WindowId::OPTION).unwrap();
        let mut visibility = visibility_fixture(&registry, &mut geometry);
        visibility.focus(&mut desktop, &mut registry, &geometry, &status).unwrap();
        assert_eq!(visibility.focused_record, Some(registry.node(&status).unwrap().record));
        assert!(visibility.nodes[&status].focused);
        let first = registry.node(&registry.primary).unwrap().children.first().unwrap();
        assert_eq!(child_instance(&registry, &registry.primary, first).unwrap(), status);
        assert_eq!(desktop.windows[option_index].id, WindowId::OPTION);
        visibility.hide(&mut desktop, &mut registry, &mut geometry, &chat).unwrap();
        assert!(!desktop.instance(&chat).unwrap().visible);
        assert!(!desktop.instance(&message).unwrap().visible);
        assert!(!visibility.visible(&chat).unwrap());
        assert!(!visibility.visible(&message).unwrap());
        assert_eq!(visibility.focused_record, Some(registry.node(&status).unwrap().record));
    }

    #[test]
    fn concrete_show_system_message_moves_notifications_and_optional_chat_child() {
        let (mut desktop, mut registry, mut geometry) = fixture();
        let chat = desktop.window(WindowId::CHAT).unwrap().instance();
        let message = desktop.window(WindowId::SYSTEM_MSG).unwrap().instance();
        let mut controls = Vec::new();
        for (owner, slot) in [
            (registry.primary.clone(), 0x17c),
            (registry.primary.clone(), 0x180),
            (registry.primary.clone(), 0x184),
            (chat.clone(), 0x1b0),
        ] {
            let child = WindowInstance::default();
            registry.register_child(&owner, child.clone(), 2, 0x1019f8b8).unwrap();
            geometry
                .nodes
                .insert(child.clone(), test_node(PositioningRect { x: 99, y: 99, width: 32.0, height: 32.0 }));
            geometry.owner_slots.insert((owner, slot), child.clone());
            controls.push(child);
        }
        let mut visibility = visibility_fixture(&registry, &mut geometry);
        visibility.hide(&mut desktop, &mut registry, &mut geometry, &message).unwrap();
        geometry.rearrange_system_message(&mut desktop, &registry, &visibility, &message).unwrap();
        assert_eq!(geometry.rect(&controls[3]).unwrap().y, 109);
        for (child, x) in controls[..3].iter().zip([5, 42, 79]) {
            let rect = geometry.rect(child).unwrap();
            assert_eq!((rect.x, rect.y), (x, 263));
        }
        visibility.show(&mut desktop, &mut registry, &mut geometry, &message).unwrap();
        assert!(desktop.instance(&message).unwrap().visible);
        assert_eq!(geometry.rect(&controls[3]).unwrap().y, -21);
        for (child, x) in controls[..3].iter().zip([5, 42, 79]) {
            let rect = geometry.rect(child).unwrap();
            assert_eq!((rect.x, rect.y), (x, 133));
        }
    }

    #[test]
    fn show_rejects_unported_dispatch_before_visibility_mutation() {
        let (mut desktop, mut registry, mut geometry) = fixture();
        let chat = desktop.window(WindowId::CHAT).unwrap().instance();
        let mut visibility = visibility_fixture(&registry, &mut geometry);
        visibility.hide(&mut desktop, &mut registry, &mut geometry, &chat).unwrap();
        assert_eq!(
            visibility.show(&mut desktop, &mut registry, &mut geometry, &chat),
            Err(InitializeError::UnsupportedVisibility(chat.clone(), 0x10053c30))
        );
        assert!(!desktop.instance(&chat).unwrap().visible);
    }

    #[test]
    fn base_show_requires_minimized_association_and_hides_it_before_showing_owner() {
        let (mut desktop, mut registry, mut geometry) = fixture();
        let status = desktop.window(WindowId::STATUS).unwrap().instance();
        let minimized = WindowInstance::default();
        registry.register_child(&status, minimized.clone(), 2, 0x1019f8b8).unwrap();
        geometry.nodes.insert(minimized.clone(), test_node(PositioningRect { x: 0, y: 0, width: 32.0, height: 32.0 }));
        let mut visibility = visibility_fixture(&registry, &mut geometry);
        visibility.nodes.get_mut(&status).unwrap().flags |= 0x2000;
        assert_eq!(
            visibility.show(&mut desktop, &mut registry, &mut geometry, &status),
            Err(InitializeError::MissingControlState(status.clone(), 0x3c))
        );
        visibility.nodes.get_mut(&status).unwrap().minimized = Some(minimized.clone());
        visibility.show(&mut desktop, &mut registry, &mut geometry, &status).unwrap();
        assert!(desktop.instance(&status).unwrap().visible);
        assert!(visibility.visible(&minimized).unwrap());
    }
}
