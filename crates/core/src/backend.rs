use std::collections::HashMap;

use crate::elements::{Button, Text};
use crate::layout::{Board, Row};
use crate::object::{Object, World};

#[derive(Clone, Debug)]
pub enum SnowComponent {
    Text {
        text: String,
        id: u64,
    },
    Button {
        text: String,
        id: u64,
    },
    Row {
        children: Vec<SnowComponent>,
    },
    Column {
        children: Vec<SnowComponent>,
    },
    Box {
        width: f64,
        height: f64,
        children: Vec<SnowComponent>,
    },
}

#[derive(Clone, Debug)]
pub struct SnowComponentInstance {
    pub id: u64,
    pub component: SnowComponent,
    pub state: HashMap<u64, String>,
}

impl Default for SnowComponent {
    fn default() -> Self {
        Self::Text {
            text: String::new(),
            id: 0,
        }
    }
}

impl SnowComponentInstance {
    pub fn new(component: SnowComponent) -> Self {
        let mut state = HashMap::new();
        Self::collect_state(&component, &mut state);
        Self {
            id: 0,
            component,
            state,
        }
    }

    fn collect_state(component: &SnowComponent, state: &mut HashMap<u64, String>) {
        match component {
            SnowComponent::Text { text, id } | SnowComponent::Button { text, id } => {
                state.insert(*id, text.clone());
            }
            SnowComponent::Row { children }
            | SnowComponent::Column { children }
            | SnowComponent::Box { children, .. } => {
                for child in children {
                    Self::collect_state(child, state);
                }
            }
        }
    }

    pub fn update_text(&mut self, id: u64, text: impl Into<String>) {
        self.state.insert(id, text.into());
    }

    pub fn bind_state(&mut self) -> SnowNode {
        let mut node = self.component.clone().into_node();
        for (id, text) in &self.state {
            node.set_text_by_id(*id, text.clone());
        }
        node
    }

    pub fn into_object(mut self) -> Object {
        self.bind_state().into()
    }

    pub fn into_world(self) -> World {
        World {
            root: self.into_object(),
        }
    }
}

impl SnowComponent {
    pub fn into_object(self) -> Object {
        match self {
            SnowComponent::Text { text, .. } => {
                let leaked: &'static str = Box::leak(text.into_boxed_str());
                Object::from(Text { text: leaked, ..Text::default() })
            }
            SnowComponent::Button { text, .. } => {
                let leaked: &'static str = Box::leak(text.into_boxed_str());
                Object::from(Button { text: leaked })
            }
            SnowComponent::Row { children } => Object::from(Row {
                children: children
                    .into_iter()
                    .map(SnowComponent::into_object)
                    .collect(),
            }),
            SnowComponent::Column { children } => Object::from(Row {
                children: children
                    .into_iter()
                    .map(SnowComponent::into_object)
                    .collect(),
            }),
            SnowComponent::Box {
                width: _,
                height: _,
                children,
            } => Object::from(Board {
                width: crate::types::Size::ViewportWidth,
                height: crate::types::Size::ViewportHeight,
                h_align: crate::types::HAlign::Center,
                v_align: crate::types::VAlign::Middle,
                children: children
                    .into_iter()
                    .map(SnowComponent::into_object)
                    .collect(),
            }),
        }
    }

    pub fn text(id: u64, text: impl Into<String>) -> Self {
        Self::Text {
            id,
            text: text.into(),
        }
    }

    pub fn button(id: u64, text: impl Into<String>) -> Self {
        Self::Button {
            id,
            text: text.into(),
        }
    }

    pub fn row(children: Vec<SnowComponent>) -> Self {
        Self::Row { children }
    }

    pub fn column(children: Vec<SnowComponent>) -> Self {
        Self::Column { children }
    }

    pub fn box_(width: f64, height: f64, children: Vec<SnowComponent>) -> Self {
        Self::Box {
            width,
            height,
            children,
        }
    }

    pub fn into_node(self) -> SnowNode {
        match self {
            SnowComponent::Text { text, id } => SnowNode::Text { text, id },
            SnowComponent::Button { text, id } => SnowNode::Button { text, id },
            SnowComponent::Row { children } => SnowNode::Row {
                children: children.into_iter().map(SnowComponent::into_node).collect(),
            },
            SnowComponent::Column { children } => SnowNode::Column {
                children: children.into_iter().map(SnowComponent::into_node).collect(),
            },
            SnowComponent::Box {
                width,
                height,
                children,
            } => SnowNode::Box {
                width,
                height,
                children: children.into_iter().map(SnowComponent::into_node).collect(),
            },
        }
    }

