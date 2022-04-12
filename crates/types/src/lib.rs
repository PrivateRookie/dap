use serde::{Deserialize, Serialize};
mod protocol;
pub use protocol::*;

pub trait FromReq: Sized + Serialize {
    const COMMAND: &'static str;
    type Ret;

    fn from_req(req: Request) -> OneOf<Result<(i64, Self), serde_json::Error>, Request>;

    fn into_req(self, seq: i64) -> Request {
        Request {
            arguments: Some(serde_json::to_value(self).unwrap()),
            command: Self::COMMAND.to_string(),
            seq,
            type_: "request".to_string(),
        }
    }

    fn can_cast(req: &Request) -> bool {
        Self::COMMAND == req.command
    }
}

#[macro_export]
macro_rules! impl_req {
    ($type:ty, $method:literal, $ret:path) => {
        impl $crate::FromReq for $type {
            const COMMAND: &'static str = $method;
            type Ret = $ret;

            fn from_req(
                req: $crate::Request,
            ) -> $crate::OneOf<Result<(i64, Self), serde_json::Error>, $crate::Request> {
                use $crate::{OneOf, Request};
                if <Self as $crate::FromReq>::can_cast(&req) {
                    let Request { seq, arguments, .. } = req;
                    OneOf::This(
                        serde_json::from_value(
                            arguments.unwrap_or_else(|| serde_json::Value::Null),
                        )
                        .map(|params| (seq, params)),
                    )
                } else {
                    OneOf::Other(req)
                }
            }
        }

        impl $type {
            /// helper function for user do not need to remember
            /// result type of a request
            pub fn ret(result: $ret) -> $ret {
                result
            }
        }
    };
    ($type:ty, $method:literal) => {
        impl_req!($type, $method, serde_json::Value);
    };
}

impl_req!(AttachRequestArguments, "attach");
impl_req!(CompletionsArguments, "completions", CompletionsResponseBody);
impl_req!(ConfigurationDoneArguments, "configurationDone");
impl_req!(ContinueArguments, "continue", ContinueResponseBody);
impl_req!(
    DataBreakpointInfoArguments,
    "dataBreakpointInfo",
    DataBreakpointInfoResponseBody
);
impl_req!(DisconnectArguments, "disconnect");
impl_req!(EvaluateArguments, "evaluate", EvaluateResponseBody);
impl_req!(
    ExceptionInfoArguments,
    "exceptionInfo",
    ExceptionInfoResponseBody
);
impl_req!(GotoArguments, "goto");
impl_req!(GotoTargetsArguments, "gotoTargets", GotoTargetsResponseBody);
impl_req!(InitializeRequestArguments, "initialize", Capabilities);
impl_req!(LaunchRequestArguments, "launch");
impl_req!(
    LoadedSourcesArguments,
    "loadedSources",
    LoadedSourcesResponseBody
);
impl_req!(ModulesArguments, "modules", ModulesResponseBody);
impl_req!(NextArguments, "next");
impl_req!(PauseArguments, "pause");
impl_req!(RestartArguments, "restart");
impl_req!(RestartFrameArguments, "restartFrame");
impl_req!(ReverseContinueArguments, "reverseContinue");
impl_req!(ScopesArguments, "scopes", ScopesResponseBody);
impl_req!(
    SetBreakpointsArguments,
    "setBreakpoints",
    SetBreakpointsResponseBody
);
impl_req!(
    SetDataBreakpointsArguments,
    "setDataBreakpoints",
    SetDataBreakpointsResponseBody
);
impl_req!(SetExceptionBreakpointsArguments, "setExceptionBreakpoints");
impl_req!(
    SetExpressionArguments,
    "setExpression",
    SetExpressionResponseBody
);
impl_req!(
    SetFunctionBreakpointsArguments,
    "setFunctionBreakpoints",
    SetFunctionBreakpointsResponseBody
);
impl_req!(SetVariableArguments, "setVariable", SetVariableResponseBody);
impl_req!(SourceArguments, "source", SourceResponseBody);
impl_req!(StackTraceArguments, "stackTrace", StackTraceResponseBody);
impl_req!(StepBackArguments, "stepBack");
impl_req!(StepInArguments, "stepIn");
impl_req!(
    StepInTargetsArguments,
    "stepInTargets",
    StepInTargetsResponseBody
);
impl_req!(StepOutArguments, "stepOut");
impl_req!(TerminateArguments, "terminate");
impl_req!(TerminateThreadsArguments, "terminateThreads");
impl_req!(ThreadsRequestArguments, "threads", ThreadsResponseBody);
impl_req!(VariablesArguments, "variables", VariablesResponseBody);

