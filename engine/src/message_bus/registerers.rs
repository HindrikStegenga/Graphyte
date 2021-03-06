use super::*;
use crate::engine_stages::{RenderStageMessageContext, UpdateStageMessageContext};
use crossbeam::channel::*;
use std::marker::PhantomData;

pub struct AnyMessageRegisterer<'a> {
    builder: &'a mut MessageBusBuilder,
    handler_type: MessageHandlerType,
}

impl<'a> AnyMessageRegisterer<'a> {
    pub fn new(builder: &'a mut MessageBusBuilder, handler_type: MessageHandlerType) -> Self {
        Self {
            builder,
            handler_type,
        }
    }

    pub fn register<M: Message>(&mut self) -> Receiver<M> {
        self.builder.add_update_handler::<M>(self.handler_type)
    }
}

pub struct RenderMessageRegisterer<'a, T: 'static> {
    registerer: AnyMessageRegisterer<'a>,
    receivers: &'a mut Vec<Box<dyn AnyRenderMessageReceiver<T>>>,
    _phantom: PhantomData<fn(T)>,
}

impl<'a, T: 'static> RenderMessageRegisterer<'a, T> {
    pub fn new(
        registerer: AnyMessageRegisterer<'a>,
        receivers: &'a mut Vec<Box<dyn AnyRenderMessageReceiver<T>>>,
    ) -> Self {
        Self {
            registerer,
            receivers,
            _phantom: Default::default(),
        }
    }

    pub fn register<'c, M: Message>(&mut self)
    where
        T: for<'b> MessageHandler<RenderStageMessageContext<'b>, M>,
    {
        let receiver = self.registerer.register::<M>();
        self.receivers
            .push(Box::new(MessageReceiver::<_, M, T>::new(receiver)));
    }
}

pub struct UpdateMessageRegisterer<'a, T: 'static> {
    registerer: AnyMessageRegisterer<'a>,
    receivers: &'a mut Vec<Box<dyn AnyUpdateMessageReceiver<T>>>,
    _phantom: PhantomData<fn(T)>,
}

impl<'a, T: 'static> UpdateMessageRegisterer<'a, T> {
    pub fn new(
        registerer: AnyMessageRegisterer<'a>,
        receivers: &'a mut Vec<Box<dyn AnyUpdateMessageReceiver<T>>>,
    ) -> Self {
        Self {
            registerer,
            receivers,
            _phantom: Default::default(),
        }
    }

    pub fn register<M: Message>(&mut self)
    where
        T: for<'b> MessageHandler<UpdateStageMessageContext<'b>, M>,
    {
        let receiver = self.registerer.register::<M>();
        self.receivers
            .push(Box::new(MessageReceiver::<_, M, T>::new(receiver)));
    }
}