    pub fn into_world(self) -> World {
        World {
            root: self.into_object(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SnowNode {
    Text {
        text: String,
        id: u64,
    },
    Button {
        text: String,
        id: u64,
    },
    Row {
        children: Vec<SnowNode>,
    },
    Column {
        children: Vec<SnowNode>,
    },
    Box {
        width: f64,
        height: f64,
        children: Vec<SnowNode>,
    },
}

impl Default for SnowNode {
    fn default() -> Self {
        Self::Text {
            text: String::new(),
            id: 0,
        }
    }
}

impl From<SnowNode> for Object {
    fn from(node: SnowNode) -> Self {
        match node {
            SnowNode::Text { text, .. } => {
                let leaked: &'static str = Box::leak(text.into_boxed_str());
                Object::from(Text { text: leaked, ..Text::default() })
            }
            SnowNode::Button { text, .. } => {
                let leaked: &'static str = Box::leak(text.into_boxed_str());
                Object::from(Button { text: leaked })
            }
            SnowNode::Row { children } => Object::from(Row {
                children: children.into_iter().map(Object::from).collect(),
            }),
            SnowNode::Column { children } => Object::from(Row {
                children: children.into_iter().map(Object::from).collect(),
            }),
            SnowNode::Box {
                width: _,
                height: _,
                children,
            } => Object::from(Board {
                width: crate::types::Size::ViewportWidth,
                height: crate::types::Size::ViewportHeight,
                h_align: crate::types::HAlign::Center,
                v_align: crate::types::VAlign::Middle,
                children: children.into_iter().map(Object::from).collect(),
            }),
        }
    }
}

impl From<SnowComponent> for Object {
    fn from(component: SnowComponent) -> Self {
        component.into_object()
    }
}

impl From<Object> for SnowNode {
    fn from(object: Object) -> Self {
        match object {
            Object::Board(board) => SnowNode::Box {
                width: 0.0,
                height: 0.0,
                children: board.children.into_iter().map(SnowNode::from).collect(),
            },
            Object::Girl(_) => SnowNode::Column { children: vec![] },
            Object::Card(card) => SnowNode::Column {
                children: card.children.into_iter().map(SnowNode::from).collect(),
            },
            Object::Row(row) => SnowNode::Row {
                children: row.children.into_iter().map(SnowNode::from).collect(),
            },
            Object::Element(crate::elements::Element::Text(text)) => SnowNode::Text {
                text: text.text.to_string(),
                id: 0,
            },
            Object::Element(crate::elements::Element::Button(button)) => SnowNode::Button {
                text: button.text.to_string(),
                id: 0,
            },
            Object::Element(crate::elements::Element::Form(form)) => SnowNode::Column {
                children: form.children.into_iter().map(SnowNode::from).collect(),
            },
            Object::Element(crate::elements::Element::TextInput(_)) => SnowNode::Text {
                text: String::new(),
                id: 0,
            },
            Object::Element(crate::elements::Element::Switch(switch_)) => SnowNode::Column {
                children: switch_.children.into_iter().map(SnowNode::from).collect(),
            },
            Object::Element(crate::elements::Element::TextClock(_)) => SnowNode::Text {
                text: String::new(),
                id: 0,
            },
            Object::DynamicText { value } => SnowNode::Text {
                text: value(),
                id: 0,
            },
        }
    }
}

impl From<SnowWorld> for World {
    fn from(world: SnowWorld) -> Self {
        Self {
            root: world.root.into(),
        }
    }
}

impl From<World> for SnowWorld {
    fn from(world: World) -> Self {
        Self::new(world.root.into())
    }
}

impl SnowNode {
    pub fn text(id: u64, text: impl Into<String>) -> Self {
        Self::Text {
            id,
            text: text.into(),
        }
    }

    pub fn button(id: u64, text: impl Into<String>) -> Self {
        Self::Button {
            id,
            text: text.into(),
        }
    }

    pub fn column(children: Vec<SnowNode>) -> Self {
        Self::Column { children }
    }

    pub fn row(children: Vec<SnowNode>) -> Self {
        Self::Row { children }
    }

    pub fn find_button(&self, target_id: u64) -> Option<&SnowNode> {
        match self {
            SnowNode::Button { id, .. } if *id == target_id => Some(self),
            SnowNode::Button { .. } => None,
            SnowNode::Row { children }
            | SnowNode::Column { children }
            | SnowNode::Box { children, .. } => children
                .iter()
                .find_map(|child| child.find_button(target_id)),
            SnowNode::Text { .. } => None,
        }
    }

    pub fn find_text(&self, target_id: u64) -> Option<&SnowNode> {
        match self {
            SnowNode::Text { id, .. } if *id == target_id => Some(self),
            SnowNode::Text { .. } => None,
            SnowNode::Row { children }
            | SnowNode::Column { children }
            | SnowNode::Box { children, .. } => {
                children.iter().find_map(|child| child.find_text(target_id))
            }
            SnowNode::Button { .. } => None,
        }
    }

    pub fn find_button_mut<'a>(&'a mut self, target_id: u64) -> Option<&'a mut SnowNode> {
        match self {
            SnowNode::Button { id, .. } if *id == target_id => Some(self),
            SnowNode::Button { .. } => None,
            SnowNode::Row { children }
            | SnowNode::Column { children }
            | SnowNode::Box { children, .. } => {
                for child in children.iter_mut() {
                    if let Some(found) = child.find_button_mut(target_id) {
                        return Some(found);
                    }
                }
                None
            }
            SnowNode::Text { .. } => None,
        }
    }

    pub fn set_text_by_id(&mut self, target_id: u64, text: impl Into<String>) {
        let value = text.into();
        if let Some(button) = self.find_button_mut(target_id) {
            if let SnowNode::Button { text: current, .. } = button {
                *current = value.clone();
            }
        }
        if let Some(target) = self.find_text_mut(target_id) {
            if let SnowNode::Text { text: current, .. } = target {
                *current = value;
            }
        }
    }

    fn find_text_mut<'a>(&'a mut self, target_id: u64) -> Option<&'a mut SnowNode> {
        match self {
            SnowNode::Text { id, .. } if *id == target_id => Some(self),
            SnowNode::Text { .. } => None,
            SnowNode::Row { children }
            | SnowNode::Column { children }
            | SnowNode::Box { children, .. } => {
                for child in children.iter_mut() {
                    if let Some(found) = child.find_text_mut(target_id) {
                        return Some(found);
                    }
                }
                None
            }
            SnowNode::Button { .. } => None,
        }
    }

    pub fn set_button_text(&mut self, target_id: u64, text: impl Into<String>) {
        self.set_text_by_id(target_id, text);
    }