pub trait FromEvent: Sized + Serialize {
    const EVENT: &'static str;

    fn from_event(event: Event) -> OneOf<Result<(i64, Self), serde_json::Error>, Event>;

    fn into_event(self, seq: i64) -> Event {
        Event {
            body: Some(serde_json::to_value(self).unwrap()),
            event: Self::EVENT.to_string(),
            seq,
            type_: "event".to_string(),
        }
    }

    fn can_cast(event: &Event) -> bool {
        Self::EVENT == event.event
    }
}

#[macro_export]
macro_rules! impl_evt {
    ($type:ty, $method:literal) => {
        impl $crate::FromEvent for $type {
            const EVENT: &'static str = $method;

            fn from_event(
                req: $crate::Event,
            ) -> $crate::OneOf<Result<(i64, Self), serde_json::Error>, $crate::Event> {
                use $crate::{Event, OneOf};
                if <Self as $crate::FromEvent>::can_cast(&req) {
                    let Event { seq, body, .. } = req;
                    OneOf::This(
                        serde_json::from_value(body.unwrap_or_else(|| serde_json::Value::Null))
                            .map(|params| (seq, params)),
                    )
                } else {
                    OneOf::Other(req)
                }
            }
        }
    };
}

impl_evt!(BreakpointEventBody, "breakpoint");
impl_evt!(CapabilitiesEventBody, "capabilities");
impl_evt!(ContinuedEventBody, "continued");
impl_evt!(ExitedEventBody, "exited");
impl_evt!(InitializedEventBody, "initialized");
// impl_evt!(Invalid, "invalidated");
impl_evt!(LoadedSourceEventBody, "loadedSource");
// impl_evt!(MemoryEventBody, "memory");
impl_req!(ModuleEventBody, "module");
impl_req!(OutputEventBody, "output");
impl_req!(ProcessEventBody, "process");
// impl_req!(ProcessEndEventBody, "progressEnd");
// impl_req!(ProcessStartEventBody, "progressStart");
// impl_req!(ProcessUpdateEventBody, "progressUpdate");
impl_req!(StoppedEventBody, "stopped");
impl_req!(TerminatedEventBody, "terminated");
impl_req!(ThreadEventBody, "thread");

impl Response {
    pub fn ok_with<T: Serialize, B: Into<Option<T>>>(seq: i64, command: &str, body: B) -> Response {
        Response {
            body: body.into().map(|v| serde_json::to_value(v).unwrap()),
            command: command.to_string(),
            message: None,
            request_seq: seq,
            seq,
            success: true,
            type_: "response".to_string(),
        }
    }

    pub fn ok<T: FromReq, B: Into<Option<T>>>(seq: i64, body: B) -> Response {
        Response {
            body: body.into().map(|v| serde_json::to_value(v).unwrap()),
            command: T::COMMAND.to_string(),
            message: None,
            request_seq: seq,
            seq,
            success: true,
            type_: "response".to_string(),
        }
    }

