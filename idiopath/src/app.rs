// Copyright 2022 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use druid_shell::{kurbo::Point, piet::Piet, WindowHandle};

use crate::{
    event::Event,
    id::Id,
    view::{Cx, View},
    widget::{RawEvent, Widget},
};

pub struct App<T, V: View<T>, F: FnMut(&mut T) -> V>
where
    V::Element: Widget,
{
    data: T,
    app_logic: F,
    view: Option<V>,
    id: Option<Id>,
    state: Option<V::State>,
    element: Option<V::Element>,
    events: Vec<Event>,
    cx: Cx,
}

impl<T, V: View<T>, F: FnMut(&mut T) -> V> App<T, V, F>
where
    V::Element: Widget,
{
    pub fn new(data: T, app_logic: F) -> Self {
        let cx = Cx::new();
        App {
            data,
            app_logic,
            view: None,
            id: None,
            state: None,
            element: None,
            events: Vec::new(),
            cx,
        }
    }

    pub fn ensure_app(&mut self) {
        if self.view.is_none() {
            let view = (self.app_logic)(&mut self.data);
            let (id, state, element) = view.build(&mut self.cx);
            self.view = Some(view);
            self.id = Some(id);
            self.state = Some(state);
            self.element = Some(element);
        }
    }

    pub fn connect(&mut self, _window_handle: WindowHandle) {
        // This will be needed for wiring up async but is a stub for now.
        //self.cx.set_handle(window_handle.get_idle_handle());
    }

    pub fn paint(&mut self, piet: &mut Piet) {
        self.ensure_app();
        let element = self.element.as_mut().unwrap();
        element.layout();
        element.paint(piet, Point::ZERO);
    }

    pub fn mouse_down(&mut self, point: Point) {
        self.event(RawEvent::MouseDown(point));
    }

    fn event(&mut self, event: RawEvent) {
        self.ensure_app();
        let element = self.element.as_mut().unwrap();
        element.event(&event, &mut self.events);
        self.run_app_logic();
    }

    pub fn run_app_logic(&mut self) {
        for event in self.events.drain(..) {
            let id_path = &event.id_path[1..];
            self.view.as_ref().unwrap().event(
                id_path,
                self.state.as_mut().unwrap(),
                event.body,
                &mut self.data,
            );
        }
        // Re-rendering should be more lazy.
        let view = (self.app_logic)(&mut self.data);
        view.rebuild(
            &mut self.cx,
            self.view.as_ref().unwrap(),
            self.id.as_mut().unwrap(),
            self.state.as_mut().unwrap(),
            self.element.as_mut().unwrap(),
        );
        assert!(self.cx.is_empty(), "id path imbalance on rebuild");
        self.view = Some(view);
    }
}