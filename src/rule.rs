use crate::{Handler, Session};
use std::future::Future;
use std::pin::Pin;

pub trait Rule<C> {
    fn rule(&self, session: &Session<C>) -> bool;
    fn layer<H>(self, handler: H) -> LayeredRule<Self, H>
    where
        Self: Sized,
        H: Handler<C>,
    {
        LayeredRule {
            rule: self,
            handler,
        }
    }
}

pub struct LayeredRule<R, H> {
    pub rule: R,
    pub handler: H,
}

impl<R, H, C> Handler<C> for LayeredRule<R, H>
where
    R: Rule<C> + Sync,
    H: Handler<C> + Sync,
    C: 'static,
{
    fn _match(&self, session: &Session<C>) -> bool {
        if self.rule.rule(session) {
            self.handler._match(session)
        } else {
            false
        }
    }
    fn _pre_handle(&self, session: &mut Session<C>) {
        self.handler._pre_handle(session)
    }
    fn handle<'a, 't>(
        &'a self,
        session: Session<C>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.handler.handle(session)
    }
}

pub struct RuleFn<I>(I);

impl<I, C> Rule<C> for RuleFn<I>
where
    I: Fn(&Session<C>) -> bool,
{
    fn rule(&self, session: &Session<C>) -> bool {
        self.0(session)
    }
}

pub fn rule_fn<I, C>(rule: I) -> RuleFn<I>
where
    I: Fn(&Session<C>) -> bool,
{
    RuleFn(rule)
}
