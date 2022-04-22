use crate::{Handler, Session};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use walle_core::app::StandardArcBot;
use walle_core::{
    EventContent, EventHandler, MessageContent, MetaContent, NoticeContent, RequestContent, Resps,
    StandardAction, StandardEvent,
};

pub(crate) type TempPlugins = Arc<Mutex<HashMap<String, Plugin<EventContent>>>>;

#[derive(Default)]
pub struct Plugins {
    pub message: Vec<Plugin<MessageContent>>,
    pub notice: Vec<Plugin<NoticeContent>>,
    pub request: Vec<Plugin<RequestContent>>,
    pub meta: Vec<Plugin<MetaContent>>,
    temp: TempPlugins,
}

impl Plugins {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn add_message_plugin(mut self, plugin: Plugin<MessageContent>) -> Self {
        self.message.push(plugin);
        self
    }
    pub fn add_notice_plugin(mut self, plugin: Plugin<NoticeContent>) -> Self {
        self.notice.push(plugin);
        self
    }
    pub fn add_request_plugin(mut self, plugin: Plugin<RequestContent>) -> Self {
        self.request.push(plugin);
        self
    }
    pub fn add_meta_plugin(mut self, plugin: Plugin<MetaContent>) -> Self {
        self.meta.push(plugin);
        self
    }
}

#[async_trait]
impl EventHandler<StandardEvent, StandardAction, Resps> for Plugins {
    async fn handle(&self, bot: StandardArcBot, event: StandardEvent) {
        let session = Session::new(bot, event, self.temp.clone());
        if let Some(p) = {
            let mut temp_plugins = self.temp.lock().await;
            let mut found: Option<String> = None;
            for (k, plugin) in temp_plugins.iter() {
                if plugin.matcher._match(&session) {
                    found = Some(k.clone());
                    break;
                }
            }
            found.and_then(|i| temp_plugins.remove(&i))
        } {
            p.matcher.handle(session).await;
            return;
        }
        let (bot, event) = (session.bot, session.event);
        if let Ok(event) = event.try_into() {
            let session = Session::new(bot, event, self.temp.clone());
            for plugin in &self.message {
                plugin.handle(&session).await;
            }
        }
    }
}

#[derive(Clone)]
pub struct Plugin<C> {
    pub name: String,
    pub description: String,
    pub matcher: Arc<dyn Handler<C> + Sync + Send + 'static>,
}

impl<C> Plugin<C>
where
    C: Clone + Send + Sync + 'static,
{
    pub fn new<T0: ToString, T1: ToString>(
        name: T0,
        description: T1,
        matcher: impl Handler<C> + Sync + Send + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            matcher: Arc::new(matcher),
        }
    }

    pub async fn handle(&self, session: &Session<C>) {
        if self.matcher._match(&session) {
            let matcher = self.matcher.clone();
            let mut session = session.clone();
            tokio::spawn(async move {
                matcher._pre_handle(&mut session);
                matcher.handle(session).await
            });
        }
    }
}