    pub fn err<T: Serialize, B: Into<Option<T>>>(
        seq: i64,
        command: &str,
        message: String,
        body: B,
    ) -> Response {
        Response {
            body: body.into().map(|v| serde_json::to_value(v).unwrap()),
            command: command.to_string(),
            message: Some(message),
            request_seq: seq,
            seq,
            success: false,
            type_: "response".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneOf<T, O> {
    This(T),
    Other(O),
}

impl<T, O> OneOf<T, O> {
    /// map `OneOf<T, O> -> OneOf<X, O>`
    pub fn map_t<X>(self, f: impl FnOnce(T) -> X) -> OneOf<X, O> {
        match self {
            OneOf::This(t) => OneOf::This(f(t)),
            OneOf::Other(u) => OneOf::Other(u),
        }
    }

    /// map `OneOf<T, O> -> OneOf<T, X>`
    pub fn map_o<X>(self, f: impl FnOnce(O) -> X) -> OneOf<T, X> {
        match self {
            OneOf::This(t) => OneOf::This(t),
            OneOf::Other(u) => OneOf::Other(f(u)),
        }
    }

    /// make `OneOf<T, O>` -> T
    pub fn unify(self, f: impl FnOnce(O) -> T) -> T {
        match self {
            OneOf::This(t) => t,
            OneOf::Other(u) => f(u),
        }
    }

    /// `OneOf<T, O> -> OneOf<O, T>`
    pub fn transpose(self) -> OneOf<O, T> {
        match self {
            OneOf::This(t) => OneOf::Other(t),
            OneOf::Other(u) => OneOf::This(u),
        }
    }
}

impl<T, O> OneOf<T, OneOf<T, O>> {
    /// `OneOf<T, OneOf<T, O>>` -> OneOf<T, O>`
    pub fn flat_o(self) -> OneOf<T, O> {
        match self {
            OneOf::This(t) => OneOf::This(t),
            OneOf::Other(u) => u.map_t(|x| x),
        }
    }

    /// apply flat_o then apply map_t
    pub fn flat_o_map_t<X>(self, f: impl FnOnce(T) -> X) -> OneOf<X, O> {
        self.flat_o().map_t(f)
    }

    /// apply flat_o then apply map_o
    pub fn flat_o_map_o<X>(self, f: impl FnOnce(O) -> X) -> OneOf<T, X> {
        self.flat_o().map_o(f)
    }
}

impl<T: Default, U> Default for OneOf<T, U> {
    fn default() -> Self {
        OneOf::This(T::default())
    }
}

#[derive(Clone, PartialEq, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OneOf3<T, A, O> {
    This(T),
    Among(A),
    Other(O),
}

impl<T: Default, O> OneOf<T, O> {
    pub fn default_this() -> Self {
        Self::This(T::default())
    }

    pub fn opt_default_this() -> Option<Self> {
        Some(Self::default_this())
    }
}

impl<T, O: Default> OneOf<T, O> {
    pub fn default_other() -> Self {
        Self::Other(O::default())
    }

    pub fn opt_default_other() -> Option<Self> {
        Some(Self::default_other())
    }
}

pub struct ReqWithContext<C, T, H: FnOnce(C, i64, serde_json::Error) -> T>((Request, C, H));

impl Request {
    pub fn with<C, T, H: FnOnce(C, i64, serde_json::Error) -> T>(
        self,
        ctx: C,
        err_handler: H,
    ) -> ReqWithContext<C, T, H> {
        ReqWithContext((self, ctx, err_handler))
    }
}

impl<C, T, H> ReqWithContext<C, T, H>
where
    H: FnOnce(C, i64, serde_json::Error) -> T,
{
    pub fn then<R, F, I>(self, f: F) -> OneOf<OneOf<I, T>, Self>
    where
        C: Clone,
        R: FromReq,
        F: FnOnce(C, i64, R) -> I,
    {
        let (req, ctx, handler) = self.0;
        let seq = req.seq;
        match R::from_req(req) {
            OneOf::This(res) => {
                let ret = match res {
                    Ok((req_id, req)) => OneOf::This(f(ctx.clone(), req_id, req)),
                    Err(e) => OneOf::Other(handler(ctx.clone(), seq, e)),
                };
                OneOf::This(ret)
            }
            OneOf::Other(req) => OneOf::Other(Self((req, ctx, handler))),
        }
    }

    pub fn group<F, I>(self, f: F) -> OneOf<I, Self>
    where
        F: FnOnce(OneOf<I, Self>) -> OneOf<I, Self>,
    {
        f(OneOf::Other(self))
    }

    pub fn split(self) -> (Request, C, H) {
        self.0
    }
}

impl<I, C, T, H> OneOf<OneOf<I, T>, ReqWithContext<C, T, H>>
where
    H: FnOnce(C, i64, serde_json::Error) -> T,
{
    /// if previous handler does not match method field, pass alternative handler
    pub fn or_else<F, R>(self, f: F) -> OneOf<OneOf<I, T>, ReqWithContext<C, T, H>>
    where
        C: Clone,
        R: FromReq,
        F: FnOnce(C, i64, R) -> I,
    {
        self.map_o(|req| req.then(f)).flat_o()
    }

    pub fn group<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
}

pub struct EventWithContext<C, T, H: FnOnce(C, i64, serde_json::Error) -> T>((Event, C, H));

impl Event {
    pub fn with<C, T, H: FnOnce(C, i64, serde_json::Error) -> T>(
        self,
        ctx: C,
        err_handler: H,
    ) -> EventWithContext<C, T, H> {
        EventWithContext((self, ctx, err_handler))
    }
}

impl<C, T, H> EventWithContext<C, T, H>
where
    H: FnOnce(C, i64, serde_json::Error) -> T,
{
    /// passing handler for current request
    pub fn then<N, F, I>(self, f: F) -> OneOf<OneOf<I, T>, Self>
    where
        C: Clone,
        N: FromEvent,
        F: FnOnce(C, i64, N) -> I,
    {
        let (event, ctx, handler) = self.0;
        let seq = event.seq;
        match N::from_event(event) {
            OneOf::This(res) => {
                let ret = match res {
                    Ok((seq, event)) => OneOf::This(f(ctx.clone(), seq, event)),
                    Err(e) => OneOf::Other(handler(ctx.clone(), seq, e)),
                };
                OneOf::This(ret)
            }
            OneOf::Other(event) => OneOf::Other(Self((event, ctx, handler))),
        }
    }

    pub fn group<F, I>(self, f: F) -> OneOf<I, Self>
    where
        F: FnOnce(OneOf<I, Self>) -> OneOf<I, Self>,
    {
        f(OneOf::Other(self))
    }

    pub fn split(self) -> (Event, C, H) {
        self.0
    }
}

impl<I, C, T, H> OneOf<OneOf<I, T>, EventWithContext<C, T, H>>
where
    H: FnOnce(C, i64, serde_json::Error) -> T,
{
    /// if previous handler does not match method field, pass alternative handler
    pub fn or_else<F, R>(self, f: F) -> OneOf<OneOf<I, T>, EventWithContext<C, T, H>>
    where
        C: Clone,
        R: FromEvent,
        F: FnOnce(C, i64, R) -> I,
    {
        self.map_o(|req| req.then(f)).flat_o()
    }

    pub fn group<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
}

#[cfg(feature = "async")]
mod async_impl {
    use super::{EventWithContext, FromEvent, FromReq, ReqWithContext};
    use crate::OneOf;
    use std::future::Future;

    impl<T, O> OneOf<T, O> {
        /// async version of `unify`, allowing pass async handler function
        pub async fn async_unify<F: Future<Output = T>>(self, f: impl FnOnce(O) -> F) -> T {
            match self {
                OneOf::This(t) => t,
                OneOf::Other(u) => {
                    let t = f(u).await;
                    t
                }
            }
        }
    }

    impl<C, T, H, FT> ReqWithContext<C, T, H>
    where
        H: FnOnce(C, i64, serde_json::Error) -> T,
        T: Future<Output = FT>,
    {
        /// async version of `then`, passing async handler
        pub async fn async_then<R, F, I, IF>(self, f: F) -> OneOf<OneOf<I, FT>, Self>
        where
            C: Clone,
            R: FromReq,
            IF: Future<Output = I>,
            F: FnOnce(C, i64, R) -> IF,
        {
            let (req, ctx, handler) = self.0;
            let req_id = req.seq;
            match R::from_req(req) {
                OneOf::This(res) => match res {
                    Ok((req_id, req)) => {
                        OneOf::This(OneOf::This(f(ctx.clone(), req_id, req).await))
                    }
                    Err(e) => OneOf::This(OneOf::Other(handler(ctx, req_id, e).await)),
                },
                OneOf::Other(req) => OneOf::Other(Self((req, ctx, handler))),
            }
        }

        pub async fn async_group<F, I, IFut>(self, f: F) -> OneOf<I, Self>
        where
            IFut: Future<Output = OneOf<I, Self>>,
            F: FnOnce(OneOf<I, Self>) -> IFut,
        {
            f(OneOf::Other(self)).await
        }
    }

    impl<I, C, T, H, FT> OneOf<OneOf<I, FT>, ReqWithContext<C, T, H>>
    where
        H: FnOnce(C, i64, serde_json::Error) -> T,
        T: Future<Output = FT>,
    {
        /// async version of `or_else`, passing async handler
        pub async fn async_or_else<F, R, IF>(
            self,
            f: F,
        ) -> OneOf<OneOf<I, FT>, ReqWithContext<C, T, H>>
        where
            C: Clone,
            R: FromReq,
            IF: Future<Output = I>,
            F: FnOnce(C, i64, R) -> IF,
        {
            let ret = match self {
                OneOf::This(t) => OneOf::This(t),
                OneOf::Other(o) => {
                    let o = o.async_then(f).await;
                    OneOf::Other(o)
                }
            };
            ret.flat_o()
        }

        pub async fn async_group<F, Fut>(self, f: F) -> Self
        where
            Fut: Future<Output = Self>,
            F: FnOnce(Self) -> Fut,
        {
            f(self).await
        }
    }

    impl<C, T, H, FT> EventWithContext<C, T, H>
    where
        H: FnOnce(C, i64, serde_json::Error) -> T,
        T: Future<Output = FT>,
    {
        /// async version of `then`, passing async handler
        pub async fn async_then<R, F, I, IF>(self, f: F) -> OneOf<OneOf<I, FT>, Self>
        where
            C: Clone,
            R: FromEvent,
            IF: Future<Output = I>,
            F: FnOnce(C, i64, R) -> IF,
        {
            let (event, ctx, handler) = self.0;
            let seq = event.seq;
            match R::from_event(event) {
                OneOf::This(res) => match res {
                    Ok((seq, event)) => OneOf::This(OneOf::This(f(ctx.clone(), seq, event).await)),
                    Err(e) => OneOf::This(OneOf::Other(handler(ctx, seq, e).await)),
                },
                OneOf::Other(req) => OneOf::Other(Self((req, ctx, handler))),
            }
        }

        pub async fn async_group<F, I, IFut>(self, f: F) -> OneOf<I, Self>
        where
            IFut: Future<Output = OneOf<I, Self>>,
            F: FnOnce(OneOf<I, Self>) -> IFut,
        {
            f(OneOf::Other(self)).await
        }
    }

    impl<I, C, T, H, FT> OneOf<OneOf<I, FT>, EventWithContext<C, T, H>>
    where
        H: FnOnce(C, i64, serde_json::Error) -> T,
        T: Future<Output = FT>,
    {
        /// async version of `or_else`, passing async handler
        pub async fn async_or_else<F, R, IF>(
            self,
            f: F,
        ) -> OneOf<OneOf<I, FT>, EventWithContext<C, T, H>>
        where
            C: Clone,
            R: FromEvent,
            IF: Future<Output = I>,
            F: FnOnce(C, i64, R) -> IF,
        {
            let ret = match self {
                OneOf::This(t) => OneOf::This(t),
                OneOf::Other(o) => {
                    let o = o.async_then(f).await;
                    OneOf::Other(o)
                }
            };
            ret.flat_o()
        }

        pub async fn async_group<F, Fut>(self, f: F) -> Self
        where
            Fut: Future<Output = Self>,
            F: FnOnce(Self) -> Fut,
        {
            f(self).await
        }
    }
}