    pub fn update_by_id(&mut self, target_id: u64, text: impl Into<String>) -> bool {
        let value = text.into();
        let mut updated = false;

        if let Some(button) = self.find_button_mut(target_id) {
            if let SnowNode::Button { text: current, .. } = button {
                *current = value.clone();
                updated = true;
            }
        }
        if let Some(target) = self.find_text_mut(target_id) {
            if let SnowNode::Text { text: current, .. } = target {
                *current = value;
                updated = true;
            }
        }

        updated
    }

    pub fn apply_update(&mut self, update: &SnowUpdate) {
        match update {
            SnowUpdate::SetText { id, text } | SnowUpdate::SetButtonText { id, text } => {
                let _ = self.update_by_id(*id, text.clone());
            }
        }
    }

    pub fn reconcile(&self, snapshot: &SnowNode) -> SnowNode {
        match (self, snapshot) {
            (SnowNode::Text { id: left_id, .. }, SnowNode::Text { id: right_id, .. })
                if left_id == right_id =>
            {
                self.clone()
            }
            (SnowNode::Button { id: left_id, .. }, SnowNode::Button { id: right_id, .. })
                if left_id == right_id =>
            {
                self.clone()
            }
            (
                SnowNode::Row {
                    children: left_children,
                },
                SnowNode::Row {
                    children: right_children,
                },
            ) => {
                let mut merged = Vec::with_capacity(right_children.len());
                for (left, right) in left_children.iter().zip(right_children.iter()) {
                    merged.push(left.reconcile(right));
                }
                for extra in right_children.iter().skip(left_children.len()) {
                    merged.push(extra.clone());
                }
                SnowNode::Row { children: merged }
            }
            (
                SnowNode::Column {
                    children: left_children,
                },
                SnowNode::Column {
                    children: right_children,
                },
            ) => {
                let mut merged = Vec::with_capacity(right_children.len());
                for (left, right) in left_children.iter().zip(right_children.iter()) {
                    merged.push(left.reconcile(right));
                }
                for extra in right_children.iter().skip(left_children.len()) {
                    merged.push(extra.clone());
                }
                SnowNode::Column { children: merged }
            }
            (
                SnowNode::Box {
                    width: _,
                    height: _,
                    children: left_children,
                },
                SnowNode::Box {
                    width,
                    height,
                    children: right_children,
                },
            ) => {
                let mut merged = Vec::with_capacity(right_children.len());
                for (left, right) in left_children.iter().zip(right_children.iter()) {
                    merged.push(left.reconcile(right));
                }
                for extra in right_children.iter().skip(left_children.len()) {
                    merged.push(extra.clone());
                }
                SnowNode::Box {
                    width: *width,
                    height: *height,
                    children: merged,
                }
            }
            _ => snapshot.clone(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SnowWorld {
    pub root: SnowNode,
}

impl SnowWorld {
    pub fn new(root: SnowNode) -> Self {
        Self { root }
    }

    pub fn button_ids(&self) -> Vec<u64> {
        let mut ids = Vec::new();
        self.collect_button_ids(&self.root, &mut ids, 2);
        ids
    }

    fn collect_button_ids(&self, node: &SnowNode, ids: &mut Vec<u64>, next_id: u64) {
        match node {
            SnowNode::Button { .. } => {
                ids.push(next_id);
            }
            SnowNode::Row { children }
            | SnowNode::Column { children }
            | SnowNode::Box { children, .. } => {
                let mut current = next_id;
                for child in children {
                    self.collect_button_ids(child, ids, current);
                    current += 1;
                }
            }
            SnowNode::Text { .. } => {}
        }
    }

    pub fn apply_message(&mut self, message: &SnowMessage) {
        for update in message.to_updates() {
            self.apply_update(&update);
        }
    }

    pub fn apply_update(&mut self, update: &SnowUpdate) {
        self.root.apply_update(update);
    }

    pub fn reconcile(&mut self, snapshot: &SnowWorld) {
        self.root = self.root.reconcile(&snapshot.root);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnowAction {
    ButtonClicked { button_id: u64, count: usize },
    SetText { id: u64, text: String },
    SetButtonText { id: u64, text: String },
}

impl SnowAction {
    pub fn to_update(&self) -> SnowUpdate {
        match self {
            SnowAction::ButtonClicked { button_id, count } => SnowUpdate::SetButtonText {
                id: *button_id,
                text: format!("Clicked {count} times"),
            },
            SnowAction::SetText { id, text } => SnowUpdate::SetText {
                id: *id,
                text: text.clone(),
            },
            SnowAction::SetButtonText { id, text } => SnowUpdate::SetButtonText {
                id: *id,
                text: text.clone(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnowMessage {
    ButtonClicked { button_id: u64, count: usize },
}

impl SnowMessage {
    pub fn to_updates(&self) -> Vec<SnowUpdate> {
        match self {
            SnowMessage::ButtonClicked { button_id, count } => vec![
                SnowUpdate::SetButtonText {
                    id: *button_id,
                    text: format!("Clicked {count} times"),
                },
                SnowUpdate::SetText {
                    id: 1,
                    text: format!("Snow UI + Masonry ({count})"),
                },
            ],
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SnowUpdate {
    SetText { id: u64, text: String },
    SetButtonText { id: u64, text: String },
}

#[derive(Clone, Debug, Default)]
pub struct SnowState {
    pub values: HashMap<u64, String>,
}

impl SnowState {
    pub fn set(&mut self, id: u64, value: impl Into<String>) {
        self.values.insert(id, value.into());
    }

    pub fn get(&self, id: u64, fallback: &str) -> String {
        self.values
            .get(&id)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    }

    pub fn apply_update(&mut self, update: &SnowUpdate) {
        match update {
            SnowUpdate::SetText { id, text } | SnowUpdate::SetButtonText { id, text } => {
                self.set(*id, text.clone());
            }
        }
    }

    pub fn apply_message(&mut self, message: &SnowMessage) {
        for update in message.to_updates() {
            self.apply_update(&update);
        }
    }
}

pub struct SnowRuntime {
    adapter: masonry_backend::MasonryAdapter,
    last_world: SnowWorld,
}

impl SnowRuntime {
    pub fn new() -> Self {
        Self {
            adapter: masonry_backend::MasonryAdapter::new(),
            last_world: SnowWorld::default(),
        }
    }

    pub fn mount(&mut self, world: SnowWorld) {
        self.adapter.set_world(world.clone());
        self.last_world = world;
    }

    pub fn mount_component(&mut self, component: SnowComponent) {
        self.mount(SnowWorld::new(component.into_node()));
    }

    pub fn set_root(&mut self, root: SnowNode) {
        self.mount(SnowWorld::new(root));
    }

    pub fn world(&self) -> &SnowWorld {
        self.adapter.world()
    }

    pub fn dispatch(&mut self, message: &SnowMessage) {
        self.adapter.handle_message(message);
        self.last_world = self.adapter.world().clone();
    }

    pub fn update(&mut self, message: &SnowMessage) {
        self.dispatch(message);
    }

    pub fn handle_event(&mut self, message: &SnowMessage) {
        self.update(message);
    }

    pub fn render(&mut self) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        self.adapter.render()
    }

    pub fn step(
        &mut self,
        message: &SnowMessage,
    ) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        self.update(message);
        self.render()
    }

    pub fn run(
        &mut self,
        message: &SnowMessage,
    ) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        self.step(message)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SnowView {
    pub component: SnowComponent,
    pub state: HashMap<u64, String>,
}

impl SnowView {
    pub fn new(component: SnowComponent) -> Self {
        let mut state = HashMap::new();
        Self::collect_state(&component, &mut state);
        Self { component, state }
    }

    fn collect_state(component: &SnowComponent, state: &mut HashMap<u64, String>) {
        match component {
            SnowComponent::Text { text, id } | SnowComponent::Button { text, id } => {
                state.insert(*id, text.clone());
            }
            SnowComponent::Row { children }
            | SnowComponent::Column { children }
            | SnowComponent::Box { children, .. } => {
                for child in children {
                    Self::collect_state(child, state);
                }
            }
        }
    }

    pub fn set_text(&mut self, id: u64, text: impl Into<String>) {
        self.state.insert(id, text.into());
    }

    pub fn apply_update(&mut self, update: &SnowUpdate) {
        match update {
            SnowUpdate::SetText { id, text } | SnowUpdate::SetButtonText { id, text } => {
                self.state.insert(*id, text.clone());
            }
        }
    }

    pub fn apply_message(&mut self, message: &SnowMessage) {
        for update in message.to_updates() {
            self.apply_update(&update);
        }
    }

    pub fn update_text(&mut self, id: u64, text: impl Into<String>) {
        self.apply_update(&SnowUpdate::SetText {
            id,
            text: text.into(),
        });
    }

    pub fn update_button_text(&mut self, id: u64, text: impl Into<String>) {
        self.apply_update(&SnowUpdate::SetButtonText {
            id,
            text: text.into(),
        });
    }

    pub fn bind_state_to_node(&self) -> SnowNode {
        let mut node = self.component.clone().into_node();
        for (id, text) in &self.state {
            node.set_text_by_id(*id, text.clone());
        }
        node
    }

    pub fn into_node(self) -> SnowNode {
        self.bind_state_to_node()
    }
}

pub struct SnowApp {
    runtime: SnowRuntime,
    view: Option<SnowView>,
    instance: Option<SnowComponentInstance>,
}

impl SnowApp {
    pub fn new(world: SnowWorld) -> Self {
        let mut runtime = SnowRuntime::new();
        runtime.mount(world);
        Self {
            runtime,
            view: None,
            instance: None,
        }
    }

    pub fn instance(&self) -> Option<&SnowComponentInstance> {
        self.instance.as_ref()
    }

    pub fn instance_mut(&mut self) -> Option<&mut SnowComponentInstance> {
        self.instance.as_mut()
    }

    pub fn from_root(root: SnowNode) -> Self {
        Self::with_root(root)
    }

    pub fn from_component(component: SnowComponent) -> Self {
        let view = SnowView::new(component.clone());
        let root = component.clone().into_node();
        let instance = SnowComponentInstance::new(component);
        let mut app = Self::with_root(root);
        app.view = Some(view);
        app.instance = Some(instance);
        app
    }

    pub fn from_library_world(world: World) -> Self {
        let root = world.root.clone().into();
        let mut app = Self::with_root(root);
        app.runtime.mount(world.clone().into());
        app
    }

    pub fn from_world(world: World) -> Self {
        Self::from_library_world(world)
    }

    pub fn with_world(world: World) -> Self {
        Self::from_world(world)
    }

    pub fn from_view(view: SnowView) -> Self {
        let root = view.clone().into_node();
        let instance = SnowComponentInstance::new(view.component.clone());
        let mut app = Self::with_root(root);
        app.view = Some(view);
        app.instance = Some(instance);
        app
    }

    pub fn with_root(root: SnowNode) -> Self {
        Self::new(SnowWorld::new(root))
    }

    pub fn with_component(component: SnowComponent) -> Self {
        Self::from_component(component)
    }

    pub fn with_view(view: SnowView) -> Self {
        Self::from_view(view)
    }

    pub fn mount(&mut self, world: SnowWorld) {
        self.runtime.mount(world);
        self.view = None;
        self.instance = None;
    }

    pub fn mount_component(&mut self, component: SnowComponent) {
        self.runtime.mount_component(component.clone());
        self.view = Some(SnowView::new(component.clone()));
        self.instance = Some(SnowComponentInstance::new(component));
    }

    pub fn mount_library_world(&mut self, world: World) {
        self.runtime.mount(world.clone().into());
        self.view = None;
        self.instance = None;
    }

    pub fn set_root(&mut self, root: SnowNode) {
        self.runtime.set_root(root);
        self.view = None;
        self.instance = None;
    }

    pub fn world(&self) -> &SnowWorld {
        self.runtime.world()
    }

    pub fn update(&mut self, message: &SnowMessage) {
        if let Some(view) = self.view.as_mut() {
            view.apply_message(message);
            if let Some(instance) = self.instance.as_mut() {
                instance.component = view.component.clone();
                instance.state = view.state.clone();
            }
            self.runtime.set_root(view.clone().into_node());
        }
        if let Some(instance) = self.instance.as_mut() {
            for update in message.to_updates() {
                match update {
                    SnowUpdate::SetText { id, text } | SnowUpdate::SetButtonText { id, text } => {
                        instance.update_text(id, text);
                    }
                }
            }
        }
        self.runtime.update(message);
    }

    pub fn dispatch_action(&mut self, action: &SnowAction) {
        let update = action.to_update();

        if let Some(view) = self.view.as_mut() {
            view.apply_update(&update);
            if let Some(instance) = self.instance.as_mut() {
                instance.component = view.component.clone();
                instance.state = view.state.clone();
            }
            self.runtime.set_root(view.clone().into_node());
        }

        if let Some(instance) = self.instance.as_mut() {
            match &update {
                SnowUpdate::SetText { id, text } | SnowUpdate::SetButtonText { id, text } => {
                    instance.update_text(*id, text.clone());
                }
            }
        }

        self.runtime.adapter.apply_update(&update);
    }

    pub fn update_view(&mut self, update: &SnowUpdate) {
        if let Some(view) = self.view.as_mut() {
            view.apply_update(update);
            if let Some(instance) = self.instance.as_mut() {
                instance.component = view.component.clone();
                instance.state = view.state.clone();
            }
            self.runtime.set_root(view.clone().into_node());
        }
    }

    pub fn handle_event(&mut self, message: &SnowMessage) {
        self.update(message);
    }

    pub fn render(&mut self) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        self.runtime.render()
    }

    pub fn render_library_world(&mut self) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        let world: World = self.runtime.world().clone().into();
        self.runtime.adapter.render_library_world(&world)
    }

    pub fn step(
        &mut self,
        message: &SnowMessage,
    ) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        self.runtime.step(message)
    }

    pub fn run(
        &mut self,
        message: &SnowMessage,
    ) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        self.runtime.run(message)
    }

    pub fn launch(
        &mut self,
        message: &SnowMessage,
    ) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        self.run(message)
    }

    pub fn start(
        &mut self,
        message: &SnowMessage,
    ) -> masonry::core::NewWidget<masonry::widgets::Flex> {
        self.run(message)
    }
}

pub mod masonry_backend {
    use super::{SnowMessage, SnowNode, SnowState, SnowUpdate, SnowWorld};
    use masonry::core::{ErasedAction, NewWidget};
    use masonry::widgets::{Button, ButtonPress, Flex, Label};

    pub struct MasonryAdapter {
        state: SnowState,
        world: SnowWorld,
    }

    impl MasonryAdapter {
        pub fn state(&self) -> &SnowState {
            &self.state
        }

        pub fn new() -> Self {
            Self {
                state: SnowState::default(),
                world: SnowWorld::default(),
            }
        }

        pub fn set_world(&mut self, world: SnowWorld) {
            self.world = world;
        }

        pub fn world(&self) -> &SnowWorld {
            &self.world
        }

        pub fn set_text(&mut self, id: u64, text: impl Into<String>) {
            self.state.set(id, text);
            self.world.root.set_button_text(id, self.state.get(id, ""));
        }

        pub fn sync_state_from_world(&mut self) {
            self.state.values.clear();
            self.state
                .values
                .extend(self.collect_text_values(&self.world.root));
        }

        pub fn apply_update(&mut self, update: &SnowUpdate) {
            self.world.apply_update(update);
            self.sync_state_from_world();
        }

        fn collect_text_values(&self, node: &SnowNode) -> Vec<(u64, String)> {
            let mut values = Vec::new();

            match node {
                SnowNode::Text { text, id } => values.push((*id, text.clone())),
                SnowNode::Button { text, id } => values.push((*id, text.clone())),
                SnowNode::Row { children }
                | SnowNode::Column { children }
                | SnowNode::Box { children, .. } => {
                    for child in children {
                        values.extend(self.collect_text_values(child));
                    }
                }
            }

            values
        }

        pub fn build_world(&self, world: &SnowWorld) -> NewWidget<Flex> {
            NewWidget::new(self.build_node(&world.root))
        }

        pub fn handle_message(&mut self, message: &SnowMessage) {
            for update in message.to_updates() {
                self.world.apply_update(&update);
                self.state.apply_update(&update);
            }
            self.sync_state_from_world();
        }

        pub fn render(&mut self) -> NewWidget<Flex> {
            self.sync_state_from_world();
            self.build_world(&self.world)
        }

        pub fn render_library_world(&mut self, world: &crate::object::World) -> NewWidget<Flex> {
            world.into_masonry_widget()
        }

        pub fn apply_message_and_rebuild(
            &mut self,
            world: &SnowWorld,
            message: &SnowMessage,
        ) -> NewWidget<Flex> {
            let previous = self.world.clone();
            self.world.reconcile(world);
            self.handle_message(message);
            let changed = self.world.root.reconcile(&previous.root);
            self.world.root = changed;
            self.render()
        }

        fn build_node(&self, node: &SnowNode) -> Flex {
            match node {
                SnowNode::Text { text, id } => {
                    let display = self.state.get(*id, text);
                    let mut column = Flex::column();
                    column = column.with_fixed(NewWidget::new(Label::new(display.as_str())));
                    column
                }
                SnowNode::Button { text, id } => {
                    let display = self.state.get(*id, text);
                    let mut column = Flex::column();
                    column = column.with_fixed(NewWidget::new(Button::with_text(display.as_str())));
                    column
                }
                SnowNode::Row { children } => {
                    let mut row = Flex::row();
                    for child in children {
                        let child_widget = NewWidget::new(self.build_node(child));
                        row = row.with_fixed(child_widget);
                    }
                    row
                }
                SnowNode::Column { children } => {
                    let mut column = Flex::column();
                    for child in children {
                        let child_widget = NewWidget::new(self.build_node(child));
                        column = column.with_fixed(child_widget);
                    }
                    column
                }
                SnowNode::Box { children, .. } => {
                    let mut box_widget = Flex::column();
                    for child in children {
                        let child_widget = NewWidget::new(self.build_node(child));
                        box_widget = box_widget.with_fixed(child_widget);
                    }
                    box_widget
                }
            }
        }

        pub fn handle_button_click(&mut self, button_id: u64) -> SnowMessage {
            let current = self
                .state
                .values
                .get(&button_id)
                .and_then(|v| v.parse::<usize>().ok())
                .unwrap_or(0);
            let next = current + 1;
            self.state.set(button_id, next.to_string());
            SnowMessage::ButtonClicked {
                button_id,
                count: next,
            }
        }

        pub fn apply_message(&mut self, message: &SnowMessage) {
            for update in message.to_updates() {
                self.world.apply_update(&update);
                self.state.apply_update(&update);
            }
            self.sync_state_from_world();
        }

        pub fn dispatch_action(&mut self, action: &ErasedAction) -> Option<SnowMessage> {
            if action.is::<ButtonPress>() {
                let target_button = self.world.button_ids().into_iter().next().unwrap_or(2);
                Some(self.handle_button_click(target_button))
            } else {
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::masonry_backend::MasonryAdapter;
    use super::{SnowMessage, SnowNode, SnowUpdate, SnowWorld};

    #[test]
    fn button_click_updates_live_world_text() {
        let mut adapter = MasonryAdapter::new();
        let initial = SnowWorld {
            root: SnowNode::Column {
                children: vec![
                    SnowNode::Text {
                        text: "Snow UI + Masonry".to_string(),
                        id: 1,
                    },
                    SnowNode::Button {
                        text: "Clicked 0 times".to_string(),
                        id: 2,
                    },
                ],
            },
        };
        adapter.set_world(initial.clone());

        let next = SnowWorld {
            root: SnowNode::Column {
                children: vec![
                    SnowNode::Text {
                        text: "Snow UI + Masonry".to_string(),
                        id: 1,
                    },
                    SnowNode::Button {
                        text: "Clicked 0 times".to_string(),
                        id: 2,
                    },
                ],
            },
        };

        let _ = adapter.apply_message_and_rebuild(
            &next,
            &SnowMessage::ButtonClicked {
                button_id: 2,
                count: 1,
            },
        );

        let button = adapter.world().root.find_button(2);
        assert!(button.is_some());
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "Clicked 1 times"));
    }

    #[test]
    fn adapter_syncs_state_from_world() {
        let mut adapter = MasonryAdapter::new();
        adapter.set_world(SnowWorld {
            root: SnowNode::Column {
                children: vec![
                    SnowNode::Text {
                        text: "Snow UI + Masonry".to_string(),
                        id: 1,
                    },
                    SnowNode::Button {
                        text: "Clicked 7 times".to_string(),
                        id: 2,
                    },
                ],
            },
        });

        adapter.sync_state_from_world();

        assert_eq!(adapter.state().get(1, ""), "Snow UI + Masonry");
        assert_eq!(adapter.state().get(2, ""), "Clicked 7 times");
    }

    #[test]
    fn update_api_changes_node_text_by_id() {
        let mut root = SnowNode::Column {
            children: vec![
                SnowNode::Text {
                    text: "before".to_string(),
                    id: 5,
                },
                SnowNode::Button {
                    text: "old".to_string(),
                    id: 6,
                },
            ],
        };

        root.apply_update(&SnowUpdate::SetText {
            id: 5,
            text: "after".to_string(),
        });
        root.apply_update(&SnowUpdate::SetButtonText {
            id: 6,
            text: "new".to_string(),
        });

        assert!(
            matches!(root.find_button(6), Some(SnowNode::Button { text, .. }) if text == "new")
        );
        assert!(matches!(root.find_text(5), Some(SnowNode::Text { text, .. }) if text == "after"));
    }

    #[test]
    fn button_click_message_creates_expected_updates() {
        let updates = SnowMessage::ButtonClicked {
            button_id: 2,
            count: 3,
        }
        .to_updates();

        assert_eq!(updates.len(), 2);
        assert!(matches!(
            updates[0],
            SnowUpdate::SetButtonText { id: 2, ref text } if text == "Clicked 3 times"
        ));
        assert!(matches!(
            updates[1],
            SnowUpdate::SetText { id: 1, ref text } if text == "Snow UI + Masonry (3)"
        ));
    }

    #[test]
    fn runtime_mount_and_dispatch_update_world() {
        let mut runtime = super::SnowRuntime::new();
        let world = SnowWorld {
            root: SnowNode::Column {
                children: vec![
                    SnowNode::Text {
                        text: "Snow UI + Masonry".to_string(),
                        id: 1,
                    },
                    SnowNode::Button {
                        text: "Clicked 0 times".to_string(),
                        id: 2,
                    },
                ],
            },
        };

        runtime.mount(world.clone());
        runtime.dispatch(&SnowMessage::ButtonClicked {
            button_id: 2,
            count: 1,
        });

        let button = runtime.last_world.root.find_button(2);
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "Clicked 1 times"));
    }

    #[test]
    fn runtime_step_handles_message_and_renders() {
        let mut runtime = super::SnowRuntime::new();
        runtime.mount(SnowWorld {
            root: SnowNode::Column {
                children: vec![
                    SnowNode::Text {
                        text: "Snow UI + Masonry".to_string(),
                        id: 1,
                    },
                    SnowNode::Button {
                        text: "Clicked 0 times".to_string(),
                        id: 2,
                    },
                ],
            },
        });

        let _ = runtime.step(&SnowMessage::ButtonClicked {
            button_id: 2,
            count: 2,
        });

        let button = runtime.last_world.root.find_button(2);
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "Clicked 2 times"));
    }

    #[test]
    fn app_wrapper_steps_world_and_updates_button() {
        let mut app = super::SnowApp::new(SnowWorld {
            root: SnowNode::Column {
                children: vec![
                    SnowNode::Text {
                        text: "Snow UI + Masonry".to_string(),
                        id: 1,
                    },
                    SnowNode::Button {
                        text: "Clicked 0 times".to_string(),
                        id: 2,
                    },
                ],
            },
        });

        let _ = app.step(&SnowMessage::ButtonClicked {
            button_id: 2,
            count: 4,
        });

        let button = app.world().root.find_button(2);
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "Clicked 4 times"));
    }

    #[test]
    fn app_run_updates_and_renders_in_one_call() {
        let mut app = super::SnowApp::new(SnowWorld {
            root: SnowNode::Column {
                children: vec![
                    SnowNode::Text {
                        text: "Snow UI + Masonry".to_string(),
                        id: 1,
                    },
                    SnowNode::Button {
                        text: "Clicked 0 times".to_string(),
                        id: 2,
                    },
                ],
            },
        });

        let _ = app.run(&SnowMessage::ButtonClicked {
            button_id: 2,
            count: 5,
        });

        let button = app.world().root.find_button(2);
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "Clicked 5 times"));
    }

    #[test]
    fn component_constructors_and_world_builder_work() {
        let root = SnowNode::column(vec![
            SnowNode::text(1, "hello"),
            SnowNode::button(2, "press"),
        ]);

        let world = SnowWorld::new(root.clone());
        let app = super::SnowApp::with_root(root.clone());

        assert!(matches!(world.root, SnowNode::Column { .. }));
        assert!(matches!(app.world().root, SnowNode::Column { .. }));
    }

    #[test]
    fn app_mount_and_root_replacement_keep_runtime_in_sync() {
        let mut app = super::SnowApp::new(SnowWorld::new(SnowNode::text(1, "before")));
        app.mount(SnowWorld::new(SnowNode::button(2, "after")));
        app.set_root(SnowNode::button(3, "final"));

        let button = app.world().root.find_button(3);
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "final"));
    }

    #[test]
    fn component_api_builds_a_root_tree() {
        let component = super::SnowComponent::column(vec![
            super::SnowComponent::text(1, "hello"),
            super::SnowComponent::button(2, "press"),
        ]);

        let root = component.clone().into_node();
        let app = super::SnowApp::with_component(component);

        assert!(matches!(root, SnowNode::Column { .. }));
        assert!(matches!(app.world().root, SnowNode::Column { .. }));
    }

    #[test]
    fn stateful_view_tracks_component_state_and_mounts_into_app() {
        let mut view = super::SnowView::new(super::SnowComponent::column(vec![
            super::SnowComponent::text(1, "hello"),
            super::SnowComponent::button(2, "press"),
        ]));

        view.set_text(1, "hello again");
        let app = super::SnowApp::with_view(view.clone());

        assert_eq!(view.state.get(&1).unwrap(), "hello again");
        assert!(matches!(app.world().root, SnowNode::Column { .. }));
    }

    #[test]
    fn app_lifecycle_supports_mount_component_and_handle_event() {
        let mut app = super::SnowApp::new(super::SnowWorld::new(super::SnowNode::column(vec![
            super::SnowNode::text(1, "before"),
            super::SnowNode::button(2, "Clicked 0 times"),
        ])));

        app.mount_component(super::SnowComponent::column(vec![
            super::SnowComponent::text(1, "Snow UI + Masonry"),
            super::SnowComponent::button(2, "Clicked 0 times"),
        ]));
        app.handle_event(&super::SnowMessage::ButtonClicked {
            button_id: 2,
            count: 7,
        });

        let button = app.world().root.find_button(2);
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "Clicked 7 times"));
    }

    #[test]
    fn view_apply_message_updates_its_bound_component() {
        let mut view = super::SnowView::new(super::SnowComponent::column(vec![
            super::SnowComponent::text(1, "before"),
            super::SnowComponent::button(2, "Clicked 0 times"),
        ]));

        view.apply_message(&super::SnowMessage::ButtonClicked {
            button_id: 2,
            count: 3,
        });

        assert_eq!(view.state.get(&2), Some(&"Clicked 3 times".to_string()));
        assert!(matches!(
            view.component.into_node(),
            SnowNode::Column { .. }
        ));
    }

    #[test]
    fn app_public_constructors_and_view_mount_are_stable() {
        let view = super::SnowView::new(super::SnowComponent::column(vec![
            super::SnowComponent::text(1, "hello"),
            super::SnowComponent::button(2, "view"),
        ]));

        let from_root = super::SnowApp::from_root(super::SnowNode::text(3, "root"));
        let from_component =
            super::SnowApp::from_component(super::SnowComponent::text(4, "component"));
        let from_view = super::SnowApp::from_view(view);

        assert!(matches!(from_root.world().root, SnowNode::Text { .. }));
        assert!(matches!(from_component.world().root, SnowNode::Text { .. }));
        assert!(matches!(from_view.world().root, SnowNode::Column { .. }));
    }

    #[test]
    fn reducer_style_updates_are_supported_by_view_and_app() {
        let mut view = super::SnowView::new(super::SnowComponent::button(2, "before"));
        view.update_button_text(2, "after");

        let mut app = super::SnowApp::from_view(view.clone());
        app.update_view(&super::SnowUpdate::SetButtonText {
            id: 2,
            text: "final".to_string(),
        });

        assert_eq!(view.state.get(&2), Some(&"after".to_string()));
        assert!(
            matches!(app.world().root.find_button(2), Some(SnowNode::Button { text, .. }) if text == "final")
        );
    }

    #[test]
    fn action_layer_converts_to_message_and_dispatches() {
        let mut app = super::SnowApp::from_component(super::SnowComponent::button(2, "before"));
        app.dispatch_action(&super::SnowAction::ButtonClicked {
            button_id: 2,
            count: 8,
        });

        let button = app.world().root.find_button(2);
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "Clicked 8 times"));
    }

    #[test]
    fn app_launch_and_start_methods_follow_the_same_lifecycle() {
        let mut app = super::SnowApp::from_component(super::SnowComponent::button(2, "before"));
        let _ = app.launch(&super::SnowMessage::ButtonClicked {
            button_id: 2,
            count: 9,
        });
        let _ = app.start(&super::SnowMessage::ButtonClicked {
            button_id: 2,
            count: 10,
        });

        let button = app.world().root.find_button(2);
        assert!(
            matches!(button, Some(SnowNode::Button { text, .. }) if text == "Clicked 10 times")
        );
    }

    #[test]
    fn state_binding_syncs_view_state_back_into_the_root_tree() {
        let mut view = super::SnowView::new(super::SnowComponent::button(2, "before"));
        view.update_button_text(2, "after");

        let root = view.bind_state_to_node();
        let button = root.find_button(2);
        assert!(matches!(button, Some(SnowNode::Button { text, .. }) if text == "after"));
    }

    #[test]
    fn component_roundtrips_into_library_object_and_world() {
        let component = super::SnowComponent::column(vec![
            super::SnowComponent::text(1, "hello"),
            super::SnowComponent::button(2, "press"),
        ]);

        let object: crate::object::Object = component.clone().into();
        let world: crate::object::World = super::SnowWorld::new(component.into_node()).into();

        assert!(matches!(object, crate::object::Object::Row(_)));
        assert!(matches!(world.root, crate::object::Object::Row(_)));
    }

    #[test]
    fn component_into_world_matches_the_library_world_bridge() {
        let component = super::SnowComponent::column(vec![
            super::SnowComponent::text(1, "hello"),
            super::SnowComponent::button(2, "press"),
        ]);

        let world = component.into_world();
        assert!(matches!(world.root, crate::object::Object::Row(_)));
    }

    #[test]
    fn component_instance_owns_state_and_converts_to_world() {
        let mut instance =
            super::SnowComponentInstance::new(super::SnowComponent::button(2, "before"));
        instance.update_text(2, "after");

        let world = instance.into_world();
        let button: crate::object::Object = world.root.clone().into();
        let _ = button;

        assert!(matches!(
            world.root,
            crate::object::Object::Element(crate::elements::Element::Button(_))
        ));
    }

    #[test]
    fn library_world_applies_runtime_message_to_button_text() {
        let mut world = crate::object::World {
            root: crate::object::Object::Element(crate::elements::Element::Button(
                crate::elements::Button { text: "before" },
            )),
        };

        world.apply_message(&super::SnowMessage::ButtonClicked {
            button_id: 0,
            count: 3,
        });

        match &world.root {
            crate::object::Object::Element(crate::elements::Element::Button(button)) => {
                assert_eq!(button.text, "Clicked 3 times");
            }
            other => panic!("unexpected object shape: {other:?}"),
        }
    }

    #[test]
    fn app_exposes_active_component_instance_state() {
        let mut app = super::SnowApp::from_component(super::SnowComponent::button(2, "before"));
        app.update(&super::SnowMessage::ButtonClicked {
            button_id: 2,
            count: 6,
        });

        assert!(app.instance().is_some());
        assert_eq!(
            app.instance().unwrap().state.get(&2),
            Some(&"Clicked 6 times".to_string())
        );
    }

    #[test]
    fn app_can_be_built_from_real_library_world() {
        let world = crate::object::World {
            root: crate::object::Object::Row(crate::layout::Row {
                children: vec![crate::object::Object::Element(
                    crate::elements::Element::Button(crate::elements::Button { text: "hello" }),
                )],
            }),
        };

        let app = super::SnowApp::from_world(world.clone());
        let app2 = super::SnowApp::with_world(world);

        assert!(matches!(app.world().root, SnowNode::Row { .. }));
        assert!(matches!(app2.world().root, SnowNode::Row { .. }));
    }
}
